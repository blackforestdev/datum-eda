//! Exact source/owned-component deletion compensation; no semantic rebinding.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

/// Canonical sheet inverses precede metadata derivation, so reversing alone can
/// restore a CI before its Symbol. Move only dependent CI births after sources.
pub(in crate::substrate) fn order_restoration(ops: &mut Vec<Operation>) {
    // Work backwards so each still-unvisited operation keeps its original index.
    // Only move a CI across the exact Symbol births it needs; unrelated trailing
    // operations retain their position relative to the restored binding.
    for index in (0..ops.len()).rev() {
        let Operation::CreateComponentInstance {
            component_instance, ..
        } = &ops[index]
        else {
            continue;
        };
        let required: BTreeSet<Uuid> = component_instance["placed_symbol_refs"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|r| serde_json::from_value(r["object_id"].clone()).ok())
            .collect();
        let last = ops
            .iter()
            .enumerate()
            .filter_map(|(i, op)| match op {
                Operation::CreateSchematicSymbol { symbol_id, .. }
                    if required.contains(symbol_id) =>
                {
                    Some(i)
                }
                _ => None,
            })
            .last();
        if let Some(last) = last.filter(|last| *last > index) {
            let binding = ops.remove(index);
            ops.insert(last, binding);
        }
    }
}

// Return source identity and the exact recorded payload; only existing supported
// deletion classes enter this family. The bool indicates a source birth.
fn source(op: &Operation) -> Option<(Uuid, Uuid, &'static str, &Value, bool)> {
    macro_rules! pair {
        ($($create:ident, $delete:ident, $id:ident, $payload:ident, $class:literal;)*) => {
            match op { $(
                Operation::$create { sheet_id, $id, $payload } => Some((*sheet_id, *$id, $class, $payload, true)),
                Operation::$delete { sheet_id, $id, $payload } => Some((*sheet_id, *$id, $class, $payload, false)),
            )* _ => None }
        }
    }
    pair! {
        CreateSchematicSymbol, DeleteSchematicSymbol, symbol_id, symbol, "symbols";
        CreateSchematicWire, DeleteSchematicWire, wire_id, wire, "wires";
        CreateSchematicLabel, DeleteSchematicLabel, label_id, label, "labels";
        CreateSchematicPort, DeleteSchematicPort, port_id, port, "ports";
        CreateSchematicBus, DeleteSchematicBus, bus_id, bus, "buses";
        CreateSchematicBusEntry, DeleteSchematicBusEntry, bus_entry_id, bus_entry, "bus_entries";
        CreateSchematicJunction, DeleteSchematicJunction, junction_id, junction, "junctions";
        CreateSchematicNoConnect, DeleteSchematicNoConnect, noconnect_id, noconnect, "noconnects";
        CreateSchematicText, DeleteSchematicText, text_id, text, "texts";
    }
}
fn opposite(op: &Operation) -> Option<Operation> {
    if source(op).is_some()
        || matches!(
            op,
            Operation::CreateComponentInstance { .. } | Operation::DeleteComponentInstance { .. }
        )
    {
        let mut raw = serde_json::to_value(op).ok()?;
        let kind = raw["kind"].as_str()?;
        let next = if let Some(tail) = kind.strip_prefix("create_") {
            format!("delete_{tail}")
        } else {
            format!("create_{}", kind.strip_prefix("delete_")?)
        };
        raw["kind"] = json!(next);
        return serde_json::from_value(raw).ok();
    }
    Some(match op {
        Operation::CreateElectricalIdentity { record } => Operation::DeleteElectricalIdentity {
            record: record.clone(),
        },
        Operation::DeleteElectricalIdentity { record } => Operation::CreateElectricalIdentity {
            record: record.clone(),
        },
        Operation::SetElectricalIdentity { previous, record } => Operation::SetElectricalIdentity {
            previous: Box::new(record.clone()),
            record: previous.as_ref().clone(),
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
    })
}
fn inverse(ops: &[Operation]) -> Option<Vec<Operation>> {
    let mut result: Vec<_> = ops
        .iter()
        .filter(|o| source(o).is_some())
        .chain(ops.iter().filter(|o| source(o).is_none()))
        .map(opposite)
        .collect::<Option<_>>()?;
    result.reverse();
    order_restoration(&mut result);
    Some(result)
}
fn capture(target: &TransactionRecord) -> Option<BTreeMap<Uuid, Uuid>> {
    if inverse(&target.operations).as_ref() != Some(&target.inverse_operations) {
        return None;
    }
    let mut symbols = BTreeMap::new();
    let mut sheet = None;
    let mut direction = None;
    let mut ids = BTreeSet::new();
    for op in &target.operations {
        if let Some((owner, id, class, _, create)) = source(op) {
            if sheet.replace(owner).is_some_and(|old| old != owner)
                || !ids.insert(id)
                || direction.replace(create).is_some_and(|old| old != create)
            {
                return None;
            }
            if class == "symbols" {
                symbols.insert(id, create);
            }
        }
    }
    if direction == Some(true)
        && target.transaction_kind == crate::substrate::TransactionKind::Normal
    {
        return None;
    }
    let mut members = BTreeMap::new();
    for op in &target.operations {
        let (id, raw, create) = match op {
            Operation::CreateComponentInstance {
                component_instance_id,
                component_instance,
            } => (*component_instance_id, component_instance, true),
            Operation::DeleteComponentInstance {
                component_instance_id,
                component_instance,
            } => (*component_instance_id, component_instance, false),
            _ => continue,
        };
        let symbol: Uuid =
            serde_json::from_value(raw["placed_symbol_refs"][0]["object_id"].clone()).ok()?;
        super::member(raw, id, symbol)?;
        let parsed: crate::substrate::ComponentInstanceShard =
            serde_json::from_value(json!({"schema_version":1,"component_instance":raw})).ok()?;
        if symbols.get(&symbol) != Some(&create)
            || members.values().any(|s| *s == symbol)
            || parsed
                .component_instance
                .library_bindings
                .values()
                .any(|b| {
                    !matches!(
                        b.binding_role,
                        crate::substrate::LibraryBindingRole::Part
                            | crate::substrate::LibraryBindingRole::Symbol
                    ) || !b.local_override_refs.is_empty()
                })
            || members.insert(id, symbol).is_some()
        {
            return None;
        }
    }
    (!members.is_empty()).then_some(members)
}
fn normalized(target: &TransactionRecord, prior: &[TransactionRecord]) -> Option<Vec<Operation>> {
    let members = capture(target)?;
    let index = prior
        .iter()
        .position(|t| t.transaction_id == target.transaction_id)?;
    let mut ops = target.inverse_operations.clone();
    let births: BTreeMap<_, _> = ops
        .iter()
        .filter_map(|op| match op {
            Operation::CreateSchematicSymbol {
                symbol_id, symbol, ..
            } => Some((
                *symbol_id,
                symbol
                    .get("object_revision")
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
            )),
            _ => None,
        })
        .collect();
    for op in &mut ops {
        match op {
            Operation::CreateComponentInstance {
                component_instance_id,
                component_instance,
            } => {
                let id = *component_instance_id;
                let symbol = members[&id];
                if super::prior_state(prior, id, symbol)?.0.is_some()
                    || super::prior_state(&prior[..=index], id, symbol)?
                        .0
                        .is_some()
                {
                    return None;
                }
                component_instance["object_revision"] = json!(0);
                component_instance["placed_symbol_refs"][0]["object_revision"] =
                    json!(births.get(&symbol)?);
            }
            Operation::DeleteComponentInstance {
                component_instance_id,
                component_instance,
            } => {
                let id = *component_instance_id;
                let symbol = members[&id];
                let historical = super::prior_state(&prior[..=index], id, symbol)?.0?;
                let current = super::prior_state(prior, id, symbol)?.0?;
                if historical != *component_instance
                    || !super::same_component_semantics(component_instance, &current, id, symbol)
                {
                    return None;
                }
                *component_instance = current;
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
    let Some(members) = capture(target) else {
        return Ok(false);
    };
    if !super::super::journal_history::current_target(&model.journal, target)
        || batch.operations != target.inverse_operations
    {
        return Err(invalid());
    }
    let ops = normalized(target, &model.journal).ok_or_else(invalid)?;
    for op in &ops {
        if let Some((sheet, id, class, raw, create)) = source(op) {
            let owner = model.objects.get(&sheet).ok_or_else(invalid)?;
            let shard = model
                .source_shards
                .iter()
                .find(|s| s.shard_id == owner.source_shard_id)
                .ok_or_else(invalid)?;
            let value =
                model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
            let current = value[class].get(id.to_string());
            if value["uuid"] != json!(sheet)
                || if create {
                    current.is_some() || model.objects.contains_key(&id)
                } else {
                    current != Some(raw) || !model.objects.contains_key(&id)
                }
            {
                return Err(invalid());
            }
        }
    }
    for (id, symbol) in members {
        let (current, epoch) =
            super::prior_state(&model.journal, id, symbol).ok_or_else(invalid)?;
        if let Some(current) = current {
            let object = model.objects.get(&id).ok_or_else(invalid)?;
            let shard = model
                .source_shards
                .iter()
                .find(|s| s.shard_id == object.source_shard_id)
                .ok_or_else(invalid)?;
            let raw =
                model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?;
            if raw["component_instance"] != current
                || !model.component_instances.contains_key(&id)
                || model.objects.get(&symbol).map(|o| o.object_revision.0) != Some(epoch)
            {
                return Err(invalid());
            }
        } else if model.objects.contains_key(&id)
            || model.component_instances.contains_key(&id)
            || model.objects.contains_key(&symbol)
        {
            return Err(invalid());
        }
    }
    batch.operations = ops;
    Ok(true)
}
pub(super) fn matches(
    tx: &TransactionRecord,
    target: &TransactionRecord,
    prior: &[TransactionRecord],
) -> bool {
    super::super::journal_history::selected_target(prior, target, tx.transaction_kind)
        && normalized(target, prior).is_some_and(|ops| {
            tx.operations == ops && inverse(&ops).as_ref() == Some(&tx.inverse_operations)
        })
}

#[cfg(test)]
#[path = "schematic_binding_history_delete_tests.rs"]
mod tests;
