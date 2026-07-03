use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::api::native_write::library::{PoolLibraryObjectTarget, PoolLibraryOperationSpec};
use eda_engine::substrate::ProjectResolver;
use serde_json::Value;
use uuid::Uuid;

use super::command_project_library::{
    NativeProjectPoolLibraryObjectMutationView, commit_pool_library_operations,
    pool_library_mutation_view, validate_project_local_pool_path,
};
use super::command_project_library_payload::read_project_pool_object_payload;

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
    let (footprint_id, relative_path) =
        legacy_target_footprint_for_package(pool_path, package_id, &model)?;
    let previous_object = read_project_pool_object_payload(root, &relative_path, footprint_id)?;
    let mut object = previous_object.clone();
    let pads = object
        .as_object_mut()
        .context("pool footprint payload must be a JSON object")?
        .entry("pads")
        .or_insert_with(|| serde_json::json!({}));
    let pads = pads
        .as_object_mut()
        .context("pool footprint pads field must be an object")?;
    if pads.contains_key(&pad_id.to_string()) {
        bail!("pool footprint {footprint_id} already has pad {pad_id}");
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
        format!(
            "route legacy package pad {pad_id} on package {package_id} to footprint {footprint_id}"
        ),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::at_relative_path(
                footprint_id,
                "footprints",
                relative_path.clone(),
            ),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_package_pad",
        pool_path,
        "footprints",
        footprint_id,
        &relative_path,
    )
}

pub(super) fn legacy_target_footprint_for_package(
    pool_path: &str,
    package_id: Uuid,
    model: &eda_engine::substrate::DesignModel,
) -> Result<(Uuid, String)> {
    let prefix = format!("{pool_path}/footprints/");
    let mut candidates = model
        .objects
        .iter()
        .filter(|(_, object)| object.domain == "pool" && object.kind == "footprints")
        .filter_map(|(footprint_id, object)| {
            model
                .source_shards
                .iter()
                .find(|shard| shard.shard_id == object.source_shard_id)
                .filter(|shard| shard.relative_path.starts_with(&prefix))
                .map(|shard| (*footprint_id, shard.relative_path.clone()))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| left.1.cmp(&right.1));
    let mut matches = Vec::new();
    for (footprint_id, relative_path) in candidates {
        let footprint = model
            .materialized_source_shard_value_by_relative_path(&relative_path)
            .with_context(|| format!("failed to materialize {relative_path}"))?;
        if footprint.get("package").and_then(Value::as_str) == Some(package_id.to_string().as_str())
        {
            matches.push((footprint_id, relative_path));
        }
    }
    match matches.len() {
        1 => Ok(matches.remove(0)),
        0 => bail!(
            "legacy package pad compatibility requires one footprint for package {package_id}; create a footprint and use set-pool-footprint-pad for new land-pattern authoring"
        ),
        _ => bail!(
            "legacy package pad compatibility is ambiguous for package {package_id}; multiple footprints exist in pool {pool_path}"
        ),
    }
}
