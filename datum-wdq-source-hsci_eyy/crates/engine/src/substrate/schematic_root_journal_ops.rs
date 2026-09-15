use uuid::Uuid;

use super::{EngineError, Operation};

pub(super) fn apply_schematic_root_operation(
    schematic_value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<bool, EngineError> {
    match operation {
        Operation::CreateSchematicWaiver {
            schematic_id,
            waiver_id,
            waiver,
        } => {
            require_schematic_root(schematic_value, schematic_id)?;
            require_payload_id("waiver", waiver, waiver_id)?;
            let mut waivers = array_field(schematic_value, "waivers")?;
            if waivers
                .iter()
                .any(|existing| payload_id("waiver", existing).ok() == Some(*waiver_id))
            {
                return Err(EngineError::Validation(format!(
                    "schematic waiver {waiver_id} already exists"
                )));
            }
            waivers.push(waiver.clone());
            set_array_field(
                schematic_value,
                "waivers",
                serde_json::Value::Array(waivers),
            )?;
            Ok(true)
        }
        Operation::DeleteSchematicWaiver {
            schematic_id,
            waiver_id,
            ..
        } => {
            require_schematic_root(schematic_value, schematic_id)?;
            let mut waivers = array_field(schematic_value, "waivers")?;
            let before_len = waivers.len();
            waivers.retain(|existing| payload_id("waiver", existing).ok() != Some(*waiver_id));
            if waivers.len() == before_len {
                return Err(EngineError::Validation(format!(
                    "schematic waiver {waiver_id} not found"
                )));
            }
            set_array_field(
                schematic_value,
                "waivers",
                serde_json::Value::Array(waivers),
            )?;
            Ok(true)
        }
        Operation::CreateSchematicDeviation {
            schematic_id,
            deviation_id,
            deviation,
        } => {
            require_schematic_root(schematic_value, schematic_id)?;
            require_payload_id("deviation", deviation, deviation_id)?;
            let mut deviations = array_field(schematic_value, "deviations")?;
            if deviations
                .iter()
                .any(|existing| payload_id("deviation", existing).ok() == Some(*deviation_id))
            {
                return Err(EngineError::Validation(format!(
                    "schematic deviation {deviation_id} already exists"
                )));
            }
            deviations.push(deviation.clone());
            set_array_field(
                schematic_value,
                "deviations",
                serde_json::Value::Array(deviations),
            )?;
            Ok(true)
        }
        Operation::DeleteSchematicDeviation {
            schematic_id,
            deviation_id,
            ..
        } => {
            require_schematic_root(schematic_value, schematic_id)?;
            let mut deviations = array_field(schematic_value, "deviations")?;
            let before_len = deviations.len();
            deviations
                .retain(|existing| payload_id("deviation", existing).ok() != Some(*deviation_id));
            if deviations.len() == before_len {
                return Err(EngineError::Validation(format!(
                    "schematic deviation {deviation_id} not found"
                )));
            }
            set_array_field(
                schematic_value,
                "deviations",
                serde_json::Value::Array(deviations),
            )?;
            Ok(true)
        }
        Operation::CreateSchematicSheet {
            schematic_id,
            sheet_id,
            relative_path,
            sheet,
        } => {
            require_schematic_root(schematic_value, schematic_id)?;
            require_payload_id("sheet", sheet, sheet_id)?;
            let object = schematic_value.as_object_mut().ok_or_else(|| {
                EngineError::Validation("schematic root is not an object".to_string())
            })?;
            let sheets = object
                .get_mut("sheets")
                .and_then(serde_json::Value::as_object_mut)
                .ok_or_else(|| {
                    EngineError::Validation("schematic root missing sheets map".to_string())
                })?;
            if sheets.contains_key(&sheet_id.to_string()) {
                return Err(EngineError::Validation(format!(
                    "schematic sheet {sheet_id} already exists"
                )));
            }
            if sheets
                .values()
                .any(|path| path.as_str() == Some(relative_path.as_str()))
            {
                return Err(EngineError::Validation(format!(
                    "schematic sheet path already exists: {relative_path}"
                )));
            }
            sheets.insert(
                sheet_id.to_string(),
                serde_json::Value::String(relative_path.clone()),
            );
            Ok(true)
        }
        Operation::DeleteSchematicSheet {
            schematic_id,
            sheet_id,
            relative_path,
            ..
        } => {
            require_schematic_root(schematic_value, schematic_id)?;
            let object = schematic_value.as_object_mut().ok_or_else(|| {
                EngineError::Validation("schematic root is not an object".to_string())
            })?;
            let sheets = object
                .get_mut("sheets")
                .and_then(serde_json::Value::as_object_mut)
                .ok_or_else(|| {
                    EngineError::Validation("schematic root missing sheets map".to_string())
                })?;
            let removed = sheets.remove(&sheet_id.to_string()).ok_or_else(|| {
                EngineError::Validation(format!("schematic sheet {sheet_id} not found"))
            })?;
            if removed.as_str() != Some(relative_path.as_str()) {
                return Err(EngineError::Validation(format!(
                    "schematic sheet path mismatch for {sheet_id}: expected {relative_path}, found {removed}"
                )));
            }
            Ok(true)
        }
        Operation::CreateSchematicDefinition {
            schematic_id,
            definition_id,
            relative_path,
            definition,
        } => {
            require_schematic_root(schematic_value, schematic_id)?;
            require_payload_id("definition", definition, definition_id)?;
            let object = schematic_value.as_object_mut().ok_or_else(|| {
                EngineError::Validation("schematic root is not an object".to_string())
            })?;
            let definitions = object
                .get_mut("definitions")
                .and_then(serde_json::Value::as_object_mut)
                .ok_or_else(|| {
                    EngineError::Validation("schematic root missing definitions map".to_string())
                })?;
            if definitions.contains_key(&definition_id.to_string()) {
                return Err(EngineError::Validation(format!(
                    "schematic definition {definition_id} already exists"
                )));
            }
            if definitions
                .values()
                .any(|path| path.as_str() == Some(relative_path.as_str()))
            {
                return Err(EngineError::Validation(format!(
                    "schematic definition path already exists: {relative_path}"
                )));
            }
            definitions.insert(
                definition_id.to_string(),
                serde_json::Value::String(relative_path.clone()),
            );
            Ok(true)
        }
        Operation::DeleteSchematicDefinition {
            schematic_id,
            definition_id,
            relative_path,
            ..
        } => {
            require_schematic_root(schematic_value, schematic_id)?;
            let object = schematic_value.as_object_mut().ok_or_else(|| {
                EngineError::Validation("schematic root is not an object".to_string())
            })?;
            let definitions = object
                .get_mut("definitions")
                .and_then(serde_json::Value::as_object_mut)
                .ok_or_else(|| {
                    EngineError::Validation("schematic root missing definitions map".to_string())
                })?;
            let removed = definitions
                .remove(&definition_id.to_string())
                .ok_or_else(|| {
                    EngineError::Validation(format!(
                        "schematic definition {definition_id} not found"
                    ))
                })?;
            if removed.as_str() != Some(relative_path.as_str()) {
                return Err(EngineError::Validation(format!(
                    "schematic definition path mismatch for {definition_id}: expected {relative_path}, found {removed}"
                )));
            }
            Ok(true)
        }
        Operation::CreateSchematicSheetInstance {
            schematic_id,
            instance_id,
            instance,
        } => {
            require_schematic_root(schematic_value, schematic_id)?;
            require_payload_id("sheet instance", instance, instance_id)?;
            let mut instances = array_field(schematic_value, "instances")?;
            if instances
                .iter()
                .any(|existing| payload_id("sheet instance", existing).ok() == Some(*instance_id))
            {
                return Err(EngineError::Validation(format!(
                    "schematic sheet instance {instance_id} already exists"
                )));
            }
            instances.push(instance.clone());
            set_array_field(
                schematic_value,
                "instances",
                serde_json::Value::Array(instances),
            )?;
            Ok(true)
        }
        Operation::DeleteSchematicSheetInstance {
            schematic_id,
            instance_id,
            ..
        } => {
            require_schematic_root(schematic_value, schematic_id)?;
            let mut instances = array_field(schematic_value, "instances")?;
            let before_len = instances.len();
            instances.retain(|existing| {
                payload_id("sheet instance", existing).ok() != Some(*instance_id)
            });
            if instances.len() == before_len {
                return Err(EngineError::Validation(format!(
                    "schematic sheet instance {instance_id} not found"
                )));
            }
            set_array_field(
                schematic_value,
                "instances",
                serde_json::Value::Array(instances),
            )?;
            Ok(true)
        }
        Operation::SetSchematicSheetInstance {
            schematic_id,
            instance_id,
            previous_instance,
            instance,
        } => {
            require_schematic_root(schematic_value, schematic_id)?;
            require_payload_id("previous sheet instance", previous_instance, instance_id)?;
            require_payload_id("sheet instance", instance, instance_id)?;
            let mut instances = array_field(schematic_value, "instances")?;
            let mut replaced = false;
            for existing in &mut instances {
                if payload_id("sheet instance", existing).ok() == Some(*instance_id) {
                    *existing = instance.clone();
                    replaced = true;
                    break;
                }
            }
            if !replaced {
                return Err(EngineError::Validation(format!(
                    "schematic sheet instance {instance_id} not found"
                )));
            }
            set_array_field(
                schematic_value,
                "instances",
                serde_json::Value::Array(instances),
            )?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(super) fn inverse_schematic_root_operation(
    schematic_value: &mut serde_json::Value,
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) -> Result<(), EngineError> {
    match operation {
        Operation::CreateSchematicWaiver {
            schematic_id,
            waiver_id,
            waiver,
        } => {
            inverse_operations.push(Operation::DeleteSchematicWaiver {
                schematic_id: *schematic_id,
                waiver_id: *waiver_id,
                waiver: waiver.clone(),
            });
            apply_schematic_root_operation(schematic_value, operation)?;
        }
        Operation::DeleteSchematicWaiver {
            schematic_id,
            waiver_id,
            ..
        } => {
            let waiver = find_payload(schematic_value, "waivers", "waiver", waiver_id)?;
            inverse_operations.push(Operation::CreateSchematicWaiver {
                schematic_id: *schematic_id,
                waiver_id: *waiver_id,
                waiver,
            });
            apply_schematic_root_operation(schematic_value, operation)?;
        }
        Operation::CreateSchematicDeviation {
            schematic_id,
            deviation_id,
            deviation,
        } => {
            inverse_operations.push(Operation::DeleteSchematicDeviation {
                schematic_id: *schematic_id,
                deviation_id: *deviation_id,
                deviation: deviation.clone(),
            });
            apply_schematic_root_operation(schematic_value, operation)?;
        }
        Operation::DeleteSchematicDeviation {
            schematic_id,
            deviation_id,
            ..
        } => {
            let deviation = find_payload(schematic_value, "deviations", "deviation", deviation_id)?;
            inverse_operations.push(Operation::CreateSchematicDeviation {
                schematic_id: *schematic_id,
                deviation_id: *deviation_id,
                deviation,
            });
            apply_schematic_root_operation(schematic_value, operation)?;
        }
        Operation::CreateSchematicSheet {
            schematic_id,
            sheet_id,
            relative_path,
            sheet,
        } => {
            inverse_operations.push(Operation::DeleteSchematicSheet {
                schematic_id: *schematic_id,
                sheet_id: *sheet_id,
                relative_path: relative_path.clone(),
                sheet: sheet.clone(),
            });
            apply_schematic_root_operation(schematic_value, operation)?;
        }
        Operation::DeleteSchematicSheet {
            schematic_id,
            sheet_id,
            relative_path,
            sheet,
        } => {
            inverse_operations.push(Operation::CreateSchematicSheet {
                schematic_id: *schematic_id,
                sheet_id: *sheet_id,
                relative_path: relative_path.clone(),
                sheet: sheet.clone(),
            });
            apply_schematic_root_operation(schematic_value, operation)?;
        }
        Operation::CreateSchematicDefinition {
            schematic_id,
            definition_id,
            relative_path,
            definition,
        } => {
            inverse_operations.push(Operation::DeleteSchematicDefinition {
                schematic_id: *schematic_id,
                definition_id: *definition_id,
                relative_path: relative_path.clone(),
                definition: definition.clone(),
            });
            apply_schematic_root_operation(schematic_value, operation)?;
        }
        Operation::DeleteSchematicDefinition {
            schematic_id,
            definition_id,
            relative_path,
            definition,
        } => {
            inverse_operations.push(Operation::CreateSchematicDefinition {
                schematic_id: *schematic_id,
                definition_id: *definition_id,
                relative_path: relative_path.clone(),
                definition: definition.clone(),
            });
            apply_schematic_root_operation(schematic_value, operation)?;
        }
        Operation::CreateSchematicSheetInstance {
            schematic_id,
            instance_id,
            instance,
        } => {
            inverse_operations.push(Operation::DeleteSchematicSheetInstance {
                schematic_id: *schematic_id,
                instance_id: *instance_id,
                instance: instance.clone(),
            });
            apply_schematic_root_operation(schematic_value, operation)?;
        }
        Operation::DeleteSchematicSheetInstance {
            schematic_id,
            instance_id,
            instance,
        } => {
            inverse_operations.push(Operation::CreateSchematicSheetInstance {
                schematic_id: *schematic_id,
                instance_id: *instance_id,
                instance: instance.clone(),
            });
            apply_schematic_root_operation(schematic_value, operation)?;
        }
        Operation::SetSchematicSheetInstance {
            schematic_id,
            instance_id,
            previous_instance,
            instance,
        } => {
            inverse_operations.push(Operation::SetSchematicSheetInstance {
                schematic_id: *schematic_id,
                instance_id: *instance_id,
                previous_instance: instance.clone(),
                instance: previous_instance.clone(),
            });
            apply_schematic_root_operation(schematic_value, operation)?;
        }
        _ => {}
    }
    Ok(())
}

fn require_schematic_root(
    schematic_value: &serde_json::Value,
    schematic_id: &Uuid,
) -> Result<(), EngineError> {
    let actual = schematic_value
        .get("uuid")
        .and_then(serde_json::Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or_else(|| {
            EngineError::Validation("schematic root missing uuid for waiver edit".to_string())
        })?;
    if &actual != schematic_id {
        return Err(EngineError::Validation(format!(
            "schematic root id mismatch: expected {schematic_id}, found {actual}"
        )));
    }
    Ok(())
}

fn array_field(
    schematic_value: &serde_json::Value,
    field: &str,
) -> Result<Vec<serde_json::Value>, EngineError> {
    let value = schematic_value
        .get(field)
        .ok_or_else(|| EngineError::Validation(format!("schematic root missing {field} array")))?;
    value.as_array().cloned().ok_or_else(|| {
        EngineError::Validation(format!("schematic root {field} field is not an array"))
    })
}

fn set_array_field(
    schematic_value: &mut serde_json::Value,
    field: &str,
    values: serde_json::Value,
) -> Result<(), EngineError> {
    let object = schematic_value
        .as_object_mut()
        .ok_or_else(|| EngineError::Validation("schematic root is not an object".to_string()))?;
    object.insert(field.to_string(), values);
    Ok(())
}

fn require_payload_id(
    label: &str,
    payload: &serde_json::Value,
    payload_uuid: &Uuid,
) -> Result<(), EngineError> {
    let actual = payload_id(label, payload)?;
    if &actual != payload_uuid {
        return Err(EngineError::Validation(format!(
            "{label} payload id mismatch: expected {payload_uuid}, found {actual}"
        )));
    }
    Ok(())
}

fn payload_id(label: &str, payload: &serde_json::Value) -> Result<Uuid, EngineError> {
    payload
        .get("uuid")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| EngineError::Validation(format!("schematic {label} missing uuid")))
        .and_then(|value| {
            Uuid::parse_str(value)
                .map_err(|error| EngineError::Validation(format!("invalid {label} uuid: {error}")))
        })
}

fn find_payload(
    schematic_value: &serde_json::Value,
    field: &str,
    label: &str,
    payload_id_value: &Uuid,
) -> Result<serde_json::Value, EngineError> {
    array_field(schematic_value, field)?
        .into_iter()
        .find(|payload| payload_id(label, payload).ok() == Some(*payload_id_value))
        .ok_or_else(|| {
            EngineError::Validation(format!("schematic {label} {payload_id_value} not found"))
        })
}
