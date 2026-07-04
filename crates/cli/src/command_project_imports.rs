use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use eda_engine::api::native_write::imports::{
    build_eagle_library_import, build_kicad_board_import, eagle_library_import_map_relative_path,
    kicad_board_import_map_relative_path,
};
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::import::eagle::import_library_file;
use eda_engine::import::ids_sidecar::compute_source_hash_file;
use eda_engine::import::kicad::{
    KiCadBoardImportIdentity, import_board_document_with_import_map_identities,
};
use eda_engine::pool::Pool;
use eda_engine::substrate::{ImportMapEntry, ProjectResolver, SourceShardKind};
use serde::Serialize;
use uuid::Uuid;

use super::command_project_imports_eagle_import_map::eagle_pool_import_map_entries;

use crate::command_project::cli_commit_source;

// Deterministic import identity/path derivations are engine-owned now
// (native_write::imports); re-exported for the sibling import modules.
pub(super) use eda_engine::api::native_write::imports::{
    eagle_pool_import_key, eagle_pool_object_refs, eagle_pool_relative_path,
    source_shard_id_for_relative_path,
};

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
    let import_map_relative_path = kicad_board_import_map_relative_path(source);
    let import_map_entry_count = import_map_entries.len();
    let write = build_kicad_board_import(
        &before,
        WriteProvenance::new(
            "datum-eda-cli",
            cli_commit_source()?,
            format!("import KiCad board {}", source.display()),
        ),
        board_id,
        &board,
        import_map_entries,
        source,
    )?;
    let created_object_count = write.created_object_count;
    if let Some(prepared) = write.prepared {
        let mut model = before;
        commit_prepared(&mut model, root, prepared)?;
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
        import_map_entry_count,
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
    let (pool, _report) = import_library_file(source)
        .with_context(|| format!("failed to import Eagle library {}", source.display()))?;
    let source_hash = compute_source_hash_file(source)?;
    let existing_eagle_identity_count = eagle_pool_object_ids(&pool)
        .into_iter()
        .filter(|object_id| before.objects.contains_key(object_id))
        .count();
    let import_map_entries =
        eagle_pool_import_map_entries(&before.import_map, pool_path, &pool, source, &source_hash);
    let import_map_relative_path = eagle_library_import_map_relative_path(source);
    let import_map_entry_count = import_map_entries.len();
    let write = build_eagle_library_import(
        &before,
        WriteProvenance::new(
            "datum-eda-cli",
            cli_commit_source()?,
            format!("import Eagle library {}", source.display()),
        ),
        pool_path,
        &pool,
        import_map_entries,
        source,
    )?;
    let created_object_count = write.created_object_count;
    if let Some(prepared) = write.prepared {
        let mut model = before;
        commit_prepared(&mut model, root, prepared)?;
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
        import_map_entry_count,
        created_object_count,
        reused_existing_identity_count: existing_eagle_identity_count,
    })
}

fn eagle_pool_object_ids(pool: &Pool) -> Vec<Uuid> {
    eagle_pool_object_refs(pool)
        .into_iter()
        .map(|(_, object_id)| object_id)
        .collect()
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
