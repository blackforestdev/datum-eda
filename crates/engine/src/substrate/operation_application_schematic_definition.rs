use super::{CommitDiff, DesignModel, DomainObject, EngineError, ObjectRevision, Operation};

pub(super) fn apply_schematic_definition_operation(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    operation: &Operation,
) -> Result<bool, EngineError> {
    match operation {
        Operation::CreateSchematicDefinition {
            schematic_id,
            definition_id,
            relative_path,
            ..
        } => {
            if model.objects.contains_key(definition_id) {
                return Err(EngineError::Validation(format!(
                    "schematic definition {definition_id} already exists"
                )));
            }
            let root = model
                .objects
                .get_mut(schematic_id)
                .ok_or(EngineError::NotFound {
                    object_type: "schematic_root",
                    uuid: *schematic_id,
                })?;
            root.object_revision = ObjectRevision(root.object_revision.0 + 1);
            diff.modified.push(*schematic_id);
            let shard_id = uuid::Uuid::new_v5(
                &uuid::Uuid::NAMESPACE_URL,
                format!("datum-eda:source-shard:schematic/{relative_path}").as_bytes(),
            );
            model.objects.insert(
                *definition_id,
                DomainObject {
                    object_id: *definition_id,
                    object_revision: ObjectRevision(0),
                    source_shard_id: shard_id,
                    domain: "schematic".to_string(),
                    kind: "schematic_definition".to_string(),
                },
            );
            diff.created.push(*definition_id);
            Ok(true)
        }
        Operation::DeleteSchematicDefinition {
            schematic_id,
            definition_id,
            ..
        } => {
            let root = model
                .objects
                .get_mut(schematic_id)
                .ok_or(EngineError::NotFound {
                    object_type: "schematic_root",
                    uuid: *schematic_id,
                })?;
            root.object_revision = ObjectRevision(root.object_revision.0 + 1);
            diff.modified.push(*schematic_id);
            if model.objects.remove(definition_id).is_some() {
                diff.deleted.push(*definition_id);
                Ok(true)
            } else {
                Err(EngineError::NotFound {
                    object_type: "schematic_definition",
                    uuid: *definition_id,
                })
            }
        }
        _ => Ok(false),
    }
}
