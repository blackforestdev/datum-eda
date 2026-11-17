//! Authenticate occurrence-only library state carried by native component birth
//! or deletion. Asset revision adoption retains its separate history owner.
use crate::{
    library_assets::lock::ComponentBinding,
    substrate::{
        Operation, TransactionRecord, component_instance::PersistedComponentInstance,
        library_asset_operations::LibraryAssetOperation, library_asset_resolver::server_policy,
    },
};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

/// A managed-to-managed assignment may change only the portable Part for the
/// same recorded occurrence keys. This is a positive delta proof, not permission
/// to ignore arbitrary library operations in an applied Proposal receipt.
pub(in crate::substrate) fn assignment_operation(tx: &TransactionRecord) -> Option<&Operation> {
    if tx.operations.iter().any(|op| {
        !matches!(
            op,
            Operation::SetSchematicSymbol { .. }
                | Operation::SetComponentInstance { .. }
                | Operation::SetProposalMetadata { .. }
                | Operation::LibraryAssets { .. }
        )
    }) {
        return None;
    }
    let states: Vec<_> = tx
        .operations
        .iter()
        .filter(|op| matches!(op, Operation::LibraryAssets { .. }))
        .collect();
    let [operation] = states.as_slice() else {
        return None;
    };
    let Operation::LibraryAssets {
        operation:
            LibraryAssetOperation::SetState {
                previous,
                state,
                content,
                ..
            },
    } = operation
    else {
        return None;
    };
    if !content.is_empty() || previous.catalog != state.catalog {
        return None;
    }
    let limit = server_policy().ok()?.metadata_bytes;
    let (mut before, _) = previous.decode(limit).ok()?;
    let (mut after, _) = state.decode(limit).ok()?;
    let old = std::mem::take(&mut before.bindings);
    let new = std::mem::take(&mut after.bindings);
    if before != after || old == new {
        return None;
    }
    let components: Vec<_> = tx
        .operations
        .iter()
        .filter_map(|op| match op {
            Operation::SetComponentInstance {
                component_instance_id,
                previous_component_instance,
                component_instance,
            } => Some((
                *component_instance_id,
                previous_component_instance,
                component_instance,
            )),
            _ => None,
        })
        .collect();
    let [(id, previous_ci, current_ci)] = components.as_slice() else {
        return None;
    };
    let previous_ci: PersistedComponentInstance =
        serde_json::from_value((*previous_ci).clone()).ok()?;
    let current_ci: PersistedComponentInstance =
        serde_json::from_value((*current_ci).clone()).ok()?;
    if previous_ci.uuid != *id
        || current_ci.uuid != *id
        || previous_ci
            .placed_symbol_refs
            .iter()
            .map(|r| r.object_id)
            .collect::<BTreeSet<_>>()
            != current_ci
                .placed_symbol_refs
                .iter()
                .map(|r| r.object_id)
                .collect()
        || previous_ci.placed_symbol_refs.is_empty()
        || previous_ci.placed_package_refs != current_ci.placed_package_refs
    {
        return None;
    }
    let old_part = previous_ci.part_ref.as_ref()?.object_id;
    let new_part = current_ci.part_ref.as_ref()?.object_id;
    if old_part == new_part {
        return None;
    }
    let old_asset = before
        .assets
        .iter()
        .find(|a| a.asset.asset_id == old_part)?;
    let new_asset = before
        .assets
        .iter()
        .find(|a| a.asset.asset_id == new_part)?;
    let rows: Vec<_> = old
        .iter()
        .filter(|r| r.component_instance_id == *id)
        .collect();
    if rows.is_empty() || rows.iter().any(|r| r.component != old_asset.asset) {
        return None;
    }
    let symbols: BTreeSet<_> = current_ci
        .placed_symbol_refs
        .iter()
        .map(|r| r.object_id)
        .collect();
    let supplied: BTreeSet<_> = tx
        .operations
        .iter()
        .filter_map(|op| match op {
            Operation::SetSchematicSymbol {
                symbol_id, symbol, ..
            } if symbol
                .get("part")
                .and_then(serde_json::Value::as_str)
                .and_then(|v| v.parse::<Uuid>().ok())
                == Some(new_part) =>
            {
                Some(*symbol_id)
            }
            _ => None,
        })
        .collect();
    if supplied != symbols {
        return None;
    }
    let mut expected = old;
    for row in &mut expected {
        if row.component_instance_id == *id {
            row.component = new_asset.asset.clone();
        }
    }
    if expected != new {
        return None;
    }
    Some(*operation)
}

pub(in crate::substrate) fn assignment_family(tx: &TransactionRecord) -> bool {
    if assignment_operation(tx).is_none() || !paired_state_inverse(tx) {
        return false;
    }
    let mut inverse = tx.clone();
    inverse.operations = tx.inverse_operations.clone();
    inverse.inverse_operations = tx.operations.clone();
    assignment_operation(&inverse).is_some()
}

fn paired_state_inverse(tx: &TransactionRecord) -> bool {
    let mut expected = Vec::new();
    for op in &tx.operations {
        if matches!(op, Operation::LibraryAssets { .. })
            && crate::substrate::journal_operation_hooks::inverse_non_core_operation(
                op,
                &mut expected,
            )
            .is_err()
        {
            return false;
        }
    }
    let actual: Vec<_> = tx
        .inverse_operations
        .iter()
        .filter(|op| matches!(op, Operation::LibraryAssets { .. }))
        .cloned()
        .collect();
    expected.len() == 1 && expected == actual
}

fn qualified(operations: &[Operation], prior: &[TransactionRecord]) -> Option<()> {
    let states: Vec<_> = operations
        .iter()
        .filter_map(|op| match op {
            Operation::LibraryAssets { operation } => Some(operation),
            _ => None,
        })
        .collect();
    let [
        LibraryAssetOperation::SetState {
            previous,
            state,
            content,
            ..
        },
    ] = states.as_slice()
    else {
        return None;
    };
    if !content.is_empty() || previous.catalog != state.catalog {
        return None;
    }
    let limit = server_policy().ok()?.metadata_bytes;
    let (mut before, _) = previous.decode(limit).ok()?;
    let (mut after, _) = state.decode(limit).ok()?;
    let old = std::mem::take(&mut before.bindings);
    let new = std::mem::take(&mut after.bindings);
    if before != after || old == new {
        return None;
    }
    let mut sheets = BTreeMap::new();
    for op in prior.iter().flat_map(|tx| &tx.operations) {
        match op {
            Operation::CreateSchematicSheet {
                sheet_id,
                relative_path,
                ..
            } => {
                sheets.insert(*sheet_id, format!("schematic/{relative_path}"));
            }
            Operation::DeleteSchematicSheet { sheet_id, .. } => {
                sheets.remove(sheet_id);
            }
            _ => {}
        }
    }
    let mut expected_old = old.clone();
    let mut expected_new = old;
    let mut seen = BTreeSet::new();
    for op in operations {
        let (id, raw, born) = match op {
            Operation::CreateComponentInstance {
                component_instance_id,
                component_instance,
            } => (*component_instance_id, component_instance, true),
            Operation::DeleteComponentInstance {
                component_instance_id,
                component_instance,
            } => (*component_instance_id, component_instance, false),
            // A Set belongs to the assignment/revision semantic owner, never
            // this exact birth/deletion proof.
            Operation::SetComponentInstance { .. } => return None,
            Operation::CreateSchematicSymbol { .. }
            | Operation::DeleteSchematicSymbol { .. }
            | Operation::CreateElectricalIdentity { .. }
            | Operation::SetElectricalIdentity { .. }
            | Operation::DeleteElectricalIdentity { .. }
            | Operation::LibraryAssets { .. } => continue,
            _ => return None,
        };
        if !seen.insert(id) {
            return None;
        }
        let ci: PersistedComponentInstance = serde_json::from_value(raw.clone()).ok()?;
        if ci.uuid != id || !ci.placed_package_refs.is_empty() || ci.placed_symbol_refs.len() != 1 {
            return None;
        }
        let part = ci.part_ref.as_ref()?.object_id;
        let active = before
            .assets
            .iter()
            .find(|asset| asset.asset.asset_id == part)?;
        let symbol_id = ci.placed_symbol_refs[0].object_id;
        let symbols: Vec<_> = operations
            .iter()
            .filter_map(|op| match op {
                Operation::CreateSchematicSymbol {
                    sheet_id,
                    symbol_id: symbol,
                    symbol: raw,
                } if born && *symbol == symbol_id => Some((*sheet_id, raw)),
                Operation::DeleteSchematicSymbol {
                    sheet_id,
                    symbol_id: symbol,
                    symbol: raw,
                } if !born && *symbol == symbol_id => Some((*sheet_id, raw)),
                _ => None,
            })
            .collect();
        let [(sheet, symbol)] = symbols.as_slice() else {
            return None;
        };
        if symbol
            .get("part")
            .and_then(serde_json::Value::as_str)?
            .parse::<Uuid>()
            .ok()?
            != part
        {
            return None;
        }
        let row = ComponentBinding {
            component_instance_id: id,
            source_path: sheets.get(sheet)?.clone(),
            // This strict exact-inverse family is the ordinary unique root
            // sheet placement. Hierarchical assignment retains its own owner.
            instance_path: Vec::new(),
            component: active.asset.clone(),
        };
        if born {
            if expected_new.iter().any(|r| r.component_instance_id == id) {
                return None;
            }
            expected_new.push(row);
        } else {
            if expected_old
                .iter()
                .filter(|r| r.component_instance_id == id)
                .count()
                != 1
                || !expected_old.contains(&row)
            {
                return None;
            }
            expected_new.retain(|r| r.component_instance_id != id);
        }
    }
    if seen.is_empty() {
        return None;
    }
    let sort = |rows: &mut Vec<ComponentBinding>| {
        rows.sort_by(|a, b| {
            (&a.source_path, &a.instance_path, a.component_instance_id).cmp(&(
                &b.source_path,
                &b.instance_path,
                b.component_instance_id,
            ))
        })
    };
    sort(&mut expected_old);
    sort(&mut expected_new);
    (expected_new == new).then_some(())
}

pub(in crate::substrate) fn exact_native_family(
    target: &TransactionRecord,
    prior: &[TransactionRecord],
) -> bool {
    assignment_family(target)
        || paired_state_inverse(target)
            && qualified(&target.operations, prior).is_some()
            && qualified(&target.inverse_operations, prior).is_some()
}
