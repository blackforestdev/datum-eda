use uuid::Uuid;

use super::EngineError;

pub(super) fn sheet_map_value<'a>(
    sheet_value: &'a serde_json::Value,
    map_name: &str,
    object_id: Uuid,
) -> Result<&'a serde_json::Value, EngineError> {
    sheet_value
        .get(map_name)
        .and_then(serde_json::Value::as_object)
        .and_then(|values| values.get(&object_id.to_string()))
        .ok_or(EngineError::NotFound {
            object_type: "schematic_sheet_object",
            uuid: object_id,
        })
}

pub(super) fn insert_sheet_map_value(
    sheet_value: &mut serde_json::Value,
    map_name: &str,
    object_id: Uuid,
    value: serde_json::Value,
) -> Result<(), EngineError> {
    validate_payload_uuid(map_name, object_id, &value)?;
    if map_name == "bus_entries" {
        validate_bus_entry_refs(sheet_value, object_id, &value)?;
    }
    sheet_map_mut(sheet_value, map_name)?.insert(object_id.to_string(), value);
    Ok(())
}

pub(super) fn remove_sheet_map_value(
    sheet_value: &mut serde_json::Value,
    map_name: &str,
    object_id: Uuid,
) -> Result<serde_json::Value, EngineError> {
    sheet_map_mut(sheet_value, map_name)?
        .remove(&object_id.to_string())
        .ok_or(EngineError::NotFound {
            object_type: "schematic_sheet_object",
            uuid: object_id,
        })
}

pub(super) fn sheet_uuid(sheet_value: &serde_json::Value) -> Option<Uuid> {
    sheet_value
        .get("uuid")
        .and_then(serde_json::Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
}

fn validate_payload_uuid(
    map_name: &str,
    object_id: Uuid,
    value: &serde_json::Value,
) -> Result<(), EngineError> {
    let payload_uuid = payload_uuid(value)
        .ok_or_else(|| EngineError::Validation(format!("{map_name} payload missing uuid")))?;
    let payload_uuid = Uuid::parse_str(payload_uuid)
        .map_err(|_| EngineError::Validation(format!("{map_name} payload has invalid uuid")))?;
    if payload_uuid != object_id {
        return Err(EngineError::Validation(format!(
            "{map_name} payload uuid {payload_uuid} does not match operation object {object_id}"
        )));
    }
    Ok(())
}

fn payload_uuid(value: &serde_json::Value) -> Option<&str> {
    value
        .get("uuid")
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            let object = value.as_object()?;
            if object.len() != 1 {
                return None;
            }
            object
                .values()
                .next()
                .and_then(|nested| nested.get("uuid"))
                .and_then(serde_json::Value::as_str)
        })
}

fn validate_bus_entry_refs(
    sheet_value: &serde_json::Value,
    bus_entry_id: Uuid,
    value: &serde_json::Value,
) -> Result<(), EngineError> {
    let bus_id = value
        .get("bus")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            EngineError::Validation(format!("bus entry {bus_entry_id} payload missing bus"))
        })?;
    let bus_id = Uuid::parse_str(bus_id).map_err(|_| {
        EngineError::Validation(format!(
            "bus entry {bus_entry_id} payload has invalid bus uuid"
        ))
    })?;
    sheet_map_value(sheet_value, "buses", bus_id)?;
    if let Some(wire_id) = value.get("wire").and_then(serde_json::Value::as_str) {
        let wire_id = Uuid::parse_str(wire_id).map_err(|_| {
            EngineError::Validation(format!(
                "bus entry {bus_entry_id} payload has invalid wire uuid"
            ))
        })?;
        sheet_map_value(sheet_value, "wires", wire_id)?;
    }
    Ok(())
}

fn sheet_map_mut<'a>(
    sheet_value: &'a mut serde_json::Value,
    map_name: &str,
) -> Result<&'a mut serde_json::Map<String, serde_json::Value>, EngineError> {
    sheet_value
        .get_mut(map_name)
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| EngineError::Validation(format!("schematic sheet missing {map_name} map")))
}
