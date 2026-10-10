//! Generic object replacement owns payload admission and the canonical edit lifecycle.
use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::api::native_write::library::{
    PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write,
};
use eda_engine::api::native_write::library_definition::{
    validate_footprint_replacement_references, validate_part_replacement_references,
    validate_symbol_replacement_references,
};
use eda_engine::api::native_write::library_pin_pad_map::validate_pin_pad_map_replacement_references;
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::substrate::{CommitSource, ProjectResolver};
use uuid::Uuid;

use super::super::payload::{read_pool_library_object_payload, read_project_pool_object_payload};
use super::{
    NativeProjectPoolLibraryObjectMutationView, definition_validation_error,
    pool_library_mutation_view, pool_library_relative_path, validate_pool_library_object_kind,
    validate_project_local_pool_path,
};

pub(crate) fn set_native_project_pool_library_object(
    root: &Path,
    pool_path: &str,
    object_kind: &str,
    object_id: Uuid,
    source: &Path,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    validate_pool_library_object_kind(object_kind)?;
    let relative_path = pool_library_relative_path(pool_path, object_kind, object_id);
    // Validate existence and payload uuid exactly as the pre-migration CLI
    // did; the facade re-sources the previous payload from the same model view.
    read_project_pool_object_payload(root, &relative_path, object_id)?;
    let object = read_pool_library_object_payload(source, object_id)?;
    let mut model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let source = crate::cli_commit_source()?;
    if source == CommitSource::Cli {
        match object_kind {
            "footprints" => validate_footprint_replacement_references(&model, &object)
                .map_err(definition_validation_error)?,
            "parts" => validate_part_replacement_references(&model, &relative_path, &object)
                .map_err(definition_validation_error)?,
            "pin_pad_maps" => {
                validate_pin_pad_map_replacement_references(&model, &relative_path, &object)
                    .map_err(definition_validation_error)?
            }
            "symbols" => validate_symbol_replacement_references(&model, &relative_path, &object)
                .map_err(definition_validation_error)?,
            _ => {}
        }
    }
    // Validate and prepare against the same resolved revision; the canonical
    // commit guard rejects an intervening edit rather than accepting stale links.
    let prepared = build_pool_library_write(
        &model,
        WriteProvenance::new(
            "datum-eda-cli",
            source,
            format!("set native pool library object {object_id}"),
        ),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, object_kind, object_id),
            object,
        }],
    )?;
    commit_prepared(&mut model, root, prepared)?;
    pool_library_mutation_view(
        root,
        "set",
        pool_path,
        object_kind,
        object_id,
        &relative_path,
    )
}
