use std::path::Path;

use uuid::Uuid;

use super::source_shard::{
    dirty_state_for_materialized_shard, source_shard_taxon_for_path,
    validate_source_shard_ownership_path, validate_source_shard_schema_version,
};
use super::{
    EngineError, ResolveDiagnostic, SourceShardDirtyState, SourceShardKind, SourceShardRef,
    source_shard_authority_for_kind,
};

pub(super) fn source_shard_ref_for_value(
    project_root: &Path,
    kind: SourceShardKind,
    relative_path: String,
    value: &serde_json::Value,
) -> Result<SourceShardRef, EngineError> {
    validate_source_shard_ownership_path(&kind, &relative_path)?;
    let schema_version = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64);
    validate_source_shard_schema_version(&kind, &relative_path, schema_version)?;
    let bytes = format!(
        "{}\n",
        crate::ir::serialization::to_json_deterministic(value)?
    );
    let content_hash = super::sha256_hex(bytes.as_bytes());
    Ok(SourceShardRef {
        shard_id: Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("datum-eda:source-shard:{relative_path}").as_bytes(),
        ),
        kind: kind.clone(),
        taxon: source_shard_taxon_for_path(&kind, &relative_path),
        path: project_root.join(&relative_path),
        relative_path: relative_path.clone(),
        authority: source_shard_authority_for_kind(&kind),
        dirty_state: dirty_state_for_materialized_shard(
            project_root,
            &relative_path,
            &content_hash,
        ),
        schema_version,
        content_hash,
    })
}

pub(super) fn source_shard_ref_for_bytes(
    kind: SourceShardKind,
    path: std::path::PathBuf,
    relative_path: String,
    schema_version: Option<u64>,
    bytes: &[u8],
    diagnostic_code: &str,
) -> Result<SourceShardRef, ResolveDiagnostic> {
    validate_source_shard_ownership_path(&kind, &relative_path).map_err(|error| {
        ResolveDiagnostic {
            code: diagnostic_code.to_string(),
            message: error.to_string(),
            path: Some(path.clone()),
        }
    })?;
    validate_source_shard_schema_version(&kind, &relative_path, schema_version).map_err(
        |error| ResolveDiagnostic {
            code: diagnostic_code.to_string(),
            message: error.to_string(),
            path: Some(path.clone()),
        },
    )?;
    Ok(SourceShardRef {
        shard_id: Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("datum-eda:source-shard:{relative_path}").as_bytes(),
        ),
        authority: source_shard_authority_for_kind(&kind),
        taxon: source_shard_taxon_for_path(&kind, &relative_path),
        kind,
        path,
        relative_path,
        dirty_state: SourceShardDirtyState::Clean,
        schema_version,
        content_hash: super::sha256_hex(bytes),
    })
}

pub(super) fn source_shard_ref_for_staged_write(
    kind: SourceShardKind,
    path: std::path::PathBuf,
    relative_path: String,
    schema_version: Option<u64>,
    content_hash: String,
) -> Result<SourceShardRef, EngineError> {
    validate_source_shard_ownership_path(&kind, &relative_path)?;
    validate_source_shard_schema_version(&kind, &relative_path, schema_version)?;
    Ok(SourceShardRef {
        shard_id: Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("datum-eda:source-shard:{relative_path}").as_bytes(),
        ),
        authority: source_shard_authority_for_kind(&kind),
        taxon: source_shard_taxon_for_path(&kind, &relative_path),
        kind,
        path,
        relative_path,
        dirty_state: SourceShardDirtyState::Clean,
        schema_version,
        content_hash,
    })
}
