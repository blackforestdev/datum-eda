use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::api::native_write::library::{
    PoolLibraryObjectTarget, PoolLibraryOperationSpec, pool_package_payload,
};
use eda_engine::substrate::ProjectResolver;
use uuid::Uuid;

use super::library::{
    NativeProjectPoolLibraryObjectMutationView, commit_pool_library_operations,
    pool_library_mutation_view, pool_library_relative_path, validate_project_local_pool_path,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn create_native_project_pool_package(
    root: &Path,
    pool_path: &str,
    package_id: Uuid,
    name: String,
    pad_id: Option<Uuid>,
    padstack_id: Option<Uuid>,
    pad_name: String,
    x_nm: i64,
    y_nm: i64,
    layer: i32,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let legacy_pad = match (pad_id, padstack_id) {
        (Some(pad_id), Some(padstack_id)) => Some((pad_id, padstack_id)),
        (Some(_), None) | (None, Some(_)) => bail!(
            "legacy package pad compatibility requires both --pad and --padstack; prefer create-pool-footprint for land-pattern pads"
        ),
        (None, None) => None,
    };
    if legacy_pad.is_some() && layer <= 0 {
        bail!("package pad layer must be positive");
    }
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    if let Some((_, padstack_id)) = legacy_pad
        && model
            .objects
            .get(&padstack_id)
            .filter(|object| object.domain == "pool" && object.kind == "padstacks")
            .is_none()
    {
        bail!("missing pool padstack {padstack_id} for package {package_id}");
    }
    let relative_path = pool_library_relative_path(pool_path, "packages", package_id);
    let pads = legacy_pad
        .map(|(pad_id, padstack_id)| {
            serde_json::json!({
                pad_id.to_string(): {
                    "uuid": pad_id,
                    "name": pad_name,
                    "position": {"x": x_nm, "y": y_nm},
                    "padstack": padstack_id,
                    "layer": layer
                }
            })
        })
        .unwrap_or_else(|| serde_json::json!({}));
    let object = pool_package_payload(package_id, &name, pads);
    commit_pool_library_operations(
        root,
        format!("create native pool package {package_id}"),
        Some(pool_path),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new(pool_path, "packages", package_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "create_package",
        pool_path,
        "packages",
        package_id,
        &relative_path,
    )
}
