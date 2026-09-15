use uuid::Uuid;

use super::board_json_maps::{
    board_map_value, insert_board_map_value, remove_board_map_value, replace_board_map_value,
};
use super::board_list_journal_ops::{apply_board_list_operation, inverse_board_list_operation};
use super::board_package_json::{
    board_package_field, board_package_value, insert_board_package, remove_board_package,
    set_board_package_field,
};
use super::board_package_move::{set_board_package_side, translate_board_package_pads};
use super::board_root_journal_ops::{apply_board_root_operation, inverse_board_root_operation};
use super::{EngineError, Operation};

pub(super) fn apply_board_operation(
    board_value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<bool, EngineError> {
    if let Some(applied) = apply_board_list_operation(board_value, operation)? {
        return Ok(applied);
    }
    if let Some(applied) = apply_board_root_operation(board_value, operation)? {
        return Ok(applied);
    }
    match operation {
        Operation::BumpObjectRevision { .. } => Ok(false),
        Operation::CreateBoardPackage {
            package_id,
            package,
            materialized,
        } => {
            insert_board_package(board_value, *package_id, package.clone())?;
            apply_board_package_materialization(board_value, *package_id, materialized)?;
            Ok(true)
        }
        Operation::DeleteBoardPackage { package_id, .. } => {
            remove_board_package(board_value, *package_id)?;
            clear_board_package_materialization(board_value, *package_id);
            Ok(true)
        }
        Operation::SetBoardPackagePart {
            package_id,
            part_id,
        } => {
            set_board_package_uuid_field(board_value, *package_id, "part", *part_id)?;
            Ok(true)
        }
        Operation::SetBoardPackagePackage {
            package_id,
            package_ref_id,
            previous_materialized: _,
            materialized,
        } => {
            set_board_package_uuid_field(board_value, *package_id, "package", *package_ref_id)?;
            clear_board_package_materialization(board_value, *package_id);
            apply_board_package_materialization(board_value, *package_id, materialized)?;
            Ok(true)
        }
        Operation::SetBoardPackageValue { package_id, value } => {
            set_board_package_value(board_value, *package_id, value)?;
            Ok(true)
        }
        Operation::SetBoardPackageReference {
            package_id,
            reference,
        } => {
            set_board_package_string_field(board_value, *package_id, "reference", reference)?;
            Ok(true)
        }
        Operation::SetBoardPackagePosition { package_id, x, y } => {
            move_board_package_position(board_value, *package_id, *x, *y)?;
            Ok(true)
        }
        Operation::SetBoardPackageLayer { package_id, layer } => {
            set_board_package_i32_field(board_value, *package_id, "layer", *layer)?;
            Ok(true)
        }
        Operation::SetComponentSide { package_id, layer } => {
            set_board_package_side(board_value, *package_id, *layer)?;
            Ok(true)
        }
        Operation::SetBoardPackageRotation {
            package_id,
            rotation,
        } => {
            set_board_package_i32_field(board_value, *package_id, "rotation", *rotation)?;
            Ok(true)
        }
        Operation::SetBoardPackageLocked { package_id, locked } => {
            set_board_package_bool_field(board_value, *package_id, "locked", *locked)?;
            Ok(true)
        }
        Operation::CreateBoardPad { pad_id, pad } => {
            insert_board_map_value(board_value, "pads", *pad_id, pad.clone())?;
            Ok(true)
        }
        Operation::SetBoardPad { pad_id, pad } => {
            replace_board_map_value(board_value, "pads", *pad_id, pad.clone())?;
            Ok(true)
        }
        Operation::DeleteBoardPad { pad_id, .. } => {
            remove_board_map_value(board_value, "pads", *pad_id)?;
            Ok(true)
        }
        Operation::CreateBoardTrack { track_id, track } => {
            insert_board_map_value(board_value, "tracks", *track_id, track.clone())?;
            Ok(true)
        }
        Operation::SetBoardTrack { track_id, track } => {
            replace_board_map_value(board_value, "tracks", *track_id, track.clone())?;
            Ok(true)
        }
        Operation::DeleteBoardTrack { track_id, .. } => {
            remove_board_map_value(board_value, "tracks", *track_id)?;
            Ok(true)
        }
        Operation::CreateBoardVia { via_id, via } => {
            insert_board_map_value(board_value, "vias", *via_id, via.clone())?;
            Ok(true)
        }
        Operation::SetBoardVia { via_id, via } => {
            replace_board_map_value(board_value, "vias", *via_id, via.clone())?;
            Ok(true)
        }
        Operation::DeleteBoardVia { via_id, .. } => {
            remove_board_map_value(board_value, "vias", *via_id)?;
            Ok(true)
        }
        Operation::CreateBoardZone { zone_id, zone } => {
            insert_board_map_value(board_value, "zones", *zone_id, zone.clone())?;
            Ok(true)
        }
        Operation::SetBoardZone { zone_id, zone } => {
            replace_board_map_value(board_value, "zones", *zone_id, zone.clone())?;
            Ok(true)
        }
        Operation::DeleteBoardZone { zone_id, .. } => {
            remove_board_map_value(board_value, "zones", *zone_id)?;
            Ok(true)
        }
        Operation::CreateBoardNet { net_id, net } => {
            insert_board_map_value(board_value, "nets", *net_id, net.clone())?;
            Ok(true)
        }
        Operation::SetBoardNet { net_id, net } => {
            replace_board_map_value(board_value, "nets", *net_id, net.clone())?;
            Ok(true)
        }
        Operation::DeleteBoardNet { net_id, .. } => {
            remove_board_map_value(board_value, "nets", *net_id)?;
            Ok(true)
        }
        Operation::CreateBoardNetClass {
            net_class_id,
            net_class,
        } => {
            insert_board_map_value(board_value, "net_classes", *net_class_id, net_class.clone())?;
            Ok(true)
        }
        Operation::SetBoardNetClass {
            net_class_id,
            net_class,
        } => {
            replace_board_map_value(board_value, "net_classes", *net_class_id, net_class.clone())?;
            Ok(true)
        }
        Operation::DeleteBoardNetClass { net_class_id, .. } => {
            remove_board_map_value(board_value, "net_classes", *net_class_id)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(super) fn inverse_board_operation(
    board_value: &mut serde_json::Value,
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) -> Result<(), EngineError> {
    if inverse_board_list_operation(board_value, operation, inverse_operations)? {
        return Ok(());
    }
    if inverse_board_root_operation(board_value, operation, inverse_operations)? {
        return Ok(());
    }
    match operation {
        Operation::BumpObjectRevision { .. } => {}
        Operation::CreateBoardPackage {
            package_id,
            package,
            materialized,
        } => {
            inverse_operations.push(Operation::DeleteBoardPackage {
                package_id: *package_id,
                package: package.clone(),
                materialized: materialized.clone(),
            });
            insert_board_package(board_value, *package_id, package.clone())?;
            apply_board_package_materialization(board_value, *package_id, materialized)?;
        }
        Operation::DeleteBoardPackage { package_id, .. } => {
            let package = board_package_value(board_value, *package_id)?.clone();
            let materialized = board_package_materialization_payload(board_value, *package_id);
            inverse_operations.push(Operation::CreateBoardPackage {
                package_id: *package_id,
                package: package.clone(),
                materialized: materialized.clone(),
            });
            remove_board_package(board_value, *package_id)?;
            clear_board_package_materialization(board_value, *package_id);
        }
        Operation::SetBoardPackagePart {
            package_id,
            part_id,
        } => {
            let previous = board_package_uuid_field(board_value, *package_id, "part")?;
            inverse_operations.push(Operation::SetBoardPackagePart {
                package_id: *package_id,
                part_id: previous,
            });
            set_board_package_uuid_field(board_value, *package_id, "part", *part_id)?;
        }
        Operation::SetBoardPackagePackage {
            package_id,
            package_ref_id,
            previous_materialized: _,
            materialized,
        } => {
            let previous = board_package_uuid_field(board_value, *package_id, "package")?;
            let previous_materialized =
                board_package_materialization_payload(board_value, *package_id);
            inverse_operations.push(Operation::SetBoardPackagePackage {
                package_id: *package_id,
                package_ref_id: previous,
                previous_materialized: materialized.clone(),
                materialized: previous_materialized,
            });
            set_board_package_uuid_field(board_value, *package_id, "package", *package_ref_id)?;
            clear_board_package_materialization(board_value, *package_id);
            apply_board_package_materialization(board_value, *package_id, materialized)?;
        }
        Operation::SetBoardPackageValue { package_id, value } => {
            let previous = board_package_string_field(board_value, *package_id, "value")?;
            inverse_operations.push(Operation::SetBoardPackageValue {
                package_id: *package_id,
                value: previous,
            });
            set_board_package_value(board_value, *package_id, value)?;
        }
        Operation::SetBoardPackageReference {
            package_id,
            reference,
        } => {
            let previous = board_package_string_field(board_value, *package_id, "reference")?;
            inverse_operations.push(Operation::SetBoardPackageReference {
                package_id: *package_id,
                reference: previous,
            });
            set_board_package_string_field(board_value, *package_id, "reference", reference)?;
        }
        Operation::SetBoardPackagePosition { package_id, x, y } => {
            let previous = board_package_position(board_value, *package_id)?;
            inverse_operations.push(Operation::SetBoardPackagePosition {
                package_id: *package_id,
                x: previous.0,
                y: previous.1,
            });
            move_board_package_position(board_value, *package_id, *x, *y)?;
        }
        Operation::SetBoardPackageLayer { package_id, layer } => {
            let previous = board_package_i32_field(board_value, *package_id, "layer")?;
            inverse_operations.push(Operation::SetBoardPackageLayer {
                package_id: *package_id,
                layer: previous,
            });
            set_board_package_i32_field(board_value, *package_id, "layer", *layer)?;
        }
        Operation::SetComponentSide { package_id, layer } => {
            let previous = board_package_i32_field(board_value, *package_id, "layer")?;
            inverse_operations.push(Operation::SetComponentSide {
                package_id: *package_id,
                layer: previous,
            });
            set_board_package_side(board_value, *package_id, *layer)?;
        }
        Operation::SetBoardPackageRotation {
            package_id,
            rotation,
        } => {
            let previous = board_package_i32_field(board_value, *package_id, "rotation")?;
            inverse_operations.push(Operation::SetBoardPackageRotation {
                package_id: *package_id,
                rotation: previous,
            });
            set_board_package_i32_field(board_value, *package_id, "rotation", *rotation)?;
        }
        Operation::SetBoardPackageLocked { package_id, locked } => {
            let previous = board_package_bool_field(board_value, *package_id, "locked")?;
            inverse_operations.push(Operation::SetBoardPackageLocked {
                package_id: *package_id,
                locked: previous,
            });
            set_board_package_bool_field(board_value, *package_id, "locked", *locked)?;
        }
        Operation::CreateBoardPad { pad_id, pad } => {
            inverse_operations.push(Operation::DeleteBoardPad {
                pad_id: *pad_id,
                pad: pad.clone(),
            });
            insert_board_map_value(board_value, "pads", *pad_id, pad.clone())?;
        }
        Operation::SetBoardPad { pad_id, pad } => {
            let previous = board_map_value(board_value, "pads", *pad_id)?.clone();
            inverse_operations.push(Operation::SetBoardPad {
                pad_id: *pad_id,
                pad: previous,
            });
            replace_board_map_value(board_value, "pads", *pad_id, pad.clone())?;
        }
        Operation::DeleteBoardPad { pad_id, .. } => {
            let previous = board_map_value(board_value, "pads", *pad_id)?.clone();
            inverse_operations.push(Operation::CreateBoardPad {
                pad_id: *pad_id,
                pad: previous,
            });
            remove_board_map_value(board_value, "pads", *pad_id)?;
        }
        Operation::CreateBoardTrack { track_id, track } => {
            inverse_operations.push(Operation::DeleteBoardTrack {
                track_id: *track_id,
                track: track.clone(),
            });
            insert_board_map_value(board_value, "tracks", *track_id, track.clone())?;
        }
        Operation::SetBoardTrack { track_id, track } => {
            let previous = board_map_value(board_value, "tracks", *track_id)?.clone();
            inverse_operations.push(Operation::SetBoardTrack {
                track_id: *track_id,
                track: previous,
            });
            replace_board_map_value(board_value, "tracks", *track_id, track.clone())?;
        }
        Operation::DeleteBoardTrack { track_id, .. } => {
            let previous = board_map_value(board_value, "tracks", *track_id)?.clone();
            inverse_operations.push(Operation::CreateBoardTrack {
                track_id: *track_id,
                track: previous,
            });
            remove_board_map_value(board_value, "tracks", *track_id)?;
        }
        Operation::CreateBoardVia { via_id, via } => {
            inverse_operations.push(Operation::DeleteBoardVia {
                via_id: *via_id,
                via: via.clone(),
            });
            insert_board_map_value(board_value, "vias", *via_id, via.clone())?;
        }
        Operation::SetBoardVia { via_id, via } => {
            let previous = board_map_value(board_value, "vias", *via_id)?.clone();
            inverse_operations.push(Operation::SetBoardVia {
                via_id: *via_id,
                via: previous,
            });
            replace_board_map_value(board_value, "vias", *via_id, via.clone())?;
        }
        Operation::DeleteBoardVia { via_id, .. } => {
            let previous = board_map_value(board_value, "vias", *via_id)?.clone();
            inverse_operations.push(Operation::CreateBoardVia {
                via_id: *via_id,
                via: previous,
            });
            remove_board_map_value(board_value, "vias", *via_id)?;
        }
        Operation::CreateBoardZone { zone_id, zone } => {
            inverse_operations.push(Operation::DeleteBoardZone {
                zone_id: *zone_id,
                zone: zone.clone(),
            });
            insert_board_map_value(board_value, "zones", *zone_id, zone.clone())?;
        }
        Operation::SetBoardZone { zone_id, zone } => {
            let previous = board_map_value(board_value, "zones", *zone_id)?.clone();
            inverse_operations.push(Operation::SetBoardZone {
                zone_id: *zone_id,
                zone: previous,
            });
            replace_board_map_value(board_value, "zones", *zone_id, zone.clone())?;
        }
        Operation::DeleteBoardZone { zone_id, .. } => {
            let previous = board_map_value(board_value, "zones", *zone_id)?.clone();
            inverse_operations.push(Operation::CreateBoardZone {
                zone_id: *zone_id,
                zone: previous,
            });
            remove_board_map_value(board_value, "zones", *zone_id)?;
        }
        Operation::CreateBoardNet { net_id, net } => {
            inverse_operations.push(Operation::DeleteBoardNet {
                net_id: *net_id,
                net: net.clone(),
            });
            insert_board_map_value(board_value, "nets", *net_id, net.clone())?;
        }
        Operation::SetBoardNet { net_id, net } => {
            let previous = board_map_value(board_value, "nets", *net_id)?.clone();
            inverse_operations.push(Operation::SetBoardNet {
                net_id: *net_id,
                net: previous,
            });
            replace_board_map_value(board_value, "nets", *net_id, net.clone())?;
        }
        Operation::DeleteBoardNet { net_id, .. } => {
            let previous = board_map_value(board_value, "nets", *net_id)?.clone();
            inverse_operations.push(Operation::CreateBoardNet {
                net_id: *net_id,
                net: previous,
            });
            remove_board_map_value(board_value, "nets", *net_id)?;
        }
        Operation::CreateBoardNetClass {
            net_class_id,
            net_class,
        } => {
            inverse_operations.push(Operation::DeleteBoardNetClass {
                net_class_id: *net_class_id,
                net_class: net_class.clone(),
            });
            insert_board_map_value(board_value, "net_classes", *net_class_id, net_class.clone())?;
        }
        Operation::SetBoardNetClass {
            net_class_id,
            net_class,
        } => {
            let previous = board_map_value(board_value, "net_classes", *net_class_id)?.clone();
            inverse_operations.push(Operation::SetBoardNetClass {
                net_class_id: *net_class_id,
                net_class: previous,
            });
            replace_board_map_value(board_value, "net_classes", *net_class_id, net_class.clone())?;
        }
        Operation::DeleteBoardNetClass { net_class_id, .. } => {
            let previous = board_map_value(board_value, "net_classes", *net_class_id)?.clone();
            inverse_operations.push(Operation::CreateBoardNetClass {
                net_class_id: *net_class_id,
                net_class: previous,
            });
            remove_board_map_value(board_value, "net_classes", *net_class_id)?;
        }
        _ => {}
    }
    Ok(())
}

fn set_board_package_value(
    board_value: &mut serde_json::Value,
    package_id: Uuid,
    value: &str,
) -> Result<(), EngineError> {
    set_board_package_string_field(board_value, package_id, "value", value)
}

fn board_package_string_field(
    board_value: &serde_json::Value,
    package_id: Uuid,
    field: &str,
) -> Result<String, EngineError> {
    let value = board_package_field(board_value, package_id, field)?;
    value.as_str().map(str::to_string).ok_or_else(|| {
        EngineError::Validation(format!(
            "board package {package_id} field {field} is not a string"
        ))
    })
}

fn board_package_i32_field(
    board_value: &serde_json::Value,
    package_id: Uuid,
    field: &str,
) -> Result<i32, EngineError> {
    let value = board_package_field(board_value, package_id, field)?;
    let number = value.as_i64().ok_or_else(|| {
        EngineError::Validation(format!(
            "board package {package_id} field {field} is not an integer"
        ))
    })?;
    i32::try_from(number).map_err(|_| {
        EngineError::Validation(format!(
            "board package {package_id} field {field} is outside i32 range"
        ))
    })
}

fn board_package_bool_field(
    board_value: &serde_json::Value,
    package_id: Uuid,
    field: &str,
) -> Result<bool, EngineError> {
    let value = board_package_field(board_value, package_id, field)?;
    value.as_bool().ok_or_else(|| {
        EngineError::Validation(format!(
            "board package {package_id} field {field} is not a bool"
        ))
    })
}

fn board_package_uuid_field(
    board_value: &serde_json::Value,
    package_id: Uuid,
    field: &str,
) -> Result<Uuid, EngineError> {
    let value = board_package_string_field(board_value, package_id, field)?;
    Uuid::parse_str(&value).map_err(|error| {
        EngineError::Validation(format!(
            "board package {package_id} field {field} is not a UUID: {error}"
        ))
    })
}

fn board_package_position(
    board_value: &serde_json::Value,
    package_id: Uuid,
) -> Result<(i64, i64), EngineError> {
    let position = board_package_field(board_value, package_id, "position")?
        .as_object()
        .ok_or_else(|| {
            EngineError::Validation(format!(
                "board package {package_id} position is not an object"
            ))
        })?;
    let x = position
        .get("x")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| {
            EngineError::Validation(format!(
                "board package {package_id} position.x is not an integer"
            ))
        })?;
    let y = position
        .get("y")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| {
            EngineError::Validation(format!(
                "board package {package_id} position.y is not an integer"
            ))
        })?;
    Ok((x, y))
}

fn set_board_package_string_field(
    board_value: &mut serde_json::Value,
    package_id: Uuid,
    field: &str,
    value: &str,
) -> Result<(), EngineError> {
    set_board_package_field(
        board_value,
        package_id,
        field,
        serde_json::Value::String(value.to_string()),
    )
}

fn set_board_package_i32_field(
    board_value: &mut serde_json::Value,
    package_id: Uuid,
    field: &str,
    value: i32,
) -> Result<(), EngineError> {
    set_board_package_field(
        board_value,
        package_id,
        field,
        serde_json::Value::Number(value.into()),
    )
}

fn set_board_package_bool_field(
    board_value: &mut serde_json::Value,
    package_id: Uuid,
    field: &str,
    value: bool,
) -> Result<(), EngineError> {
    set_board_package_field(
        board_value,
        package_id,
        field,
        serde_json::Value::Bool(value),
    )
}

fn set_board_package_uuid_field(
    board_value: &mut serde_json::Value,
    package_id: Uuid,
    field: &str,
    value: Uuid,
) -> Result<(), EngineError> {
    set_board_package_string_field(board_value, package_id, field, &value.to_string())
}

fn clear_board_package_materialization(board_value: &mut serde_json::Value, package_id: Uuid) {
    let key = package_id.to_string();
    for map_name in [
        "component_silkscreen",
        "component_silkscreen_texts",
        "component_silkscreen_arcs",
        "component_silkscreen_circles",
        "component_silkscreen_polygons",
        "component_silkscreen_polylines",
        "component_mechanical_lines",
        "component_mechanical_texts",
        "component_mechanical_polygons",
        "component_mechanical_polylines",
        "component_mechanical_circles",
        "component_mechanical_arcs",
        "component_pads",
        "component_models_3d",
    ] {
        if let Some(map) = board_value
            .get_mut(map_name)
            .and_then(serde_json::Value::as_object_mut)
        {
            map.remove(&key);
        }
    }
}

fn board_package_materialization_payload(
    board_value: &serde_json::Value,
    package_id: Uuid,
) -> serde_json::Value {
    let key = package_id.to_string();
    let mut payload = serde_json::Map::new();
    for map_name in component_materialization_map_names() {
        if let Some(value) = board_value
            .get(map_name)
            .and_then(serde_json::Value::as_object)
            .and_then(|map| map.get(&key))
        {
            payload.insert(map_name.to_string(), value.clone());
        }
    }
    serde_json::Value::Object(payload)
}

fn apply_board_package_materialization(
    board_value: &mut serde_json::Value,
    package_id: Uuid,
    materialized: &serde_json::Value,
) -> Result<(), EngineError> {
    let key = package_id.to_string();
    let payload = materialized.as_object().ok_or_else(|| {
        EngineError::Validation(
            "board package materialization payload is not an object".to_string(),
        )
    })?;
    for map_name in component_materialization_map_names() {
        let Some(value) = payload.get(map_name) else {
            continue;
        };
        let map = board_value
            .get_mut(map_name)
            .and_then(serde_json::Value::as_object_mut)
            .ok_or_else(|| {
                EngineError::Validation(format!("board shard missing {map_name} map"))
            })?;
        map.insert(key.clone(), value.clone());
    }
    Ok(())
}

fn component_materialization_map_names() -> [&'static str; 14] {
    [
        "component_silkscreen",
        "component_silkscreen_texts",
        "component_silkscreen_arcs",
        "component_silkscreen_circles",
        "component_silkscreen_polygons",
        "component_silkscreen_polylines",
        "component_mechanical_lines",
        "component_mechanical_texts",
        "component_mechanical_polygons",
        "component_mechanical_polylines",
        "component_mechanical_circles",
        "component_mechanical_arcs",
        "component_pads",
        "component_models_3d",
    ]
}

fn move_board_package_position(
    board_value: &mut serde_json::Value,
    package_id: Uuid,
    x: i64,
    y: i64,
) -> Result<(), EngineError> {
    let (previous_x, previous_y) = board_package_position(board_value, package_id)?;
    let dx = x - previous_x;
    let dy = y - previous_y;
    set_board_package_field(
        board_value,
        package_id,
        "position",
        serde_json::json!({ "x": x, "y": y }),
    )?;
    translate_board_package_pads(board_value, package_id, dx, dy)?;
    Ok(())
}
