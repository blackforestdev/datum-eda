use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Result, bail};
use eda_engine::substrate::{
    CommitDiff, CommitReport, Operation, OperationBatch, ProjectResolver, Proposal,
    ProposalApplyBlocker, ProposalCreateRequest, ProposalSource, ProposalStatus,
    apply_accepted_proposal, create_draft_proposal_from_batch, preview_proposal_diff,
    review_proposal_status, validate_proposal_apply,
};
use serde::Serialize;
use uuid::Uuid;

use crate::ProposalSourceArg;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectProposalsView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) proposal_count: usize,
    pub(crate) proposals: BTreeMap<Uuid, Proposal>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectProposalView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) proposal_id: Uuid,
    pub(crate) proposal: Proposal,
    pub(crate) validation: NativeProjectProposalValidationView,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectProposalPreviewView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) proposal_id: Uuid,
    pub(crate) prepared_against: String,
    pub(crate) preview_after_model_revision: String,
    pub(crate) affected_objects: Vec<String>,
    pub(crate) diff: CommitDiff,
    pub(crate) render_deltas: Vec<NativeProjectProposalRenderDeltaView>,
    pub(crate) validation: NativeProjectProposalValidationView,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectProposalRenderDeltaView {
    pub(crate) delta_kind: &'static str,
    pub(crate) object_id: String,
    pub(crate) primitive_kind: &'static str,
    pub(crate) layer_id: String,
    pub(crate) end_layer_id: Option<String>,
    pub(crate) width_nm: i64,
    pub(crate) drill_nm: Option<i64>,
    pub(crate) diameter_nm: Option<i64>,
    pub(crate) path: Vec<NativeProjectProposalRenderPointView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectProposalRenderPointView {
    pub(crate) x: i64,
    pub(crate) y: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectProposalCreateView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) proposal_id: Uuid,
    pub(crate) proposal: Proposal,
    pub(crate) validation: NativeProjectProposalValidationView,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectProposalValidationView {
    pub(crate) contract: &'static str,
    pub(crate) policy: &'static str,
    pub(crate) approval_path: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) proposal_id: Uuid,
    pub(crate) status: ProposalStatus,
    pub(crate) prepared_against: String,
    pub(crate) prepared_against_current_model: bool,
    pub(crate) batch_revision_guard_matches: bool,
    pub(crate) acceptance_required: bool,
    pub(crate) current_revision_required: bool,
    pub(crate) revision_guard_required: bool,
    pub(crate) check_source_evidence_required: bool,
    pub(crate) can_apply: bool,
    pub(crate) blocker_codes: Vec<String>,
    pub(crate) blockers: Vec<ProposalApplyBlocker>,
}

pub(crate) fn query_native_project_proposals(root: &Path) -> Result<NativeProjectProposalsView> {
    let model = ProjectResolver::new(root).resolve()?;
    Ok(NativeProjectProposalsView {
        contract: "proposals_query_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        proposal_count: model.proposals.len(),
        proposals: model.proposals,
    })
}

pub(crate) fn create_native_project_proposal(
    root: &Path,
    batch_path: &Path,
    rationale: String,
    proposal_id: Option<Uuid>,
    source: ProposalSourceArg,
    checks_run: Vec<Uuid>,
    finding_fingerprints: Vec<String>,
) -> Result<NativeProjectProposalCreateView> {
    let batch = serde_json::from_str::<OperationBatch>(&std::fs::read_to_string(batch_path)?)?;
    let mut model = ProjectResolver::new(root).resolve()?;
    let source = match source {
        ProposalSourceArg::Manual => ProposalSource::Manual,
        ProposalSourceArg::Cli => ProposalSource::Cli,
        ProposalSourceArg::Tool => ProposalSource::Tool,
        ProposalSourceArg::Assistant => ProposalSource::Assistant,
        ProposalSourceArg::Check => ProposalSource::Check,
        ProposalSourceArg::Import => ProposalSource::Import,
    };
    let proposal = create_draft_proposal_from_batch(
        &mut model,
        root,
        ProposalCreateRequest {
            proposal_id,
            batch,
            rationale,
            source,
            checks_run,
            finding_fingerprints,
        },
    )?;
    let validation = validate_proposal_in_model(&model, proposal.proposal_id, &proposal);
    Ok(NativeProjectProposalCreateView {
        contract: "proposal_create_v1",
        action: "create_proposal",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        proposal_id: proposal.proposal_id,
        proposal,
        validation,
    })
}

pub(crate) fn show_native_project_proposal(
    root: &Path,
    proposal_id: Uuid,
) -> Result<NativeProjectProposalView> {
    let model = ProjectResolver::new(root).resolve()?;
    let proposal = model
        .proposals
        .get(&proposal_id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("proposal {proposal_id} not found"))?;
    Ok(NativeProjectProposalView {
        contract: "proposal_show_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0.clone(),
        proposal_id,
        validation: validate_proposal_in_model(&model, proposal_id, &proposal),
        proposal,
    })
}

pub(crate) fn preview_native_project_proposal(
    root: &Path,
    proposal_id: Uuid,
) -> Result<NativeProjectProposalPreviewView> {
    let model = ProjectResolver::new(root).resolve()?;
    let preview = preview_proposal_diff(&model, proposal_id)?;
    Ok(NativeProjectProposalPreviewView {
        contract: "proposal_preview_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: preview.current_model_revision.0,
        proposal_id,
        prepared_against: preview.prepared_against.0,
        preview_after_model_revision: preview.preview_after_model_revision.0,
        affected_objects: preview
            .affected_objects
            .iter()
            .map(|object| object.to_string())
            .collect(),
        diff: preview.diff,
        render_deltas: proposal_render_deltas(
            &model
                .proposals
                .get(&proposal_id)
                .ok_or_else(|| anyhow::anyhow!("proposal {proposal_id} not found"))?
                .batch,
        )?,
        validation: validate_proposal_in_model(
            &model,
            proposal_id,
            model
                .proposals
                .get(&proposal_id)
                .ok_or_else(|| anyhow::anyhow!("proposal {proposal_id} not found"))?,
        ),
    })
}

fn proposal_render_deltas(
    batch: &OperationBatch,
) -> Result<Vec<NativeProjectProposalRenderDeltaView>> {
    batch
        .operations
        .iter()
        .filter_map(|operation| match operation {
            Operation::CreateBoardTrack { track_id, track } => Some(proposal_track_render_delta(
                "create",
                track_id.to_string(),
                track,
            )),
            Operation::SetBoardTrack { track_id, track } => Some(proposal_track_render_delta(
                "set",
                track_id.to_string(),
                track,
            )),
            Operation::CreateBoardVia { via_id, via } => {
                Some(proposal_via_render_delta("create", via_id.to_string(), via))
            }
            Operation::SetBoardVia { via_id, via } => {
                Some(proposal_via_render_delta("set", via_id.to_string(), via))
            }
            _ => None,
        })
        .collect()
}

fn proposal_track_render_delta(
    delta_kind: &'static str,
    object_id: String,
    track: &serde_json::Value,
) -> Result<NativeProjectProposalRenderDeltaView> {
    let track: eda_engine::board::Track = serde_json::from_value(track.clone())?;
    Ok(NativeProjectProposalRenderDeltaView {
        delta_kind,
        object_id,
        primitive_kind: "track_path",
        layer_id: format!("L{}", track.layer),
        end_layer_id: None,
        width_nm: track.width,
        drill_nm: None,
        diameter_nm: None,
        path: vec![
            NativeProjectProposalRenderPointView {
                x: track.from.x,
                y: track.from.y,
            },
            NativeProjectProposalRenderPointView {
                x: track.to.x,
                y: track.to.y,
            },
        ],
    })
}

fn proposal_via_render_delta(
    delta_kind: &'static str,
    object_id: String,
    via: &serde_json::Value,
) -> Result<NativeProjectProposalRenderDeltaView> {
    let via: eda_engine::board::Via = serde_json::from_value(via.clone())?;
    Ok(NativeProjectProposalRenderDeltaView {
        delta_kind,
        object_id,
        primitive_kind: "via",
        layer_id: format!("L{}", via.from_layer),
        end_layer_id: Some(format!("L{}", via.to_layer)),
        width_nm: via.diameter,
        drill_nm: Some(via.drill),
        diameter_nm: Some(via.diameter),
        path: vec![NativeProjectProposalRenderPointView {
            x: via.position.x,
            y: via.position.y,
        }],
    })
}

pub(crate) fn validate_native_project_proposal(
    root: &Path,
    proposal_id: Uuid,
) -> Result<NativeProjectProposalValidationView> {
    let model = ProjectResolver::new(root).resolve()?;
    let proposal = model
        .proposals
        .get(&proposal_id)
        .ok_or_else(|| anyhow::anyhow!("proposal {proposal_id} not found"))?;
    Ok(validate_proposal_in_model(&model, proposal_id, proposal))
}

fn validate_proposal_in_model(
    model: &eda_engine::substrate::DesignModel,
    proposal_id: Uuid,
    _proposal: &Proposal,
) -> NativeProjectProposalValidationView {
    let validation =
        validate_proposal_apply(model, proposal_id).expect("proposal was already resolved");
    let blocker_codes = validation
        .blockers
        .iter()
        .map(|blocker| blocker.code.clone())
        .collect::<Vec<_>>();
    NativeProjectProposalValidationView {
        contract: "proposal_validation_v1",
        policy: "accepted_revision_guarded_source_policy_v1",
        approval_path: "draft_review_accept_then_apply",
        project_id: model.project.project_id.to_string(),
        model_revision: validation.current_model_revision.0,
        proposal_id,
        status: validation.status,
        prepared_against: validation.prepared_against.0,
        prepared_against_current_model: validation.prepared_against_current_model,
        batch_revision_guard_matches: validation.batch_revision_guard_matches,
        acceptance_required: true,
        current_revision_required: true,
        revision_guard_required: true,
        check_source_evidence_required: true,
        can_apply: validation.can_apply,
        blocker_codes,
        blockers: validation.blockers,
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectProposalReviewView {
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) proposal_id: Uuid,
    pub(crate) status: ProposalStatus,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectProposalApplyView {
    pub(crate) action: &'static str,
    pub(crate) policy: &'static str,
    pub(crate) approval_path: &'static str,
    pub(crate) project_id: String,
    pub(crate) proposal_id: Uuid,
    pub(crate) status: ProposalStatus,
    pub(crate) transaction_id: Uuid,
    pub(crate) validation: NativeProjectProposalValidationView,
    pub(crate) commit: CommitReport,
}

pub(crate) fn review_native_project_proposal(
    root: &Path,
    proposal_id: Uuid,
    status: ProposalStatus,
) -> Result<NativeProjectProposalReviewView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let proposal = review_proposal_status(&mut model, root, proposal_id, status)?;
    Ok(NativeProjectProposalReviewView {
        action: "review_proposal",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        proposal_id,
        status: proposal.status,
    })
}

pub(crate) fn defer_native_project_proposal(
    root: &Path,
    proposal_id: Uuid,
) -> Result<NativeProjectProposalReviewView> {
    review_native_project_proposal(root, proposal_id, ProposalStatus::Deferred)
}

pub(crate) fn apply_native_project_proposal(
    root: &Path,
    proposal_id: Uuid,
) -> Result<NativeProjectProposalApplyView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let validation = validate_native_project_proposal(root, proposal_id)?;
    if !validation.can_apply {
        bail!(
            "proposal {proposal_id} is not applyable: {}",
            validation
                .blockers
                .iter()
                .map(|blocker| format!("{}: {}", blocker.code, blocker.message))
                .collect::<Vec<_>>()
                .join("; ")
        );
    }
    let project_id = model.project.project_id.to_string();
    let commit = apply_accepted_proposal(&mut model, root, proposal_id)?;
    Ok(NativeProjectProposalApplyView {
        action: "apply_proposal",
        policy: validation.policy,
        approval_path: validation.approval_path,
        project_id,
        proposal_id,
        status: ProposalStatus::Applied,
        transaction_id: commit.transaction.transaction_id,
        validation,
        commit,
    })
}

pub(crate) fn accept_and_apply_native_project_proposal(
    root: &Path,
    proposal_id: Uuid,
) -> Result<NativeProjectProposalApplyView> {
    review_native_project_proposal(root, proposal_id, ProposalStatus::Accepted)?;
    apply_native_project_proposal(root, proposal_id)
}
