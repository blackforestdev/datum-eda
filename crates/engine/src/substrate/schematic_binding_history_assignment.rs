//! Existing single-Symbol Part assignment recorded-family classifier.
use super::*;
pub(super) fn capture(target: &TransactionRecord) -> Option<(Uuid, Uuid)> {
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
    for op in target.operations.iter().chain(&target.inverse_operations) {
        let raw = match op {
            Operation::CreateComponentInstance {
                component_instance, ..
            }
            | Operation::DeleteComponentInstance {
                component_instance, ..
            }
            | Operation::SetComponentInstance {
                component_instance, ..
            } => component_instance,
            _ => continue,
        };
        if raw["placed_symbol_refs"]
            .as_array()
            .is_none_or(|r| r.len() != 1 || r[0]["object_id"] != serde_json::json!(pair.0))
        {
            return None;
        }
    }
    (counts == (1, 1)).then_some(pair)
}
