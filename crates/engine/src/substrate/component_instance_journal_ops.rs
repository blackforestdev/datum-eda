use std::collections::BTreeMap;
use std::path::Path;

use uuid::Uuid;

use super::{
    COMPONENT_INSTANCE_SHARD_SCHEMA_VERSION, ComponentInstance, EngineError, ObjectId, Operation,
    OperationBatch, SourceShardKind, TransactionRecord,
    component_instance::persisted_component_instance_from_value,
    journal::{StagedShardWrite, stage_new_shard_write},
    operation_application_component_instance::authored_relative_path,
};

pub(super) fn stage_component_instance_operation(
    project_root: &Path,
    batch: &OperationBatch,
    operation: &Operation,
) -> Result<Option<StagedShardWrite>, EngineError> {
    let Some((object_id, value, delete)) = component_instance_operation_write(operation) else {
        return Ok(None);
    };
    let relative_path = authored_relative_path(object_id);
    let destination = project_root.join(&relative_path);
    if delete {
        return Ok(Some(StagedShardWrite {
            destination,
            staged: None,
            kind: SourceShardKind::ComponentInstance,
            relative_path,
            content_hash: String::new(),
            schema_version: None,
            delete: true,
        }));
    }
    let wrapper = wrap_payload(value.clone());
    stage_new_shard_write(
        project_root,
        batch,
        SourceShardKind::ComponentInstance,
        &relative_path,
        &wrapper,
    )
    .map(Some)
}

pub(super) fn maybe_stage_component_instance_operation(
    project_root: &Path,
    batch: &OperationBatch,
    operation: &Operation,
    staged: &mut Vec<StagedShardWrite>,
) -> Result<(), EngineError> {
    if let Some(write) = stage_component_instance_operation(project_root, batch, operation)? {
        staged.push(write);
    }
    Ok(())
}

pub(super) fn inverse_component_instance_operation(
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) {
    match operation {
        Operation::CreateComponentInstance {
            component_instance_id,
            component_instance,
        } => inverse_operations.push(Operation::DeleteComponentInstance {
            component_instance_id: *component_instance_id,
            component_instance: component_instance.clone(),
        }),
        Operation::DeleteComponentInstance {
            component_instance_id,
            component_instance,
        } => inverse_operations.push(Operation::CreateComponentInstance {
            component_instance_id: *component_instance_id,
            component_instance: component_instance.clone(),
        }),
        Operation::SetComponentInstance {
            component_instance_id,
            previous_component_instance,
            component_instance,
        } => inverse_operations.push(Operation::SetComponentInstance {
            component_instance_id: *component_instance_id,
            previous_component_instance: component_instance.clone(),
            component_instance: previous_component_instance.clone(),
        }),
        _ => {}
    }
}

pub(super) fn apply_component_instance_shard_operation(
    shard_kind: &SourceShardKind,
    value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<bool, EngineError> {
    if shard_kind != &SourceShardKind::ComponentInstance {
        return Ok(false);
    }
    let Some((object_id, payload, delete)) = component_instance_operation_write(operation) else {
        return Ok(false);
    };
    let current_id = value
        .get("component_instance")
        .and_then(|entry| entry.get("uuid"))
        .and_then(serde_json::Value::as_str);
    if current_id != Some(object_id.to_string().as_str()) {
        return Ok(false);
    }
    *value = if delete {
        serde_json::Value::Null
    } else {
        wrap_payload(payload.clone())
    };
    Ok(true)
}

pub(super) fn component_instance_operation_write(
    operation: &Operation,
) -> Option<(Uuid, &serde_json::Value, bool)> {
    match operation {
        Operation::CreateComponentInstance {
            component_instance_id,
            component_instance,
        } => Some((*component_instance_id, component_instance, false)),
        Operation::DeleteComponentInstance {
            component_instance_id,
            component_instance,
        } => Some((*component_instance_id, component_instance, true)),
        Operation::SetComponentInstance {
            component_instance_id,
            component_instance,
            ..
        } => Some((*component_instance_id, component_instance, false)),
        _ => None,
    }
}

pub(super) fn apply_component_instance_journal_to_map(
    journal: &[TransactionRecord],
    component_instances: &mut BTreeMap<ObjectId, ComponentInstance>,
) -> Result<(), EngineError> {
    for transaction in journal {
        for operation in &transaction.operations {
            match operation {
                Operation::CreateComponentInstance {
                    component_instance_id,
                    component_instance,
                }
                | Operation::SetComponentInstance {
                    component_instance_id,
                    component_instance,
                    ..
                } => {
                    let component_instance =
                        persisted_component_instance_from_value(component_instance)?;
                    component_instances.insert(*component_instance_id, component_instance);
                }
                Operation::DeleteComponentInstance {
                    component_instance_id,
                    ..
                } => {
                    component_instances.remove(component_instance_id);
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub(super) fn wrap_payload(payload: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "schema_version": COMPONENT_INSTANCE_SHARD_SCHEMA_VERSION,
        "component_instance": payload
    })
}
