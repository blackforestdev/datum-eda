use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::api::native_write::library::{PoolLibraryObjectTarget, PoolLibraryOperationSpec};
use eda_engine::substrate::ProjectResolver;
use uuid::Uuid;

use super::command_project_library::{
    NativeProjectPoolLibraryObjectMutationView, commit_pool_library_operations,
    pool_library_mutation_view, pool_library_relative_path, validate_project_local_pool_path,
};
use super::command_project_library_payload::read_project_pool_object_payload;

pub(crate) fn set_native_project_pool_part_bindings(
    root: &Path,
    pool_path: &str,
    part_id: Uuid,
    default_footprint: Option<Uuid>,
    clear_default_footprint: bool,
    default_pin_pad_map: Option<Uuid>,
    clear_default_pin_pad_map: bool,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    if default_footprint.is_some() && clear_default_footprint {
        bail!("default-footprint and clear-default-footprint are mutually exclusive");
    }
    if default_pin_pad_map.is_some() && clear_default_pin_pad_map {
        bail!("default-pin-pad-map and clear-default-pin-pad-map are mutually exclusive");
    }
    if default_footprint.is_none()
        && !clear_default_footprint
        && default_pin_pad_map.is_none()
        && !clear_default_pin_pad_map
    {
        bail!("set-pool-part-bindings requires at least one binding update");
    }

    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    if let Some(footprint_id) = default_footprint {
        if model
            .objects
            .get(&footprint_id)
            .filter(|object| object.domain == "pool" && object.kind == "footprints")
            .is_none()
        {
            bail!("missing pool footprint {footprint_id} for part {part_id}");
        }
    }
    if let Some(pin_pad_map_id) = default_pin_pad_map {
        if model
            .objects
            .get(&pin_pad_map_id)
            .filter(|object| object.domain == "pool" && object.kind == "pin_pad_maps")
            .is_none()
        {
            bail!("missing pool pin_pad_map {pin_pad_map_id} for part {part_id}");
        }
    }

    let relative_path = pool_library_relative_path(pool_path, "parts", part_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, part_id)?;
    let mut object = previous_object.clone();
    let object_map = object
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("part {part_id} payload is not an object"))?;
    if clear_default_footprint {
        object_map.insert("default_footprint".to_string(), serde_json::Value::Null);
    } else if let Some(default_footprint) = default_footprint {
        object_map.insert(
            "default_footprint".to_string(),
            serde_json::Value::String(default_footprint.to_string()),
        );
    }
    if clear_default_pin_pad_map {
        object_map.insert("default_pin_pad_map".to_string(), serde_json::Value::Null);
    } else if let Some(default_pin_pad_map) = default_pin_pad_map {
        object_map.insert(
            "default_pin_pad_map".to_string(),
            serde_json::Value::String(default_pin_pad_map.to_string()),
        );
    }
    commit_pool_library_operations(
        root,
        format!("set native pool part {part_id} default bindings"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "parts", part_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_part_bindings",
        pool_path,
        "parts",
        part_id,
        &relative_path,
    )
}
