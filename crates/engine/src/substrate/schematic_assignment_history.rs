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
    if refs.len() != 1 || refs[0]["object_id"] != json!(symbol) {
        return None;
    }
    refs[0]["object_revision"].as_u64()
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
                    epoch = Some(member(component_instance, id, symbol)?);
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
        value["placed_symbol_refs"][0]["object_revision"] = Value::Null;
        Some(value)
    };
    normalize(a).is_some_and(|a| normalize(b).as_ref() == Some(&a))
}
fn normalized(target: &TransactionRecord, prior: &[TransactionRecord]) -> Option<Vec<Operation>> {
    let members = family::capture(target)?;
    let index = prior
        .iter()
        .position(|tx| tx.transaction_id == target.transaction_id)?;
    let recorded_after = &prior[..=index];
    let mut ops = target.inverse_operations.clone();
    for op in &mut ops {
        let Some(id) = family::component_id(op) else {
            continue;
        };
        let symbol = members[&id];
        let (current, epoch) = prior_state(prior, id, symbol)?;
        let (historical, _) = prior_state(recorded_after, id, symbol)?;
        let next_pin = epoch.checked_add(1)?;
        match op {
            Operation::CreateComponentInstance {
                component_instance, ..
            } => {
                if current.is_some() || historical.is_some() {
                    return None;
                }
                member(component_instance, id, symbol)?;
                component_instance["object_revision"] = json!(0);
                component_instance["placed_symbol_refs"][0]["object_revision"] = json!(next_pin);
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
                component_instance["placed_symbol_refs"][0]["object_revision"] = json!(next_pin);
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
    if deletion::prepare(model, target, batch)? {
        return Ok(true);
    }
    let Some(members) = family::capture(target) else {
        return Ok(false);
    };
    if !super::journal_history::current_target(&model.journal, target)
        || batch.operations != target.inverse_operations
    {
        return Err(invalid());
    }
    validate_source_preimages(model, target)?;
    for (id, symbol) in members {
        let (current, epoch) = prior_state(&model.journal, id, symbol).ok_or_else(invalid)?;
        if model.objects.get(&symbol).map(|o| o.object_revision.0) != Some(epoch) {
            return Err(invalid());
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
fn sheet_operation(op: &Operation) -> bool {
    matches!(
        op,
        Operation::SetSchematicSymbol { .. }
            | Operation::CreateSchematicWire { .. }
            | Operation::DeleteSchematicWire { .. }
    )
}
fn inverse(ops: &[Operation], target: &TransactionRecord) -> Option<Vec<Operation>> {
    let mut inverse = Vec::new();
    // Canonical journal inversion visits source sheets before non-core metadata,
    // then reverses that accumulated list (not simply the input operation order).
    for op in ops
        .iter()
        .filter(|op| sheet_operation(op))
        .chain(ops.iter().filter(|op| !sheet_operation(op)))
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
    deletion::matches(tx, target, prior)
        || (super::journal_history::selected_target(prior, target, tx.transaction_kind)
            && normalized(target, prior).is_some_and(|ops| {
                tx.operations == ops
                    && inverse(&ops, target).as_ref() == Some(&tx.inverse_operations)
            }))
}

#[cfg(test)]
#[path = "schematic_assignment_history_tests.rs"]
mod tests;
