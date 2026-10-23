//! PM057 D2-H1: new current metadata over immutable recorded semantic choices.
use super::*;
use crate::substrate::{
    ComponentInstanceShard, ElectricalIdentity, NetCorrespondenceStatus, ObjectRevision,
    SourceShardKind,
    component_pad_source::{ComponentPadMaterializationStatus, component_pad_projection},
    net_correspondence_status,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn invalid(message: &str) -> EngineError {
    EngineError::Validation(format!("component binding compensation: {message}"))
}
fn next(model: &DesignModel, id: Uuid) -> Result<ObjectRevision, EngineError> {
    model
        .objects
        .get(&id)
        .and_then(|o| o.object_revision.0.checked_add(1))
        .map(ObjectRevision)
        .ok_or_else(|| invalid("missing object or revision overflow"))
}
fn pad_ids(value: &Value) -> Result<BTreeSet<Uuid>, EngineError> {
    let mut ids = BTreeSet::new();
    for pad in value
        .get("component_pads")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let id = serde_json::from_value(pad["uuid"].clone())?;
        if !ids.insert(id) {
            return Err(invalid("duplicate recorded Pad identity"));
        }
    }
    Ok(ids)
}
fn ci_semantics(value: &Value) -> Value {
    let mut value = value.clone();
    value["object_revision"] = Value::Null;
    if let Some(refs) = value["placed_package_refs"].as_array_mut() {
        for reference in refs {
            reference["object_revision"] = Value::Null;
        }
    }
    value
}
fn net_semantics(record: &crate::substrate::ElectricalIdentityRecord, ci: Uuid) -> Value {
    let mut value = serde_json::to_value(record).expect("typed electrical record");
    value["object_revision"] = Value::Null;
    if let Some(evidence) = value["identity"]["evidence"].as_array_mut() {
        for e in evidence {
            if e["component_instance"]["object_id"] == json!(ci) {
                e["component_instance"]["object_revision"] = Value::Null;
                e["board_pad"]["object_revision"] = Value::Null;
            }
        }
    }
    value
}
fn current_ci(
    model: &DesignModel,
    id: Uuid,
    package: Uuid,
    expected: &Value,
) -> Result<(), EngineError> {
    let object = model
        .objects
        .get(&id)
        .ok_or_else(|| invalid("missing CI registration"))?;
    let shard = model
        .source_shards
        .iter()
        .find(|s| {
            s.shard_id == object.source_shard_id && s.kind == SourceShardKind::ComponentInstance
        })
        .ok_or_else(|| invalid("missing authored CI source"))?;
    let value = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
    let source: ComponentInstanceShard = serde_json::from_value(value.clone())?;
    let ci = source.component_instance;
    let current = |r: &crate::substrate::RevisionedRef| {
        model
            .objects
            .get(&r.object_id)
            .is_some_and(|o| o.object_revision == r.object_revision)
    };
    if source.schema_version != crate::substrate::COMPONENT_INSTANCE_SHARD_SCHEMA_VERSION
        || ci.uuid != id
        || ci.object_revision != object.object_revision
        || value["component_instance"] != *expected
        || ci.placed_package_refs.len() != 1
        || ci.placed_package_refs[0].object_id != package
        || !ci.placed_package_refs.iter().all(current)
        || !ci.placed_symbol_refs.iter().all(current)
        || !ci.part_ref.as_ref().is_none_or(current)
        || !ci.library_bindings.values().all(|b| {
            model
                .objects
                .get(&b.target_object_id)
                .is_some_and(|o| o.object_revision == b.pinned_object_revision)
                && b.local_override_refs.iter().all(current)
        })
    {
        return Err(invalid("changed or stale recorded CI preimage"));
    }
    Ok(())
}

pub(super) fn prepare(
    model: &DesignModel,
    project_root: &Path,
    target: &TransactionRecord,
    batch: &mut OperationBatch,
) -> Result<bool, EngineError> {
    let packages: Vec<_> = batch
        .operations
        .iter()
        .filter_map(|op| {
            if let Operation::SetBoardPackagePackage { package_id, .. } = op {
                Some(*package_id)
            } else {
                None
            }
        })
        .collect();
    let components: Vec<_> = batch
        .operations
        .iter()
        .filter_map(|op| {
            if let Operation::SetComponentInstance {
                component_instance_id,
                ..
            } = op
            {
                Some(*component_instance_id)
            } else {
                None
            }
        })
        .collect();
    if packages.is_empty() || components.is_empty() {
        return Ok(false);
    }
    if packages.len() != 1
        || components.len() != 1
        || batch.operations.iter().any(|op| {
            !matches!(
                op,
                Operation::SetBoardPackagePackage { .. }
                    | Operation::SetComponentInstance { .. }
                    | Operation::SetElectricalIdentity { .. }
            )
        })
    {
        return Err(invalid("unsupported or duplicate recorded refresh family"));
    }
    if model.journal.last().map(|t| t.transaction_id) != Some(target.transaction_id)
        || batch.operations != target.inverse_operations
    {
        return Err(invalid(
            "compensation is not the actual linked recorded inverse",
        ));
    }
    let package = packages[0];
    let ci = components[0];
    let caller = model;
    let caller_ci = batch
        .operations
        .iter()
        .find_map(|op| {
            if let Operation::SetComponentInstance {
                previous_component_instance,
                ..
            } = op
            {
                Some(previous_component_instance)
            } else {
                None
            }
        })
        .unwrap();
    current_ci(caller, ci, package, caller_ci)?;
    // Match the durable source basis the canonical commit captures under its
    // write lease. A known electrical registration-cache discrepancy is not
    // permission to replace caller semantics or weaken a revision guard.
    let durable = crate::substrate::ProjectResolver::new(project_root).resolve()?;
    if durable.project.project_id != caller.project.project_id
        || durable.model_revision != caller.model_revision
        || serde_json::to_vec(&durable.journal.last())? != serde_json::to_vec(&Some(target))?
        || serde_json::to_vec(&durable.journal)? != serde_json::to_vec(&caller.journal)?
        || serde_json::to_vec(&durable.component_instances)?
            != serde_json::to_vec(&caller.component_instances)?
        || serde_json::to_vec(&durable.electrical_identities)?
            != serde_json::to_vec(&caller.electrical_identities)?
    {
        return Err(invalid(
            "durable Project revision or actual linked history changed",
        ));
    }
    let caller_dependencies: BTreeMap<_, _> = caller
        .electrical_identities
        .values()
        .filter_map(|r| {
            if let ElectricalIdentity::NetRelationship {
                logical_net,
                evidence,
                ..
            } = &r.identity
                && evidence
                    .iter()
                    .any(|e| e.component_instance.object_id == ci)
            {
                Some((r.id, (*logical_net, r)))
            } else {
                None
            }
        })
        .collect();
    let durable_dependencies: BTreeMap<_, _> = durable
        .electrical_identities
        .values()
        .filter_map(|r| {
            if let ElectricalIdentity::NetRelationship {
                logical_net,
                evidence,
                ..
            } = &r.identity
                && evidence
                    .iter()
                    .any(|e| e.component_instance.object_id == ci)
            {
                Some((r.id, (*logical_net, r)))
            } else {
                None
            }
        })
        .collect();
    if caller_dependencies != durable_dependencies
        || caller_dependencies.values().any(|(net, _)| {
            net_correspondence_status(caller, *net) != NetCorrespondenceStatus::Complete
        })
    {
        return Err(invalid(
            "caller and durable component Net preimages disagree or are stale",
        ));
    }
    if caller.objects.keys().collect::<Vec<_>>() != durable.objects.keys().collect::<Vec<_>>() {
        return Err(invalid(
            "caller and durable source object inventory differs",
        ));
    }
    for (id, object) in &caller.objects {
        let current = &durable.objects[id];
        let mut expected = serde_json::to_value(object)?;
        let mut actual = serde_json::to_value(current)?;
        if object.object_revision != current.object_revision {
            let record = caller_dependencies
                .get(id)
                .map(|(_, r)| *r)
                .ok_or_else(|| invalid("unsupported caller object epoch drift"))?;
            let advances = durable
                .journal
                .iter()
                .filter(|t| t.diff.modified.contains(id))
                .count() as u64;
            if object.object_revision != record.object_revision
                || record.object_revision.0.checked_add(advances) != Some(current.object_revision.0)
            {
                return Err(invalid(
                    "unsupported electrical registration counter discrepancy",
                ));
            }
            expected["object_revision"] = Value::Null;
            actual["object_revision"] = Value::Null;
        }
        if expected != actual {
            return Err(invalid("caller and durable source registration differs"));
        }
    }
    let model = &durable;
    let recorded_ci = batch
        .operations
        .iter()
        .find_map(|op| {
            if let Operation::SetComponentInstance {
                component_instance, ..
            } = op
            {
                Some(component_instance.clone())
            } else {
                None
            }
        })
        .unwrap();
    let recorded_ci_revision: ObjectRevision =
        serde_json::from_value(recorded_ci["object_revision"].clone())?;
    let ci_revision = next(model, ci)?;
    let package_revision = next(model, package)?;
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let mut actual = serde_json::Map::new();
    for (name, map) in board
        .as_object()
        .ok_or_else(|| invalid("missing BoardRoot"))?
    {
        if name.starts_with("component_")
            && let Some(v) = map.get(package.to_string())
        {
            actual.insert(name.clone(), v.clone());
        }
    }
    let projection = component_pad_projection(model, &board)?;
    let mut operations = batch.operations.clone();
    let mut pad_revisions = BTreeMap::new();
    let mut touched = BTreeSet::from([ci, package]);
    for op in &mut operations {
        if let Operation::SetBoardPackagePackage {
            previous_materialized,
            materialized,
            package_ref_id,
            ..
        } = op
        {
            let mut supplied = previous_materialized
                .as_object()
                .cloned()
                .ok_or_else(|| invalid("invalid recorded package preimage"))?;
            supplied.remove("pad_source_transition");
            if supplied != actual
                || board["packages"][package.to_string()]["package"] != json!(package_ref_id)
            {
                return Err(invalid("changed recorded package preimage"));
            }
            let old_ids = pad_ids(previous_materialized)?;
            for id in &old_ids {
                if caller.objects.get(id).map(|o| o.object_revision)
                    != model.objects.get(id).map(|o| o.object_revision)
                {
                    return Err(invalid("caller and durable Pad epochs disagree"));
                }
                if projection.unavailable.contains_key(id)
                    || !projection.pads.contains_key(id)
                    || projection.statuses.get(id)
                        != Some(&ComponentPadMaterializationStatus::Current)
                {
                    return Err(invalid("current Pad binding is unavailable"));
                }
                touched.insert(*id);
            }
            for id in pad_ids(materialized)? {
                let revision = if old_ids.contains(&id) {
                    next(model, id)?
                } else {
                    if model.objects.contains_key(&id) {
                        return Err(invalid("foreign recorded allocation"));
                    }
                    ObjectRevision(0)
                };
                pad_revisions.insert(id, revision);
            }
            if let Some(pads) = materialized["component_pads"].as_array_mut() {
                for pad in pads {
                    if pad["source_binding"]["component_instance"]["object_id"] != json!(ci)
                        || pad["source_binding"]["component_instance"]["object_revision"]
                            != json!(recorded_ci_revision)
                    {
                        return Err(invalid("recorded Pad has another component owner"));
                    }
                    pad["source_binding"]["component_instance"]["object_revision"] =
                        json!(ci_revision);
                }
            }
        }
    }
    for (key, pads) in board["component_pads"]
        .as_object()
        .into_iter()
        .flat_map(|m| m.iter())
    {
        if *key != package.to_string()
            && pads
                .as_array()
                .into_iter()
                .flatten()
                .any(|p| p["source_binding"]["component_instance"]["object_id"] == json!(ci))
        {
            return Err(invalid("another package depends on the refreshed CI"));
        }
    }
    let required: BTreeSet<_> = model
        .electrical_identities
        .values()
        .filter_map(|r| {
            if let ElectricalIdentity::NetRelationship { evidence, .. } = &r.identity {
                evidence
                    .iter()
                    .any(|e| e.component_instance.object_id == ci)
                    .then_some(r.id)
            } else {
                None
            }
        })
        .collect();
    let mut supplied = BTreeSet::new();
    for op in &mut operations {
        match op {
            Operation::SetComponentInstance {
                component_instance_id,
                previous_component_instance,
                component_instance,
            } => {
                if *component_instance_id != ci
                    || ci_semantics(previous_component_instance) != ci_semantics(component_instance)
                {
                    return Err(invalid(
                        "recorded refresh changes component binding semantics",
                    ));
                }
                current_ci(model, ci, package, previous_component_instance)?;
                component_instance["object_revision"] = json!(ci_revision);
                component_instance["placed_package_refs"][0]["object_revision"] =
                    json!(package_revision);
            }
            Operation::SetElectricalIdentity { previous, record } => {
                if !supplied.insert(record.id)
                    || previous.id != record.id
                    || !required.contains(&record.id)
                    || model.electrical_identities.get(&record.id) != Some(previous.as_ref())
                    || net_semantics(previous, ci) != net_semantics(record, ci)
                {
                    return Err(invalid(
                        "changed, foreign or duplicate recorded Net preimage",
                    ));
                }
                let ElectricalIdentity::NetRelationship {
                    logical_net,
                    evidence,
                    ..
                } = &mut record.identity
                else {
                    return Err(invalid("unsupported electrical compensation"));
                };
                if net_correspondence_status(model, *logical_net)
                    != NetCorrespondenceStatus::Complete
                {
                    return Err(invalid("current Net evidence is unavailable"));
                }
                record.object_revision = next(model, record.id)?;
                touched.insert(record.id);
                for e in evidence
                    .iter_mut()
                    .filter(|e| e.component_instance.object_id == ci)
                {
                    if e.component_instance.object_revision != recorded_ci_revision {
                        return Err(invalid("recorded Net and component snapshots disagree"));
                    }
                    e.component_instance.object_revision = ci_revision;
                    e.board_pad.object_revision =
                        *pad_revisions.get(&e.board_pad.object_id).ok_or_else(|| {
                            invalid("recorded Net Pad is absent from final materialization")
                        })?;
                }
            }
            _ => {}
        }
    }
    if supplied != required {
        return Err(invalid("incomplete affected Net dependencies"));
    }
    let mut candidate_batch = batch.clone();
    candidate_batch.operations = operations;
    for id in touched {
        candidate_batch
            .operations
            .push(Operation::GuardObjectRevision {
                object_id: id,
                expected_object_revision: model.objects[&id].object_revision,
            });
    }
    let mut prospective = model.clone();
    prospective.commit_without_direct_policy(candidate_batch.clone())?;
    // The clone still reads immutable current disk bytes. Use the existing
    // pending-operation source owner, not a changed hash or a guessed snapshot.
    let mut final_board = board;
    {
        let mut pass = crate::substrate::board_list_journal_ops::BoardPass::new(&mut final_board);
        for operation in &candidate_batch.operations {
            pass.apply(operation)?;
        }
    }
    let final_projection = crate::substrate::component_pad_source::projection_with_operations(
        &prospective,
        &final_board,
        &candidate_batch.operations,
    )?;
    for id in pad_revisions.keys() {
        if final_projection.unavailable.contains_key(id)
            || !final_projection.pads.contains_key(id)
            || final_projection.statuses.get(id)
                != Some(&ComponentPadMaterializationStatus::Current)
        {
            return Err(invalid("prospective Pad reference graph is incomplete"));
        }
    }
    let basis = crate::substrate::electrical_basis::ElectricalBasis::new(
        &prospective,
        &candidate_batch.operations,
    )?;
    let membership = crate::substrate::electrical_net_membership::index(
        &prospective,
        &candidate_batch.operations,
    )?;
    for id in required {
        let ElectricalIdentity::NetRelationship { logical_net, .. } =
            prospective.electrical_identities[&id].identity
        else {
            unreachable!()
        };
        if crate::substrate::electrical_identity_validation::correspondence_with_basis(
            &prospective,
            logical_net,
            Some(&basis),
            Some(&membership),
        ) != NetCorrespondenceStatus::Complete
        {
            return Err(invalid("prospective Net reference graph is incomplete"));
        }
    }
    *batch = candidate_batch;
    Ok(true)
}
