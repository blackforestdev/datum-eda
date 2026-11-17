//! Whole selected-target compensation for same-pad physical library updates.
//! Source maps and epochs come from actual recorded operations, never a model
//! with a manufactured journal/cursor or a blanket historical-current rule.
use super::*;
use crate::substrate::{
    DomainObject, ElectricalIdentity, ElectricalIdentityRecord, ModelRevision,
    component_pad_source::{ComponentPadSource, PinnedPadLibrarySource},
    operation_application_objects::apply_operation_to_objects,
    transaction_links::component_binding::{latest_package, pad_epochs},
};
use serde_json::json;
mod proof;
mod receipt;
pub(in crate::substrate) use proof::{
    Proof, qualified_package_transaction, qualified_package_transaction_with_proof,
};
mod sources;
use sources::RecordedPool as Pool;
pub(crate) fn qualify_plan(
    model: &DesignModel,
    operations: &[Operation],
) -> Result<(), EngineError> {
    receipt::verify(
        operations,
        &inverse(operations)?,
        &model.journal,
        &model.model_revision,
        &mut Proof::new(&model.journal),
    )
}

pub(super) fn family(tx: &TransactionRecord) -> bool {
    tx.operations
        .iter()
        .any(|op| matches!(op, Operation::SetBoardPackagePackage { .. }))
        && tx.operations.iter().any(|op| {
            matches!(
                op,
                Operation::LibraryAssets {
                    operation: LibraryAssetOperation::SetState { .. }
                }
            )
        })
}

fn inverse(operations: &[Operation]) -> Result<Vec<Operation>, EngineError> {
    let mut result = Vec::new();
    for operation in operations {
        if let Operation::SetBoardPackagePackage {
            package_id,
            package_ref_id,
            previous_materialized,
            materialized,
        } = operation
        {
            let mut before = previous_materialized.clone();
            before
                .as_object_mut()
                .ok_or_else(|| invalid("physical preimage is not an object"))?
                .remove("pad_source_transition");
            result.push(Operation::SetBoardPackagePackage {
                package_id: *package_id,
                package_ref_id: *package_ref_id,
                previous_materialized: materialized.clone(),
                materialized: before,
            });
        }
    }
    // The journal collects Board inverses first, then non-core inverses in
    // operation order, and finally reverses the complete array.
    for operation in operations {
        if !matches!(operation, Operation::SetBoardPackagePackage { .. }) {
            if !matches!(
                operation,
                Operation::LibraryAssets {
                    operation: LibraryAssetOperation::SetState { .. }
                } | Operation::SetPoolLibraryObject { .. }
                    | Operation::SetComponentInstance { .. }
                    | Operation::SetElectricalIdentity { .. }
            ) {
                return Err(invalid(
                    "physical receipt contains an unsupported operation",
                ));
            }
            crate::substrate::journal_operation_hooks::inverse_non_core_operation(
                operation,
                &mut result,
            )?;
        }
    }
    result.reverse();
    Ok(result)
}

fn latest_electrical(prior: &[TransactionRecord]) -> BTreeMap<Uuid, ElectricalIdentityRecord> {
    let mut records = BTreeMap::new();
    for op in prior.iter().flat_map(|tx| &tx.operations) {
        match op {
            Operation::CreateElectricalIdentity { record }
            | Operation::SetElectricalIdentity { record, .. } => {
                records.insert(record.id, record.clone());
            }
            Operation::DeleteElectricalIdentity { record } => {
                records.remove(&record.id);
            }
            _ => {}
        }
    }
    records
}

// Match the existing Format-4 validate_native rule before a typed projection
// can discard fields. History must refuse unsupported raw Pads, never erase them.
fn canonical_pads(raw: &Value) -> Result<Vec<ComponentPadSource>, EngineError> {
    let pads: Vec<ComponentPadSource> = serde_json::from_value(raw.clone())?;
    if serde_json::to_value(&pads)? != *raw {
        return Err(invalid(
            "physical history requires canonical native Pad records",
        ));
    }
    Ok(pads)
}

fn package_basis(
    prior: &[TransactionRecord],
    package: Uuid,
    proof: &mut Proof<'_>,
) -> Result<(Value, ObjectRevision), EngineError> {
    let (_, materialized) = latest_package(prior, package)
        .ok_or_else(|| invalid("package source producer is absent"))?;
    let ci: Uuid = serde_json::from_value(
        materialized["component_pads"][0]["source_binding"]["component_instance"]["object_id"]
            .clone(),
    )?;
    let current = latest_ci(prior, ci)?;
    let refs: Vec<crate::substrate::RevisionedRef> =
        serde_json::from_value(current["placed_package_refs"].clone())?;
    let [reference] = refs.as_slice() else {
        return Err(invalid(
            "physical history requires one declared package anchor",
        ));
    };
    if reference.object_id != package {
        return Err(invalid("physical history package anchor differs"));
    }
    proof.package(prior, reference)
}

fn net_semantics(
    record: &ElectricalIdentityRecord,
    users: &BTreeSet<Uuid>,
) -> Result<Value, EngineError> {
    let mut value = serde_json::to_value(record)?;
    value["object_revision"] = Value::Null;
    if let Some(evidence) = value
        .pointer_mut("/identity/evidence")
        .and_then(Value::as_array_mut)
    {
        for row in evidence {
            if row["component_instance"]["object_id"]
                .as_str()
                .and_then(|s| s.parse::<Uuid>().ok())
                .is_some_and(|id| users.contains(&id))
            {
                for field in [
                    "component_instance",
                    "board_pad",
                    "library_pad",
                    "pin_pad_map",
                ] {
                    if !row[field].is_null() {
                        row[field]["object_revision"] = Value::Null;
                    }
                }
            }
        }
    }
    Ok(value)
}

fn normalized(
    target: &TransactionRecord,
    prior: &[TransactionRecord],
    proof: &mut Proof<'_>,
) -> Result<Vec<Operation>, EngineError> {
    let index = prior
        .iter()
        .position(|tx| tx == target)
        .ok_or_else(|| invalid("physical target is not the recorded transaction"))?;
    receipt::verify(
        &target.operations,
        &target.inverse_operations,
        &prior[..index],
        &target.before_model_revision,
        proof,
    )?;
    if !family(target) || inverse(&target.operations)? != target.inverse_operations {
        return Err(invalid(
            "physical target inverse is not its whole canonical receipt",
        ));
    }
    let latest = prior
        .last()
        .ok_or_else(|| invalid("physical compensation has no accepted journal"))?;
    let mut operations = target.inverse_operations.clone();
    let mut pool = Pool::history(prior)?;
    let mut changed = BTreeSet::new();
    for op in &mut operations {
        if let Operation::SetPoolLibraryObject {
            object_id,
            relative_path,
            object_kind,
            previous_object,
            object,
        } = op
        {
            if !matches!(object_kind.as_str(), "footprints" | "padstacks")
                || !changed.insert(*object_id)
            {
                return Err(invalid(
                    "physical compensation changes unsupported or duplicate definitions",
                ));
            }
            let current = pool
                .values
                .get(relative_path)
                .ok_or_else(|| invalid("physical source lacks a recorded current witness"))?
                .clone();
            if engineering(current.clone()) != engineering(previous_object.clone()) {
                return Err(invalid("physical source semantic preimage changed"));
            }
            *previous_object = current;
            if object.get("object_revision").is_some() {
                object["object_revision"] =
                    json!(increment(pool.objects[object_id].object_revision.0)?);
            }
            crate::substrate::library_asset_journal_ops::admission::qualify_geometry_delta(
                previous_object,
                object,
                object_kind,
            )?;
        }
    }
    operations.sort_by_key(|op| match op {
        Operation::SetPoolLibraryObject { .. } => 0,
        Operation::SetBoardPackagePackage { .. } => 1,
        Operation::SetComponentInstance { .. } => 2,
        Operation::SetElectricalIdentity { .. } => 3,
        _ => 4,
    });
    for op in &operations {
        pool.apply(op, None)?;
    }
    let mut users = BTreeSet::new();
    let mut ci_epochs = BTreeMap::new();
    let mut package_epochs = BTreeMap::new();
    for op in &operations {
        if let Operation::SetComponentInstance {
            component_instance_id,
            ..
        } = op
        {
            if !users.insert(*component_instance_id) {
                return Err(invalid("duplicate physical component compensation"));
            }
            let current = latest_ci(prior, *component_instance_id)?;
            ci_epochs.insert(
                *component_instance_id,
                ObjectRevision(increment(
                    current["object_revision"]
                        .as_u64()
                        .ok_or_else(|| invalid("current component epoch missing"))?,
                )?),
            );
        }
        if let Operation::SetBoardPackagePackage { package_id, .. } = op {
            if package_epochs
                .insert(
                    *package_id,
                    ObjectRevision(increment(package_basis(prior, *package_id, proof)?.1.0)?),
                )
                .is_some()
            {
                return Err(invalid("duplicate physical package compensation"));
            }
        }
    }
    let mut pad_bindings = BTreeMap::new();
    let mut needed = BTreeSet::new();
    for op in &mut operations {
        let Operation::SetBoardPackagePackage {
            package_id,
            package_ref_id,
            previous_materialized,
            materialized,
        } = op
        else {
            continue;
        };
        let (current_ref, current) = latest_package(prior, *package_id)
            .ok_or_else(|| invalid("package has no recorded current source"))?;
        let current_pads = canonical_pads(&current["component_pads"])?;
        canonical_pads(&previous_materialized["component_pads"])?;
        let mut pads = canonical_pads(&materialized["component_pads"])?;
        if current_ref != *package_ref_id {
            return Err(invalid("physical compensation changes package identity"));
        }
        let mut expected = previous_materialized.clone();
        expected
            .as_object_mut()
            .ok_or_else(|| invalid("invalid package preimage"))?
            .remove("pad_source_transition");
        if package_semantics(&current)? != package_semantics(&expected)? {
            return Err(invalid("physical package semantic preimage changed"));
        }
        let current_pads: BTreeMap<Uuid, ComponentPadSource> = current_pads
            .into_iter()
            .map(|pad| (pad.uuid, pad))
            .collect();
        if pads.len() != current_pads.len() {
            return Err(invalid("physical compensation changes pad topology"));
        }
        for pad in &mut pads {
            let current = current_pads
                .get(&pad.uuid)
                .ok_or_else(|| invalid("physical compensation replaces a pad"))?;
            let old = current
                .source_binding
                .as_ref()
                .ok_or_else(|| invalid("current physical source is unbound"))?;
            let binding = pad
                .source_binding
                .as_mut()
                .ok_or_else(|| invalid("compensation downgrades a bound source"))?;
            if binding.package_id != *package_id
                || binding.project_id != old.project_id
                || binding.board_id != old.board_id
            {
                return Err(invalid("compensation changes physical source ownership"));
            }
            for (pin, now) in [
                (&mut binding.part, &old.part),
                (&mut binding.package, &old.package),
                (&mut binding.land_pattern, &old.land_pattern),
                (&mut binding.library_pad, &old.library_pad),
                (&mut binding.padstack, &old.padstack),
            ] {
                if pin.object_id != now.object_id || pin.owner_id != now.owner_id {
                    return Err(invalid("compensation changes library identity"));
                }
                pool.pin(pin, now, latest.transaction_id)?;
            }
            let (Some(map), Some(old_map)) = (&mut binding.pin_pad_map, &old.pin_pad_map) else {
                return Err(invalid("compensation changes mapping identity"));
            };
            if map.object_id != old_map.object_id {
                return Err(invalid("compensation changes mapping identity"));
            }
            pool.pin(map, old_map, latest.transaction_id)?;
            let ci = binding
                .component_instance
                .as_mut()
                .ok_or_else(|| invalid("physical compensation has no component"))?;
            if old
                .component_instance
                .as_ref()
                .is_none_or(|old| old.object_id != ci.object_id)
            {
                return Err(invalid("physical compensation rebinds a component"));
            }
            ci.object_revision = *ci_epochs
                .get(&ci.object_id)
                .ok_or_else(|| invalid("physical compensation omitted its component"))?;
            binding.materialized_against = latest.after_model_revision.clone();
            if let Some(snapshot) = &mut binding.process_snapshot {
                if let Some(value) = pool.values.get(&binding.padstack.relative_path) {
                    snapshot.raw_padstack = value.clone();
                }
                if let Some(value) = pool.values.get(&binding.land_pattern.relative_path) {
                    snapshot.raw_footprint = value.clone();
                }
            }
            pad.validate_frozen()?;
            needed.insert(pad.uuid);
            pad_bindings.insert(pad.uuid, pad.source_binding.as_ref().unwrap().clone());
        }
        materialized["component_pads"] = serde_json::to_value(pads)?;
        materialized
            .as_object_mut()
            .unwrap()
            .remove("pad_source_transition");
        *previous_materialized = current;
    }
    let pad_epochs = pad_epochs(prior, &needed)
        .ok_or_else(|| invalid("Pad epochs are not producer-qualified"))?;
    for op in &mut operations {
        if let Operation::SetComponentInstance {
            component_instance_id,
            previous_component_instance,
            component_instance,
        } = op
        {
            let current = latest_ci(prior, *component_instance_id)?;
            if ci_intent(&current) != ci_intent(previous_component_instance)
                || ci_intent(&current) != ci_intent(component_instance)
            {
                return Err(invalid(
                    "physical compensation changes component membership or selection",
                ));
            }
            component_instance["object_revision"] =
                serde_json::to_value(ci_epochs[component_instance_id])?;
            for reference in component_instance["placed_package_refs"]
                .as_array_mut()
                .ok_or_else(|| invalid("missing package refs"))?
            {
                let id: Uuid = serde_json::from_value(reference["object_id"].clone())?;
                reference["object_revision"] = serde_json::to_value(
                    *package_epochs
                        .get(&id)
                        .ok_or_else(|| invalid("component has an unsupported other package"))?,
                )?;
            }
            // Same identity choices only: current unchanged Part/Map epochs are
            // retained; changed native Footprint pins use actual projected maps.
            component_instance["part_ref"] = current["part_ref"].clone();
            if let Some(bindings) = component_instance["library_bindings"].as_object_mut() {
                for (key, binding) in bindings {
                    let id: Uuid = serde_json::from_value(binding["target_object_id"].clone())?;
                    binding["pinned_object_revision"] = pool.objects.get(&id).map_or_else(
                        || current["library_bindings"][key]["pinned_object_revision"].clone(),
                        |object| json!(object.object_revision),
                    );
                }
            }
            *previous_component_instance = current;
        }
    }
    let records = latest_electrical(prior);
    let mut updated_records = BTreeSet::new();
    for op in &mut operations {
        if let Operation::SetElectricalIdentity { previous, record } = op {
            let current = records
                .get(&record.id)
                .ok_or_else(|| invalid("electrical source is no longer present"))?;
            if net_semantics(current, &users)? != net_semantics(previous, &users)?
                || net_semantics(current, &users)? != net_semantics(record, &users)?
            {
                return Err(invalid("physical compensation changes electrical meaning"));
            }
            if !updated_records.insert(record.id) {
                return Err(invalid("duplicate electrical compensation"));
            }
            record.object_revision = ObjectRevision(increment(current.object_revision.0)?);
            let ElectricalIdentity::NetRelationship { evidence, .. } = &mut record.identity else {
                return Err(invalid("unsupported electrical compensation"));
            };
            for row in evidence
                .iter_mut()
                .filter(|row| users.contains(&row.component_instance.object_id))
            {
                let binding = pad_bindings
                    .get(&row.board_pad.object_id)
                    .ok_or_else(|| invalid("electrical Pad has no complete compensation"))?;
                row.component_instance.object_revision =
                    ci_epochs[&row.component_instance.object_id];
                row.board_pad.object_revision =
                    ObjectRevision(increment(pad_epochs[&row.board_pad.object_id])?);
                row.library_pad = Some(crate::substrate::RevisionedRef {
                    object_id: binding.library_pad.object_id,
                    object_revision: binding.library_pad.object_revision,
                });
                let map = binding.pin_pad_map.as_ref().unwrap();
                row.pin_pad_map = crate::substrate::RevisionedRef {
                    object_id: map.object_id,
                    object_revision: map.object_revision,
                };
            }
            *previous = Box::new(current.clone());
        }
    }
    for record in records.values() {
        if matches!(&record.identity,ElectricalIdentity::NetRelationship {evidence,..} if evidence.iter().any(|row|users.contains(&row.component_instance.object_id)))
            && !updated_records.contains(&record.id)
        {
            return Err(invalid("physical compensation omitted electrical evidence"));
        }
    }
    let states: Vec<_> = operations
        .iter()
        .filter_map(|op| match op {
            Operation::LibraryAssets { operation } => Some(operation),
            _ => None,
        })
        .collect();
    let [state] = states.as_slice() else {
        return Err(invalid("physical compensation needs one library state"));
    };
    let latest_state = prior
        .iter()
        .rev()
        .flat_map(|tx| tx.operations.iter().rev())
        .find_map(|op| match op {
            Operation::LibraryAssets { operation } => Some(operation.state()),
            _ => None,
        })
        .transpose()?
        .ok_or_else(|| invalid("current library state missing"))?;
    if let LibraryAssetOperation::SetState { previous, .. } = state {
        if previous != latest_state {
            return Err(invalid("physical library state preimage changed"));
        }
    }
    Ok(operations)
}

fn package_semantics(value: &Value) -> Result<Value, EngineError> {
    let mut value = value.clone();
    value
        .as_object_mut()
        .ok_or_else(|| invalid("invalid package materialization"))?
        .remove("pad_source_transition");
    if let Some(pads) = value["component_pads"].as_array_mut() {
        for pad in pads {
            if let Some(binding) = pad["source_binding"].as_object_mut() {
                binding.remove("materialized_against");
                for field in [
                    "part",
                    "package",
                    "land_pattern",
                    "library_pad",
                    "padstack",
                    "pin_pad_map",
                ] {
                    if let Some(pin) = binding.get_mut(field).and_then(Value::as_object_mut) {
                        for field in [
                            "object_revision",
                            "owner_value_hash",
                            "record_hash",
                            "journal_tip",
                        ] {
                            pin.remove(field);
                        }
                    }
                }
                if let Some(ci) = binding
                    .get_mut("component_instance")
                    .and_then(Value::as_object_mut)
                {
                    ci.remove("object_revision");
                }
                if let Some(snapshot) = binding
                    .get_mut("process_snapshot")
                    .and_then(Value::as_object_mut)
                {
                    for field in ["raw_footprint", "raw_padstack"] {
                        if let Some(raw) = snapshot.get_mut(field).and_then(Value::as_object_mut) {
                            raw.remove("object_revision");
                        }
                    }
                }
            }
        }
    }
    Ok(value)
}

pub(super) fn prepare(
    model: &DesignModel,
    target: &TransactionRecord,
    batch: &mut OperationBatch,
) -> Result<bool, EngineError> {
    if !crate::substrate::journal_history::current_target(&model.journal, target)
        || batch.operations != target.inverse_operations
    {
        return Err(invalid(
            "physical compensation is not the exact current whole inverse",
        ));
    }
    let operations = normalized(target, &model.journal, &mut Proof::new(&model.journal))?;
    let pool = Pool::history(&model.journal)?;
    for (id, object) in &pool.objects {
        if model.objects.get(id).is_none_or(|actual| actual != object) {
            return Err(invalid(
                "recorded pool epoch differs from actual resolved source",
            ));
        }
    }
    batch.operations = operations;
    Ok(true)
}

pub(super) fn matches(
    tx: &TransactionRecord,
    target: &TransactionRecord,
    prior: &[TransactionRecord],
) -> bool {
    matches_with_proof(tx, target, prior, &mut Proof::new(prior))
}

fn matches_with_proof(
    tx: &TransactionRecord,
    target: &TransactionRecord,
    prior: &[TransactionRecord],
    proof: &mut Proof<'_>,
) -> bool {
    crate::substrate::journal_history::selected_target(prior, target, tx.transaction_kind)
        && prior
            .last()
            .is_some_and(|last| last.after_model_revision == tx.before_model_revision)
        && normalized(target, prior, proof).is_ok_and(|operations| {
            operations == tx.operations
                && inverse(&operations).is_ok_and(|inverse| inverse == tx.inverse_operations)
        })
}
