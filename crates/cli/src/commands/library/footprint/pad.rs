//! Manual Footprint pad command lifecycle over shared engine validation.
use super::super::library::{
    NativeProjectPoolLibraryObjectMutationView, commit_pool_library_operations,
    definition_validation_error, pool_library_mutation_view, pool_library_relative_path,
    validate_project_local_pool_path,
};
use super::super::payload::read_project_pool_object_payload;
use super::ensure_pool_object_exists;
use anyhow::Result;
use eda_engine::api::native_write::library::{PoolLibraryObjectTarget, PoolLibraryOperationSpec};
use eda_engine::api::native_write::library_definition::{
    FootprintPadDefinitionInput, footprint_with_pad,
};
use std::path::Path;
use uuid::Uuid;
#[allow(clippy::too_many_arguments)]
pub(crate) fn set_native_project_pool_footprint_pad(
    root: &Path,
    pool_path: &str,
    footprint_id: Uuid,
    pad_id: Uuid,
    padstack_id: Uuid,
    pad_name: String,
    x_nm: i64,
    y_nm: i64,
    layer: i32,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let input = FootprintPadDefinitionInput {
        pad_id,
        padstack_id,
        name: &pad_name,
        x_nm,
        y_nm,
        layer,
    };
    input.validate().map_err(definition_validation_error)?;
    ensure_pool_object_exists(root, padstack_id, "padstacks", "padstack")?;
    let relative_path = pool_library_relative_path(pool_path, "footprints", footprint_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, footprint_id)?;
    let object = footprint_with_pad(footprint_id, previous_object, input)
        .map_err(definition_validation_error)?;
    commit_pool_library_operations(
        root,
        format!("set native pool footprint {footprint_id} pad {pad_id}"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "footprints", footprint_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_footprint_pad",
        pool_path,
        "footprints",
        footprint_id,
        &relative_path,
    )
}
