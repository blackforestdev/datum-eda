use uuid::Uuid;

use super::EngineError;

pub(crate) fn board_map_value<'a>(
    board_value: &'a serde_json::Value,
    map_name: &str,
    object_id: Uuid,
) -> Result<&'a serde_json::Value, EngineError> {
    let key = object_id.to_string();
    let map = board_value
        .get(map_name)
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| EngineError::Validation(format!("board shard missing {map_name} map")))?;
    map.get(&key).ok_or(EngineError::NotFound {
        object_type: "board_object",
        uuid: object_id,
    })
}

pub(crate) fn insert_board_map_value(
    board_value: &mut serde_json::Value,
    map_name: &str,
    object_id: Uuid,
    value: serde_json::Value,
) -> Result<(), EngineError> {
    let key = object_id.to_string();
    let map = board_value
        .get_mut(map_name)
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| EngineError::Validation(format!("board shard missing {map_name} map")))?;
    if map.contains_key(&key) {
        return Ok(());
    }
    map.insert(key, value);
    Ok(())
}

pub(crate) fn replace_board_map_value(
    board_value: &mut serde_json::Value,
    map_name: &str,
    object_id: Uuid,
    value: serde_json::Value,
) -> Result<(), EngineError> {
    let key = object_id.to_string();
    let map = board_value
        .get_mut(map_name)
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| EngineError::Validation(format!("board shard missing {map_name} map")))?;
    if !map.contains_key(&key) {
        return Err(EngineError::NotFound {
            object_type: "board_object",
            uuid: object_id,
        });
    }
    map.insert(key, value);
    Ok(())
}

pub(crate) fn remove_board_map_value(
    board_value: &mut serde_json::Value,
    map_name: &str,
    object_id: Uuid,
) -> Result<serde_json::Value, EngineError> {
    let key = object_id.to_string();
    let map = board_value
        .get_mut(map_name)
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| EngineError::Validation(format!("board shard missing {map_name} map")))?;
    Ok(map.remove(&key).unwrap_or(serde_json::Value::Null))
}
