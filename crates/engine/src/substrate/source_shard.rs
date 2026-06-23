use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;

use uuid::Uuid;

use super::{
    DomainObject, EngineError, ImportKey, ImportMapEntry, ObjectId, ResolveDiagnostic,
    SourceShardDirtyState, SourceShardKind, SourceShardRef, collect_uuid_objects,
    domain_for_shard_kind, sha256_hex, source_shard_authority_for_kind,
};

pub(super) fn read_source_shard(
    project_root: &Path,
    kind: SourceShardKind,
    relative_path: &str,
    value: Option<&serde_json::Value>,
) -> Result<SourceShardRef, EngineError> {
    let path = project_root.join(relative_path);
    let bytes = std::fs::read(&path)?;
    let parsed;
    let value = match value {
        Some(value) => value,
        None => {
            parsed = serde_json::from_slice::<serde_json::Value>(&bytes)?;
            &parsed
        }
    };
    let schema_version = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64);
    validate_source_shard_schema_version(&kind, relative_path, schema_version)?;
    Ok(SourceShardRef {
        shard_id: Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("datum-eda:source-shard:{relative_path}").as_bytes(),
        ),
        authority: source_shard_authority_for_kind(&kind),
        kind,
        path,
        relative_path: relative_path.to_string(),
        dirty_state: SourceShardDirtyState::Clean,
        schema_version,
        content_hash: sha256_hex(&bytes),
    })
}

pub(super) fn collect_referenced_shards(
    project_root: &Path,
    value: &serde_json::Value,
    _parent_shard: &SourceShardRef,
    shards: &mut Vec<SourceShardRef>,
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    import_map: &mut BTreeMap<ImportKey, ImportMapEntry>,
    diagnostics: &mut Vec<ResolveDiagnostic>,
) -> Result<(), EngineError> {
    let Some(object) = value.as_object() else {
        return Ok(());
    };
    for (key, kind) in [
        ("sheets", SourceShardKind::SchematicSheet),
        ("definitions", SourceShardKind::SchematicDefinition),
    ] {
        let Some(map) = object.get(key).and_then(serde_json::Value::as_object) else {
            continue;
        };
        for relative in map.values().filter_map(serde_json::Value::as_str) {
            let relative_path = format!("schematic/{relative}");
            let path = project_root.join(&relative_path);
            let bytes = match std::fs::read(&path) {
                Ok(bytes) => bytes,
                Err(error)
                    if kind == SourceShardKind::SchematicSheet
                        && error.kind() == ErrorKind::NotFound =>
                {
                    diagnostics.push(ResolveDiagnostic {
                        code: "missing_referenced_schematic_sheet".to_string(),
                        message: error.to_string(),
                        path: Some(path),
                    });
                    continue;
                }
                Err(error) => return Err(error.into()),
            };
            let value = serde_json::from_slice::<serde_json::Value>(&bytes)?;
            let schema_version = value
                .get("schema_version")
                .and_then(serde_json::Value::as_u64);
            let kind = kind.clone();
            validate_source_shard_schema_version(&kind, &relative_path, schema_version)?;
            shards.push(SourceShardRef {
                shard_id: Uuid::new_v5(
                    &Uuid::NAMESPACE_URL,
                    format!("datum-eda:source-shard:{relative_path}").as_bytes(),
                ),
                authority: source_shard_authority_for_kind(&kind),
                dirty_state: SourceShardDirtyState::Clean,
                kind,
                path,
                relative_path,
                schema_version,
                content_hash: sha256_hex(&bytes),
            });
            let shard = shards.last().expect("referenced shard was just pushed");
            collect_uuid_objects(
                &value,
                shard,
                domain_for_shard_kind(&shard.kind),
                objects,
                import_map,
            );
        }
    }
    Ok(())
}

fn validate_source_shard_schema_version(
    kind: &SourceShardKind,
    relative_path: &str,
    schema_version: Option<u64>,
) -> Result<(), EngineError> {
    const SUPPORTED_SCHEMA_VERSION: u64 = 1;
    if let Some(version) = schema_version
        && version > SUPPORTED_SCHEMA_VERSION
    {
        return Err(EngineError::Validation(format!(
            "unsupported {kind:?} schema_version {version} in {relative_path}; supported <= {SUPPORTED_SCHEMA_VERSION}"
        )));
    }
    Ok(())
}
