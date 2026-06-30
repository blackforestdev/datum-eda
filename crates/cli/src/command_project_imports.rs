use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use eda_engine::board::Board;
use eda_engine::import::eagle::import_library_file;
use eda_engine::import::ids_sidecar::compute_source_hash_file;
use eda_engine::import::kicad::{
    KiCadBoardImportIdentity, import_board_document_with_import_map_identities,
};
use eda_engine::pool::Pool;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, ImportMapEntry, ImportMapShard, Operation, OperationBatch,
    ProjectResolver, SourceShardKind,
};
use serde::Serialize;
use uuid::Uuid;

use super::NativeProjectManifest;
use super::command_project_imports_eagle_import_map::eagle_pool_import_map_entries;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectKiCadBoardImportView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: Uuid,
    pub(crate) source_path: String,
    pub(crate) import_map_path: String,
    pub(crate) imported_package_count: usize,
    pub(crate) imported_pad_count: usize,
    pub(crate) imported_track_count: usize,
    pub(crate) imported_via_count: usize,
    pub(crate) imported_zone_count: usize,
    pub(crate) import_map_entry_count: usize,
    pub(crate) created_object_count: usize,
    pub(crate) reused_existing_identity_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectEagleLibraryImportView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: Uuid,
    pub(crate) source_path: String,
    pub(crate) pool_path: String,
    pub(crate) import_map_path: String,
    pub(crate) imported_unit_count: usize,
    pub(crate) imported_symbol_count: usize,
    pub(crate) imported_entity_count: usize,
    pub(crate) imported_part_count: usize,
    pub(crate) imported_package_count: usize,
    pub(crate) imported_padstack_count: usize,
    pub(crate) import_map_entry_count: usize,
    pub(crate) created_object_count: usize,
    pub(crate) reused_existing_identity_count: usize,
}

pub(crate) fn import_native_project_kicad_board(
    root: &Path,
    source: &Path,
) -> Result<NativeProjectKiCadBoardImportView> {
    let before = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let board_shard_id = before
        .source_shards
        .iter()
        .find(|shard| shard.kind == SourceShardKind::BoardRoot)
        .map(|shard| shard.shard_id)
        .context("native project has no resolver-visible board root shard")?;
    let (board, _report, identities) =
        import_board_document_with_import_map_identities(source, &before.import_map)
            .with_context(|| format!("failed to import KiCad board {}", source.display()))?;
    let source_hash = compute_source_hash_file(source)?;
    let board_root = before
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .context("failed to materialize native board root before KiCad board import")?;
    let board_id = board_root
        .get("uuid")
        .and_then(|value| value.as_str())
        .ok_or_else(|| anyhow::anyhow!("native board root missing uuid before KiCad import"))?
        .parse::<Uuid>()
        .context("failed to parse native board uuid before KiCad import")?;
    let mut operations = board_import_create_operations(&before.objects, &board)?;
    let imported_outline = serde_json::to_value(&board.outline)?;
    if board_root.get("outline") != Some(&imported_outline) {
        operations.push(Operation::SetBoardOutline {
            board_id,
            outline: imported_outline,
        });
    }
    let imported_stackup = serde_json::to_value(&board.stackup)?;
    if board_root.get("stackup") != Some(&imported_stackup) {
        operations.push(Operation::SetBoardStackup {
            board_id,
            stackup: imported_stackup,
        });
    }
    let reused_existing_identity_count = identities
        .iter()
        .filter(|identity| before.import_map.contains_key(&identity.import_key))
        .count();
    let import_map_entries = board_import_map_entries(
        &before.import_map,
        &identities,
        board_shard_id,
        source,
        &source_hash,
    );
    let import_map_relative_path = board_import_map_relative_path(source);
    if !import_map_entries.is_empty() {
        operations.push(Operation::CreateImportMapShard {
            relative_path: import_map_relative_path.clone(),
            shard: serde_json::to_value(ImportMapShard {
                schema_version: 1,
                entries: import_map_entries.clone(),
            })?,
        });
    }
    let created_object_count = operations
        .iter()
        .filter(|operation| {
            matches!(
                operation,
                Operation::CreateBoardPackage { .. }
                    | Operation::CreateBoardPad { .. }
                    | Operation::CreateBoardTrack { .. }
                    | Operation::CreateBoardVia { .. }
                    | Operation::CreateBoardZone { .. }
                    | Operation::CreateBoardNet { .. }
            )
        })
        .count();
    if !operations.is_empty() {
        let mut model = before;
        model.commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "datum-eda-cli".to_string(),
                    source: CommitSource::Cli,
                    reason: format!("import KiCad board {}", source.display()),
                },
                operations,
            },
        )?;
    }
    let after_write = ProjectResolver::new(root).resolve().with_context(|| {
        format!(
            "failed to resolve imported board objects {}",
            root.display()
        )
    })?;
    for identity in &identities {
        after_write
            .objects
            .get(&identity.object_id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "imported board object {} ({}) was not resolver-visible",
                    identity.object_id,
                    identity.object_family
                )
            })?;
    }
    Ok(NativeProjectKiCadBoardImportView {
        contract: "native_project_kicad_board_import_v1",
        project_id: after_write.project.project_id,
        source_path: source.display().to_string(),
        import_map_path: root.join(&import_map_relative_path).display().to_string(),
        imported_package_count: board.packages.len(),
        imported_pad_count: board.pads.len(),
        imported_track_count: board.tracks.len(),
        imported_via_count: board.vias.len(),
        imported_zone_count: board.zones.len(),
        import_map_entry_count: import_map_entries.len(),
        created_object_count,
        reused_existing_identity_count,
    })
}

pub(crate) fn import_native_project_eagle_library(
    root: &Path,
    source: &Path,
    pool_path: &str,
) -> Result<NativeProjectEagleLibraryImportView> {
    validate_project_local_pool_path(pool_path)?;
    let before = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let project_manifest: NativeProjectManifest = serde_json::from_value(
        before
            .materialized_source_shard_value(SourceShardKind::ProjectManifest)
            .context("failed to materialize project manifest")?,
    )
    .context("failed to parse resolver-materialized project manifest")?;
    let (pool, _report) = import_library_file(source)
        .with_context(|| format!("failed to import Eagle library {}", source.display()))?;
    let source_hash = compute_source_hash_file(source)?;
    let mut operations = Vec::new();
    if !project_manifest
        .pools
        .iter()
        .any(|pool| pool.path == pool_path)
    {
        operations.push(Operation::AddProjectPoolRef {
            path: pool_path.to_string(),
            priority: next_pool_priority(&project_manifest.pools),
        });
    }
    operations.extend(eagle_pool_create_operations(
        &before.objects,
        pool_path,
        &pool,
    )?);
    let existing_eagle_identity_count = eagle_pool_object_ids(&pool)
        .into_iter()
        .filter(|object_id| before.objects.contains_key(object_id))
        .count();
    let import_map_entries =
        eagle_pool_import_map_entries(&before.import_map, pool_path, &pool, source, &source_hash);
    let import_map_relative_path = eagle_library_import_map_relative_path(source);
    if !import_map_entries.is_empty() {
        operations.push(Operation::CreateImportMapShard {
            relative_path: import_map_relative_path.clone(),
            shard: serde_json::to_value(ImportMapShard {
                schema_version: 1,
                entries: import_map_entries.clone(),
            })?,
        });
    }
    let created_object_count = operations
        .iter()
        .filter(|operation| matches!(operation, Operation::CreatePoolLibraryObject { .. }))
        .count();
    if !operations.is_empty() {
        let mut model = before;
        model.commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "datum-eda-cli".to_string(),
                    source: CommitSource::Cli,
                    reason: format!("import Eagle library {}", source.display()),
                },
                operations,
            },
        )?;
    }
    let after_write = ProjectResolver::new(root).resolve().with_context(|| {
        format!(
            "failed to resolve imported Eagle library {}",
            root.display()
        )
    })?;
    for object_id in eagle_pool_object_ids(&pool) {
        after_write.objects.get(&object_id).ok_or_else(|| {
            anyhow::anyhow!("imported Eagle pool object {object_id} was not resolver-visible")
        })?;
    }
    Ok(NativeProjectEagleLibraryImportView {
        contract: "native_project_eagle_library_import_v1",
        project_id: after_write.project.project_id,
        source_path: source.display().to_string(),
        pool_path: pool_path.to_string(),
        import_map_path: root.join(&import_map_relative_path).display().to_string(),
        imported_unit_count: pool.units.len(),
        imported_symbol_count: pool.symbols.len(),
        imported_entity_count: pool.entities.len(),
        imported_part_count: pool.parts.len(),
        imported_package_count: pool.packages.len(),
        imported_padstack_count: pool.padstacks.len(),
        import_map_entry_count: import_map_entries.len(),
        created_object_count,
        reused_existing_identity_count: existing_eagle_identity_count,
    })
}

fn eagle_pool_create_operations(
    existing_objects: &std::collections::BTreeMap<Uuid, eda_engine::substrate::DomainObject>,
    pool_path: &str,
    pool: &Pool,
) -> Result<Vec<Operation>> {
    let mut operations = Vec::new();
    push_eagle_pool_object_operations(
        &mut operations,
        existing_objects,
        pool_path,
        "units",
        pool.units.iter().map(|(id, object)| (*id, object)),
    )?;
    push_eagle_pool_object_operations(
        &mut operations,
        existing_objects,
        pool_path,
        "symbols",
        pool.symbols.iter().map(|(id, object)| (*id, object)),
    )?;
    push_eagle_pool_object_operations(
        &mut operations,
        existing_objects,
        pool_path,
        "entities",
        pool.entities.iter().map(|(id, object)| (*id, object)),
    )?;
    push_eagle_pool_object_operations(
        &mut operations,
        existing_objects,
        pool_path,
        "padstacks",
        pool.padstacks.iter().map(|(id, object)| (*id, object)),
    )?;
    push_eagle_pool_object_operations(
        &mut operations,
        existing_objects,
        pool_path,
        "packages",
        pool.packages.iter().map(|(id, object)| (*id, object)),
    )?;
    push_eagle_pool_object_operations(
        &mut operations,
        existing_objects,
        pool_path,
        "parts",
        pool.parts.iter().map(|(id, object)| (*id, object)),
    )?;
    Ok(operations)
}

fn push_eagle_pool_object_operations<T: serde::Serialize>(
    operations: &mut Vec<Operation>,
    existing_objects: &std::collections::BTreeMap<Uuid, eda_engine::substrate::DomainObject>,
    pool_path: &str,
    object_kind: &str,
    objects: impl Iterator<Item = (Uuid, T)>,
) -> Result<()> {
    let mut objects: Vec<_> = objects.collect();
    objects.sort_by_key(|(id, _)| *id);
    for (object_id, object) in objects {
        if existing_objects.contains_key(&object_id) {
            continue;
        }
        operations.push(Operation::CreatePoolLibraryObject {
            object_id,
            relative_path: eagle_pool_relative_path(pool_path, object_kind, object_id),
            object_kind: object_kind.to_string(),
            object: eagle_pool_object_payload(object)?,
        });
    }
    Ok(())
}

fn eagle_pool_object_payload<T: serde::Serialize>(object: T) -> Result<serde_json::Value> {
    let mut object = serde_json::to_value(object)?;
    let document = object
        .as_object_mut()
        .context("imported Eagle pool object must serialize as a JSON object")?;
    document.insert("schema_version".to_string(), serde_json::json!(1));
    Ok(object)
}

fn eagle_pool_object_ids(pool: &Pool) -> Vec<Uuid> {
    eagle_pool_object_refs(pool)
        .into_iter()
        .map(|(_, object_id)| object_id)
        .collect()
}

pub(super) fn eagle_pool_object_refs(pool: &Pool) -> Vec<(&'static str, Uuid)> {
    let mut refs = Vec::new();
    refs.extend(pool.units.keys().copied().map(|id| ("units", id)));
    refs.extend(pool.symbols.keys().copied().map(|id| ("symbols", id)));
    refs.extend(pool.entities.keys().copied().map(|id| ("entities", id)));
    refs.extend(pool.padstacks.keys().copied().map(|id| ("padstacks", id)));
    refs.extend(pool.packages.keys().copied().map(|id| ("packages", id)));
    refs.extend(pool.parts.keys().copied().map(|id| ("parts", id)));
    refs.sort_by_key(|(_, object_id)| *object_id);
    refs
}

pub(super) fn eagle_pool_import_key(source: &Path, object_kind: &str, object_id: Uuid) -> String {
    format!("eagle:lbr:{}:{object_kind}:{object_id}", source.display())
}

pub(super) fn eagle_pool_relative_path(
    pool_path: &str,
    object_kind: &str,
    object_id: Uuid,
) -> String {
    format!("{pool_path}/{object_kind}/{object_id}.json")
}

fn eagle_library_import_map_relative_path(source: &Path) -> String {
    let import_id = Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum-eda:eagle-library-import-map:{}", source.display()).as_bytes(),
    );
    format!(".datum/import_map/eagle-library-{import_id}.json")
}

fn board_import_create_operations(
    existing_objects: &std::collections::BTreeMap<Uuid, eda_engine::substrate::DomainObject>,
    board: &Board,
) -> Result<Vec<Operation>> {
    let mut operations = Vec::new();
    let mut nets: Vec<_> = board.nets.values().collect();
    nets.sort_by_key(|net| net.uuid);
    for net in nets {
        if existing_objects.contains_key(&net.uuid) {
            continue;
        }
        operations.push(Operation::CreateBoardNet {
            net_id: net.uuid,
            net: serde_json::to_value(net)?,
        });
    }
    let mut packages: Vec<_> = board.packages.values().collect();
    packages.sort_by_key(|package| package.uuid);
    for package in packages {
        if existing_objects.contains_key(&package.uuid) {
            continue;
        }
        operations.push(Operation::CreateBoardPackage {
            package_id: package.uuid,
            package: serde_json::to_value(package)?,
            materialized: serde_json::json!({}),
        });
    }
    let mut pads: Vec<_> = board.pads.values().collect();
    pads.sort_by_key(|pad| pad.uuid);
    for pad in pads {
        if existing_objects.contains_key(&pad.uuid) {
            continue;
        }
        operations.push(Operation::CreateBoardPad {
            pad_id: pad.uuid,
            pad: serde_json::to_value(pad)?,
        });
    }
    let mut tracks: Vec<_> = board.tracks.values().collect();
    tracks.sort_by_key(|track| track.uuid);
    for track in tracks {
        if existing_objects.contains_key(&track.uuid) {
            continue;
        }
        operations.push(Operation::CreateBoardTrack {
            track_id: track.uuid,
            track: serde_json::to_value(track)?,
        });
    }
    let mut vias: Vec<_> = board.vias.values().collect();
    vias.sort_by_key(|via| via.uuid);
    for via in vias {
        if existing_objects.contains_key(&via.uuid) {
            continue;
        }
        operations.push(Operation::CreateBoardVia {
            via_id: via.uuid,
            via: serde_json::to_value(via)?,
        });
    }
    let mut zones: Vec<_> = board.zones.values().collect();
    zones.sort_by_key(|zone| zone.uuid);
    for zone in zones {
        if existing_objects.contains_key(&zone.uuid) {
            continue;
        }
        operations.push(Operation::CreateBoardZone {
            zone_id: zone.uuid,
            zone: serde_json::to_value(zone)?,
        });
    }
    Ok(operations)
}

fn board_import_map_entries(
    existing_import_map: &std::collections::BTreeMap<String, ImportMapEntry>,
    identities: &[KiCadBoardImportIdentity],
    board_shard_id: Uuid,
    source: &Path,
    source_hash: &str,
) -> Vec<ImportMapEntry> {
    let source_path = source.display().to_string();
    let mut desired = std::collections::BTreeMap::new();
    for identity in identities {
        desired.insert(
            identity.import_key.clone(),
            ImportMapEntry {
                import_key: identity.import_key.clone(),
                object_id: identity.object_id,
                source_shard_id: board_shard_id,
                status: eda_engine::substrate::ImportMapEntryStatus::Active,
                source_tool: "kicad".to_string(),
                source_path: source_path.clone(),
                source_object_ref: board_source_object_ref(identity),
                source_hash: source_hash.to_string(),
            },
        );
    }
    for (import_key, entry) in existing_import_map {
        if desired.contains_key(import_key)
            || !is_same_kicad_board_source_entry(entry, &source_path)
        {
            continue;
        }
        let mut entry = entry.clone();
        entry.status = eda_engine::substrate::ImportMapEntryStatus::MissingInSource;
        entry.source_hash = source_hash.to_string();
        desired.insert(import_key.clone(), entry);
    }
    if desired.iter().all(|(import_key, entry)| {
        existing_import_map
            .get(import_key)
            .is_some_and(|existing| existing == entry)
    }) {
        return Vec::new();
    }
    desired.into_values().collect()
}

fn board_source_object_ref(identity: &KiCadBoardImportIdentity) -> String {
    let family = match identity.object_family {
        "board_footprint" => "board-footprint",
        "board_pad" => "board-pad",
        "board_segment" => "board-segment",
        "board_via" => "board-via",
        "board_zone" => "board-zone",
        family => family,
    };
    format!("{family}:{}", identity.source_uuid)
}

fn is_same_kicad_board_source_entry(entry: &ImportMapEntry, source_path: &str) -> bool {
    entry.source_tool == "kicad"
        && entry.source_path == source_path
        && (entry.import_key.starts_with("kicad:board-footprint:")
            || entry.import_key.starts_with("kicad:board-pad:")
            || entry.import_key.starts_with("kicad:board-segment:")
            || entry.import_key.starts_with("kicad:board-via:")
            || entry.import_key.starts_with("kicad:board-zone:"))
}

fn board_import_map_relative_path(source: &Path) -> String {
    let import_id = Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum-eda:kicad-board-import-map:{}", source.display()).as_bytes(),
    );
    format!(".datum/import_map/kicad-board-{import_id}.json")
}

pub(super) fn validate_project_local_pool_path(pool_path: &str) -> Result<()> {
    let path = PathBuf::from(pool_path);
    if pool_path.trim().is_empty() || path.is_absolute() {
        bail!("project pool path must be a non-empty relative path");
    }
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        bail!("project pool path must not contain parent-directory components");
    }
    Ok(())
}

pub(super) fn next_pool_priority(pools: &[super::NativeProjectPoolRef]) -> u32 {
    pools.iter().map(|pool| pool.priority).max().unwrap_or(0) + 1
}

pub(super) fn source_shard_id_for_relative_path(relative_path: &str) -> Uuid {
    Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum-eda:source-shard:{relative_path}").as_bytes(),
    )
}
