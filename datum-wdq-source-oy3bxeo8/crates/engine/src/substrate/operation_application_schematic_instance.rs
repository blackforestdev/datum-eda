use super::{CommitDiff, DesignModel, DomainObject, EngineError, ObjectRevision, Operation};

pub(super) fn apply_schematic_instance_operation(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    operation: &Operation,
) -> Result<bool, EngineError> {
    match operation {
        Operation::CreateSchematicSheetInstance {
            schematic_id,
            instance_id,
            ..
        } => {
            if model.objects.contains_key(instance_id) {
                return Err(EngineError::Validation(format!(
                    "schematic sheet instance {instance_id} already exists"
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
            let root_shard_id = root.source_shard_id;
            diff.modified.push(*schematic_id);
            model.objects.insert(
                *instance_id,
                DomainObject {
                    object_id: *instance_id,
                    object_revision: ObjectRevision(0),
                    source_shard_id: root_shard_id,
                    domain: "schematic".to_string(),
                    kind: "schematic_sheet_instance".to_string(),
                },
            );
            diff.created.push(*instance_id);
            Ok(true)
        }
        Operation::DeleteSchematicSheetInstance {
            schematic_id,
            instance_id,
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
            if model.objects.remove(instance_id).is_some() {
                diff.deleted.push(*instance_id);
            }
            Ok(true)
        }
        Operation::SetSchematicSheetInstance {
            schematic_id,
            instance_id,
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
            let root_shard_id = root.source_shard_id;
            diff.modified.push(*schematic_id);
            if let Some(instance) = model.objects.get_mut(instance_id) {
                instance.object_revision = ObjectRevision(instance.object_revision.0 + 1);
                diff.modified.push(*instance_id);
            } else {
                model.objects.insert(
                    *instance_id,
                    DomainObject {
                        object_id: *instance_id,
                        object_revision: ObjectRevision(1),
                        source_shard_id: root_shard_id,
                        domain: "schematic".to_string(),
                        kind: "schematic_sheet_instance".to_string(),
                    },
                );
                diff.created.push(*instance_id);
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}
