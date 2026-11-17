//! Component Properties is a purpose of the existing manual Proposal lifecycle.
use super::*;
use crate::api::native_write::{
    context::PreparedWrite, schematic_assignment,
    schematic_library::part_placement::CompletePartSelection,
};

pub(crate) const REASON: &str = "Edit component properties";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentPropertiesRequest {
    pub proposal_id: Uuid,
    pub symbol: Uuid,
    pub instance_path: Vec<Uuid>,
    pub reference: String,
    pub value: String,
    pub component: CompletePartSelection,
}

pub(super) fn create(
    model: &mut DesignModel,
    root: &std::path::Path,
    request: &ComponentPropertiesRequest,
) -> Result<(), EngineError> {
    let prepared = schematic_assignment::build_properties(
        model,
        WriteProvenance::new("datum-gui", CommitSource::Manual, REASON),
        request.symbol,
        &request.instance_path,
        &request.component,
        &request.reference,
        &request.value,
    )?;
    create_write(
        model,
        root,
        request.proposal_id,
        prepared,
        "Apply the selected component properties and confirmed physical choice",
    )
}

/// Legacy assignment keeps its exact producer/consent path; only its common
/// Proposal packaging lives with the new complete-properties purpose.
#[allow(clippy::too_many_arguments)]
pub(super) fn create_assignment(
    model: &mut DesignModel,
    root: &std::path::Path,
    id: Uuid,
    symbol: Uuid,
    path: &[Uuid],
    part: Uuid,
    gate: Uuid,
    component: Option<&CompletePartSelection>,
) -> Result<(), EngineError> {
    let prepared = schematic_assignment::build_configured(
        model,
        WriteProvenance::new("datum-gui", CommitSource::Manual, "Assign schematic Part"),
        symbol,
        path,
        part,
        gate,
        component,
    )?;
    create_write(
        model,
        root,
        id,
        prepared,
        "Assign compatible Part and Gate to selected Symbol",
    )
}

fn create_write(
    model: &mut DesignModel,
    root: &std::path::Path,
    id: Uuid,
    mut write: PreparedWrite,
    rationale: &str,
) -> Result<(), EngineError> {
    write.batch.batch_id = id;
    create_draft_proposal_from_batch(
        model,
        root,
        ProposalCreateRequest {
            proposal_id: Some(id),
            batch: write.batch,
            rationale: rationale.into(),
            source: ProposalSource::Manual,
            checks_run: vec![],
            finding_fingerprints: vec![],
        },
    )?;
    Ok(())
}
