use std::collections::{BTreeMap, BTreeSet};

use super::*;

pub(super) fn active_deleted_track_uuids(undo_stack: &[TransactionRecord]) -> BTreeSet<uuid::Uuid> {
    let mut deleted = BTreeSet::new();
    for transaction in undo_stack {
        if let TransactionRecord::DeleteTrack { track } = transaction {
            deleted.insert(track.uuid);
        }
    }
    deleted
}

pub(super) fn active_deleted_via_uuids(undo_stack: &[TransactionRecord]) -> BTreeSet<uuid::Uuid> {
    let mut deleted = BTreeSet::new();
    for transaction in undo_stack {
        if let TransactionRecord::DeleteVia { via } = transaction {
            deleted.insert(via.uuid);
        }
    }
    deleted
}

pub(super) fn active_deleted_component_uuids(
    undo_stack: &[TransactionRecord],
) -> BTreeSet<uuid::Uuid> {
    let mut deleted = BTreeSet::new();
    for transaction in undo_stack {
        if let TransactionRecord::DeleteComponent { package, .. } = transaction {
            deleted.insert(package.uuid);
        }
    }
    deleted
}

pub(super) fn active_moved_components(
    undo_stack: &[TransactionRecord],
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut moved = BTreeMap::new();
    for transaction in undo_stack {
        match transaction {
            TransactionRecord::MoveComponent { after, .. }
            | TransactionRecord::RotateComponent { after, .. } => {
                moved.insert(after.uuid, after.clone());
            }
            _ => {}
        }
    }
    moved
}

pub(super) fn active_set_value_components(
    undo_stack: &[TransactionRecord],
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut valued = BTreeMap::new();
    for transaction in undo_stack {
        if let TransactionRecord::SetValue { after, .. } = transaction {
            valued.insert(after.uuid, after.clone());
        }
    }
    valued
}

pub(super) fn active_set_reference_components(
    undo_stack: &[TransactionRecord],
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut referenced = BTreeMap::new();
    for transaction in undo_stack {
        if let TransactionRecord::SetReference { after, .. } = transaction {
            referenced.insert(after.uuid, after.clone());
        }
    }
    referenced
}

pub(super) fn active_assigned_part_components(
    undo_stack: &[TransactionRecord],
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut assigned = BTreeMap::new();
    for transaction in undo_stack {
        if let TransactionRecord::AssignPart { after, .. } = transaction {
            assigned.insert(after.uuid, after.clone());
        }
    }
    assigned
}

pub(super) fn merge_component_value_overrides(
    valued_components: &BTreeMap<uuid::Uuid, crate::board::PlacedPackage>,
    assigned_components: &BTreeMap<uuid::Uuid, crate::board::PlacedPackage>,
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut merged = valued_components.clone();
    for (uuid, package) in assigned_components {
        merged.insert(*uuid, package.clone());
    }
    merged
}

pub(super) fn active_package_rewritten_components(board: &Board) -> BTreeSet<uuid::Uuid> {
    board
        .packages
        .values()
        .filter(|package| package.package != uuid::Uuid::nil())
        .map(|package| package.uuid)
        .collect()
}

pub(super) fn filter_component_map(
    components: &BTreeMap<uuid::Uuid, crate::board::PlacedPackage>,
    exclude: &BTreeSet<uuid::Uuid>,
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    components
        .iter()
        .filter(|(uuid, _)| !exclude.contains(uuid))
        .map(|(uuid, package)| (*uuid, package.clone()))
        .collect()
}
