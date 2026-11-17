//! Library revision intent and current-epoch compensation. Assignment receipts
//! and metadata proofs retain their separate owners and authority.
use super::*;
use crate::{
    library_assets::{
        AssetKind, AssetRevisionRef, ClosureLimits, inspect_closure, lock::ProjectLibraryLock,
    },
    substrate::{DesignModel, ObjectRevision, Operation, TransactionKind, TransactionRecord},
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;
mod physical;
pub(crate) use physical::qualify_plan;
pub(in crate::substrate) use physical::{
    Proof, qualified_package_transaction, qualified_package_transaction_with_proof,
};

fn invalid(message: &str) -> EngineError {
    refusal(format!("library revision history: {message}"))
}
fn increment(value: u64) -> Result<u64, EngineError> {
    value
        .checked_add(1)
        .ok_or_else(|| invalid("native revision overflow"))
}
fn transition(
    operations: &[Operation],
) -> Result<Option<(Uuid, ProjectLibraryLock, ProjectLibraryLock)>, EngineError> {
    let states: Vec<_> = operations
        .iter()
        .filter_map(|op| match op {
            Operation::LibraryAssets { operation } => Some(operation),
            _ => None,
        })
        .collect();
    let [operation] = states.as_slice() else {
        if states.is_empty() {
            return Ok(None);
        }
        return Err(invalid("multiple library transitions"));
    };
    let LibraryAssetOperation::SetState {
        project_id,
        previous,
        state,
        ..
    } = operation
    else {
        return Ok(None);
    };
    let limit = super::super::library_asset_resolver::server_policy()?.metadata_bytes;
    Ok(Some((
        *project_id,
        previous.decode(limit)?.0,
        state.decode(limit)?.0,
    )))
}
fn bound(
    lock: &ProjectLibraryLock,
    component: Uuid,
) -> Result<Option<AssetRevisionRef>, EngineError> {
    let refs: BTreeSet<_> = lock
        .bindings
        .iter()
        .filter(|b| b.component_instance_id == component)
        .map(|b| b.component.clone())
        .collect();
    if refs.len() > 1 {
        return Err(invalid("conflicting component Parts"));
    }
    Ok(refs.into_iter().next())
}
fn part_id(value: &Value) -> Option<Uuid> {
    value
        .pointer("/part_ref/object_id")
        .and_then(Value::as_str)
        .and_then(|id| id.parse().ok())
}
fn selection(value: &Value) -> Value {
    let mut value = value.clone();
    if let Some(map) = value.as_object_mut() {
        map.remove("object_revision");
    }
    for key in ["placed_symbol_refs", "placed_package_refs"] {
        if let Some(refs) = value[key].as_array_mut() {
            for reference in refs {
                if let Some(fields) = reference.as_object_mut() {
                    fields.remove("object_revision");
                }
            }
        }
    }
    value
}
fn latest_ci(prior: &[TransactionRecord], id: Uuid) -> Result<Value, EngineError> {
    let mut current = None;
    for op in prior.iter().flat_map(|tx| &tx.operations) {
        match op {
            Operation::CreateComponentInstance {
                component_instance_id,
                component_instance,
            }
            | Operation::SetComponentInstance {
                component_instance_id,
                component_instance,
                ..
            } if *component_instance_id == id => current = Some(component_instance.clone()),
            Operation::DeleteComponentInstance {
                component_instance_id,
                ..
            } if *component_instance_id == id => current = None,
            _ => {}
        }
    }
    current.ok_or_else(|| invalid("component has no canonical postimage"))
}

/// Resolve an explicit lock adoption, never a catalog head or a guessed latest
/// native revision. Later selection changes remain with their existing owner.
pub(in crate::substrate) fn component_adoption(
    model: &DesignModel,
    component: Uuid,
    part: Uuid,
    current: &Value,
) -> Result<Option<usize>, EngineError> {
    if model.project.schema_version != Some(2) {
        return Ok(None);
    }
    let mut current = current.clone();
    for (index, tx) in model.journal.iter().enumerate().rev() {
        let changes: Vec<_> = tx
            .operations
            .iter()
            .filter_map(|op| match op {
                Operation::SetComponentInstance {
                    component_instance_id,
                    previous_component_instance,
                    component_instance,
                } if *component_instance_id == component => {
                    Some((Some(previous_component_instance), component_instance))
                }
                Operation::CreateComponentInstance {
                    component_instance_id,
                    component_instance,
                } if *component_instance_id == component => Some((None, component_instance)),
                _ => None,
            })
            .collect();
        if changes.len() > 1 {
            return Err(invalid("ambiguous component transition"));
        }
        if changes.first().is_some_and(|(_, after)| **after != current) {
            return Err(invalid(
                "component postimage differs from its recorded chain",
            ));
        }
        if let Some((project, before_lock, after_lock)) = transition(&tx.operations)? {
            let before_ref = bound(&before_lock, component)?;
            let after_ref = bound(&after_lock, component)?;
            if let Some(reference) =
                after_ref.filter(|r| r.asset_id == part && Some(r) != before_ref.as_ref())
            {
                if project != model.project.project_id {
                    return Err(invalid("foreign project revision history"));
                }
                let Ok(before_ci) = latest_ci(&model.journal[..index], component) else {
                    // A first CI birth remains with the existing paired Symbol
                    // birth/assignment owner, not this revision authority.
                    return Ok(None);
                };
                if part_id(&before_ci) != Some(part) || part_id(&current) != Some(part) {
                    return Ok(None);
                }
                for key in [
                    "placed_symbol_refs",
                    "placed_package_refs",
                    "placed_symbol_roles",
                    "placed_package_roles",
                ] {
                    if selection(&before_ci)[key] != selection(&current)[key] {
                        return Err(invalid("revision adoption changes component membership"));
                    }
                }
                let expected_epoch = before_ci["object_revision"]
                    .as_u64()
                    .ok_or_else(|| invalid("prior component epoch missing"))?;
                let expected_epoch = if changes.is_empty() {
                    expected_epoch
                } else {
                    increment(expected_epoch)?
                };
                if current["object_revision"].as_u64() != Some(expected_epoch) {
                    return Err(invalid(
                        "revision adoption has no exact current component epoch",
                    ));
                }
                if tx.transaction_kind != TransactionKind::Normal {
                    let targets = super::super::journal_history::targets(&model.journal[..index])
                        .map_err(EngineError::Operation)?;
                    let target = targets
                        .target(tx.transaction_kind)
                        .ok_or_else(|| invalid("compensation target missing"))?;
                    if !matches(tx, target, &model.journal[..index]) {
                        return Err(invalid("unproved library compensation"));
                    }
                }
                authenticate_payloads(model, index, &after_lock, &reference, component, &current)?;
                return Ok(Some(index));
            }
        }
        if let Some((before, _)) = changes.first() {
            let Some(before) = before else {
                return Ok(None);
            };
            if selection(before) != selection(&current) {
                return Ok(None);
            }
            current = (*before).clone();
        }
    }
    Ok(None)
}
fn authenticate_payloads(
    model: &DesignModel,
    index: usize,
    lock: &ProjectLibraryLock,
    root: &AssetRevisionRef,
    component_id: Uuid,
    component: &Value,
) -> Result<(), EngineError> {
    let project = model
        .source_shards
        .iter()
        .find(|s| s.kind == SourceShardKind::ProjectManifest)
        .and_then(|s| s.path.parent())
        .ok_or_else(|| invalid("project source missing"))?;
    let limits = super::super::library_asset_resolver::server_policy()?;
    let state = super::super::library_asset_resolver::read_state(project, limits)?;
    let snapshot = resolve_state(project, &state, &BTreeMap::new(), limits)?;
    let closure = inspect_closure(
        &[root.clone()],
        snapshot.assets(),
        ClosureLimits {
            max_nodes: limits.files,
            max_depth: limits.depth,
        },
    )?;
    if !closure.graph_report().valid {
        return Err(invalid("historical Part closure invalid"));
    }
    let mut revisions = BTreeMap::new();
    for (reference, asset) in closure.assets() {
        let entry = lock
            .assets
            .iter()
            .find(|entry| entry.asset == *reference)
            .ok_or_else(|| invalid("declared dependency is not locked at this adoption"))?;
        let kind = asset
            .revision()
            .kind
            .pool_kind()
            .ok_or_else(|| invalid("owned model history unsupported"))?;
        let owner = model
            .objects
            .get(&reference.asset_id)
            .ok_or_else(|| invalid("native definition missing"))?;
        if !model
            .source_shards
            .iter()
            .any(|s| s.shard_id == owner.source_shard_id && s.relative_path == entry.source_path)
        {
            return Err(invalid("native definition source owner changed"));
        }
        let (payload, revision) =
            model.pool_source_at_transaction(reference.asset_id, kind, index)?;
        asset.payload().verify_materialization(
            &payload,
            super::super::library_asset_resolver::counter_policy(
                asset.revision().kind,
                asset.revision().payload_schema_version,
            )?,
        )?;
        revisions.insert(reference.asset_id, revision);
    }
    if snapshot.assets()[root].revision().kind != AssetKind::Part
        || component["part_ref"]["object_revision"]
            != serde_json::to_value(revisions[&root.asset_id])?
    {
        return Err(invalid("Part pin is not the actual adopted native epoch"));
    }
    let typed: super::super::component_instance::PersistedComponentInstance =
        serde_json::from_value(component.clone())?;
    if typed.uuid != component_id {
        return Err(invalid("component declaration identity changed"));
    }
    for binding in typed.library_bindings.values() {
        if let Some(revision) = revisions.get(&binding.target_object_id)
            && binding.pinned_object_revision != *revision
        {
            return Err(invalid("declared binding is not the adopted native epoch"));
        }
        for pin in &binding.local_override_refs {
            if let Some(revision) = revisions.get(&pin.object_id)
                && pin.object_revision != *revision
            {
                return Err(invalid("override pin differs from adopted native epoch"));
            }
        }
    }
    Ok(())
}

fn supported(operations: &[Operation]) -> bool {
    operations.iter().all(|op| {
        matches!(
            op,
            Operation::LibraryAssets {
                operation: LibraryAssetOperation::SetState { .. }
            } | Operation::CreatePoolLibraryObject { .. }
                | Operation::SetPoolLibraryObject { .. }
                | Operation::DeletePoolLibraryObject { .. }
                | Operation::SetComponentInstance { .. }
                | Operation::SetProposalMetadata { .. }
        )
    })
}
fn inverse(operations: &[Operation]) -> Result<Vec<Operation>, EngineError> {
    if !supported(operations) {
        return Err(invalid("mixed compensation belongs to another owner"));
    }
    let mut inverse = Vec::new();
    for operation in operations {
        super::super::journal_operation_hooks::inverse_non_core_operation(operation, &mut inverse)?;
    }
    inverse.reverse();
    Ok(inverse)
}
fn root_epoch(prior: &[TransactionRecord], id: Uuid) -> Result<ObjectRevision, EngineError> {
    let mut epoch = None;
    let mut preceding = 0u64;
    for operation in prior.iter().flat_map(|tx| &tx.operations) {
        match operation {
            Operation::CreatePoolLibraryObject {
                object_id, object, ..
            } if *object_id == id => {
                epoch = Some(super::super::object_revision_for(object).0);
                preceding = 0;
            }
            Operation::SetPoolLibraryObject {
                object_id,
                previous_object,
                ..
            } if *object_id == id => {
                let before = epoch.unwrap_or(
                    super::super::object_revision_for(previous_object)
                        .0
                        .checked_add(preceding)
                        .ok_or_else(|| invalid("native epoch overflow"))?,
                );
                epoch = Some(increment(before)?);
            }
            Operation::DeletePoolLibraryObject { object_id, .. } if *object_id == id => {
                epoch = None;
                preceding = 0;
            }
            operation
                if super::super::operation_application_pool_objects::root_revision_advance(
                    operation, id,
                ) != 0 =>
            {
                if let Some(value) = epoch.as_mut() {
                    *value = increment(*value)?;
                } else {
                    preceding = increment(preceding)?;
                }
            }
            _ => {}
        }
    }
    epoch
        .map(ObjectRevision)
        .ok_or_else(|| invalid("native epoch has no recorded source witness"))
}
fn pool_value(prior: &[TransactionRecord], id: Uuid) -> Result<Value, EngineError> {
    let mut value = None;
    for operation in prior.iter().flat_map(|tx| &tx.operations) {
        match operation {
            Operation::CreatePoolLibraryObject {
                object_id, object, ..
            }
            | Operation::SetPoolLibraryObject {
                object_id, object, ..
            } if *object_id == id => value = Some(object.clone()),
            Operation::DeletePoolLibraryObject { object_id, .. } if *object_id == id => {
                value = None
            }
            _ => {}
        }
    }
    value.ok_or_else(|| invalid("pool preimage has no recorded source"))
}
fn engineering(mut value: Value) -> Value {
    if let Some(fields) = value.as_object_mut() {
        fields.remove("object_revision");
    }
    value
}
fn ci_intent(value: &Value) -> Value {
    let mut value = selection(value);
    if !value["part_ref"].is_null() {
        value["part_ref"]["object_revision"] = Value::Null;
    }
    if let Some(bindings) = value
        .get_mut("library_bindings")
        .and_then(Value::as_object_mut)
    {
        for binding in bindings.values_mut() {
            binding["pinned_object_revision"] = Value::Null;
            if let Some(refs) = binding
                .get_mut("local_override_refs")
                .and_then(Value::as_array_mut)
            {
                for pin in refs {
                    pin["object_revision"] = Value::Null;
                }
            }
        }
    }
    value
}
fn pin_epoch(
    pin: &mut Value,
    current: &Value,
    revisions: &BTreeMap<Uuid, ObjectRevision>,
) -> Result<(), EngineError> {
    let Some(id) = pin["object_id"]
        .as_str()
        .and_then(|id| id.parse::<Uuid>().ok())
    else {
        return Err(invalid("invalid native pin"));
    };
    if let Some(epoch) = revisions.get(&id) {
        pin["object_revision"] = serde_json::to_value(epoch)?;
    } else if current["object_id"] == pin["object_id"] {
        pin["object_revision"] = current["object_revision"].clone();
    }
    Ok(())
}
fn normalized(
    target: &TransactionRecord,
    prior: &[TransactionRecord],
) -> Result<Vec<Operation>, EngineError> {
    if transition(&target.operations)?.is_none() || !supported(&target.inverse_operations) {
        return Err(invalid("not a library compensation family"));
    }
    if inverse(&target.operations)? != target.inverse_operations {
        return Err(invalid(
            "target inverse differs from the canonical library operation owner",
        ));
    }
    let current_state = prior
        .iter()
        .rev()
        .flat_map(|tx| tx.operations.iter().rev())
        .find_map(|op| match op {
            Operation::LibraryAssets { operation } => Some(operation.state()),
            _ => None,
        })
        .transpose()?
        .ok_or_else(|| invalid("current library state is unavailable"))?;
    let mut operations = target.inverse_operations.clone();
    for operation in &operations {
        if let Operation::LibraryAssets {
            operation: LibraryAssetOperation::SetState { previous, .. },
        } = operation
            && previous != current_state
        {
            return Err(invalid(
                "library compensation preimage differs from current recorded state",
            ));
        }
    }
    // Native definitions must exist at their new counters before CI references
    // are applied. Never grant availability from future pending operations.
    operations.sort_by_key(|op| match op {
        Operation::CreatePoolLibraryObject { .. }
        | Operation::SetPoolLibraryObject { .. }
        | Operation::DeletePoolLibraryObject { .. } => 0,
        Operation::SetComponentInstance { .. } => 1,
        Operation::LibraryAssets { .. } => 2,
        _ => 3,
    });
    let mut owners = BTreeSet::new();
    let mut revisions = BTreeMap::new();
    for operation in &mut operations {
        match operation {
            Operation::SetPoolLibraryObject {
                object_id,
                previous_object,
                object,
                ..
            } => {
                if !owners.insert(*object_id) {
                    return Err(invalid("duplicate native materialization in compensation"));
                }
                let current = pool_value(prior, *object_id)?;
                if engineering(current.clone()) != engineering(previous_object.clone()) {
                    return Err(invalid("pool preimage changed"));
                }
                *previous_object = current;
                let epoch = ObjectRevision(increment(root_epoch(prior, *object_id)?.0)?);
                if object.get("object_revision").is_some() {
                    object["object_revision"] = serde_json::to_value(epoch)?;
                }
                revisions.insert(*object_id, epoch);
            }
            Operation::CreatePoolLibraryObject {
                object_id, object, ..
            } => {
                if !owners.insert(*object_id) {
                    return Err(invalid("duplicate native materialization in compensation"));
                }
                revisions.insert(*object_id, super::super::object_revision_for(object));
            }
            Operation::DeletePoolLibraryObject { object_id, .. } => {
                if !owners.insert(*object_id) {
                    return Err(invalid("duplicate native materialization in compensation"));
                }
            }
            _ => {}
        }
    }
    let mut components = BTreeSet::new();
    for operation in &mut operations {
        if let Operation::SetComponentInstance {
            component_instance_id,
            previous_component_instance,
            component_instance,
        } = operation
        {
            if !components.insert(*component_instance_id) {
                return Err(invalid("duplicate component compensation"));
            }
            let current = latest_ci(prior, *component_instance_id)?;
            for value in [
                &current,
                &*previous_component_instance,
                &*component_instance,
            ] {
                serde_json::from_value::<
                    super::super::component_instance::PersistedComponentInstance,
                >(value.clone())?;
            }
            if ci_intent(&current) != ci_intent(previous_component_instance) {
                return Err(invalid("component semantic preimage changed"));
            }
            let mut next = component_instance.clone();
            for key in [
                "placed_symbol_refs",
                "placed_package_refs",
                "placed_symbol_roles",
                "placed_package_roles",
            ] {
                if ci_intent(&current)[key] != ci_intent(&next)[key] {
                    return Err(invalid("compensation changes component membership"));
                }
                if let Some(value) = current.get(key) {
                    next[key] = value.clone();
                } else {
                    next.as_object_mut().unwrap().remove(key);
                }
            }
            next["object_revision"] = increment(
                current["object_revision"]
                    .as_u64()
                    .ok_or_else(|| invalid("component epoch missing"))?,
            )?
            .into();
            if !next["part_ref"].is_null() {
                pin_epoch(&mut next["part_ref"], &current["part_ref"], &revisions)?;
            }
            if let Some(bindings) = next
                .get_mut("library_bindings")
                .and_then(Value::as_object_mut)
            {
                for (key, binding) in bindings {
                    let id: Uuid = serde_json::from_value(binding["target_object_id"].clone())?;
                    if let Some(epoch) = revisions.get(&id) {
                        binding["pinned_object_revision"] = serde_json::to_value(epoch)?;
                    } else if current["library_bindings"][key]["target_object_id"]
                        == binding["target_object_id"]
                    {
                        binding["pinned_object_revision"] =
                            current["library_bindings"][key]["pinned_object_revision"].clone();
                    }
                    if let Some(refs) = binding
                        .get_mut("local_override_refs")
                        .and_then(Value::as_array_mut)
                    {
                        for (i, pin) in refs.iter_mut().enumerate() {
                            pin_epoch(
                                pin,
                                &current["library_bindings"][key]["local_override_refs"][i],
                                &revisions,
                            )?;
                        }
                    }
                }
            }
            *previous_component_instance = current;
            *component_instance = next;
        }
    }
    Ok(operations)
}

/// Library compensation cannot fall through to the legacy exact-inverse path,
/// which would copy obsolete CI counters into a newer native materialization.
pub(in crate::substrate) fn requires_owned_compensation(
    target: &TransactionRecord,
    prior: &[TransactionRecord],
) -> bool {
    if super::binding_history::exact_native_family(target, prior) {
        return false;
    }
    target.operations.iter().any(|op| {
        matches!(
            op,
            Operation::LibraryAssets {
                operation: LibraryAssetOperation::SetState { .. }
            }
        )
    })
}

pub(in crate::substrate) fn prepare(
    model: &DesignModel,
    target: &TransactionRecord,
    batch: &mut OperationBatch,
) -> Result<bool, EngineError> {
    if physical::family(target) {
        return physical::prepare(model, target, batch);
    }
    if transition(&target.operations)?.is_none() {
        return Ok(false);
    }
    if super::binding_history::exact_native_family(target, &model.journal) {
        return Ok(false);
    }
    if !supported(&target.operations) {
        return Err(invalid(
            "mixed library compensation requires its physical source owner",
        ));
    }
    if !super::super::journal_history::current_target(&model.journal, target)
        || batch.operations != target.inverse_operations
    {
        return Err(invalid("compensation is not the exact selected target"));
    }
    let operations = normalized(target, &model.journal)?;
    for operation in &operations {
        if let Operation::SetPoolLibraryObject { object_id, .. } = operation
            && model.objects.get(object_id).map(|o| o.object_revision)
                != Some(root_epoch(&model.journal, *object_id)?)
        {
            return Err(invalid(
                "recorded native epoch differs from the durable model",
            ));
        }
    }
    batch.operations = operations;
    Ok(true)
}
pub(in crate::substrate) fn matches(
    tx: &TransactionRecord,
    target: &TransactionRecord,
    prior: &[TransactionRecord],
) -> bool {
    if physical::family(target) {
        return physical::matches(tx, target, prior);
    }
    super::super::journal_history::selected_target(prior, target, tx.transaction_kind)
        && normalized(target, prior).is_ok_and(|operations| {
            operations == tx.operations
                && inverse(&operations).is_ok_and(|expected| expected == tx.inverse_operations)
        })
}
