//! Admit only recorded assignment or pose-and-endpoint inverse families.
use super::{Operation, TransactionRecord, Uuid, Value};
use std::collections::BTreeMap;
#[path = "schematic_binding_history_fields.rs"]
mod fields;

pub(super) fn capture(target: &TransactionRecord) -> Option<BTreeMap<Uuid, Uuid>> {
    if let Some((symbol, component)) = assignment(target) {
        return Some(BTreeMap::from([(component, symbol)]));
    }
    transform(target)
}
pub(super) fn component_id(op: &Operation) -> Option<Uuid> {
    match op {
        Operation::CreateComponentInstance {
            component_instance_id,
            ..
        }
        | Operation::SetComponentInstance {
            component_instance_id,
            ..
        }
        | Operation::DeleteComponentInstance {
            component_instance_id,
            ..
        } => Some(*component_instance_id),
        _ => None,
    }
}

#[derive(Default)]
struct PoseOps<'a> {
    sheet: Option<Uuid>,
    symbols: BTreeMap<Uuid, &'a Value>,
    components: BTreeMap<Uuid, (&'a Value, &'a Value)>,
    removed_wires: BTreeMap<Uuid, &'a Value>,
    added_wires: BTreeMap<Uuid, &'a Value>,
}
impl<'a> PoseOps<'a> {
    fn capture(ops: &'a [Operation]) -> Option<Self> {
        let mut out = Self::default();
        for op in ops {
            let sheet = match op {
                Operation::SetSchematicSymbol {
                    sheet_id,
                    symbol_id,
                    symbol,
                } => {
                    if out.symbols.insert(*symbol_id, symbol).is_some() {
                        return None;
                    }
                    Some(*sheet_id)
                }
                Operation::DeleteSchematicWire {
                    sheet_id,
                    wire_id,
                    wire,
                } => {
                    if out.removed_wires.insert(*wire_id, wire).is_some() {
                        return None;
                    }
                    Some(*sheet_id)
                }
                Operation::CreateSchematicWire {
                    sheet_id,
                    wire_id,
                    wire,
                } => {
                    if out.added_wires.insert(*wire_id, wire).is_some() {
                        return None;
                    }
                    Some(*sheet_id)
                }
                Operation::SetComponentInstance {
                    component_instance_id,
                    previous_component_instance,
                    component_instance,
                } => {
                    if out
                        .components
                        .insert(
                            *component_instance_id,
                            (previous_component_instance, component_instance),
                        )
                        .is_some()
                    {
                        return None;
                    }
                    None
                }
                _ => return None,
            };
            if let Some(sheet) = sheet {
                if out.sheet.replace(sheet).is_some_and(|old| old != sheet) {
                    return None;
                }
            }
        }
        (!out.symbols.is_empty() && !out.components.is_empty()).then_some(out)
    }
}
fn transform(target: &TransactionRecord) -> Option<BTreeMap<Uuid, Uuid>> {
    let forward = PoseOps::capture(&target.operations)?;
    let inverse = PoseOps::capture(&target.inverse_operations)?;
    if forward.sheet != inverse.sheet
        || forward.symbols.keys().ne(inverse.symbols.keys())
        || forward.components.keys().ne(inverse.components.keys())
        || forward.removed_wires != inverse.added_wires
        || forward.added_wires != inverse.removed_wires
        || forward.removed_wires.keys().ne(forward.added_wires.keys())
    {
        return None;
    }
    for (id, next) in &forward.symbols {
        if !symbol_pose(inverse.symbols[id], next)
            && !(forward.symbols.len() == 1
                && forward.removed_wires.is_empty()
                && fields::symbol_fields(inverse.symbols[id], next))
        {
            return None;
        }
    }
    for (id, old) in &forward.removed_wires {
        let next = forward.added_wires[id];
        let mut allowed = (*old).clone();
        for key in ["from", "to"] {
            replace_point(&mut allowed[key], &next[key])?;
        }
        if allowed != *next {
            return None;
        }
    }
    let mut owners = BTreeMap::new();
    for (id, (old, next)) in forward.components {
        if inverse.components[&id] != (next, old) {
            return None;
        }
        let refs = old["placed_symbol_refs"].as_array()?;
        if refs.len() != 1 {
            return None;
        }
        let symbol: Uuid = serde_json::from_value(refs[0]["object_id"].clone()).ok()?;
        if !forward.symbols.contains_key(&symbol) || owners.values().any(|owner| *owner == symbol) {
            return None;
        }
        super::member(old, id, symbol)?;
        super::member(next, id, symbol)?;
        let mut allowed = old.clone();
        allowed["object_revision"] = next["object_revision"].clone();
        allowed["placed_symbol_refs"][0]["object_revision"] =
            next["placed_symbol_refs"][0]["object_revision"].clone();
        if allowed != *next {
            return None;
        }
        owners.insert(id, symbol);
    }
    Some(owners)
}
fn replace_point(old: &mut Value, next: &Value) -> Option<()> {
    old.as_object()?;
    next["x"].as_i64()?;
    next["y"].as_i64()?;
    old["x"] = next["x"].clone();
    old["y"] = next["y"].clone();
    Some(())
}
fn symbol_pose(old: &Value, next: &Value) -> bool {
    let check = || -> Option<Value> {
        let mut allowed = old.clone();
        replace_point(&mut allowed["position"], &next["position"])?;
        for key in ["rotation", "mirrored"] {
            if let Some(value) = next.get(key) {
                allowed[key] = value.clone();
            } else {
                allowed.as_object_mut()?.remove(key);
            }
        }
        for key in ["pins", "fields"] {
            match (allowed.get_mut(key), next.get(key)) {
                (None, None) => {}
                (Some(old), Some(next)) => {
                    let (old, next) = (old.as_array_mut()?, next.as_array()?);
                    if old.len() != next.len() {
                        return None;
                    }
                    for (a, b) in old.iter_mut().zip(next) {
                        match (a.get_mut("position"), b.get("position")) {
                            (Some(a), Some(b)) if a.is_object() && b.is_object() => {
                                replace_point(a, b)?
                            }
                            (a, b) if a.as_deref() == b => {}
                            _ => return None,
                        }
                    }
                }
                _ => return None,
            }
        }
        Some(allowed)
    };
    check().as_ref() == Some(next)
}

fn assignment(target: &TransactionRecord) -> Option<(Uuid, Uuid)> {
    let ops = &target.inverse_operations;
    let mut symbol = None;
    let mut component = None;
    for op in ops {
        match op {
            Operation::SetSchematicSymbol {
                sheet_id,
                symbol_id,
                symbol: next,
                ..
            } if symbol.is_none() => {
                let paired: Vec<_> = target
                    .operations
                    .iter()
                    .filter_map(|op| match op {
                        Operation::SetSchematicSymbol {
                            sheet_id: sheet,
                            symbol_id: id,
                            symbol,
                        } if sheet == sheet_id && id == symbol_id => Some(symbol),
                        _ => None,
                    })
                    .collect();
                let [previous_symbol] = paired.as_slice() else {
                    return None;
                };
                let mut allowed = (*previous_symbol).clone();
                for key in ["part", "entity", "gate"] {
                    if let Some(value) = next.get(key) {
                        allowed[key] = value.clone();
                    } else {
                        allowed.as_object_mut()?.remove(key);
                    }
                }
                if allowed != *next {
                    return None;
                }
                symbol = Some(*symbol_id);
            }
            Operation::CreateComponentInstance {
                component_instance_id,
                ..
            }
            | Operation::SetComponentInstance {
                component_instance_id,
                ..
            }
            | Operation::DeleteComponentInstance {
                component_instance_id,
                ..
            } if component.is_none() => component = Some(*component_instance_id),
            Operation::SetProposalMetadata { .. } => {}
            _ => return None,
        }
    }
    let pair = (symbol?, component?);
    let mut counts = (0, 0);
    for op in &target.operations {
        match op {
            Operation::SetSchematicSymbol { symbol_id, .. } if *symbol_id == pair.0 => {
                counts.0 += 1
            }
            Operation::CreateComponentInstance {
                component_instance_id,
                ..
            }
            | Operation::SetComponentInstance {
                component_instance_id,
                ..
            }
            | Operation::DeleteComponentInstance {
                component_instance_id,
                ..
            } if *component_instance_id == pair.1 => counts.1 += 1,
            Operation::SetProposalMetadata { .. } => {}
            _ => return None,
        }
    }
    (counts == (1, 1)).then_some(pair)
}
