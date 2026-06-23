use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::substrate::{Operation, ProjectResolver};
use uuid::Uuid;

use super::command_project_library::{
    NativeProjectPoolLibraryObjectMutationView, commit_pool_library_operations,
    pool_library_mutation_view, pool_library_relative_path, read_pool_library_object_payload,
    validate_project_local_pool_path,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn set_native_project_pool_package_pad(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    pad_id: Uuid,
    padstack_id: Uuid,
    pad_name: String,
    x_nm: i64,
    y_nm: i64,
    layer: i32,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    if layer <= 0 {
        bail!("package pad layer must be positive");
    }
    let pad_name = pad_name.trim().to_string();
    if pad_name.is_empty() {
        bail!("package pad name must not be empty");
    }
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    if model
        .objects
        .get(&padstack_id)
        .filter(|object| object.domain == "pool" && object.kind == "padstacks")
        .is_none()
    {
        bail!("missing pool padstack {padstack_id} for package {package_id}");
    }
    if model
        .objects
        .get(&package_id)
        .filter(|object| object.domain == "pool" && object.kind == "packages")
        .is_none()
    {
        bail!("missing pool package {package_id}");
    }
    let relative_path = pool_library_relative_path(pool_path, "packages", package_id);
    let previous_object = read_pool_library_object_payload(&root.join(&relative_path), package_id)?;
    let mut object = previous_object.clone();
    let pads = object
        .as_object_mut()
        .context("pool package payload must be a JSON object")?
        .entry("pads")
        .or_insert_with(|| serde_json::json!({}));
    let pads = pads
        .as_object_mut()
        .context("pool package pads field must be an object")?;
    if pads.contains_key(&pad_id.to_string()) {
        bail!("pool package {package_id} already has pad {pad_id}");
    }
    pads.insert(
        pad_id.to_string(),
        serde_json::json!({
            "uuid": pad_id,
            "name": pad_name,
            "position": {"x": x_nm, "y": y_nm},
            "padstack": padstack_id,
            "layer": layer
        }),
    );
    commit_pool_library_operations(
        root,
        format!("set native pool package pad {pad_id} on package {package_id}"),
        vec![Operation::SetPoolLibraryObject {
            object_id: package_id,
            relative_path: relative_path.clone(),
            object_kind: "packages".to_string(),
            previous_object,
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_package_pad",
        pool_path,
        "packages",
        package_id,
        &relative_path,
    )
}
