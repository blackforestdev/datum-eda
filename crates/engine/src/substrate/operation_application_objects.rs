use std::collections::BTreeMap;

use super::{
    CommitDiff, DomainObject, EngineError, ObjectId, ObjectRevision, Operation,
    operation_application_object_revision::bump_existing_object,
};

pub(super) fn apply_operation_to_objects(
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    operation: &Operation,
    diff: Option<&mut CommitDiff>,
) -> Result<(), EngineError> {
    match operation {
        Operation::BumpObjectRevision { object_id } => {
            let object = objects.get_mut(object_id).ok_or(EngineError::NotFound {
                object_type: "domain_object",
                uuid: *object_id,
            })?;
            object.object_revision = ObjectRevision(object.object_revision.0 + 1);
            if let Some(diff) = diff {
                diff.modified.push(*object_id);
            }
            Ok(())
        }
        Operation::SetProjectName { project_id, .. } => {
            bump_existing_object(objects, *project_id, diff)
        }
        Operation::SetProjectRules { .. }
        | Operation::AddProjectPoolRef { .. }
        | Operation::DeleteProjectPoolRef { .. } => Ok(()),
        Operation::CreateProjectRule { rules_root_id, .. }
        | Operation::SetProjectRule { rules_root_id, .. }
        | Operation::DeleteProjectRule { rules_root_id, .. } => {
            bump_existing_object(objects, *rules_root_id, diff)
        }
        Operation::CreatePoolPackage {
            package_id,
            relative_path,
            package,
        } => create_pool_object(
            objects,
            diff,
            *package_id,
            relative_path,
            package,
            "packages",
        ),
        Operation::DeletePoolPackage { package_id, .. } => {
            delete_object(objects, diff, *package_id)
        }
        Operation::CreatePoolPadstack {
            padstack_id,
            relative_path,
            padstack,
        } => create_pool_object(
            objects,
            diff,
            *padstack_id,
            relative_path,
            padstack,
            "padstacks",
        ),
        Operation::DeletePoolPadstack { padstack_id, .. } => {
            delete_object(objects, diff, *padstack_id)
        }
        Operation::CreatePoolLibraryObject {
            object_id,
            relative_path,
            object_kind,
            object,
        } => create_pool_object(
            objects,
            diff,
            *object_id,
            relative_path,
            object,
            object_kind,
        ),
        Operation::SetPoolLibraryObject { object_id, .. } => {
            bump_existing_object(objects, *object_id, diff)
        }
        Operation::AttachPoolPartModel { part_id, .. }
        | Operation::DetachPoolPartModel { part_id, .. } => {
            bump_existing_object(objects, *part_id, diff)
        }
        Operation::DeletePoolLibraryObject { object_id, .. } => {
            delete_object(objects, diff, *object_id)
        }
        Operation::CreateBoardPackage { .. }
        | Operation::DeleteBoardPackage { .. }
        | Operation::CreateBoardPad { .. }
        | Operation::DeleteBoardPad { .. }
        | Operation::CreateBoardTrack { .. }
        | Operation::DeleteBoardTrack { .. }
        | Operation::CreateBoardVia { .. }
        | Operation::DeleteBoardVia { .. }
        | Operation::CreateBoardZone { .. }
        | Operation::SetBoardZone { .. }
        | Operation::DeleteBoardZone { .. }
        | Operation::CreateBoardNet { .. }
        | Operation::DeleteBoardNet { .. } => Ok(()),
        Operation::SetBoardTrack { track_id, .. } => bump_existing_object(objects, *track_id, diff),
        Operation::SetBoardVia { via_id, .. } => bump_existing_object(objects, *via_id, diff),
        Operation::SetBoardNet { net_id, .. } => bump_existing_object(objects, *net_id, diff),
        Operation::CreateBoardNetClass { .. } | Operation::DeleteBoardNetClass { .. } => Ok(()),
        Operation::SetBoardNetClass {
            net_class_id: object_id,
            ..
        }
        | Operation::SetBoardDimension {
            dimension_id: object_id,
            ..
        } => bump_existing_object(objects, *object_id, diff),
        Operation::CreateBoardDimension { .. } | Operation::DeleteBoardDimension { .. } => Ok(()),
        Operation::CreateBoardText { .. } | Operation::DeleteBoardText { .. } => Ok(()),
        Operation::SetBoardText { text_id, .. } => bump_existing_object(objects, *text_id, diff),
        Operation::CreateBoardKeepout { .. } | Operation::DeleteBoardKeepout { .. } => Ok(()),
        Operation::CreateRelationship { .. }
        | Operation::DeleteRelationship { .. }
        | Operation::SetRelationship { .. }
        | Operation::CreateVariantOverlay { .. }
        | Operation::DeleteVariantOverlay { .. }
        | Operation::SetVariantOverlay { .. }
        | Operation::CreateComponentInstance { .. }
        | Operation::DeleteComponentInstance { .. }
        | Operation::SetComponentInstance { .. }
        | Operation::SetZoneFill { .. }
        | Operation::DeleteZoneFill { .. } => Ok(()),
        Operation::CreateSchematicWire { .. }
        | Operation::DeleteSchematicWire { .. }
        | Operation::CreateSchematicJunction { .. }
        | Operation::DeleteSchematicJunction { .. }
        | Operation::CreateSchematicNoConnect { .. }
        | Operation::DeleteSchematicNoConnect { .. }
        | Operation::CreateSchematicSheet { .. }
        | Operation::DeleteSchematicSheet { .. }
        | Operation::CreateSchematicDefinition { .. }
        | Operation::DeleteSchematicDefinition { .. }
        | Operation::CreateSchematicSheetInstance { .. }
        | Operation::DeleteSchematicSheetInstance { .. }
        | Operation::SetSchematicSheetInstance { .. }
        | Operation::CreateSchematicWaiver { .. }
        | Operation::DeleteSchematicWaiver { .. }
        | Operation::CreateSchematicDeviation { .. }
        | Operation::DeleteSchematicDeviation { .. } => Ok(()),
        Operation::CreateSchematicLabel { .. } | Operation::DeleteSchematicLabel { .. } => Ok(()),
        Operation::CreateSchematicPort { .. } | Operation::DeleteSchematicPort { .. } => Ok(()),
        Operation::CreateSchematicBus { .. } | Operation::DeleteSchematicBus { .. } => Ok(()),
        Operation::CreateSchematicBusEntry { .. } | Operation::DeleteSchematicBusEntry { .. } => {
            Ok(())
        }
        Operation::CreateSchematicText { .. } | Operation::DeleteSchematicText { .. } => Ok(()),
        Operation::CreateSchematicDrawing { .. } | Operation::DeleteSchematicDrawing { .. } => {
            Ok(())
        }
        Operation::CreateSchematicSymbol { .. }
        | Operation::SetSchematicSymbol { .. }
        | Operation::DeleteSchematicSymbol { .. } => Ok(()),
        Operation::SetSchematicLabel {
            label_id: object_id,
            ..
        }
        | Operation::SetSchematicSheetName {
            sheet_id: object_id,
            ..
        }
        | Operation::SetSchematicPort {
            port_id: object_id, ..
        }
        | Operation::SetSchematicBus {
            bus_id: object_id, ..
        }
        | Operation::SetSchematicText {
            text_id: object_id, ..
        }
        | Operation::SetSchematicDrawing {
            drawing_id: object_id,
            ..
        } => bump_existing_object(objects, *object_id, diff),
        Operation::SetBoardKeepout {
            keepout_id: object_id,
            ..
        }
        | Operation::SetBoardOutline {
            board_id: object_id,
            ..
        }
        | Operation::SetBoardStackup {
            board_id: object_id,
            ..
        }
        | Operation::SetBoardName {
            board_id: object_id,
            ..
        }
        | Operation::SetBoardPad {
            pad_id: object_id, ..
        } => bump_existing_object(objects, *object_id, diff),
        Operation::SetBoardPackagePart { package_id, .. }
        | Operation::SetBoardPackagePackage { package_id, .. }
        | Operation::SetBoardPackageValue { package_id, .. }
        | Operation::SetBoardPackageReference { package_id, .. }
        | Operation::SetBoardPackagePosition { package_id, .. }
        | Operation::SetBoardPackageLayer { package_id, .. }
        | Operation::SetComponentSide { package_id, .. }
        | Operation::SetBoardPackageRotation { package_id, .. }
        | Operation::SetBoardPackageLocked { package_id, .. } => {
            bump_existing_object(objects, *package_id, diff)
        }
        _ => Ok(()),
    }
}

fn create_pool_object(
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    diff: Option<&mut CommitDiff>,
    object_id: ObjectId,
    relative_path: &str,
    value: &serde_json::Value,
    kind: &str,
) -> Result<(), EngineError> {
    if objects.contains_key(&object_id) {
        return Err(EngineError::Validation(format!(
            "pool object {object_id} already exists"
        )));
    }
    let shard_id = uuid::Uuid::new_v5(
        &uuid::Uuid::NAMESPACE_URL,
        format!("datum-eda:source-shard:{relative_path}").as_bytes(),
    );
    objects.insert(
        object_id,
        DomainObject {
            object_id,
            object_revision: ObjectRevision(
                value
                    .get("object_revision")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
            ),
            source_shard_id: shard_id,
            domain: "pool".to_string(),
            kind: kind.to_string(),
        },
    );
    if let Some(diff) = diff {
        diff.created.push(object_id);
    }
    Ok(())
}

fn delete_object(
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    diff: Option<&mut CommitDiff>,
    object_id: ObjectId,
) -> Result<(), EngineError> {
    if objects.remove(&object_id).is_some() {
        if let Some(diff) = diff {
            diff.deleted.push(object_id);
        }
        Ok(())
    } else {
        Err(EngineError::NotFound {
            object_type: "domain_object",
            uuid: object_id,
        })
    }
}
