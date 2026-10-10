//! Draft Padstack proposal lifecycle over the same manual engine definition.
use super::super::library::{definition_validation_error, validate_project_local_pool_path};
use super::propose_create_pool_library_object_value;
use crate::NativeProjectProposalCreateView;
use anyhow::Result;
use eda_engine::api::native_write::library_definition::{
    PadstackDefinitionInput, validated_padstack_payload,
};
use std::path::Path;
use uuid::Uuid;
#[allow(clippy::too_many_arguments)]
pub(crate) fn propose_create_native_project_pool_padstack(
    root: &Path,
    pool_path: &str,
    padstack_id: Uuid,
    name: String,
    aperture: Option<String>,
    diameter_nm: Option<i64>,
    width_nm: Option<i64>,
    height_nm: Option<i64>,
    drill_nm: Option<i64>,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    validate_project_local_pool_path(pool_path)?;
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
    propose_create_pool_library_object_value(
        root,
        pool_path,
        "padstacks",
        padstack_id,
        object,
        proposal_id,
        rationale,
        "create_pool_padstack_proposal",
        "Create native pool padstack",
    )
}
