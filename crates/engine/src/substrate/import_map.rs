use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(test)]
use crate::ir::serialization::to_json_deterministic;

#[cfg(test)]
use super::EngineError;
use super::{
    DomainObject, ImportKey, ImportMapEntry, ObjectId, ResolveDiagnostic, SourceShardDirtyState,
    SourceShardKind, SourceShardRef, read_json_value, sha256_hex, source_shard_authority_for_kind,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportMapShard {
    pub schema_version: u64,
    pub entries: Vec<ImportMapEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportIdentityAllocation {
    pub import_key: ImportKey,
    pub object_id: ObjectId,
    pub reused_existing: bool,
}

pub fn allocate_import_identity(
    import_map: &BTreeMap<ImportKey, ImportMapEntry>,
    import_key: impl Into<ImportKey>,
) -> ImportIdentityAllocation {
    let import_key = import_key.into();
    if let Some(entry) = import_map.get(&import_key) {
        return ImportIdentityAllocation {
            import_key,
            object_id: entry.object_id,
            reused_existing: true,
        };
    }
    let object_id = Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum-eda:import-object:{import_key}").as_bytes(),
    );
    ImportIdentityAllocation {
        import_key,
        object_id,
        reused_existing: false,
    }
}

/// Legacy/test fixture writer for pre-journal import-map sidecars.
///
/// Production import-map creation must use `Operation::CreateImportMapShard`
/// through a journaled `OperationBatch`.
#[cfg(test)]
pub(super) fn write_legacy_import_map_sidecar(
    project_root: impl AsRef<Path>,
    shard_name: &str,
    mut entries: Vec<ImportMapEntry>,
) -> Result<PathBuf, EngineError> {
    validate_import_map_shard_name(shard_name)?;
    entries.sort_by(|left, right| left.import_key.cmp(&right.import_key));
    let shard = ImportMapShard {
        schema_version: 1,
        entries,
    };
    let directory = project_root.as_ref().join(".datum/import_map");
    std::fs::create_dir_all(&directory)?;
    let path = directory.join(shard_name);
    let temp_path = directory.join(format!("{shard_name}.tmp"));
    let json = to_json_deterministic(&shard)?;
    std::fs::write(&temp_path, format!("{json}\n").as_bytes())?;
    std::fs::File::open(&temp_path)?.sync_all()?;
    std::fs::rename(&temp_path, &path)?;
    std::fs::File::open(&directory)?.sync_all()?;
    Ok(path)
}

#[cfg(test)]
fn validate_import_map_shard_name(shard_name: &str) -> Result<(), EngineError> {
    if shard_name.is_empty()
        || shard_name.contains('/')
        || shard_name.contains('\\')
        || shard_name == "."
        || shard_name == ".."
        || !shard_name.ends_with(".json")
    {
        return Err(EngineError::Validation(format!(
            "invalid import map shard name {shard_name:?}"
        )));
    }
    Ok(())
}

pub(super) fn read_import_map_shards(
    project_root: &Path,
    objects: &BTreeMap<ObjectId, DomainObject>,
    import_map: &mut BTreeMap<ImportKey, ImportMapEntry>,
) -> (Vec<SourceShardRef>, Vec<ResolveDiagnostic>) {
    let import_map_dir = project_root.join(".datum/import_map");
    let mut shards = Vec::new();
    let mut diagnostics = Vec::new();
    let Ok(entries) = std::fs::read_dir(&import_map_dir) else {
        return (shards, diagnostics);
    };
    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();

    let mut sidecar_keys = BTreeSet::new();
    for path in paths {
        let Some(filename) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let relative_path = format!(".datum/import_map/{filename}");
        let path = project_root.join(&relative_path);
        match read_import_map_shard(path, relative_path) {
            Ok((shard, import_shard)) => {
                for entry in import_shard.entries {
                    validate_and_insert_import_map_entry(
                        &shard,
                        entry,
                        objects,
                        import_map,
                        &mut sidecar_keys,
                        &mut diagnostics,
                    );
                }
                shards.push(shard);
            }
            Err(error) => diagnostics.push(error),
        }
    }

    (shards, diagnostics)
}

fn read_import_map_shard(
    path: PathBuf,
    relative_path: String,
) -> Result<(SourceShardRef, ImportMapShard), ResolveDiagnostic> {
    let bytes = std::fs::read(&path).map_err(|error| ResolveDiagnostic {
        code: "missing_import_map".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let value = read_json_value(&path).map_err(|error| ResolveDiagnostic {
        code: "invalid_import_map".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let schema_version = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64);
    let shard = SourceShardRef {
        shard_id: Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("datum-eda:source-shard:{relative_path}").as_bytes(),
        ),
        kind: SourceShardKind::ImportMap,
        path,
        relative_path,
        authority: source_shard_authority_for_kind(&SourceShardKind::ImportMap),
        dirty_state: SourceShardDirtyState::Clean,
        schema_version,
        content_hash: sha256_hex(&bytes),
    };
    let import_shard =
        serde_json::from_value::<ImportMapShard>(value).map_err(|error| ResolveDiagnostic {
            code: "invalid_import_map".to_string(),
            message: error.to_string(),
            path: Some(shard.path.clone()),
        })?;
    Ok((shard, import_shard))
}

fn validate_and_insert_import_map_entry(
    shard: &SourceShardRef,
    entry: ImportMapEntry,
    objects: &BTreeMap<ObjectId, DomainObject>,
    import_map: &mut BTreeMap<ImportKey, ImportMapEntry>,
    sidecar_keys: &mut BTreeSet<ImportKey>,
    diagnostics: &mut Vec<ResolveDiagnostic>,
) {
    if entry.import_key.trim().is_empty() {
        diagnostics.push(ResolveDiagnostic {
            code: "invalid_import_map".to_string(),
            message: "import map entry has an empty import_key".to_string(),
            path: Some(shard.path.clone()),
        });
        return;
    }
    if entry.source_tool.trim().is_empty()
        && (!entry.source_path.trim().is_empty() || !entry.source_object_ref.trim().is_empty())
    {
        diagnostics.push(ResolveDiagnostic {
            code: "invalid_import_map".to_string(),
            message: format!(
                "import map key {} has source provenance without source_tool",
                entry.import_key
            ),
            path: Some(shard.path.clone()),
        });
        return;
    }
    if !sidecar_keys.insert(entry.import_key.clone()) {
        diagnostics.push(ResolveDiagnostic {
            code: "import_map_duplicate_key".to_string(),
            message: format!("duplicate import map key {}", entry.import_key),
            path: Some(shard.path.clone()),
        });
        return;
    }
    let Some(object) = objects.get(&entry.object_id) else {
        diagnostics.push(ResolveDiagnostic {
            code: "import_map_missing_object".to_string(),
            message: format!(
                "import map key {} references missing object {}",
                entry.import_key, entry.object_id
            ),
            path: Some(shard.path.clone()),
        });
        return;
    };
    if object.source_shard_id != entry.source_shard_id {
        diagnostics.push(ResolveDiagnostic {
            code: "import_map_object_shard_mismatch".to_string(),
            message: format!(
                "import map key {} references object {} on shard {}, but resolver found shard {}",
                entry.import_key, entry.object_id, entry.source_shard_id, object.source_shard_id
            ),
            path: Some(shard.path.clone()),
        });
        return;
    }
    if let Some(existing) = import_map.get(&entry.import_key)
        && existing.object_id != entry.object_id
    {
        diagnostics.push(ResolveDiagnostic {
            code: "import_map_conflict".to_string(),
            message: format!(
                "import map key {} maps to both {} and {}",
                entry.import_key, existing.object_id, entry.object_id
            ),
            path: Some(shard.path.clone()),
        });
        return;
    }
    import_map.insert(entry.import_key.clone(), entry);
}
