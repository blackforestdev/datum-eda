use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::substrate::ProjectResolver;
use uuid::Uuid;

pub(super) fn read_pool_library_object_payload(
    path: &Path,
    object_id: Uuid,
) -> Result<serde_json::Value> {
    let object: serde_json::Value = serde_json::from_slice(
        &std::fs::read(path)
            .with_context(|| format!("failed to read pool library object {}", path.display()))?,
    )
    .with_context(|| format!("failed to parse pool library object {}", path.display()))?;
    validate_pool_library_object_payload_id(&object, object_id)?;
    Ok(object)
}

pub(super) fn read_project_pool_object_payload(
    root: &Path,
    relative_path: &str,
    object_id: Uuid,
) -> Result<serde_json::Value> {
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let object = model
        .materialized_source_shard_value_by_relative_path(relative_path)
        .with_context(|| format!("failed to materialize pool library object {relative_path}"))?;
    validate_pool_library_object_payload_id(&object, object_id)?;
    Ok(object)
}

fn validate_pool_library_object_payload_id(
    object: &serde_json::Value,
    object_id: Uuid,
) -> Result<()> {
    let payload_id = object
        .get("uuid")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("pool library object missing uuid"))?;
    let payload_id = Uuid::parse_str(payload_id)
        .with_context(|| format!("invalid pool library object uuid {payload_id}"))?;
    if payload_id != object_id {
        bail!("pool library object uuid {payload_id} does not match --object {object_id}");
    }
    Ok(())
}
