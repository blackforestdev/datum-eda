//! Direct manual Padstack command lifecycle over shared engine definitions.
use super::{
    NativeProjectPoolLibraryObjectMutationView, commit_pool_library_operations,
    definition_validation_error, pool_library_mutation_view, pool_library_relative_path,
    validate_project_local_pool_path,
};
use anyhow::Result;
use eda_engine::api::native_write::library::{PoolLibraryObjectTarget, PoolLibraryOperationSpec};
use eda_engine::api::native_write::library_definition::{
    PadstackDefinitionInput, validated_padstack_payload,
};
use std::path::Path;
use uuid::Uuid;

// CLI command handler threads individually parsed flag values.
#[allow(clippy::too_many_arguments)]
pub(crate) fn create_native_project_pool_padstack(
    root: &Path,
    pool_path: &str,
    padstack_id: Uuid,
    name: String,
    aperture: Option<String>,
    diameter_nm: Option<i64>,
    width_nm: Option<i64>,
    height_nm: Option<i64>,
    drill_nm: Option<i64>,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let relative_path = pool_library_relative_path(pool_path, "padstacks", padstack_id);
    let object = validated_padstack_payload(
        padstack_id,
        &name,
        PadstackDefinitionInput {
            aperture: aperture.as_deref(),
            diameter_nm,
            width_nm,
            height_nm,
            drill_nm,
        },
    )
    .map_err(definition_validation_error)?;
    commit_pool_library_operations(
        root,
        format!("create native pool padstack {padstack_id}"),
        Some(pool_path),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new(pool_path, "padstacks", padstack_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "create_padstack",
        pool_path,
        "padstacks",
        padstack_id,
        &relative_path,
    )
}
