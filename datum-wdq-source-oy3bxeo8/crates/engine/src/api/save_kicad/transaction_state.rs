use std::collections::{BTreeMap, BTreeSet};

use super::*;

pub(super) fn active_deleted_track_uuids(
    undo_stack: &[ImportedSessionUndoRecord],
) -> BTreeSet<uuid::Uuid> {
    let mut deleted = BTreeSet::new();
    for transaction in undo_stack {
        if let ImportedSessionUndoRecord::DeleteTrack { track } = transaction {
            deleted.insert(track.uuid);
        }
    }
    deleted
}

pub(super) fn active_deleted_via_uuids(
    undo_stack: &[ImportedSessionUndoRecord],
) -> BTreeSet<uuid::Uuid> {
    let mut deleted = BTreeSet::new();
    for transaction in undo_stack {
        if let ImportedSessionUndoRecord::DeleteVia { via } = transaction {
            deleted.insert(via.uuid);
        }
    }
    deleted
}

pub(super) fn active_deleted_component_uuids(
    undo_stack: &[ImportedSessionUndoRecord],
) -> BTreeSet<uuid::Uuid> {
    let mut deleted = BTreeSet::new();
    for transaction in undo_stack {
        if let ImportedSessionUndoRecord::DeleteComponent { package, .. } = transaction {
            deleted.insert(package.uuid);
        }
    }
    deleted
}

pub(super) fn active_moved_components(
    undo_stack: &[ImportedSessionUndoRecord],
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut moved = BTreeMap::new();
    for transaction in undo_stack {
        match transaction {
            ImportedSessionUndoRecord::MoveComponent { after, .. }
            | ImportedSessionUndoRecord::RotateComponent { after, .. } => {
                moved.insert(after.uuid, after.clone());
            }
            _ => {}
        }
    }
    moved
}

pub(super) fn active_set_value_components(
    undo_stack: &[ImportedSessionUndoRecord],
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut valued = BTreeMap::new();
    for transaction in undo_stack {
        if let ImportedSessionUndoRecord::SetValue { after, .. } = transaction {
            valued.insert(after.uuid, after.clone());
        }
    }
    valued
}

pub(super) fn active_set_reference_components(
    undo_stack: &[ImportedSessionUndoRecord],
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut referenced = BTreeMap::new();
    for transaction in undo_stack {
        if let ImportedSessionUndoRecord::SetReference { after, .. } = transaction {
            referenced.insert(after.uuid, after.clone());
        }
    }
    referenced
}

pub(super) fn active_assigned_part_components(
    undo_stack: &[ImportedSessionUndoRecord],
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut assigned = BTreeMap::new();
    for transaction in undo_stack {
        if let ImportedSessionUndoRecord::AssignPart { after, .. } = transaction {
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

pub(super) fn active_package_rewritten_components(
    undo_stack: &[ImportedSessionUndoRecord],
) -> BTreeSet<uuid::Uuid> {
    let mut rewritten = BTreeSet::new();
    collect_package_rewritten_components(undo_stack, &mut rewritten);
    rewritten
}

fn collect_package_rewritten_components(
    records: &[ImportedSessionUndoRecord],
    rewritten: &mut BTreeSet<uuid::Uuid>,
) {
    for transaction in records {
        match transaction {
            ImportedSessionUndoRecord::AssignPart { after, .. }
            | ImportedSessionUndoRecord::SetPackage { after, .. } => {
                rewritten.insert(after.uuid);
            }
            ImportedSessionUndoRecord::Batch { records, .. } => {
                collect_package_rewritten_components(records, rewritten);
            }
            _ => {}
        }
    }
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
