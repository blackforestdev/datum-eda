use std::path::{Component, Path};

use crate::pool::{
    Entity, Footprint, ModelAttachment, Package, Padstack, Part, PinPadMap, Symbol, Unit,
};

use super::{
    EngineError, Operation, OperationBatch, SourceShardKind,
    journal::{StagedShardWrite, stage_new_shard_write},
};

pub(super) fn maybe_stage_pool_operation(
    project_root: &Path,
    batch: &OperationBatch,
    operation: &Operation,
    staged: &mut Vec<StagedShardWrite>,
) -> Result<(), EngineError> {
    match operation {
        Operation::CreatePoolPackage {
            relative_path,
            package,
            ..
        } => staged.push(stage_new_shard_write(
            project_root,
            batch,
            SourceShardKind::Pool,
            relative_path,
            package,
        )?),
        Operation::DeletePoolPackage { relative_path, .. } => {
            staged.push(delete_pool_shard(project_root, relative_path));
        }
        Operation::CreatePoolPadstack {
            relative_path,
            padstack,
            ..
        } => staged.push(stage_new_shard_write(
            project_root,
            batch,
            SourceShardKind::Pool,
            relative_path,
            padstack,
        )?),
        Operation::DeletePoolPadstack { relative_path, .. } => {
            staged.push(delete_pool_shard(project_root, relative_path));
        }
        Operation::CreatePoolLibraryObject {
            object_id,
            relative_path,
            object_kind,
            object,
        } => {
            validate_pool_library_object(*object_id, relative_path, object_kind, object)?;
            staged.push(stage_new_shard_write(
                project_root,
                batch,
                SourceShardKind::Pool,
                relative_path,
                object,
            )?);
        }
        Operation::SetPoolLibraryObject {
            object_id,
            relative_path,
            object_kind,
            previous_object,
            object,
        } => {
            validate_pool_library_object(*object_id, relative_path, object_kind, previous_object)?;
            validate_pool_library_object(*object_id, relative_path, object_kind, object)?;
            staged.push(stage_new_shard_write(
                project_root,
                batch,
                SourceShardKind::Pool,
                relative_path,
                object,
            )?);
        }
        Operation::AttachPoolPartModel {
            part_id,
            relative_path,
            previous_attachments,
            attachments,
        }
        | Operation::DetachPoolPartModel {
            part_id,
            relative_path,
            previous_attachments,
            attachments,
        } => {
            let part_path = project_root.join(relative_path);
            let mut part = super::read_json_value(&part_path)?;
            apply_pool_part_model_attachments(
                *part_id,
                relative_path,
                &mut part,
                previous_attachments,
                attachments,
            )?;
            staged.push(stage_new_shard_write(
                project_root,
                batch,
                SourceShardKind::Pool,
                relative_path,
                &part,
            )?);
        }
        Operation::DeletePoolLibraryObject {
            object_id,
            relative_path,
            object_kind,
            object,
        } => {
            validate_pool_library_object(*object_id, relative_path, object_kind, object)?;
            staged.push(delete_pool_shard(project_root, relative_path));
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn apply_pool_shard_operation(
    shard_kind: &SourceShardKind,
    value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<bool, EngineError> {
    if shard_kind != &SourceShardKind::Pool {
        return Ok(false);
    }
    match operation {
        Operation::CreatePoolPackage { package, .. } => {
            *value = package.clone();
            Ok(true)
        }
        Operation::CreatePoolPadstack { padstack, .. } => {
            *value = padstack.clone();
            Ok(true)
        }
        Operation::CreatePoolLibraryObject { object, .. } => {
            *value = object.clone();
            Ok(true)
        }
        Operation::SetPoolLibraryObject { object, .. } => {
            *value = object.clone();
            Ok(true)
        }
        Operation::AttachPoolPartModel {
            part_id,
            relative_path,
            previous_attachments,
            attachments,
        }
        | Operation::DetachPoolPartModel {
            part_id,
            relative_path,
            previous_attachments,
            attachments,
        } => {
            apply_pool_part_model_attachments(
                *part_id,
                relative_path,
                value,
                previous_attachments,
                attachments,
            )?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(super) fn reconstruct_pool_shard_value(
    relative_path: &str,
    journal: &[super::TransactionRecord],
) -> Result<serde_json::Value, EngineError> {
    let mut value = None;
    for transaction in journal {
        for operation in &transaction.operations {
            match operation {
                Operation::CreatePoolPackage {
                    relative_path: operation_path,
                    package,
                    ..
                } if operation_path == relative_path => {
                    value = Some(package.clone());
                }
                Operation::CreatePoolPadstack {
                    relative_path: operation_path,
                    padstack,
                    ..
                } if operation_path == relative_path => {
                    value = Some(padstack.clone());
                }
                Operation::CreatePoolLibraryObject {
                    relative_path: operation_path,
                    object,
                    ..
                }
                | Operation::SetPoolLibraryObject {
                    relative_path: operation_path,
                    object,
                    ..
                } if operation_path == relative_path => {
                    value = Some(object.clone());
                }
                Operation::DeletePoolPackage {
                    relative_path: operation_path,
                    ..
                }
                | Operation::DeletePoolPadstack {
                    relative_path: operation_path,
                    ..
                }
                | Operation::DeletePoolLibraryObject {
                    relative_path: operation_path,
                    ..
                } if operation_path == relative_path => {
                    value = None;
                }
                Operation::AttachPoolPartModel {
                    relative_path: operation_path,
                    ..
                }
                | Operation::DetachPoolPartModel {
                    relative_path: operation_path,
                    ..
                } if operation_path == relative_path => {
                    if let Some(current) = &mut value {
                        apply_pool_shard_operation(&SourceShardKind::Pool, current, operation)?;
                    }
                }
                _ => {}
            }
        }
    }
    value.ok_or_else(|| {
        EngineError::Validation(format!(
            "missing pool shard {relative_path} has no journal create record"
        ))
    })
}

pub(super) fn inverse_pool_operation(
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) {
    match operation {
        Operation::CreatePoolPackage {
            package_id,
            relative_path,
            package,
        } => inverse_operations.push(Operation::DeletePoolPackage {
            package_id: *package_id,
            relative_path: relative_path.clone(),
            package: package.clone(),
        }),
        Operation::DeletePoolPackage {
            package_id,
            relative_path,
            package,
        } => inverse_operations.push(Operation::CreatePoolPackage {
            package_id: *package_id,
            relative_path: relative_path.clone(),
            package: package.clone(),
        }),
        Operation::CreatePoolPadstack {
            padstack_id,
            relative_path,
            padstack,
        } => inverse_operations.push(Operation::DeletePoolPadstack {
            padstack_id: *padstack_id,
            relative_path: relative_path.clone(),
            padstack: padstack.clone(),
        }),
        Operation::DeletePoolPadstack {
            padstack_id,
            relative_path,
            padstack,
        } => inverse_operations.push(Operation::CreatePoolPadstack {
            padstack_id: *padstack_id,
            relative_path: relative_path.clone(),
            padstack: padstack.clone(),
        }),
        Operation::CreatePoolLibraryObject {
            object_id,
            relative_path,
            object_kind,
            object,
        } => inverse_operations.push(Operation::DeletePoolLibraryObject {
            object_id: *object_id,
            relative_path: relative_path.clone(),
            object_kind: object_kind.clone(),
            object: object.clone(),
        }),
        Operation::SetPoolLibraryObject {
            object_id,
            relative_path,
            object_kind,
            previous_object,
            object,
        } => inverse_operations.push(Operation::SetPoolLibraryObject {
            object_id: *object_id,
            relative_path: relative_path.clone(),
            object_kind: object_kind.clone(),
            previous_object: object.clone(),
            object: previous_object.clone(),
        }),
        Operation::AttachPoolPartModel {
            part_id,
            relative_path,
            previous_attachments,
            attachments,
        } => inverse_operations.push(Operation::DetachPoolPartModel {
            part_id: *part_id,
            relative_path: relative_path.clone(),
            previous_attachments: attachments.clone(),
            attachments: previous_attachments.clone(),
        }),
        Operation::DetachPoolPartModel {
            part_id,
            relative_path,
            previous_attachments,
            attachments,
        } => inverse_operations.push(Operation::AttachPoolPartModel {
            part_id: *part_id,
            relative_path: relative_path.clone(),
            previous_attachments: attachments.clone(),
            attachments: previous_attachments.clone(),
        }),
        Operation::DeletePoolLibraryObject {
            object_id,
            relative_path,
            object_kind,
            object,
        } => inverse_operations.push(Operation::CreatePoolLibraryObject {
            object_id: *object_id,
            relative_path: relative_path.clone(),
            object_kind: object_kind.clone(),
            object: object.clone(),
        }),
        _ => {}
    }
}

fn apply_pool_part_model_attachments(
    part_id: uuid::Uuid,
    relative_path: &str,
    part: &mut serde_json::Value,
    previous_attachments: &[serde_json::Value],
    attachments: &[serde_json::Value],
) -> Result<(), EngineError> {
    validate_pool_library_object(part_id, relative_path, "parts", part)?;
    for attachment in attachments {
        validate_typed_pool_object::<ModelAttachment>(
            "part behavioural model",
            attachment.clone(),
        )?;
    }
    let document = part.as_object_mut().ok_or_else(|| {
        EngineError::Validation("pool part object must be a JSON object".to_string())
    })?;
    let current = document
        .entry("behavioural_models".to_string())
        .or_insert_with(|| serde_json::Value::Array(Vec::new()));
    let current = current.as_array_mut().ok_or_else(|| {
        EngineError::Validation("pool part behavioural_models field must be an array".to_string())
    })?;
    if current.as_slice() != previous_attachments {
        return Err(EngineError::Validation(
            "pool part behavioural_models precondition mismatch".to_string(),
        ));
    }
    *current = attachments.to_vec();
    validate_pool_library_object(part_id, relative_path, "parts", part)
}

fn delete_pool_shard(project_root: &Path, relative_path: &str) -> StagedShardWrite {
    StagedShardWrite {
        destination: project_root.join(relative_path),
        staged: None,
        kind: SourceShardKind::Pool,
        relative_path: relative_path.to_string(),
        content_hash: String::new(),
        schema_version: None,
        delete: true,
    }
}

fn validate_pool_library_object(
    object_id: uuid::Uuid,
    relative_path: &str,
    object_kind: &str,
    object: &serde_json::Value,
) -> Result<(), EngineError> {
    let Some(document) = object.as_object() else {
        return Err(EngineError::Validation(
            "pool library object must be a JSON object".to_string(),
        ));
    };
    const ALLOWED_KINDS: &[&str] = &[
        "units",
        "symbols",
        "entities",
        "parts",
        "packages",
        "footprints",
        "padstacks",
        "pin_pad_maps",
    ];
    if !ALLOWED_KINDS.contains(&object_kind) {
        return Err(EngineError::Validation(format!(
            "unsupported pool library object kind {object_kind}"
        )));
    }
    let parts: Vec<_> = Path::new(relative_path).components().collect();
    if parts.len() < 3
        || !parts
            .iter()
            .all(|part| matches!(part, Component::Normal(_)))
    {
        return Err(EngineError::Validation(format!(
            "invalid pool library object path {relative_path}"
        )));
    }
    let parent_kind = parts[parts.len() - 2].as_os_str().to_string_lossy();
    if parent_kind != object_kind {
        return Err(EngineError::Validation(format!(
            "pool library object kind {object_kind} does not match path {relative_path}"
        )));
    }
    let filename = parts[parts.len() - 1].as_os_str().to_string_lossy();
    if filename != format!("{object_id}.json") {
        return Err(EngineError::Validation(format!(
            "pool library object path {relative_path} does not match object id {object_id}"
        )));
    }
    let payload_id = object
        .get("uuid")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| EngineError::Validation("pool library object missing uuid".to_string()))?;
    let payload_id = uuid::Uuid::parse_str(payload_id)
        .map_err(|error| EngineError::Validation(format!("invalid pool library uuid: {error}")))?;
    if payload_id != object_id {
        return Err(EngineError::Validation(format!(
            "pool library object payload uuid {payload_id} does not match object id {object_id}"
        )));
    }
    match object
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
    {
        Some(1) => {}
        Some(version) => {
            return Err(EngineError::Validation(format!(
                "unsupported pool library object schema_version {version}; expected 1"
            )));
        }
        None => {
            return Err(EngineError::Validation(
                "pool library object missing schema_version".to_string(),
            ));
        }
    }
    validate_pool_library_object_shape(object_kind, document)?;
    Ok(())
}

fn validate_pool_library_object_shape(
    object_kind: &str,
    object: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), EngineError> {
    let value = serde_json::Value::Object(object.clone());
    match object_kind {
        "units" => validate_typed_pool_object::<Unit>(object_kind, value),
        "symbols" => validate_typed_pool_object::<Symbol>(object_kind, value),
        "entities" => validate_typed_pool_object::<Entity>(object_kind, value),
        "parts" => validate_typed_pool_object::<Part>(object_kind, value),
        "packages" => validate_typed_pool_object::<Package>(object_kind, value),
        "footprints" => validate_typed_pool_object::<Footprint>(object_kind, value),
        "padstacks" => validate_typed_pool_object::<Padstack>(object_kind, value),
        "pin_pad_maps" => validate_typed_pool_object::<PinPadMap>(object_kind, value),
        _ => Ok(()),
    }
}

fn validate_typed_pool_object<T>(
    object_kind: &str,
    value: serde_json::Value,
) -> Result<(), EngineError>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value::<T>(value)
        .map(|_| ())
        .map_err(|error| {
            EngineError::Validation(format!(
                "invalid pool library {object_kind} object: {error}"
            ))
        })
}
