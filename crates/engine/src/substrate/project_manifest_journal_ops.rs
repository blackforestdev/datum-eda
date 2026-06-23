use super::{EngineError, Operation};

pub(super) fn apply_project_manifest_operation(
    manifest_value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<Option<bool>, EngineError> {
    match operation {
        Operation::SetProjectName { name, .. } => {
            set_project_manifest_field(
                manifest_value,
                "name",
                serde_json::Value::String(name.clone()),
            )?;
            Ok(Some(true))
        }
        Operation::AddProjectPoolRef { path, priority } => {
            add_project_pool_ref(manifest_value, path, *priority)?;
            Ok(Some(true))
        }
        Operation::DeleteProjectPoolRef { path, .. } => {
            delete_project_pool_ref(manifest_value, path)?;
            Ok(Some(true))
        }
        _ => Ok(None),
    }
}

pub(super) fn inverse_project_manifest_operation(
    manifest_value: &mut serde_json::Value,
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) -> Result<bool, EngineError> {
    match operation {
        Operation::SetProjectName { project_id, name } => {
            let previous = project_manifest_field(manifest_value, "name")?
                .as_str()
                .ok_or_else(|| {
                    EngineError::Validation(
                        "project manifest name field is not a string".to_string(),
                    )
                })?
                .to_string();
            inverse_operations.push(Operation::SetProjectName {
                project_id: *project_id,
                name: previous,
            });
            set_project_manifest_field(
                manifest_value,
                "name",
                serde_json::Value::String(name.clone()),
            )?;
            Ok(true)
        }
        Operation::AddProjectPoolRef { path, priority } => {
            inverse_operations.push(Operation::DeleteProjectPoolRef {
                path: path.clone(),
                priority: *priority,
            });
            add_project_pool_ref(manifest_value, path, *priority)?;
            Ok(true)
        }
        Operation::DeleteProjectPoolRef { path, priority } => {
            inverse_operations.push(Operation::AddProjectPoolRef {
                path: path.clone(),
                priority: *priority,
            });
            delete_project_pool_ref(manifest_value, path)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn project_manifest_field<'a>(
    manifest_value: &'a serde_json::Value,
    field: &str,
) -> Result<&'a serde_json::Value, EngineError> {
    manifest_value
        .get(field)
        .ok_or_else(|| EngineError::Validation(format!("project manifest missing {field} field")))
}

fn set_project_manifest_field(
    manifest_value: &mut serde_json::Value,
    field: &str,
    value: serde_json::Value,
) -> Result<(), EngineError> {
    let object = manifest_value
        .as_object_mut()
        .ok_or_else(|| EngineError::Validation("project manifest is not an object".to_string()))?;
    object.insert(field.to_string(), value);
    Ok(())
}

fn add_project_pool_ref(
    manifest_value: &mut serde_json::Value,
    path: &str,
    priority: u32,
) -> Result<(), EngineError> {
    let pools = manifest_value
        .get_mut("pools")
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(|| {
            EngineError::Validation("project manifest pools field is not an array".to_string())
        })?;
    if pools
        .iter()
        .any(|pool| pool.get("path").and_then(serde_json::Value::as_str) == Some(path))
    {
        return Ok(());
    }
    pools.push(serde_json::json!({
        "path": path,
        "priority": priority
    }));
    pools.sort_by(|left, right| {
        let left_priority = left
            .get("priority")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(u64::MAX);
        let right_priority = right
            .get("priority")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(u64::MAX);
        left_priority.cmp(&right_priority).then_with(|| {
            left["path"]
                .as_str()
                .unwrap_or("")
                .cmp(right["path"].as_str().unwrap_or(""))
        })
    });
    Ok(())
}

fn delete_project_pool_ref(
    manifest_value: &mut serde_json::Value,
    path: &str,
) -> Result<(), EngineError> {
    let pools = manifest_value
        .get_mut("pools")
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(|| {
            EngineError::Validation("project manifest pools field is not an array".to_string())
        })?;
    pools.retain(|pool| pool.get("path").and_then(serde_json::Value::as_str) != Some(path));
    Ok(())
}
