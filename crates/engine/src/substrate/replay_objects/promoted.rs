//! Authored object registrations and revision diffs for promoted journal history.
use super::super::{DomainObject, ObjectId, ObjectRevision, Operation, TransactionRecord};
use std::collections::BTreeMap;
use uuid::Uuid;

pub(in crate::substrate) fn apply_promoted_journal_object_diffs(
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    journal: &[TransactionRecord],
) {
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
        for object_id in &transaction.diff.modified {
            if let Some(object) = objects.get_mut(object_id) {
                object.object_revision = ObjectRevision(object.object_revision.0 + 1);
            }
        }
        for object_id in &transaction.diff.deleted {
            objects.remove(object_id);
        }
    }
}
