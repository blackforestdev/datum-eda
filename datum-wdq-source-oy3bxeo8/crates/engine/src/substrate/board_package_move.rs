use uuid::Uuid;

use super::EngineError;

pub(super) fn translate_board_package_pads(
    board_value: &mut serde_json::Value,
    package_id: Uuid,
    dx: i64,
    dy: i64,
) -> Result<(), EngineError> {
    let Some(pads) = board_value
        .get_mut("pads")
        .and_then(serde_json::Value::as_object_mut)
    else {
        return Ok(());
    };
    for pad in pads.values_mut() {
        let Some(pad_object) = pad.as_object_mut() else {
            continue;
        };
        let owns_package = pad_object
            .get("package")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| Uuid::parse_str(value).ok())
            == Some(package_id);
        if !owns_package {
            continue;
        }
        let Some(position) = pad_object
            .get_mut("position")
            .and_then(serde_json::Value::as_object_mut)
        else {
            continue;
        };
        let x = position
            .get("x")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| {
                EngineError::Validation("board pad position.x is not an integer".to_string())
            })?;
        let y = position
            .get("y")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| {
                EngineError::Validation("board pad position.y is not an integer".to_string())
            })?;
        position.insert("x".to_string(), serde_json::Value::Number((x + dx).into()));
        position.insert("y".to_string(), serde_json::Value::Number((y + dy).into()));
    }
    Ok(())
}

pub(super) fn set_board_package_side(
    board_value: &mut serde_json::Value,
    package_id: Uuid,
    target_layer: i32,
) -> Result<(), EngineError> {
    let previous_layer = board_package_i32_field(board_value, package_id, "layer")?;
    let origin_x = board_package_position_x(board_value, package_id)?;
    set_board_package_i32_field(board_value, package_id, "layer", target_layer)?;
    if previous_layer != target_layer {
        mirror_board_package_pads_for_side(
            board_value,
            package_id,
            origin_x,
            previous_layer,
            target_layer,
        )?;
    }
    Ok(())
}

fn mirror_board_package_pads_for_side(
    board_value: &mut serde_json::Value,
    package_id: Uuid,
    origin_x: i64,
    previous_layer: i32,
    target_layer: i32,
) -> Result<(), EngineError> {
    if let Some(pads) = board_value
        .get_mut("pads")
        .and_then(serde_json::Value::as_object_mut)
    {
        for pad in pads.values_mut() {
            let Some(pad_object) = pad.as_object_mut() else {
                continue;
            };
            let owns_package = pad_object
                .get("package")
                .and_then(serde_json::Value::as_str)
                .and_then(|value| Uuid::parse_str(value).ok())
                == Some(package_id);
            if !owns_package {
                continue;
            }
            mirror_pad_object_for_side(pad_object, origin_x, previous_layer, target_layer)?;
        }
    }

    let key = package_id.to_string();
    if let Some(component_pads) = board_value
        .get_mut("component_pads")
        .and_then(serde_json::Value::as_object_mut)
        .and_then(|pads| pads.get_mut(&key))
        .and_then(serde_json::Value::as_array_mut)
    {
        for pad in component_pads {
            let Some(pad_object) = pad.as_object_mut() else {
                continue;
            };
            mirror_pad_object_for_side(pad_object, origin_x, previous_layer, target_layer)?;
        }
    }

    Ok(())
}

fn mirror_pad_object_for_side(
    pad_object: &mut serde_json::Map<String, serde_json::Value>,
    origin_x: i64,
    previous_layer: i32,
    target_layer: i32,
) -> Result<(), EngineError> {
    let Some(position) = pad_object
        .get_mut("position")
        .and_then(serde_json::Value::as_object_mut)
    else {
        return Ok(());
    };
    let x = position
        .get("x")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| {
            EngineError::Validation("board pad position.x is not an integer".to_string())
        })?;
    position.insert(
        "x".to_string(),
        serde_json::Value::Number((origin_x * 2 - x).into()),
    );
    pad_object.insert(
        "layer".to_string(),
        serde_json::Value::Number(target_layer.into()),
    );
    for field in ["copper_layers", "mask_layers", "paste_layers"] {
        swap_pad_layer_array(pad_object, field, previous_layer, target_layer)?;
    }
    if let Some(rotation) = pad_object
        .get("rotation")
        .and_then(serde_json::Value::as_i64)
    {
        let mirrored = normalize_degrees(180 - rotation);
        pad_object.insert(
            "rotation".to_string(),
            serde_json::Value::Number(mirrored.into()),
        );
    }
    Ok(())
}

fn swap_pad_layer_array(
    pad_object: &mut serde_json::Map<String, serde_json::Value>,
    field: &str,
    previous_layer: i32,
    target_layer: i32,
) -> Result<(), EngineError> {
    let Some(layers) = pad_object
        .get_mut(field)
        .and_then(serde_json::Value::as_array_mut)
    else {
        return Ok(());
    };
    for layer in layers {
        let Some(value) = layer.as_i64() else {
            return Err(EngineError::Validation(format!(
                "board pad {field} entry is not an integer"
            )));
        };
        if value == i64::from(previous_layer) {
            *layer = serde_json::Value::Number(target_layer.into());
        } else if value == i64::from(target_layer) {
            *layer = serde_json::Value::Number(previous_layer.into());
        }
    }
    Ok(())
}

fn normalize_degrees(value: i64) -> i64 {
    value.rem_euclid(360)
}

fn board_package_i32_field(
    board_value: &serde_json::Value,
    package_id: Uuid,
    field: &str,
) -> Result<i32, EngineError> {
    let value = board_package_field(board_value, package_id, field)?
        .as_i64()
        .ok_or_else(|| {
            EngineError::Validation(format!(
                "board package {package_id} field {field} is not an integer"
            ))
        })?;
    i32::try_from(value).map_err(|_| {
        EngineError::Validation(format!(
            "board package {package_id} field {field} is outside i32 range"
        ))
    })
}

fn board_package_position_x(
    board_value: &serde_json::Value,
    package_id: Uuid,
) -> Result<i64, EngineError> {
    let position = board_package_field(board_value, package_id, "position")?
        .as_object()
        .ok_or_else(|| {
            EngineError::Validation(format!(
                "board package {package_id} position is not an object"
            ))
        })?;
    position
        .get("x")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| {
            EngineError::Validation(format!(
                "board package {package_id} position.x is not an integer"
            ))
        })
}

fn board_package_field<'a>(
    board_value: &'a serde_json::Value,
    package_id: Uuid,
    field: &str,
) -> Result<&'a serde_json::Value, EngineError> {
    let key = package_id.to_string();
    let package = board_value
        .get("packages")
        .and_then(serde_json::Value::as_object)
        .and_then(|packages| packages.get(&key))
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

fn set_board_package_i32_field(
    board_value: &mut serde_json::Value,
    package_id: Uuid,
    field: &str,
    value: i32,
) -> Result<(), EngineError> {
    let key = package_id.to_string();
    let package = board_value
        .get_mut("packages")
        .and_then(serde_json::Value::as_object_mut)
        .and_then(|packages| packages.get_mut(&key))
        .ok_or(EngineError::NotFound {
            object_type: "board_package",
            uuid: package_id,
        })?;
    let package_object = package.as_object_mut().ok_or_else(|| {
        EngineError::Validation(format!("board package {package_id} is not an object"))
    })?;
    package_object.insert(field.to_string(), serde_json::Value::Number(value.into()));
    Ok(())
}
