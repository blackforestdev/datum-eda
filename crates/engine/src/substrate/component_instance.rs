use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ComponentInstance, ComponentInstanceId, DomainObject, EngineError, ObjectId, ObjectRevision,
    ResolveDiagnostic, RevisionedRef, SourceShardDirtyState, SourceShardKind, SourceShardRef,
    TransactionRecord, read_json_value, sha256_hex, source_shard_authority_for_kind,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentInstanceShard {
    pub schema_version: u64,
    pub component_instance: PersistedComponentInstance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersistedComponentInstance {
    pub uuid: ComponentInstanceId,
    pub object_revision: ObjectRevision,
    pub placed_symbol_refs: Vec<RevisionedRef>,
    pub placed_package_refs: Vec<RevisionedRef>,
}

pub(super) fn persisted_component_instance_from_value(
    value: &serde_json::Value,
) -> Result<ComponentInstance, EngineError> {
    let persisted: PersistedComponentInstance = serde_json::from_value(value.clone())?;
    Ok(component_instance_from_persisted(&persisted))
}

pub(super) fn component_instance_from_persisted(
    persisted: &PersistedComponentInstance,
) -> ComponentInstance {
    ComponentInstance {
        id: persisted.uuid,
        object_revision: persisted.object_revision,
        placed_symbol_refs: persisted
            .placed_symbol_refs
            .iter()
            .map(|reference| reference.object_id)
            .collect(),
        placed_package_refs: persisted
            .placed_package_refs
            .iter()
            .map(|reference| reference.object_id)
            .collect(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ComponentJoinKey {
    reference: String,
    part: Uuid,
}

pub(super) fn collect_component_instances(
    project_id: &Uuid,
    shards: &[SourceShardRef],
    journal: &[TransactionRecord],
    objects: &BTreeMap<ObjectId, DomainObject>,
    mut persisted_instances: BTreeMap<ComponentInstanceId, ComponentInstance>,
    diagnostics: &mut Vec<ResolveDiagnostic>,
) -> Result<BTreeMap<ComponentInstanceId, ComponentInstance>, EngineError> {
    let mut covered_symbol_refs = Vec::new();
    let mut covered_package_refs = Vec::new();
    for instance in persisted_instances.values() {
        covered_symbol_refs.extend(instance.placed_symbol_refs.iter().copied());
        covered_package_refs.extend(instance.placed_package_refs.iter().copied());
    }
    covered_symbol_refs.sort();
    covered_symbol_refs.dedup();
    covered_package_refs.sort();
    covered_package_refs.dedup();

    let mut symbols = BTreeMap::<ComponentJoinKey, Vec<ObjectId>>::new();
    let mut packages = BTreeMap::<ComponentJoinKey, Vec<ObjectId>>::new();

    for shard in shards {
        match shard.kind {
            SourceShardKind::SchematicSheet => {
                collect_symbol_join_keys(&mut symbols, shard, journal)?;
            }
            SourceShardKind::BoardRoot => {
                collect_package_join_keys(&mut packages, shard)?;
            }
            _ => {}
        }
    }

    for (key, symbol_ids) in &symbols {
        if symbol_ids
            .iter()
            .all(|symbol_id| covered_symbol_refs.contains(symbol_id))
        {
            continue;
        }
        let Some(package_ids) = packages.get(key) else {
            diagnostics.push(ResolveDiagnostic {
                code: "component_instance_unmatched_symbol".to_string(),
                message: format!(
                    "schematic component {} / part {} has no matching board package",
                    key.reference, key.part
                ),
                path: None,
            });
            continue;
        };
        let placed_symbol_refs =
            uncovered_live_object_refs(symbol_ids.clone(), objects, &covered_symbol_refs);
        let placed_package_refs =
            uncovered_live_object_refs(package_ids.clone(), objects, &covered_package_refs);
        if placed_symbol_refs.is_empty() || placed_package_refs.is_empty() {
            continue;
        }
        if placed_symbol_refs.len() != 1 || placed_package_refs.len() != 1 {
            diagnostics.push(ResolveDiagnostic {
                code: "component_instance_ambiguous_join".to_string(),
                message: format!(
                    "component {} / part {} resolves to {} schematic symbols and {} board packages; persisted ComponentInstance refs are required",
                    key.reference,
                    key.part,
                    placed_symbol_refs.len(),
                    placed_package_refs.len()
                ),
                path: None,
            });
            continue;
        }
        let id = Uuid::new_v5(
            project_id,
            format!(
                "datum-eda:component-instance:{}:{}",
                placed_symbol_refs
                    .iter()
                    .map(Uuid::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
                placed_package_refs
                    .iter()
                    .map(Uuid::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            )
            .as_bytes(),
        );
        persisted_instances.insert(
            id,
            ComponentInstance {
                id,
                object_revision: ObjectRevision(0),
                placed_symbol_refs,
                placed_package_refs,
            },
        );
    }
    for (key, package_ids) in &packages {
        if symbols.contains_key(key) {
            continue;
        }
        if uncovered_live_object_refs(package_ids.clone(), objects, &covered_package_refs)
            .is_empty()
        {
            continue;
        }
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_unmatched_package".to_string(),
            message: format!(
                "board package {} / part {} has no matching schematic component",
                key.reference, key.part
            ),
            path: None,
        });
    }

    Ok(persisted_instances)
}

pub(super) fn read_component_instance_shards(
    project_root: &Path,
    objects: &mut BTreeMap<ObjectId, DomainObject>,
) -> (
    Vec<SourceShardRef>,
    BTreeMap<ComponentInstanceId, ComponentInstance>,
    Vec<ResolveDiagnostic>,
) {
    let dir = project_root.join(".datum/component_instances");
    let mut shards = Vec::new();
    let mut instances = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return (shards, instances, diagnostics);
    };
    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();

    for path in paths {
        let Some(filename) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let relative_path = format!(".datum/component_instances/{filename}");
        let path = project_root.join(&relative_path);
        match read_component_instance_shard(path, relative_path) {
            Ok((shard, component_instance_shard)) => {
                insert_component_instance(
                    &shard,
                    component_instance_shard.component_instance,
                    objects,
                    &mut instances,
                    &mut diagnostics,
                );
                shards.push(shard);
            }
            Err(error) => diagnostics.push(error),
        }
    }

    (shards, instances, diagnostics)
}

fn read_component_instance_shard(
    path: PathBuf,
    relative_path: String,
) -> Result<(SourceShardRef, ComponentInstanceShard), ResolveDiagnostic> {
    let bytes = std::fs::read(&path).map_err(|error| ResolveDiagnostic {
        code: "missing_component_instance_shard".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let value = read_json_value(&path).map_err(|error| ResolveDiagnostic {
        code: "invalid_component_instance_shard".to_string(),
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
        kind: SourceShardKind::ComponentInstance,
        path,
        relative_path,
        authority: source_shard_authority_for_kind(&SourceShardKind::ComponentInstance),
        dirty_state: SourceShardDirtyState::Clean,
        schema_version,
        content_hash: sha256_hex(&bytes),
    };
    let component_instance_shard = serde_json::from_value::<ComponentInstanceShard>(value)
        .map_err(|error| ResolveDiagnostic {
            code: "invalid_component_instance_shard".to_string(),
            message: error.to_string(),
            path: Some(shard.path.clone()),
        })?;
    Ok((shard, component_instance_shard))
}

fn insert_component_instance(
    shard: &SourceShardRef,
    input: PersistedComponentInstance,
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    instances: &mut BTreeMap<ComponentInstanceId, ComponentInstance>,
    diagnostics: &mut Vec<ResolveDiagnostic>,
) {
    let Some(filename_id) = shard
        .path
        .file_stem()
        .and_then(|value| value.to_str())
        .and_then(|value| Uuid::parse_str(value).ok())
    else {
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_invalid_filename".to_string(),
            message: format!(
                "component instance shard filename must be <uuid>.json: {}",
                shard.relative_path
            ),
            path: Some(shard.path.clone()),
        });
        return;
    };
    if filename_id != input.uuid {
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_filename_mismatch".to_string(),
            message: format!(
                "component instance shard filename {} does not match embedded uuid {}",
                filename_id, input.uuid
            ),
            path: Some(shard.path.clone()),
        });
        return;
    }
    if instances.contains_key(&input.uuid) || objects.contains_key(&input.uuid) {
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_duplicate_id".to_string(),
            message: format!("duplicate component instance id {}", input.uuid),
            path: Some(shard.path.clone()),
        });
        return;
    }
    if input.placed_symbol_refs.is_empty() || input.placed_package_refs.is_empty() {
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_empty_refs".to_string(),
            message: format!(
                "component instance {} must reference at least one symbol and one board package",
                input.uuid
            ),
            path: Some(shard.path.clone()),
        });
        return;
    }
    if !refs_match_objects(&input.placed_symbol_refs, objects)
        || !refs_match_objects(&input.placed_package_refs, objects)
    {
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_unresolved_ref".to_string(),
            message: format!(
                "component instance {} references missing or stale symbol/package objects",
                input.uuid
            ),
            path: Some(shard.path.clone()),
        });
        return;
    }
    objects.insert(
        input.uuid,
        DomainObject {
            object_id: input.uuid,
            object_revision: input.object_revision,
            source_shard_id: shard.shard_id,
            domain: "component_instance".to_string(),
            kind: "component_instance".to_string(),
        },
    );
    instances.insert(input.uuid, component_instance_from_persisted(&input));
}

fn collect_symbol_join_keys(
    output: &mut BTreeMap<ComponentJoinKey, Vec<ObjectId>>,
    shard: &SourceShardRef,
    journal: &[TransactionRecord],
) -> Result<(), EngineError> {
    let value = materialized_schematic_sheet_value(shard, journal)?;
    let Some(symbols) = value.get("symbols").and_then(serde_json::Value::as_object) else {
        return Ok(());
    };
    for symbol in symbols.values() {
        let Some(id) = value_uuid(symbol, "uuid") else {
            continue;
        };
        let Some(reference) = value_string(symbol, "reference") else {
            continue;
        };
        let Some(part) = value_uuid(symbol, "part") else {
            continue;
        };
        output
            .entry(ComponentJoinKey { reference, part })
            .or_default()
            .push(id);
    }
    Ok(())
}

fn materialized_schematic_sheet_value(
    shard: &SourceShardRef,
    journal: &[TransactionRecord],
) -> Result<serde_json::Value, EngineError> {
    let mut value = if shard.path.exists() {
        Some(read_json_value(&shard.path)?)
    } else {
        None
    };
    for transaction in journal {
        for operation in &transaction.operations {
            match operation {
                super::Operation::CreateSchematicSheet {
                    relative_path,
                    sheet,
                    ..
                } if format!("schematic/{relative_path}") == shard.relative_path => {
                    value = Some(sheet.clone());
                }
                super::Operation::DeleteSchematicSheet { relative_path, .. }
                    if format!("schematic/{relative_path}") == shard.relative_path =>
                {
                    value = None;
                }
                _ => {}
            }
        }
    }
    value.ok_or_else(|| {
        EngineError::Validation(format!(
            "missing schematic sheet shard {} has no journal materialization",
            shard.relative_path
        ))
    })
}

fn collect_package_join_keys(
    output: &mut BTreeMap<ComponentJoinKey, Vec<ObjectId>>,
    shard: &SourceShardRef,
) -> Result<(), EngineError> {
    let value = read_json_value(&shard.path)?;
    let Some(packages) = value.get("packages").and_then(serde_json::Value::as_object) else {
        return Ok(());
    };
    for package in packages.values() {
        let Some(id) = value_uuid(package, "uuid") else {
            continue;
        };
        let Some(reference) = value_string(package, "reference") else {
            continue;
        };
        let Some(part) = value_uuid(package, "part") else {
            continue;
        };
        output
            .entry(ComponentJoinKey { reference, part })
            .or_default()
            .push(id);
    }
    Ok(())
}

fn live_object_refs(
    mut ids: Vec<ObjectId>,
    objects: &BTreeMap<ObjectId, DomainObject>,
) -> Vec<ObjectId> {
    ids.retain(|id| objects.contains_key(id));
    ids.sort();
    ids.dedup();
    ids
}

fn uncovered_live_object_refs(
    ids: Vec<ObjectId>,
    objects: &BTreeMap<ObjectId, DomainObject>,
    covered_refs: &[ObjectId],
) -> Vec<ObjectId> {
    let mut refs = live_object_refs(ids, objects);
    refs.retain(|id| !covered_refs.contains(id));
    refs
}

fn refs_match_objects(refs: &[RevisionedRef], objects: &BTreeMap<ObjectId, DomainObject>) -> bool {
    refs.iter().all(|reference| {
        objects
            .get(&reference.object_id)
            .map(|object| object.object_revision == reference.object_revision)
            .unwrap_or(false)
    })
}

fn value_uuid(value: &serde_json::Value, key: &str) -> Option<Uuid> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .and_then(|text| Uuid::parse_str(text).ok())
}

fn value_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}
