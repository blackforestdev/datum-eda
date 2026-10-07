//! Authored object registrations and revision diffs for promoted journal history.
use super::super::{DomainObject, ObjectId, ObjectRevision, Operation, TransactionRecord};
use std::collections::BTreeMap;
use uuid::Uuid;

pub(in crate::substrate) fn apply_promoted_journal_object_diffs(
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    journal: &[TransactionRecord],
) {
    let final_sources = objects.clone();
    for transaction in journal {
        for operation in &transaction.operations {
            if let Operation::CreateBoardGraphic {
                board_id,
                graphic_id,
                ..
            } = operation
                && let Some(board) = objects
                    .get(board_id)
                    .filter(|o| o.domain == "board" && o.kind == "$")
            {
                let shard_id = board.source_shard_id;
                objects.entry(*graphic_id).or_insert_with(|| DomainObject {
                    object_id: *graphic_id,
                    object_revision: ObjectRevision(0),
                    source_shard_id: shard_id,
                    domain: "board".into(),
                    kind: graphic_id.to_string(),
                });
            }
            if let Operation::CreateSchematicSheet {
                sheet_id,
                relative_path,
                ..
            } = operation
            {
                objects.entry(*sheet_id).or_insert_with(|| DomainObject {
                    object_id: *sheet_id,
                    object_revision: ObjectRevision(0),
                    source_shard_id: Uuid::new_v5(
                        &Uuid::NAMESPACE_URL,
                        format!("datum-eda:source-shard:schematic/{relative_path}").as_bytes(),
                    ),
                    domain: "schematic".to_string(),
                    kind: "schematic_sheet".to_string(),
                });
            }
            if let Operation::CreateSchematicDefinition {
                definition_id,
                relative_path,
                ..
            } = operation
            {
                objects
                    .entry(*definition_id)
                    .or_insert_with(|| DomainObject {
                        object_id: *definition_id,
                        object_revision: ObjectRevision(0),
                        source_shard_id: Uuid::new_v5(
                            &Uuid::NAMESPACE_URL,
                            format!("datum-eda:source-shard:schematic/{relative_path}").as_bytes(),
                        ),
                        domain: "schematic".to_string(),
                        kind: "schematic_definition".to_string(),
                    });
            }
            if let Operation::CreatePoolLibraryObject {
                object_id,
                relative_path,
                object_kind,
                ..
            } = operation
            {
                objects.entry(*object_id).or_insert_with(|| DomainObject {
                    object_id: *object_id,
                    object_revision: ObjectRevision(0),
                    source_shard_id: Uuid::new_v5(
                        &Uuid::NAMESPACE_URL,
                        format!("datum-eda:source-shard:{relative_path}").as_bytes(),
                    ),
                    domain: "pool".to_string(),
                    kind: object_kind.clone(),
                });
            }
        }
        for id in &transaction.diff.created {
            if let Some(source) = final_sources.get(id) {
                objects.entry(*id).or_insert_with(|| source.clone());
            }
        }
        restore_component_pad_registrations(objects, transaction);
        for object_id in &transaction.diff.modified {
            if let Some(object) = objects.get_mut(object_id) {
                // A board import can set both outline and stackup. Canonical
                // application advances the root for each operation, while the
                // diff lists each modified object only once. Restore those
                // recorded advances rather than truncating valid history.
                let root_advances = if object.domain == "board" && object.kind == "$" {
                    transaction
                        .operations
                        .iter()
                        .filter(|op| match op {
                            Operation::AdoptBoardSourceProfile { board_id, .. }
                            | Operation::SetBoardOutline { board_id, .. }
                            | Operation::SetBoardStackup { board_id, .. }
                            | Operation::SetBoardName { board_id, .. } => board_id == object_id,
                            Operation::BumpObjectRevision { object_id: id } => id == object_id,
                            _ => false,
                        })
                        .count() as u64
                } else {
                    0
                };
                object.object_revision =
                    ObjectRevision(object.object_revision.0 + root_advances.max(1));
            }
        }
        for object_id in &transaction.diff.deleted {
            objects.remove(object_id);
        }
        restore_library_source_registrations(objects, transaction);
    }
}

fn restore_component_pad_registrations(
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    transaction: &TransactionRecord,
) {
    for op in &transaction.operations {
        let (package, materialized) = match op {
            Operation::CreateBoardPackage {
                package_id,
                materialized,
                ..
            }
            | Operation::SetBoardPackagePackage {
                package_id,
                materialized,
                ..
            } => (*package_id, materialized),
            _ => continue,
        };
        let Some(shard) = objects.get(&package).map(|o| o.source_shard_id) else {
            continue;
        };
        let Some(pads) = materialized
            .get("component_pads")
            .and_then(serde_json::Value::as_array)
        else {
            continue;
        };
        for value in pads {
            if let Ok(pad) = serde_json::from_value::<
                super::super::component_pad_source::ComponentPadSource,
            >(value.clone())
                && (pad.source_binding.is_some() || super::super::component_pad_source::touches(op))
            {
                let registration = DomainObject {
                    object_id: pad.uuid,
                    object_revision: ObjectRevision(0),
                    source_shard_id: shard,
                    domain: "board".into(),
                    kind: package.to_string(),
                };
                if transaction.diff.created.contains(&pad.uuid) {
                    objects.insert(pad.uuid, registration);
                } else {
                    objects.entry(pad.uuid).or_insert(registration);
                }
            }
        }
    }
}
fn restore_library_source_registrations(
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    transaction: &TransactionRecord,
) {
    for op in &transaction.operations {
        let materialized = match op {
            Operation::CreateBoardPackage { materialized, .. }
            | Operation::SetBoardPackagePackage { materialized, .. } => materialized,
            _ => continue,
        };
        if let Some(value)=materialized.get("pad_source_transition") && let Ok(proof)=serde_json::from_value::<crate::api::native_write::board_components::pad_materialization::ComponentPadTransition>(value.clone()) {objects.extend(proof.library_source_registrations);}
    }
}
