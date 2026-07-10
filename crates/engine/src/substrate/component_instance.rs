use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ComponentInstance, ComponentInstanceAuthority, ComponentInstanceId,
    ComponentInstanceRoleMetadata, DomainObject, EngineError, LibraryBinding, LibraryBindingRole,
    ObjectId, ObjectRevision, ResolveDiagnostic, RevisionedRef, SourceShardKind, SourceShardRef,
    TransactionRecord, read_json_value, source_shard::validate_source_shard_schema_version,
    source_shard_ref_builders::source_shard_ref_for_bytes,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentInstanceShard {
    #[serde(default = "default_component_instance_shard_schema_version")]
    pub schema_version: u64,
    pub component_instance: PersistedComponentInstance,
}

pub const COMPONENT_INSTANCE_SHARD_SCHEMA_VERSION: u64 = 1;

fn default_component_instance_shard_schema_version() -> u64 {
    COMPONENT_INSTANCE_SHARD_SCHEMA_VERSION
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersistedComponentInstance {
    pub uuid: ComponentInstanceId,
    pub object_revision: ObjectRevision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part_ref: Option<RevisionedRef>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub library_bindings: BTreeMap<ObjectId, LibraryBinding>,
    pub placed_symbol_refs: Vec<RevisionedRef>,
    pub placed_package_refs: Vec<RevisionedRef>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub placed_symbol_roles: BTreeMap<ObjectId, ComponentInstanceRoleMetadata>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub placed_package_roles: BTreeMap<ObjectId, ComponentInstanceRoleMetadata>,
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
    let placed_symbol_refs = persisted
        .placed_symbol_refs
        .iter()
        .map(|reference| reference.object_id)
        .collect::<Vec<_>>();
    let placed_package_refs = persisted
        .placed_package_refs
        .iter()
        .map(|reference| reference.object_id)
        .collect::<Vec<_>>();
    ComponentInstance {
        id: persisted.uuid,
        object_revision: persisted.object_revision,
        authority: ComponentInstanceAuthority::Authored,
        part_ref: persisted
            .part_ref
            .as_ref()
            .map(|reference| reference.object_id)
            .or_else(|| part_binding(persisted).map(|binding| binding.target_object_id)),
        library_bindings: persisted.library_bindings.clone(),
        placed_symbol_roles: persisted.placed_symbol_roles.clone(),
        placed_package_roles: persisted.placed_package_roles.clone(),
        placed_symbol_refs,
        placed_package_refs,
    }
}

fn part_binding(persisted: &PersistedComponentInstance) -> Option<&LibraryBinding> {
    persisted
        .library_bindings
        .values()
        .find(|binding| binding.binding_role == LibraryBindingRole::Part)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ComponentJoinKey {
    reference: String,
    part: Uuid,
}

pub(super) fn collect_component_instances(
    shards: &[SourceShardRef],
    journal: &[TransactionRecord],
    objects: &BTreeMap<ObjectId, DomainObject>,
    persisted_instances: BTreeMap<ComponentInstanceId, ComponentInstance>,
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
    validate_source_shard_schema_version(
        &SourceShardKind::ComponentInstance,
        &relative_path,
        schema_version,
    )
    .map_err(|error| ResolveDiagnostic {
        code: "invalid_component_instance_shard".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let shard = source_shard_ref_for_bytes(
        SourceShardKind::ComponentInstance,
        path,
        relative_path,
        schema_version,
        &bytes,
        "invalid_component_instance_shard",
    )?;
    let component_instance_shard = serde_json::from_value::<ComponentInstanceShard>(value)
        .map_err(|error| ResolveDiagnostic {
            code: "invalid_component_instance_shard".to_string(),
            message: error.to_string(),
            path: Some(shard.path.clone()),
        })?;
    if component_instance_shard.schema_version != COMPONENT_INSTANCE_SHARD_SCHEMA_VERSION {
        return Err(ResolveDiagnostic {
            code: "invalid_component_instance_shard".to_string(),
            message: format!(
                "unsupported ComponentInstanceShard schema_version {}",
                component_instance_shard.schema_version
            ),
            path: Some(shard.path.clone()),
        });
    }
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
    if input.placed_symbol_refs.is_empty() {
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_empty_refs".to_string(),
            message: format!(
                "component instance {} must reference at least one symbol",
                input.uuid
            ),
            path: Some(shard.path.clone()),
        });
        return;
    }
    if !refs_match_objects(&input.placed_symbol_refs, objects)
        || (!input.placed_package_refs.is_empty()
            && !refs_match_objects(&input.placed_package_refs, objects))
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
    if let Err(message) = validate_role_map(
        &input.placed_symbol_refs,
        &input.placed_symbol_roles,
        "symbol",
    ) {
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_invalid_symbol_roles".to_string(),
            message: format!("component instance {} {message}", input.uuid),
            path: Some(shard.path.clone()),
        });
        return;
    }
    if let Err(message) = validate_role_map(
        &input.placed_package_refs,
        &input.placed_package_roles,
        "package",
    ) {
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_invalid_package_roles".to_string(),
            message: format!("component instance {} {message}", input.uuid),
            path: Some(shard.path.clone()),
        });
        return;
    }
    if let Some(part_ref) = &input.part_ref
        && !revisioned_pool_ref_matches(
            part_ref.object_id,
            part_ref.object_revision,
            objects,
            "parts",
        ) {
            diagnostics.push(ResolveDiagnostic {
                code: "component_instance_invalid_part_ref".to_string(),
                message: format!(
                    "component instance {} part_ref {} must target a current pool parts object",
                    input.uuid, part_ref.object_id
                ),
                path: Some(shard.path.clone()),
            });
            return;
        }
    if let Err(message) = validate_library_bindings(&input, objects) {
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_invalid_library_binding".to_string(),
            message: format!("component instance {} {message}", input.uuid),
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

fn validate_library_bindings(
    input: &PersistedComponentInstance,
    objects: &BTreeMap<ObjectId, DomainObject>,
) -> Result<(), String> {
    let mut part_binding = None;
    for (binding_id, binding) in &input.library_bindings {
        if *binding_id == Uuid::nil() || *binding_id == binding.target_object_id {
            return Err(format!(
                "library binding {binding_id} has invalid binding identity"
            ));
        }
        let expected_kind = match binding.binding_role {
            LibraryBindingRole::Part => "parts",
            LibraryBindingRole::Symbol => "symbols",
            LibraryBindingRole::Package => "packages",
            LibraryBindingRole::Footprint => "footprints",
            LibraryBindingRole::PinPadMap => "pin_pad_maps",
            LibraryBindingRole::ModelAttachment => "models",
        };
        if !revisioned_pool_ref_matches(
            binding.target_object_id,
            binding.pinned_object_revision,
            objects,
            expected_kind,
        ) {
            return Err(format!(
                "library binding {binding_id} target {} must resolve to current pool {expected_kind}",
                binding.target_object_id
            ));
        }
        for reference in &binding.local_override_refs {
            if !revisioned_pool_ref_matches(
                reference.object_id,
                reference.object_revision,
                objects,
                expected_kind,
            ) {
                return Err(format!(
                    "library binding {binding_id} local override {} must resolve to current pool {expected_kind}",
                    reference.object_id
                ));
            }
        }
        if binding.binding_role == LibraryBindingRole::Part
            && part_binding.replace(binding).is_some()
        {
            return Err("must not contain multiple part LibraryBindings".to_string());
        }
    }
    if let (Some(part_ref), Some(part_binding)) = (&input.part_ref, part_binding)
        && (part_ref.object_id != part_binding.target_object_id
            || part_ref.object_revision != part_binding.pinned_object_revision)
        {
            return Err(format!(
                "part_ref {}@{} does not match part LibraryBinding {}@{}",
                part_ref.object_id,
                part_ref.object_revision.0,
                part_binding.target_object_id,
                part_binding.pinned_object_revision.0
            ));
        }
    Ok(())
}

pub(super) fn validate_role_map(
    refs: &[RevisionedRef],
    roles: &BTreeMap<ObjectId, ComponentInstanceRoleMetadata>,
    label: &str,
) -> Result<(), String> {
    let ref_ids = refs
        .iter()
        .map(|reference| reference.object_id)
        .collect::<Vec<_>>();
    for (object_id, metadata) in roles {
        if metadata.role.trim().is_empty() {
            return Err(format!("{label} role for {object_id} must not be blank"));
        }
        if !metadata
            .role
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-')
            || metadata.role.len() > 64
        {
            return Err(format!(
                "{label} role for {object_id} must be a lowercase ASCII identifier"
            ));
        }
        if let Some(role_label) = &metadata.label
            && (role_label.trim().is_empty()
                || role_label.len() > 128
                || role_label.chars().any(char::is_control))
            {
                return Err(format!("{label} role label for {object_id} is invalid"));
            }
        if !ref_ids.contains(object_id) {
            return Err(format!(
                "{label} role for {object_id} must reference a placed {label}"
            ));
        }
    }
    Ok(())
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
        read_json_value(&shard.path).ok()
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
        revisioned_object_matches(reference.object_id, reference.object_revision, objects)
    })
}

fn revisioned_object_matches(
    object_id: ObjectId,
    object_revision: ObjectRevision,
    objects: &BTreeMap<ObjectId, DomainObject>,
) -> bool {
    objects
        .get(&object_id)
        .map(|object| object.object_revision == object_revision)
        .unwrap_or(false)
}

fn revisioned_pool_ref_matches(
    object_id: ObjectId,
    object_revision: ObjectRevision,
    objects: &BTreeMap<ObjectId, DomainObject>,
    expected_kind: &str,
) -> bool {
    objects
        .get(&object_id)
        .map(|object| {
            object.object_revision == object_revision
                && object.domain == "pool"
                && object.kind == expected_kind
        })
        .unwrap_or(false)
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
