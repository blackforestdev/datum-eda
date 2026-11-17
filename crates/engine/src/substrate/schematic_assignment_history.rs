//! Repin exact recorded schematic assignment/transform inverses to current epochs.
//! The same pure reconstruction validates durable history; semantic choices stay immutable.
use super::{DesignModel, EngineError, Operation, OperationBatch, TransactionRecord};
use serde_json::{Value, json};
use uuid::Uuid;
fn invalid() -> EngineError {
    EngineError::Validation("Schematic binding history preimage or revision changed".into())
}
#[path = "schematic_binding_history_delete.rs"]
mod deletion;
#[path = "schematic_binding_history_family.rs"]
mod family;
pub(super) use deletion::order_restoration as order_component_restoration;
fn member(value: &Value, id: Uuid, symbol: Uuid) -> Option<u64> {
    if value["uuid"] != json!(id) || value["placed_package_refs"].as_array()?.len() != 0 {
        return None;
    }
    let refs = value["placed_symbol_refs"].as_array()?;
    let matching: Vec<_> = refs
        .iter()
        .filter(|r| r["object_id"] == json!(symbol))
        .collect();
    let [found] = matching.as_slice() else {
        return None;
    };
    found["object_revision"].as_u64()
}
/// The last recorded pin is a post-transaction epoch. Later source modifications
/// advance it exactly as canonical schematic application does; no hash equivalence.
fn prior_state(
    prior: &[TransactionRecord],
    id: Uuid,
    symbol: Uuid,
) -> Option<(Option<Value>, u64)> {
    let mut value = None;
    let mut epoch = None;
    for tx in prior {
        if let Some(n) = epoch.as_mut() {
            if tx.diff.modified.contains(&symbol) {
                *n = u64::checked_add(*n, 1)?;
            }
        }
        for op in &tx.operations {
            match op {
                Operation::CreateComponentInstance {
                    component_instance_id,
                    component_instance,
                }
                | Operation::SetComponentInstance {
                    component_instance_id,
                    component_instance,
                    ..
                } if *component_instance_id == id => {
                    if let Some(pin) = member(component_instance, id, symbol) {
                        epoch = Some(pin);
                    }
                    value = Some(component_instance.clone());
                }
                Operation::DeleteComponentInstance {
                    component_instance_id,
                    ..
                } if *component_instance_id == id => value = None,
                _ => {}
            }
        }
    }
    Some((value, epoch?))
}
fn same_component_semantics(a: &Value, b: &Value, id: Uuid, symbol: Uuid) -> bool {
    let normalize = |value: &Value| -> Option<Value> {
        member(value, id, symbol)?;
        let mut value = value.clone();
        value["object_revision"] = Value::Null;
        for r in value["placed_symbol_refs"].as_array_mut()? {
            r["object_revision"] = Value::Null;
        }
        Some(value)
    };
    normalize(a).is_some_and(|a| normalize(b).as_ref() == Some(&a))
}
fn repin(raw: &mut Value, pins: &std::collections::BTreeMap<Uuid, u64>) -> Option<()> {
    for member in raw["placed_symbol_refs"].as_array_mut()? {
        let symbol: Uuid = serde_json::from_value(member["object_id"].clone()).ok()?;
        member["object_revision"] = json!(pins.get(&symbol)?);
    }
    Some(())
}
fn normalized(target: &TransactionRecord, prior: &[TransactionRecord]) -> Option<Vec<Operation>> {
    let members = family::capture(target, prior)?;
    let index = prior
        .iter()
        .position(|tx| tx.transaction_id == target.transaction_id)?;
    let recorded_after = &prior[..=index];
    let mut ops = target.inverse_operations.clone();
    for op in &mut ops {
        let Some(id) = family::component_id(op) else {
            continue;
        };
        let symbols = &members[&id];
        let symbol = *symbols.iter().next()?;
        let (current, _) = prior_state(prior, id, symbol)?;
        let (historical, _) = prior_state(recorded_after, id, symbol)?;
        let pins:std::collections::BTreeMap<_,_> = symbols.iter().map(|symbol| {
            let epoch = prior_state(prior, id, *symbol)?.1;
            let changed = target.inverse_operations.iter().any(|o| matches!(o, Operation::SetSchematicSymbol { symbol_id,.. } if symbol_id == symbol));
            Some((*symbol, if changed {epoch.checked_add(1)?} else {epoch}))
        }).collect::<Option<_>>()?;
        match op {
            Operation::CreateComponentInstance {
                component_instance, ..
            } => {
                if current.is_some() || historical.is_some() {
                    return None;
                }
                member(component_instance, id, symbol)?;
                component_instance["object_revision"] = json!(0);
                repin(component_instance, &pins)?;
            }
            Operation::SetComponentInstance {
                previous_component_instance,
                component_instance,
                ..
            } => {
                let current = current.as_ref()?;
                if historical.as_ref() != Some(&*previous_component_instance)
                    || !same_component_semantics(previous_component_instance, current, id, symbol)
                {
                    return None;
                }
                *previous_component_instance = current.clone();
                member(component_instance, id, symbol)?;
                component_instance["object_revision"] =
                    json!(current["object_revision"].as_u64()?.checked_add(1)?);
                repin(component_instance, &pins)?;
            }
            Operation::DeleteComponentInstance {
                component_instance, ..
            } => {
                let current = current.as_ref()?;
                if historical.as_ref() != Some(&*component_instance)
                    || !same_component_semantics(component_instance, current, id, symbol)
                {
                    return None;
                }
                *component_instance = current.clone();
            }
            _ => {}
        }
    }
    Some(ops)
}
pub(super) fn prepare(
    model: &DesignModel,
    target: &TransactionRecord,
    batch: &mut OperationBatch,
) -> Result<bool, EngineError> {
    if model.prepare_component_metadata_inverse(target, batch)? {
        return Ok(true);
    }
    if deletion::prepare(model, target, batch)? {
        return Ok(true);
    }
    let Some(members) = family::capture(target, &model.journal) else {
        return Ok(false);
    };
    if !super::journal_history::current_target(&model.journal, target)
        || batch.operations != target.inverse_operations
    {
        return Err(invalid());
    }
    validate_source_preimages(model, target)?;
    for (id, symbols) in members {
        let mut current = None;
        for symbol in symbols {
            let (raw, epoch) = prior_state(&model.journal, id, symbol).ok_or_else(invalid)?;
            if model.objects.get(&symbol).map(|o| o.object_revision.0) != Some(epoch) {
                return Err(invalid());
            }
            current = raw;
        }
        if let Some(current) = current {
            let object = model.objects.get(&id).ok_or_else(invalid)?;
            let shard = model
                .source_shards
                .iter()
                .find(|s| s.shard_id == object.source_shard_id)
                .ok_or_else(invalid)?;
            let source =
                model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
            if source["component_instance"] != current
                || model.component_instances.get(&id).is_none()
            {
                return Err(invalid());
            }
        } else if model.component_instances.contains_key(&id) {
            return Err(invalid());
        }
    }
    batch.operations = normalized(target, &model.journal).ok_or_else(invalid)?;
    Ok(true)
}
fn validate_source_preimages(
    model: &DesignModel,
    target: &TransactionRecord,
) -> Result<(), EngineError> {
    for op in &target.operations {
        let (sheet, id, class, expected) = match op {
            Operation::SetSchematicSymbol {
                sheet_id,
                symbol_id,
                symbol,
            } => (*sheet_id, *symbol_id, "symbols", symbol),
            Operation::CreateSchematicWire {
                sheet_id,
                wire_id,
                wire,
            } => (*sheet_id, *wire_id, "wires", wire),
            _ => continue,
        };
        let object = model.objects.get(&id).ok_or_else(invalid)?;
        let shard = model
            .source_shards
            .iter()
            .find(|s| s.shard_id == object.source_shard_id)
            .ok_or_else(invalid)?;
        let raw = model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
        if raw["uuid"] != json!(sheet) || raw[class][id.to_string()] != *expected {
            return Err(invalid());
        }
    }
    Ok(())
}
fn operation_sheet(op: &Operation) -> Option<Uuid> {
    match op {
        Operation::SetSchematicSymbol { sheet_id, .. }
        | Operation::CreateSchematicWire { sheet_id, .. }
        | Operation::DeleteSchematicWire { sheet_id, .. } => Some(*sheet_id),
        _ => None,
    }
}
fn inverse(ops: &[Operation], target: &TransactionRecord) -> Option<Vec<Operation>> {
    let mut inverse = Vec::new();
    // The recorded inverse encodes canonical source-shard traversal in reverse.
    // Recover that sheet order, retaining input operation order within each sheet.
    // UUID order and the flat input order are not source-shard traversal order.
    let mut sheets = Vec::new();
    for sheet in target
        .inverse_operations
        .iter()
        .rev()
        .filter_map(operation_sheet)
    {
        if !sheets.contains(&sheet) {
            sheets.push(sheet);
        }
    }
    if ops
        .iter()
        .filter_map(operation_sheet)
        .any(|sheet| !sheets.contains(&sheet))
    {
        return None;
    }
    for op in sheets
        .iter()
        .flat_map(|sheet| {
            ops.iter()
                .filter(move |op| operation_sheet(op) == Some(*sheet))
        })
        .chain(ops.iter().filter(|op| operation_sheet(op).is_none()))
    {
        inverse.push(match op {
            Operation::SetSchematicSymbol {
                sheet_id,
                symbol_id,
                ..
            } => target
                .operations
                .iter()
                .find(|op| {
                    matches!(op,
                    Operation::SetSchematicSymbol { sheet_id: sheet, symbol_id: id, .. }
                    if sheet == sheet_id && id == symbol_id)
                })?
                .clone(),
            Operation::CreateSchematicWire {
                sheet_id,
                wire_id,
                wire,
            } => Operation::DeleteSchematicWire {
                sheet_id: *sheet_id,
                wire_id: *wire_id,
                wire: wire.clone(),
            },
            Operation::DeleteSchematicWire {
                sheet_id,
                wire_id,
                wire,
            } => Operation::CreateSchematicWire {
                sheet_id: *sheet_id,
                wire_id: *wire_id,
                wire: wire.clone(),
            },
            Operation::CreateComponentInstance {
                component_instance_id,
                component_instance,
            } => Operation::DeleteComponentInstance {
                component_instance_id: *component_instance_id,
                component_instance: component_instance.clone(),
            },
            Operation::DeleteComponentInstance {
                component_instance_id,
                component_instance,
            } => Operation::CreateComponentInstance {
                component_instance_id: *component_instance_id,
                component_instance: component_instance.clone(),
            },
            Operation::SetComponentInstance {
                component_instance_id,
                previous_component_instance,
                component_instance,
            } => Operation::SetComponentInstance {
                component_instance_id: *component_instance_id,
                previous_component_instance: component_instance.clone(),
                component_instance: previous_component_instance.clone(),
            },
            Operation::SetProposalMetadata {
                proposal_id,
                relative_path,
                previous_proposal,
                proposal,
            } => Operation::SetProposalMetadata {
                proposal_id: *proposal_id,
                relative_path: relative_path.clone(),
                previous_proposal: proposal.clone(),
                proposal: previous_proposal.clone(),
            },
            Operation::LibraryAssets { .. }
                if super::library_asset_journal_ops::binding_history::assignment_family(target) =>
            {
                let mut owned = Vec::new();
                super::journal_operation_hooks::inverse_non_core_operation(op, &mut owned).ok()?;
                if owned.len() != 1 {
                    return None;
                }
                owned.pop()?
            }
            _ => return None,
        });
    }
    inverse.reverse();
    Some(inverse)
}
pub(super) fn matches(
    tx: &TransactionRecord,
    target: &TransactionRecord,
    prior: &[TransactionRecord],
) -> bool {
    DesignModel::component_metadata_inverse_matches(tx, target, prior)
        || deletion::matches(tx, target, prior)
        || (super::journal_history::selected_target(prior, target, tx.transaction_kind)
            && normalized(target, prior).is_some_and(|ops| {
                tx.operations == ops
                    && inverse(&ops, target).as_ref() == Some(&tx.inverse_operations)
            }))
}

#[cfg(test)]
#[path = "schematic_assignment_history_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "schematic_annotation_history_tests.rs"]
mod annotation_tests;
