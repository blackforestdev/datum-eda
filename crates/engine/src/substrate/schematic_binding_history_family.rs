//! Admit only recorded assignment or pose-and-endpoint inverse families.
use super::{Operation, TransactionRecord, Uuid, Value};
use std::collections::{BTreeMap, BTreeSet};
#[path = "schematic_binding_history_fields.rs"]
mod fields;

#[path = "schematic_binding_history_assignment.rs"]
mod assignment;

pub(super) fn capture(target: &TransactionRecord) -> Option<BTreeMap<Uuid, BTreeSet<Uuid>>> {
    if let Some((symbol, component)) = assignment::capture(target) {
        return Some(BTreeMap::from([(component, BTreeSet::from([symbol]))]));
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
    metadata: Vec<&'a Operation>,
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
                Operation::SetProposalMetadata { .. } => {
                    out.metadata.push(op);
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
fn transform(target: &TransactionRecord) -> Option<BTreeMap<Uuid, BTreeSet<Uuid>>> {
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
    let annotation = forward.removed_wires.is_empty()
        && forward
            .symbols
            .iter()
            .all(|(id, next)| reference_only(inverse.symbols[id], next));
    if !forward.metadata.is_empty() || !inverse.metadata.is_empty() {
        if !annotation || !paired_annotation_metadata(&forward.metadata, &inverse.metadata) {
            return None;
        }
    }
    for (id, next) in &forward.symbols {
        if !annotation
            && !symbol_pose(inverse.symbols[id], next)
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
        let next_refs = next["placed_symbol_refs"].as_array()?;
        if refs.is_empty() || refs.len() != next_refs.len() {
            return None;
        }
        let mut members = BTreeSet::new();
        let mut allowed = old.clone();
        let mut changed = false;
        for (index, reference) in refs.iter().enumerate() {
            let symbol: Uuid = serde_json::from_value(reference["object_id"].clone()).ok()?;
            if !members.insert(symbol)
                || owners
                    .values()
                    .any(|set: &BTreeSet<Uuid>| set.contains(&symbol))
            {
                return None;
            }
            super::member(old, id, symbol)?;
            super::member(next, id, symbol)?;
            if forward.symbols.contains_key(&symbol) {
                changed = true;
                allowed["placed_symbol_refs"][index]["object_revision"] =
                    next_refs[index]["object_revision"].clone();
            }
        }
        // Shared owners retain every member; only the recorded changed members
        // advance, with paired metadata and exact reference-only source above.
        if !changed {
            return None;
        }
        allowed["object_revision"] = next["object_revision"].clone();
        if allowed != *next {
            return None;
        }
        owners.insert(id, members);
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

// Multi-Symbol annotation changes only references; unrelated field/pose edits
// remain subject to their existing exact recorded families.
fn reference_only(old: &Value, next: &Value) -> bool {
    if next["reference"]
        .as_str()
        .is_none_or(|s| s.is_empty() || s.chars().any(|c| c.is_whitespace() || c.is_control()))
    {
        return false;
    }
    let mut expected = old.clone();
    expected["reference"] = next["reference"].clone();
    expected == *next
}

fn paired_annotation_metadata(forward: &[&Operation], inverse: &[&Operation]) -> bool {
    let (
        [
            Operation::SetProposalMetadata {
                proposal_id: a,
                relative_path: ap,
                previous_proposal: old,
                proposal: next,
            },
        ],
        [
            Operation::SetProposalMetadata {
                proposal_id: b,
                relative_path: bp,
                previous_proposal: back_old,
                proposal: back_next,
            },
        ],
    ) = (forward, inverse)
    else {
        return false;
    };
    a == b
        && ap == bp
        && old == back_next
        && next == back_old
        && old["batch"] == next["batch"]
        && old["proposal_id"] == next["proposal_id"]
}
