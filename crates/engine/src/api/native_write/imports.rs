//! Import (KiCad board/schematic/footprint, Eagle library) mutation builders
//! for the native write facade.
//!
//! Family I of the native-write migration: all operation authoring for the
//! import flows in `crates/cli/src/command_project_imports.rs`,
//! `crates/cli/src/command_project_imports_schematic.rs`, and
//! `crates/cli/src/command_project_imports_kicad_footprint.rs` lives here.
//! The CLI callers are thin argument-parsers: they run the (pure) format
//! importers, derive the desired [`ImportMapEntry`] sets, call a `build_*`
//! function, and commit the returned [`PreparedWrite`] via
//! [`super::commit_prepared`].
//!
//! Imports are LARGE single atomic batches: one batch per import operation so
//! undo reverts the whole import. Operation ordering is byte-for-byte the
//! CLI's historical sequence (creations sorted by object id within each
//! object family, then board outline/stackup rewrites, then the import-map
//! shard write). Wherever another family already owns a builder for an
//! operation, that builder is composed here (its per-op guard, a no-op for
//! creations, is dropped and the combined batch is re-guarded as one unit —
//! the `forward_annotation` composition pattern). Only the import-specific
//! operations (`CreateImportMapShard`, `CreatePoolPadstack`,
//! `CreatePoolPackage` for imported footprints) are authored directly.
//!
//! One deliberate delta from the pre-facade CLI: batches are built through
//! [`BatchComposer`], so the standard guard pass runs. Import creations are
//! unguarded (guard pass is a no-op — see tests), but the KiCad board
//! import's `SetBoardOutline`/`SetBoardStackup` rewrites now carry the same
//! board revision guard every other facade write carries; the historical CLI
//! committed that batch unguarded.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;
use uuid::Uuid;

use crate::board::Board;
use crate::error::EngineError;
use crate::import::kicad::ImportedKiCadFootprint;
use crate::pool::Pool;
use crate::schematic::{PlacedSymbol, Schematic, Sheet};
use crate::substrate::{
    DesignModel, ImportMapEntry, ImportMapShard, ObjectId, Operation, SourceShardKind,
};

use super::board_components::{BoardPackagePlacement, build_place_board_package};
use super::board_layout::{build_set_board_outline, build_set_board_stackup};
use super::board_routing::{
    build_place_board_net, build_place_board_pad, build_place_board_track, build_place_board_via,
    build_place_board_zone,
};
use super::context::{BatchComposer, PreparedWrite, WriteProvenance};
use super::library::{
    PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write,
    ensure_pool_ref_operation, pool_library_relative_path,
};
use super::schematic_connectivity::{
    build_create_schematic_bus, build_create_schematic_bus_entry, build_create_schematic_junction,
    build_create_schematic_label, build_create_schematic_noconnect, build_create_schematic_port,
    build_create_schematic_wire,
};
use super::schematic_sheets::{
    build_create_schematic_definition, build_create_schematic_drawing,
    build_create_schematic_sheet, build_create_schematic_sheet_instance,
    build_create_schematic_text,
};
use super::schematic_symbols::build_place_schematic_symbol;

/// A built import write: the uncommitted batch (when the import has anything
/// to do) plus the created-object count the import report surfaces.
#[derive(Debug, Clone)]
pub struct ImportWrite {
    /// The single atomic import batch; `None` when the import is a no-op
    /// (re-import with nothing new), in which case nothing must be committed.
    pub prepared: Option<PreparedWrite>,
    /// Number of `Create*` design/pool-object operations in the batch
    /// (import-map shard writes and outline/stackup rewrites excluded).
    pub created_object_count: usize,
}

// ---------------------------------------------------------------------------
// Deterministic import identity/path derivations (persistence-visible; the
// v5 seed layouts are byte-exact historical CLI conventions and must never
// drift).
// ---------------------------------------------------------------------------

/// Deterministic id of the source shard at `relative_path`
/// (v5 over `datum-eda:source-shard:<relative_path>`).
pub fn source_shard_id_for_relative_path(relative_path: &str) -> Uuid {
    Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum-eda:source-shard:{relative_path}").as_bytes(),
    )
}

/// Import-map sidecar path for a KiCad board import of `source`.
pub fn kicad_board_import_map_relative_path(source: &Path) -> String {
    let import_id = Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum-eda:kicad-board-import-map:{}", source.display()).as_bytes(),
    );
    format!(".datum/import_map/kicad-board-{import_id}.json")
}

/// Import-map sidecar path for a KiCad schematic import of `source`.
pub fn kicad_schematic_import_map_relative_path(source: &Path) -> String {
    let import_id = Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum-eda:kicad-schematic-import-map:{}", source.display()).as_bytes(),
    );
    format!(".datum/import_map/kicad-schematic-{import_id}.json")
}

/// Import-map sidecar path for an Eagle library import of `source`.
pub fn eagle_library_import_map_relative_path(source: &Path) -> String {
    let import_id = Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum-eda:eagle-library-import-map:{}", source.display()).as_bytes(),
    );
    format!(".datum/import_map/eagle-library-{import_id}.json")
}

/// Import-map sidecar path for a KiCad footprint import that resolved to
/// pool package `package_id`.
pub fn kicad_footprint_import_map_relative_path(package_id: Uuid) -> String {
    format!(".datum/import_map/kicad-footprint-{package_id}.json")
}

/// Import key of one Eagle library pool object.
pub fn eagle_pool_import_key(source: &Path, object_kind: &str, object_id: Uuid) -> String {
    format!("eagle:lbr:{}:{object_kind}:{object_id}", source.display())
}

/// Canonical shard path of one imported Eagle pool object.
pub fn eagle_pool_relative_path(pool_path: &str, object_kind: &str, object_id: Uuid) -> String {
    pool_library_relative_path(pool_path, object_kind, object_id)
}

/// Every pool object an Eagle library import materializes, as
/// `(object_kind, object_id)` pairs sorted by object id.
pub fn eagle_pool_object_refs(pool: &Pool) -> Vec<(&'static str, Uuid)> {
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

// ---------------------------------------------------------------------------
// KiCad board import
// ---------------------------------------------------------------------------

/// Build the single atomic batch for a KiCad board import: create every
/// resolver-unknown net/package/pad/track/via/zone (sorted by id within each
/// family), rewrite the board outline/stackup when the imported values
/// differ from the materialized board root, and write the import-map sidecar
/// shard when `import_map_entries` is non-empty.
pub fn build_kicad_board_import(
    model: &DesignModel,
    provenance: WriteProvenance,
    board_id: ObjectId,
    board: &Board,
    import_map_entries: Vec<ImportMapEntry>,
    source: &Path,
) -> Result<ImportWrite, EngineError> {
    let mut operations = Vec::new();

    let mut nets: Vec<_> = board.nets.values().collect();
    nets.sort_by_key(|net| net.uuid);
    for net in nets {
        if model.objects.contains_key(&net.uuid) {
            continue;
        }
        operations.extend(unguarded_operations(build_place_board_net(
            model,
            provenance.clone(),
            net,
        )?));
    }
    let mut packages: Vec<_> = board.packages.values().collect();
    packages.sort_by_key(|package| package.uuid);
    for package in packages {
        if model.objects.contains_key(&package.uuid) {
            continue;
        }
        operations.extend(unguarded_operations(build_place_board_package(
            model,
            provenance.clone(),
            &BoardPackagePlacement {
                package: package.clone(),
                materialized: serde_json::json!({}),
            },
        )?));
    }
    let mut pads: Vec<_> = board.pads.values().collect();
    pads.sort_by_key(|pad| pad.uuid);
    for pad in pads {
        if model.objects.contains_key(&pad.uuid) {
            continue;
        }
        operations.extend(unguarded_operations(build_place_board_pad(
            model,
            provenance.clone(),
            pad,
        )?));
    }
    let mut tracks: Vec<_> = board.tracks.values().collect();
    tracks.sort_by_key(|track| track.uuid);
    for track in tracks {
        if model.objects.contains_key(&track.uuid) {
            continue;
        }
        operations.extend(unguarded_operations(build_place_board_track(
            model,
            provenance.clone(),
            track,
        )?));
    }
    let mut vias: Vec<_> = board.vias.values().collect();
    vias.sort_by_key(|via| via.uuid);
    for via in vias {
        if model.objects.contains_key(&via.uuid) {
            continue;
        }
        operations.extend(unguarded_operations(build_place_board_via(
            model,
            provenance.clone(),
            via,
        )?));
    }
    let mut zones: Vec<_> = board.zones.values().collect();
    zones.sort_by_key(|zone| zone.uuid);
    for zone in zones {
        if model.objects.contains_key(&zone.uuid) {
            continue;
        }
        operations.extend(unguarded_operations(build_place_board_zone(
            model,
            provenance.clone(),
            zone,
        )?));
    }

    let board_root = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let imported_outline = serde_json::to_value(&board.outline)?;
    if board_root.get("outline") != Some(&imported_outline) {
        operations.extend(unguarded_operations(build_set_board_outline(
            model,
            provenance.clone(),
            board_id,
            &board.outline,
        )?));
    }
    let imported_stackup = serde_json::to_value(&board.stackup)?;
    if board_root.get("stackup") != Some(&imported_stackup) {
        operations.extend(unguarded_operations(build_set_board_stackup(
            model,
            provenance.clone(),
            board_id,
            &board.stackup.layers,
        )?));
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
    if !import_map_entries.is_empty() {
        operations.push(import_map_shard_operation(
            kicad_board_import_map_relative_path(source),
            import_map_entries,
        )?);
    }
    finish_import_write(model, provenance, operations, created_object_count)
}

// ---------------------------------------------------------------------------
// Eagle library import
// ---------------------------------------------------------------------------

/// Build the single atomic batch for an Eagle library import: ensure the
/// project references `pool_path` (family B's pool-ref rule), create every
/// resolver-unknown pool object (kind order `units`, `symbols`, `entities`,
/// `padstacks`, `packages`, `parts`; sorted by id within each kind; payloads
/// stamped `schema_version: 1`), and write the import-map sidecar shard when
/// `import_map_entries` is non-empty.
pub fn build_eagle_library_import(
    model: &DesignModel,
    provenance: WriteProvenance,
    pool_path: &str,
    pool: &Pool,
    import_map_entries: Vec<ImportMapEntry>,
    source: &Path,
) -> Result<ImportWrite, EngineError> {
    let mut specs = Vec::new();
    push_eagle_pool_create_specs(
        &mut specs,
        model,
        pool_path,
        "units",
        pool.units.iter().map(|(id, object)| (*id, object)),
    )?;
    push_eagle_pool_create_specs(
        &mut specs,
        model,
        pool_path,
        "symbols",
        pool.symbols.iter().map(|(id, object)| (*id, object)),
    )?;
    push_eagle_pool_create_specs(
        &mut specs,
        model,
        pool_path,
        "entities",
        pool.entities.iter().map(|(id, object)| (*id, object)),
    )?;
    push_eagle_pool_create_specs(
        &mut specs,
        model,
        pool_path,
        "padstacks",
        pool.padstacks.iter().map(|(id, object)| (*id, object)),
    )?;
    push_eagle_pool_create_specs(
        &mut specs,
        model,
        pool_path,
        "packages",
        pool.packages.iter().map(|(id, object)| (*id, object)),
    )?;
    push_eagle_pool_create_specs(
        &mut specs,
        model,
        pool_path,
        "parts",
        pool.parts.iter().map(|(id, object)| (*id, object)),
    )?;

    // Family B owns pool-library op authoring (pool-ref rule + create ops);
    // its creations carry no guards, so its batch splices verbatim.
    let library_write = build_pool_library_write(model, provenance.clone(), Some(pool_path), specs)?;
    let mut operations = library_write.batch.operations;
    let created_object_count = operations
        .iter()
        .filter(|operation| matches!(operation, Operation::CreatePoolLibraryObject { .. }))
        .count();
    if !import_map_entries.is_empty() {
        operations.push(import_map_shard_operation(
            eagle_library_import_map_relative_path(source),
            import_map_entries,
        )?);
    }
    finish_import_write(model, provenance, operations, created_object_count)
}

fn push_eagle_pool_create_specs<T: Serialize>(
    specs: &mut Vec<PoolLibraryOperationSpec>,
    model: &DesignModel,
    pool_path: &str,
    object_kind: &str,
    objects: impl Iterator<Item = (Uuid, T)>,
) -> Result<(), EngineError> {
    let mut objects: Vec<_> = objects.collect();
    objects.sort_by_key(|(id, _)| *id);
    for (object_id, object) in objects {
        if model.objects.contains_key(&object_id) {
            continue;
        }
        specs.push(PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new(pool_path, object_kind, object_id),
            object: eagle_pool_object_payload(object)?,
        });
    }
    Ok(())
}

fn eagle_pool_object_payload<T: Serialize>(object: T) -> Result<serde_json::Value, EngineError> {
    let mut object = serde_json::to_value(object)?;
    let document = object.as_object_mut().ok_or_else(|| {
        EngineError::Validation(
            "imported Eagle pool object must serialize as a JSON object".to_string(),
        )
    })?;
    document.insert("schema_version".to_string(), serde_json::json!(1));
    Ok(object)
}

// ---------------------------------------------------------------------------
// KiCad schematic import
// ---------------------------------------------------------------------------

/// One schematic sheet shard to create ahead of a KiCad schematic import
/// (the payload is the CLI-owned native sheet shard shape, pre-serialized).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KiCadSchematicSheetCreateSpec {
    pub sheet_id: ObjectId,
    pub relative_path: String,
    pub sheet: serde_json::Value,
}

/// Build the atomic batch that creates every sheet shard a KiCad schematic
/// import needs (the import's first, separately committed batch).
pub fn build_kicad_schematic_sheet_imports(
    model: &DesignModel,
    provenance: WriteProvenance,
    schematic_id: ObjectId,
    sheets: Vec<KiCadSchematicSheetCreateSpec>,
) -> Result<PreparedWrite, EngineError> {
    let mut operations = Vec::new();
    for spec in sheets {
        operations.extend(unguarded_operations(build_create_schematic_sheet(
            model,
            provenance.clone(),
            schematic_id,
            spec.sheet_id,
            &spec.relative_path,
            spec.sheet,
        )?));
    }
    BatchComposer::compose(model, provenance)
        .push_ops(operations)
        .finish()
}

/// Everything the main KiCad schematic import batch is built from.
///
/// `definition_payloads`/`instance_payloads` are the CLI-owned native shard
/// shapes for the resolver-unknown sheet definitions/instances, keyed by
/// object id (BTreeMap iteration gives the historical id-sorted op order).
pub struct KiCadSchematicImportSpec<'a> {
    pub schematic_id: ObjectId,
    pub imported: &'a Schematic,
    /// Imported sheet id → target native sheet id.
    pub sheet_id_map: &'a BTreeMap<Uuid, Uuid>,
    pub definition_payloads: BTreeMap<Uuid, serde_json::Value>,
    pub instance_payloads: BTreeMap<Uuid, serde_json::Value>,
    pub import_map_entries: Vec<ImportMapEntry>,
    pub source: &'a Path,
}

/// Build the single atomic main batch for a KiCad schematic import: sheet
/// definitions, then sheet instances (both id-sorted), then per sheet
/// (id-sorted) symbols, wires, junctions, labels, ports, buses, bus entries,
/// no-connects, texts, and drawings (each id-sorted, resolver-unknown
/// objects only), then the import-map sidecar shard when
/// `import_map_entries` is non-empty.
pub fn build_kicad_schematic_import(
    model: &DesignModel,
    provenance: WriteProvenance,
    spec: KiCadSchematicImportSpec<'_>,
) -> Result<ImportWrite, EngineError> {
    let mut operations = Vec::new();
    for (definition_id, definition) in spec.definition_payloads {
        if model.objects.contains_key(&definition_id) {
            continue;
        }
        operations.extend(unguarded_operations(build_create_schematic_definition(
            model,
            provenance.clone(),
            spec.schematic_id,
            definition_id,
            &format!("definitions/{definition_id}.json"),
            definition,
        )?));
    }
    for (instance_id, instance) in spec.instance_payloads {
        if model.objects.contains_key(&instance_id) {
            continue;
        }
        operations.extend(unguarded_operations(build_create_schematic_sheet_instance(
            model,
            provenance.clone(),
            spec.schematic_id,
            instance_id,
            instance,
        )?));
    }

    let mut sheets: Vec<&Sheet> = spec.imported.sheets.values().collect();
    sheets.sort_by_key(|sheet| sheet.uuid);
    for sheet in sheets {
        let target_sheet_id = *spec.sheet_id_map.get(&sheet.uuid).ok_or_else(|| {
            EngineError::Validation("missing schematic sheet object mapping".to_string())
        })?;
        let mut symbols: Vec<&PlacedSymbol> = sheet.symbols.values().collect();
        symbols.sort_by_key(|symbol| symbol.uuid);
        for symbol in symbols {
            if model.objects.contains_key(&symbol.uuid) {
                continue;
            }
            operations.extend(unguarded_operations(build_place_schematic_symbol(
                model,
                provenance.clone(),
                target_sheet_id,
                symbol,
                None,
            )?));
        }
        // Category order is the historical per-sheet import sequence.
        push_sorted_sheet_creates(&mut operations, model, &sheet.wires, |wire| {
            build_create_schematic_wire(model, provenance.clone(), target_sheet_id, wire)
        })?;
        push_sorted_sheet_creates(&mut operations, model, &sheet.junctions, |junction| {
            build_create_schematic_junction(model, provenance.clone(), target_sheet_id, junction)
        })?;
        push_sorted_sheet_creates(&mut operations, model, &sheet.labels, |label| {
            build_create_schematic_label(model, provenance.clone(), target_sheet_id, label)
        })?;
        push_sorted_sheet_creates(&mut operations, model, &sheet.ports, |port| {
            build_create_schematic_port(model, provenance.clone(), target_sheet_id, port)
        })?;
        push_sorted_sheet_creates(&mut operations, model, &sheet.buses, |bus| {
            build_create_schematic_bus(model, provenance.clone(), target_sheet_id, bus)
        })?;
        push_sorted_sheet_creates(&mut operations, model, &sheet.bus_entries, |bus_entry| {
            build_create_schematic_bus_entry(model, provenance.clone(), target_sheet_id, bus_entry)
        })?;
        push_sorted_sheet_creates(&mut operations, model, &sheet.noconnects, |noconnect| {
            build_create_schematic_noconnect(model, provenance.clone(), target_sheet_id, noconnect)
        })?;
        push_sorted_sheet_creates(&mut operations, model, &sheet.texts, |text| {
            build_create_schematic_text(model, provenance.clone(), target_sheet_id, text)
        })?;
        push_sorted_sheet_creates(&mut operations, model, &sheet.drawings, |drawing| {
            build_create_schematic_drawing(model, provenance.clone(), target_sheet_id, drawing)
        })?;
    }

    // Every operation so far is a schematic object creation.
    let created_object_count = operations.len();
    if !spec.import_map_entries.is_empty() {
        operations.push(import_map_shard_operation(
            kicad_schematic_import_map_relative_path(spec.source),
            spec.import_map_entries,
        )?);
    }
    finish_import_write(model, provenance, operations, created_object_count)
}

fn push_sorted_sheet_creates<T>(
    operations: &mut Vec<Operation>,
    model: &DesignModel,
    objects: &std::collections::HashMap<Uuid, T>,
    mut build: impl FnMut(&T) -> Result<PreparedWrite, EngineError>,
) -> Result<(), EngineError> {
    let mut ids: Vec<Uuid> = objects.keys().copied().collect();
    ids.sort();
    for id in ids {
        if model.objects.contains_key(&id) {
            continue;
        }
        operations.extend(unguarded_operations(build(&objects[&id])?));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// KiCad footprint import
// ---------------------------------------------------------------------------

/// Build the single atomic batch for a KiCad footprint import: ensure the
/// project references `pool_path`, and — unless the import resolved to an
/// existing identity — create the padstacks (in imported order), the pool
/// package, the footprint (payload stamped `schema_version: 1`), and the
/// import-map sidecar shard (when `import_map_entries` is non-empty).
pub fn build_kicad_footprint_import(
    model: &DesignModel,
    provenance: WriteProvenance,
    pool_path: &str,
    imported: &ImportedKiCadFootprint,
    reused_existing_identity: bool,
    import_map_entries: Vec<ImportMapEntry>,
) -> Result<ImportWrite, EngineError> {
    let mut operations = Vec::new();
    if let Some(operation) = ensure_pool_ref_operation(model, pool_path)? {
        operations.push(operation);
    }
    if !reused_existing_identity {
        for padstack in &imported.padstacks {
            operations.push(Operation::CreatePoolPadstack {
                padstack_id: padstack.uuid,
                relative_path: pool_library_relative_path(pool_path, "padstacks", padstack.uuid),
                padstack: serde_json::to_value(padstack)?,
            });
        }
        operations.push(Operation::CreatePoolPackage {
            package_id: imported.package.uuid,
            relative_path: pool_library_relative_path(pool_path, "packages", imported.package.uuid),
            package: serde_json::to_value(&imported.package)?,
        });
        let mut footprint_value = serde_json::to_value(&imported.footprint)?;
        if let Some(document) = footprint_value.as_object_mut() {
            document.insert("schema_version".to_string(), serde_json::json!(1));
        }
        operations.push(Operation::CreatePoolLibraryObject {
            object_id: imported.footprint.uuid,
            relative_path: pool_library_relative_path(
                pool_path,
                "footprints",
                imported.footprint.uuid,
            ),
            object_kind: "footprints".to_string(),
            object: footprint_value,
        });
    }
    let created_object_count = operations
        .iter()
        .filter(|operation| {
            matches!(
                operation,
                Operation::CreatePoolPadstack { .. }
                    | Operation::CreatePoolPackage { .. }
                    | Operation::CreatePoolLibraryObject { .. }
            )
        })
        .count();
    if !import_map_entries.is_empty() {
        operations.push(import_map_shard_operation(
            kicad_footprint_import_map_relative_path(imported.package.uuid),
            import_map_entries,
        )?);
    }
    finish_import_write(model, provenance, operations, created_object_count)
}

// ---------------------------------------------------------------------------
// Shared internals
// ---------------------------------------------------------------------------

/// The `CreateImportMapShard` operation for one import-map sidecar write —
/// the one operation only this family authors.
fn import_map_shard_operation(
    relative_path: String,
    entries: Vec<ImportMapEntry>,
) -> Result<Operation, EngineError> {
    Ok(Operation::CreateImportMapShard {
        relative_path,
        shard: serde_json::to_value(ImportMapShard {
            schema_version: 1,
            entries,
        })?,
    })
}

/// Splice a composed family builder's operations into the import batch,
/// dropping its per-op guards (the combined batch is re-guarded as one unit
/// by [`BatchComposer`] — the `forward_annotation` composition pattern).
fn unguarded_operations(prepared: PreparedWrite) -> impl Iterator<Item = Operation> {
    prepared
        .batch
        .operations
        .into_iter()
        .filter(|operation| !matches!(operation, Operation::GuardObjectRevision { .. }))
}

fn finish_import_write(
    model: &DesignModel,
    provenance: WriteProvenance,
    operations: Vec<Operation>,
    created_object_count: usize,
) -> Result<ImportWrite, EngineError> {
    if operations.is_empty() {
        return Ok(ImportWrite {
            prepared: None,
            created_object_count,
        });
    }
    let prepared = BatchComposer::compose(model, provenance)
        .push_ops(operations)
        .finish()?;
    Ok(ImportWrite {
        prepared: Some(prepared),
        created_object_count,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::super::test_support::{temp_project_root, write_minimal_project};
    use super::*;
    use crate::board::{Net, PlacedPackage, Stackup, StackupLayer, StackupLayerType, Track};
    use crate::ir::geometry::{Point, Polygon};
    use crate::pool::Unit;
    use crate::substrate::{
        CommitSource, ImportMapEntryStatus, ObjectRevision, ProjectResolver,
    };

    fn test_provenance() -> WriteProvenance {
        WriteProvenance::new("unit-test", CommitSource::Test, "imports facade test")
    }

    fn resolved_minimal_model(name: &str) -> (std::path::PathBuf, DesignModel, Uuid) {
        let root = temp_project_root(name);
        let project_id = Uuid::new_v4();
        let board_id = Uuid::new_v4();
        write_minimal_project(&root, project_id, board_id);
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("fixture project should resolve");
        (root, model, board_id)
    }

    fn test_import_map_entry(import_key: &str, object_id: Uuid) -> ImportMapEntry {
        ImportMapEntry {
            import_key: import_key.to_string(),
            object_id,
            source_shard_id: source_shard_id_for_relative_path("board/board.json"),
            status: ImportMapEntryStatus::Active,
            source_tool: "kicad".to_string(),
            source_path: "source.kicad_pcb".to_string(),
            source_object_ref: import_key.to_string(),
            source_hash: "sha256:test".to_string(),
        }
    }

    fn test_board(net_id: Uuid, package_id: Uuid, track_id: Uuid) -> Board {
        Board {
            uuid: Uuid::new_v4(),
            name: "Imported".to_string(),
            stackup: Stackup {
                layers: vec![StackupLayer::new(
                    0,
                    "F.Cu",
                    StackupLayerType::Copper,
                    35_000,
                )],
            },
            pad_expansion_setup: Default::default(),
            outline: Polygon {
                vertices: vec![
                    Point { x: 0, y: 0 },
                    Point { x: 10, y: 0 },
                    Point { x: 0, y: 0 },
                ],
                closed: true,
            },
            packages: HashMap::from([(
                package_id,
                PlacedPackage {
                    uuid: package_id,
                    part: Uuid::new_v4(),
                    package: Uuid::new_v4(),
                    reference: "U1".to_string(),
                    value: "IMPORTED".to_string(),
                    position: Point { x: 0, y: 0 },
                    rotation: 0,
                    layer: 0,
                    locked: false,
                },
            )]),
            pads: HashMap::new(),
            tracks: HashMap::from([(
                track_id,
                Track {
                    uuid: track_id,
                    net: net_id,
                    from: Point { x: 0, y: 0 },
                    to: Point { x: 5, y: 5 },
                    width: 250_000,
                    layer: 0,
                },
            )]),
            vias: HashMap::new(),
            zones: HashMap::new(),
            nets: HashMap::from([(net_id, Net::new(net_id, "SIG", Uuid::nil()))]),
            net_classes: HashMap::new(),
            rules: Vec::new(),
            keepouts: Vec::new(),
            dimensions: Vec::new(),
            texts: Vec::new(),
        }
    }

    #[test]
    fn kicad_board_import_matches_hand_built_operation_order_oracle() {
        let (_root, model, board_id) = resolved_minimal_model("imports_board_oracle");
        // Fixed ids ordered so the sorted-by-id family order is observable.
        let net_id = Uuid::from_u128(1);
        let package_id = Uuid::from_u128(2);
        let track_id = Uuid::from_u128(3);
        let board = test_board(net_id, package_id, track_id);
        let entries = vec![test_import_map_entry("kicad:board-segment:test", track_id)];

        let write = build_kicad_board_import(
            &model,
            test_provenance(),
            board_id,
            &board,
            entries.clone(),
            Path::new("source.kicad_pcb"),
        )
        .expect("board import should build");
        let prepared = write.prepared.expect("board import should have a batch");

        // Hand-built oracle: the historical CLI operation sequence — creates
        // sorted by id within each family (nets, packages, pads, tracks,
        // vias, zones), then outline/stackup rewrites (now preceded by the
        // facade's board revision guard), then the import-map shard.
        let oracle = vec![
            Operation::CreateBoardNet {
                net_id,
                net: serde_json::to_value(&board.nets[&net_id]).unwrap(),
            },
            Operation::CreateBoardPackage {
                package_id,
                package: serde_json::to_value(&board.packages[&package_id]).unwrap(),
                materialized: serde_json::json!({}),
            },
            Operation::CreateBoardTrack {
                track_id,
                track: serde_json::to_value(&board.tracks[&track_id]).unwrap(),
            },
            Operation::GuardObjectRevision {
                object_id: board_id,
                expected_object_revision: ObjectRevision(0),
            },
            Operation::SetBoardOutline {
                board_id,
                outline: serde_json::to_value(&board.outline).unwrap(),
            },
            Operation::SetBoardStackup {
                board_id,
                stackup: serde_json::to_value(&board.stackup).unwrap(),
            },
            Operation::CreateImportMapShard {
                relative_path: kicad_board_import_map_relative_path(Path::new(
                    "source.kicad_pcb",
                )),
                shard: serde_json::to_value(ImportMapShard {
                    schema_version: 1,
                    entries,
                })
                .unwrap(),
            },
        ];
        assert_eq!(prepared.batch.operations, oracle);
        assert_eq!(write.created_object_count, 3);
        assert_eq!(
            prepared.batch.expected_model_revision,
            Some(model.model_revision.clone())
        );
    }

    #[test]
    fn kicad_board_import_skips_resolver_known_objects() {
        let (_root, model, board_id) = resolved_minimal_model("imports_board_noop");
        let mut board = test_board(Uuid::from_u128(1), Uuid::from_u128(2), Uuid::from_u128(3));
        board.nets.clear();
        board.packages.clear();
        board.tracks.clear();

        let write = build_kicad_board_import(
            &model,
            test_provenance(),
            board_id,
            &board,
            Vec::new(),
            Path::new("source.kicad_pcb"),
        )
        .expect("board import should build");

        // No objects to create and no import-map entries: the only remaining
        // operations are the (guarded) outline/stackup rewrites against the
        // fixture board root, never a creation.
        assert_eq!(write.created_object_count, 0);
        let prepared = write.prepared.expect("outline/stackup rewrite batch");
        assert!(prepared.batch.operations.iter().all(|operation| matches!(
            operation,
            Operation::GuardObjectRevision { .. }
                | Operation::SetBoardOutline { .. }
                | Operation::SetBoardStackup { .. }
        )));
    }

    #[test]
    fn eagle_library_import_is_guard_free_and_matches_oracle() {
        let (_root, model, _board_id) = resolved_minimal_model("imports_eagle_oracle");
        let unit_id = Uuid::from_u128(7);
        let mut pool = Pool::default();
        pool.units.insert(
            unit_id,
            Unit {
                uuid: unit_id,
                name: "OPAMP".to_string(),
                manufacturer: "Test".to_string(),
                pins: HashMap::new(),
                tags: HashSet::new(),
            },
        );
        let entries = vec![test_import_map_entry("eagle:lbr:test:units", unit_id)];

        let write = build_eagle_library_import(
            &model,
            test_provenance(),
            "pool",
            &pool,
            entries.clone(),
            Path::new("source.lbr"),
        )
        .expect("eagle import should build");
        let prepared = write.prepared.expect("eagle import should have a batch");

        let mut unit_payload = serde_json::to_value(&pool.units[&unit_id]).unwrap();
        unit_payload
            .as_object_mut()
            .unwrap()
            .insert("schema_version".to_string(), serde_json::json!(1));
        let oracle = vec![
            Operation::AddProjectPoolRef {
                path: "pool".to_string(),
                priority: 1,
            },
            Operation::CreatePoolLibraryObject {
                object_id: unit_id,
                relative_path: format!("pool/units/{unit_id}.json"),
                object_kind: "units".to_string(),
                object: unit_payload,
            },
            Operation::CreateImportMapShard {
                relative_path: eagle_library_import_map_relative_path(Path::new("source.lbr")),
                shard: serde_json::to_value(ImportMapShard {
                    schema_version: 1,
                    entries,
                })
                .unwrap(),
            },
        ];
        assert_eq!(prepared.batch.operations, oracle);
        assert_eq!(write.created_object_count, 1);
        // Import creations are guard-free: the guard pass is a no-op.
        assert!(
            prepared
                .batch
                .operations
                .iter()
                .all(|operation| !matches!(operation, Operation::GuardObjectRevision { .. }))
        );
    }

    #[test]
    fn schematic_sheet_imports_compose_one_guard_free_batch() {
        let (_root, model, _board_id) = resolved_minimal_model("imports_schematic_sheets");
        let schematic_root = model
            .materialized_source_shard_value(SourceShardKind::SchematicRoot)
            .expect("schematic root should materialize");
        let schematic_id: Uuid = schematic_root["uuid"]
            .as_str()
            .unwrap()
            .parse()
            .expect("schematic uuid");
        let first = Uuid::from_u128(11);
        let second = Uuid::from_u128(12);
        let specs = vec![
            KiCadSchematicSheetCreateSpec {
                sheet_id: first,
                relative_path: format!("sheets/{first}.json"),
                sheet: serde_json::json!({ "schema_version": 1, "uuid": first }),
            },
            KiCadSchematicSheetCreateSpec {
                sheet_id: second,
                relative_path: format!("sheets/{second}.json"),
                sheet: serde_json::json!({ "schema_version": 1, "uuid": second }),
            },
        ];

        let prepared =
            build_kicad_schematic_sheet_imports(&model, test_provenance(), schematic_id, specs)
                .expect("sheet imports should build");

        assert_eq!(
            prepared.batch.operations,
            vec![
                Operation::CreateSchematicSheet {
                    schematic_id,
                    sheet_id: first,
                    relative_path: format!("sheets/{first}.json"),
                    sheet: serde_json::json!({ "schema_version": 1, "uuid": first }),
                },
                Operation::CreateSchematicSheet {
                    schematic_id,
                    sheet_id: second,
                    relative_path: format!("sheets/{second}.json"),
                    sheet: serde_json::json!({ "schema_version": 1, "uuid": second }),
                },
            ]
        );
    }

    #[test]
    fn kicad_footprint_import_matches_oracle_and_reused_identity_skips_creates() {
        let (_root, model, _board_id) = resolved_minimal_model("imports_footprint_oracle");
        let footprint_source = temp_project_root("imports_footprint_fixture")
            .join("native-import.kicad_mod");
        std::fs::write(
            &footprint_source,
            r#"(footprint "NativeImportFootprint"
  (layer "F.Cu")
  (fp_line (start -1 -0.8) (end 1 -0.8) (layer "F.SilkS") (width 0.12))
  (pad "1" smd rect (at 0 0) (size 1 1) (layers "F.Cu" "F.Paste" "F.Mask"))
)"#,
        )
        .expect("footprint fixture should write");
        let (imported, _report) =
            crate::import::kicad::import_footprint_document(&footprint_source)
                .expect("footprint should import");
        let package_id = imported.package.uuid;
        let entry = test_import_map_entry("kicad:footprint-package:test", package_id);

        let write = build_kicad_footprint_import(
            &model,
            test_provenance(),
            "pool",
            &imported,
            false,
            vec![entry.clone()],
        )
        .expect("footprint import should build");
        let prepared = write.prepared.expect("footprint import should have a batch");

        let mut oracle = vec![Operation::AddProjectPoolRef {
            path: "pool".to_string(),
            priority: 1,
        }];
        for padstack in &imported.padstacks {
            oracle.push(Operation::CreatePoolPadstack {
                padstack_id: padstack.uuid,
                relative_path: format!("pool/padstacks/{}.json", padstack.uuid),
                padstack: serde_json::to_value(padstack).unwrap(),
            });
        }
        oracle.push(Operation::CreatePoolPackage {
            package_id,
            relative_path: format!("pool/packages/{package_id}.json"),
            package: serde_json::to_value(&imported.package).unwrap(),
        });
        let mut footprint_value = serde_json::to_value(&imported.footprint).unwrap();
        footprint_value
            .as_object_mut()
            .unwrap()
            .insert("schema_version".to_string(), serde_json::json!(1));
        oracle.push(Operation::CreatePoolLibraryObject {
            object_id: imported.footprint.uuid,
            relative_path: format!("pool/footprints/{}.json", imported.footprint.uuid),
            object_kind: "footprints".to_string(),
            object: footprint_value,
        });
        oracle.push(Operation::CreateImportMapShard {
            relative_path: kicad_footprint_import_map_relative_path(package_id),
            shard: serde_json::to_value(ImportMapShard {
                schema_version: 1,
                entries: vec![entry],
            })
            .unwrap(),
        });
        assert_eq!(prepared.batch.operations, oracle);
        assert_eq!(
            write.created_object_count,
            imported.padstacks.len() + 2,
            "padstacks + package + footprint"
        );

        // Reused identity with the pool already referenced is a no-op write.
        let reused = build_kicad_footprint_import(
            &model,
            test_provenance(),
            "pool",
            &imported,
            true,
            Vec::new(),
        )
        .expect("reused footprint import should build");
        let reused_prepared = reused
            .prepared
            .expect("pool ref is still missing, so one op remains");
        assert_eq!(
            reused_prepared.batch.operations,
            vec![Operation::AddProjectPoolRef {
                path: "pool".to_string(),
                priority: 1,
            }]
        );
        assert_eq!(reused.created_object_count, 0);
    }

    #[test]
    fn import_identity_derivations_match_historical_cli_formulas() {
        let source = Path::new("/tmp/demo.kicad_pcb");
        // Byte-exact historical CLI v5 derivations (pre-migration
        // command_project_imports*.rs).
        assert_eq!(
            source_shard_id_for_relative_path("board/board.json"),
            Uuid::new_v5(
                &Uuid::NAMESPACE_URL,
                "datum-eda:source-shard:board/board.json".as_bytes(),
            )
        );
        let board_id = Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("datum-eda:kicad-board-import-map:{}", source.display()).as_bytes(),
        );
        assert_eq!(
            kicad_board_import_map_relative_path(source),
            format!(".datum/import_map/kicad-board-{board_id}.json")
        );
        let schematic_id = Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("datum-eda:kicad-schematic-import-map:{}", source.display()).as_bytes(),
        );
        assert_eq!(
            kicad_schematic_import_map_relative_path(source),
            format!(".datum/import_map/kicad-schematic-{schematic_id}.json")
        );
        let eagle_id = Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("datum-eda:eagle-library-import-map:{}", source.display()).as_bytes(),
        );
        assert_eq!(
            eagle_library_import_map_relative_path(source),
            format!(".datum/import_map/eagle-library-{eagle_id}.json")
        );
        let object_id = Uuid::from_u128(42);
        assert_eq!(
            eagle_pool_import_key(source, "units", object_id),
            format!("eagle:lbr:{}:units:{object_id}", source.display())
        );
        assert_eq!(
            eagle_pool_relative_path("pool", "units", object_id),
            format!("pool/units/{object_id}.json")
        );
        assert_eq!(
            kicad_footprint_import_map_relative_path(object_id),
            format!(".datum/import_map/kicad-footprint-{object_id}.json")
        );
    }
}
