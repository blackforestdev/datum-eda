use uuid::Uuid;

use super::EngineError;

pub(crate) fn board_package_field<'a>(
    board_value: &'a serde_json::Value,
    package_id: Uuid,
    field: &str,
) -> Result<&'a serde_json::Value, EngineError> {
    let key = package_id.to_string();
    let package = board_packages(board_value)?
        .get(&key)
        .ok_or(EngineError::NotFound {
            object_type: "board_package",
            uuid: package_id,
        })?;
    let package_object = package.as_object().ok_or_else(|| {
        EngineError::Validation(format!("board package {package_id} is not an object"))
    })?;
    package_object.get(field).ok_or_else(|| {
        EngineError::Validation(format!("board package {package_id} missing field {field}"))
    })
}

pub(crate) fn board_package_value(
    board_value: &serde_json::Value,
    package_id: Uuid,
) -> Result<&serde_json::Value, EngineError> {
    let key = package_id.to_string();
    board_packages(board_value)?
        .get(&key)
        .ok_or(EngineError::NotFound {
            object_type: "board_package",
            uuid: package_id,
        })
}

pub(crate) fn set_board_package_field(
    board_value: &mut serde_json::Value,
    package_id: Uuid,
    field: &str,
    value: serde_json::Value,
) -> Result<(), EngineError> {
    let key = package_id.to_string();
    let package = board_packages_mut(board_value)?
        .get_mut(&key)
        .ok_or(EngineError::NotFound {
            object_type: "board_package",
            uuid: package_id,
        })?;
    let package_object = package.as_object_mut().ok_or_else(|| {
        EngineError::Validation(format!("board package {package_id} is not an object"))
    })?;
    package_object.insert(field.to_string(), value);
    Ok(())
}

pub(crate) fn insert_board_package(
    board_value: &mut serde_json::Value,
    package_id: Uuid,
    package: serde_json::Value,
) -> Result<(), EngineError> {
    let key = package_id.to_string();
    let packages = board_packages_mut(board_value)?;
    if packages.contains_key(&key) {
        return Ok(());
    }
    packages.insert(key, package);
    Ok(())
}

pub(crate) fn remove_board_package(
    board_value: &mut serde_json::Value,
    package_id: Uuid,
) -> Result<serde_json::Value, EngineError> {
    Ok(board_packages_mut(board_value)?
        .remove(&package_id.to_string())
        .unwrap_or(serde_json::Value::Null))
}

fn board_packages(
    board_value: &serde_json::Value,
) -> Result<&serde_json::Map<String, serde_json::Value>, EngineError> {
    board_value
        .get("packages")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| EngineError::Validation("board shard missing packages map".to_string()))
}

fn board_packages_mut(
    board_value: &mut serde_json::Value,
) -> Result<&mut serde_json::Map<String, serde_json::Value>, EngineError> {
    board_value
        .get_mut("packages")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| EngineError::Validation("board shard missing packages map".to_string()))
}
