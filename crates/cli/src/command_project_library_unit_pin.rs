use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::substrate::Operation;
use uuid::Uuid;

use super::command_project_library::{
    NativeProjectPoolLibraryObjectMutationView, commit_pool_library_operations,
    pool_library_mutation_view, pool_library_relative_path, validate_project_local_pool_path,
};
use super::command_project_library_payload::read_project_pool_object_payload;

pub(crate) fn set_native_project_pool_unit_pin(
    root: &Path,
    pool_path: &str,
    unit_id: Uuid,
    pin_id: Uuid,
    name: String,
    direction: Option<String>,
    electrical_type: Option<String>,
    swap_group: u32,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let direction = resolve_pin_electrical_type(direction, electrical_type)?;
    let name = name.trim().to_string();
    if name.is_empty() {
        bail!("pool unit pin name must not be empty");
    }
    let relative_path = pool_library_relative_path(pool_path, "units", unit_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, unit_id)?;
    let mut object = previous_object.clone();
    let pins = object
        .as_object_mut()
        .context("pool unit payload must be a JSON object")?
        .entry("pins")
        .or_insert_with(|| serde_json::json!({}));
    let pins = pins
        .as_object_mut()
        .context("pool unit pins field must be an object")?;
    if pins.contains_key(&pin_id.to_string()) {
        bail!("pool unit {unit_id} already has pin {pin_id}");
    }
    pins.insert(
        pin_id.to_string(),
        serde_json::json!({
            "uuid": pin_id,
            "name": name,
            "direction": direction,
            "electrical_type": direction,
            "swap_group": swap_group,
            "alternates": []
        }),
    );
    commit_pool_library_operations(
        root,
        format!("set native pool unit pin {pin_id} on unit {unit_id}"),
        vec![Operation::SetPoolLibraryObject {
            object_id: unit_id,
            relative_path: relative_path.clone(),
            object_kind: "units".to_string(),
            previous_object,
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_unit_pin",
        pool_path,
        "units",
        unit_id,
        &relative_path,
    )
}

fn resolve_pin_electrical_type(
    direction: Option<String>,
    electrical_type: Option<String>,
) -> Result<String> {
    match (direction, electrical_type) {
        (Some(direction), Some(electrical_type)) if direction != electrical_type => bail!(
            "conflicting pin electrical type {electrical_type} and direction {direction}; pass only one value or use matching values"
        ),
        (Some(direction), _) => validate_pin_electrical_type(direction),
        (_, Some(electrical_type)) => validate_pin_electrical_type(electrical_type),
        (None, None) => Ok("Passive".to_string()),
    }
}

fn validate_pin_electrical_type(direction: String) -> Result<String> {
    match direction.as_str() {
        "Input" | "Output" | "Bidirectional" | "Passive" | "PowerIn" | "PowerOut"
        | "OpenCollector" | "OpenEmitter" | "TriState" | "NoConnect" => Ok(direction),
        other => bail!(
            "unsupported pin direction/electrical type {other}; expected Input, Output, Bidirectional, Passive, PowerIn, PowerOut, OpenCollector, OpenEmitter, TriState, or NoConnect"
        ),
    }
}
