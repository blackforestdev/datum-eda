//! Recorded Part assignment family, including complete same-sheet shared owners.
use super::*;
fn symbol_image(ops: &[Operation], sheet: Uuid, symbol: Uuid) -> Option<&Value> {
    let rows: Vec<_> = ops
        .iter()
        .filter_map(|op| match op {
            Operation::SetSchematicSymbol {
                sheet_id,
                symbol_id,
                symbol: raw,
            } if *sheet_id == sheet && *symbol_id == symbol => Some(raw),
            _ => None,
        })
        .collect();
    match rows.as_slice() {
        [raw] => Some(*raw),
        _ => None,
    }
}

/// Only the exact value images from a guarded, applied component assignment
/// can extend the legacy family. Canonical compensation resolves its normal
/// origin first; it never turns an arbitrary value edit into an assignment.
fn recorded_value_change(
    target: &TransactionRecord,
    prior: &[TransactionRecord],
    sheet: Uuid,
    symbol: Uuid,
    forward: &Value,
    inverse: &Value,
) -> Option<()> {
    use crate::substrate::{TransactionKind, component_instance};
    let origin = component_instance::assignment_origin(target, prior)?;
    let receipt = component_instance::assignment_receipt(origin, symbol).ok()?;
    let after = symbol_image(&origin.operations, sheet, symbol)?;
    let before = symbol_image(&origin.inverse_operations, sheet, symbol)?;
    let pair_matches = if target.transaction_kind == TransactionKind::Undo {
        forward == before && inverse == after
    } else {
        forward == after && inverse == before
    };
    if !pair_matches || !after["value"].is_string() || !before["value"].is_string() {
        return None;
    }
    let part: Uuid = serde_json::from_value(after["part"].clone()).ok()?;
    let revisions: Vec<_> = receipt
        .batch
        .operations
        .iter()
        .filter_map(|op| match op {
            Operation::GuardObjectRevision {
                object_id,
                expected_object_revision,
            } if *object_id == part => Some(expected_object_revision.0),
            _ => None,
        })
        .collect();
    let [revision] = revisions.as_slice() else {
        return None;
    };
    let owners: Vec<_> = origin
        .operations
        .iter()
        .filter_map(|op| match op {
            Operation::CreateComponentInstance {
                component_instance, ..
            }
            | Operation::SetComponentInstance {
                component_instance, ..
            } => Some(component_instance),
            _ => None,
        })
        .collect();
    let [owner] = owners.as_slice() else {
        return None;
    };
    if owner["part_ref"]["object_id"] != serde_json::json!(part)
        || owner["part_ref"]["object_revision"] != serde_json::json!(revision)
    {
        return None;
    }
    let bindings: Vec<_> = owner["library_bindings"]
        .as_object()?
        .values()
        .filter(|binding| binding["binding_role"] == "part")
        .collect();
    let [binding] = bindings.as_slice() else {
        return None;
    };
    if binding["target_object_id"] != serde_json::json!(part)
        || binding["pinned_object_revision"] != serde_json::json!(revision)
    {
        return None;
    }
    Some(())
}
pub(super) fn capture(
    target: &TransactionRecord,
    prior: &[TransactionRecord],
) -> Option<(BTreeSet<Uuid>, Uuid)> {
    let derived =
        crate::substrate::library_asset_journal_ops::binding_history::assignment_family(target);
    let ops = &target.inverse_operations;
    let shared = ops
        .iter()
        .filter(|op| matches!(op, Operation::SetSchematicSymbol { .. }))
        .count()
        > 1;
    let mut symbols = BTreeSet::new();
    let mut sheet = None;
    let mut component = None;
    for op in ops {
        match op {
            Operation::SetSchematicSymbol {
                sheet_id,
                symbol_id,
                symbol: next,
            } => {
                if !symbols.insert(*symbol_id)
                    || sheet.replace(*sheet_id).is_some_and(|old| old != *sheet_id)
                {
                    return None;
                }
                let paired: Vec<_> = target
                    .operations
                    .iter()
                    .filter_map(|op| match op {
                        Operation::SetSchematicSymbol {
                            sheet_id: owner,
                            symbol_id: id,
                            symbol,
                        } if owner == sheet_id && id == symbol_id => Some(symbol),
                        _ => None,
                    })
                    .collect();
                let [previous] = paired.as_slice() else {
                    return None;
                };
                let mut allowed = (*previous).clone();
                let fields: &[&str] = if shared {
                    &["part"]
                } else {
                    &["part", "entity", "gate"]
                };
                for key in fields {
                    if let Some(value) = next.get(*key) {
                        allowed[*key] = value.clone();
                    } else {
                        allowed.as_object_mut()?.remove(*key);
                    }
                }
                if allowed["value"] != next["value"] {
                    recorded_value_change(target, prior, *sheet_id, *symbol_id, previous, next)?;
                    allowed["value"] = next["value"].clone();
                }
                if allowed != *next {
                    return None;
                }
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
            Operation::LibraryAssets { .. } if derived => {}
            _ => return None,
        }
    }
    let component = component?;
    if symbols.is_empty() {
        return None;
    }
    let mut forward = BTreeSet::new();
    let mut owners = 0;
    for op in &target.operations {
        match op {
            Operation::SetSchematicSymbol {
                sheet_id,
                symbol_id,
                ..
            } if Some(*sheet_id) == sheet
                && symbols.contains(symbol_id)
                && forward.insert(*symbol_id) => {}
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
            } if *component_instance_id == component => owners += 1,
            Operation::SetProposalMetadata { .. } => {}
            Operation::LibraryAssets { .. } if derived => {}
            _ => return None,
        }
    }
    if forward != symbols || owners != 1 {
        return None;
    }
    for op in target.operations.iter().chain(ops) {
        let payloads = match op {
            Operation::CreateComponentInstance {
                component_instance, ..
            }
            | Operation::DeleteComponentInstance {
                component_instance, ..
            } if !shared => vec![component_instance],
            Operation::CreateComponentInstance { .. }
            | Operation::DeleteComponentInstance { .. } => return None,
            Operation::SetComponentInstance {
                component_instance,
                previous_component_instance,
                ..
            } => vec![component_instance, previous_component_instance],
            _ => continue,
        };
        for raw in payloads {
            let refs = raw["placed_symbol_refs"].as_array()?;
            let ids = refs
                .iter()
                .map(|r| serde_json::from_value::<Uuid>(r["object_id"].clone()).ok())
                .collect::<Option<BTreeSet<_>>>()?;
            if refs.len() != symbols.len() || ids != symbols {
                return None;
            }
        }
    }
    Some((symbols, component))
}

#[cfg(test)]
#[path = "schematic_binding_history_assignment_tests.rs"]
mod tests;
