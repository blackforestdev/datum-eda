//! Generic object creation owns payload admission and the canonical create lifecycle.
use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::api::native_write::library::{
    PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write,
};
use eda_engine::api::native_write::library_definition::{
    validate_footprint_creation_references, validate_part_creation_references,
    validate_symbol_creation_references,
};
use eda_engine::api::native_write::library_pin_pad_map::validate_pin_pad_map_creation_references;
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::substrate::{CommitSource, ProjectResolver};
use uuid::Uuid;

use super::super::payload::read_pool_library_object_payload;
use super::{
    NativeProjectPoolLibraryObjectMutationView, definition_validation_error,
    pool_library_mutation_view, pool_library_relative_path, validate_pool_library_object_kind,
    validate_project_local_pool_path,
};

pub(crate) fn create_native_project_pool_library_object(
    root: &Path,
    pool_path: &str,
    object_kind: &str,
    object_id: Uuid,
    source: &Path,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    validate_pool_library_object_kind(object_kind)?;
    let object = read_pool_library_object_payload(source, object_id)?;
    let relative_path = pool_library_relative_path(pool_path, object_kind, object_id);
    let mut model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let source = crate::cli_commit_source()?;
    let provenance = WriteProvenance::new(
        "datum-eda-cli",
        source,
        format!("create native pool library object {object_id}"),
    );
    // Keep manifest/ensure-pool validation before reference admission and prepare
    // against the same resolved revision whose references are checked below.
    let prepared = build_pool_library_write(
        &model,
        provenance,
        Some(pool_path),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new(pool_path, object_kind, object_id),
            object: object.clone(),
        }],
    )?;
    if source == CommitSource::Cli {
        match object_kind {
            "footprints" => validate_footprint_creation_references(&model, &relative_path, &object)
                .map_err(definition_validation_error)?,
            "parts" => validate_part_creation_references(&model, &relative_path, &object)
                .map_err(definition_validation_error)?,
            "pin_pad_maps" => {
                validate_pin_pad_map_creation_references(&model, &relative_path, &object)
                    .map_err(definition_validation_error)?
            }
            "symbols" => validate_symbol_creation_references(&model, &relative_path, &object)
                .map_err(definition_validation_error)?,
            _ => {}
        }
    }
    commit_prepared(&mut model, root, prepared)?;
    pool_library_mutation_view(
        root,
        "create",
        pool_path,
        object_kind,
        object_id,
        &relative_path,
    )
}
