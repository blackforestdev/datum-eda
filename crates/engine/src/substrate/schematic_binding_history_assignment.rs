//! Recorded Part assignment family, including complete same-sheet shared owners.
use super::*;
pub(super) fn capture(target: &TransactionRecord) -> Option<(BTreeSet<Uuid>, Uuid)> {
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
