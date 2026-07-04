use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::api::native_write::WriteProvenance;
use eda_engine::api::native_write::board_components::BoardPackageEdit;
use eda_engine::api::native_write::forward_annotation::{
    ProposalRenderDelta, build_board_component_replacement_proposal_batch, guard_proposal_batch,
    proposal_render_deltas,
};
use eda_engine::board::PlacedPackage;
use eda_engine::substrate::{
    CommitDiff, CommitReport, ProjectResolver, Proposal, ProposalApplyBlocker,
    ProposalCreateRequest, ProposalSource, ProposalStatus, apply_accepted_proposal,
    create_draft_proposal_from_batch, preview_proposal_diff_journaled, review_proposal_status,
    validate_proposal_apply,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::NativeBoardRoot;
use crate::ProposalSourceArg;
use crate::command_project::{
    board_package_materialization_payload_for_component,
    current_board_component_materialization_payload,
    load_native_project_with_resolved_board_and_model,
};

use crate::command_project::cli_commit_source;

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

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BoardComponentReplacementSpec {
    #[serde(alias = "component_uuid")]
    pub(crate) component: Uuid,
    #[serde(default, alias = "package_uuid")]
    pub(crate) package: Option<Uuid>,
    #[serde(default, alias = "part_uuid")]
    pub(crate) part: Option<Uuid>,
    #[serde(default)]
    pub(crate) value: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BoardComponentReplacementPlanSelectionSpec {
    #[serde(alias = "component", alias = "component_uuid")]
    pub(crate) uuid: Uuid,
    #[serde(default, alias = "package")]
    pub(crate) package_uuid: Option<Uuid>,
    #[serde(default, alias = "part")]
    pub(crate) part_uuid: Option<Uuid>,
    #[serde(default)]
    pub(crate) value: Option<String>,
}

impl From<BoardComponentReplacementPlanSelectionSpec> for BoardComponentReplacementSpec {
    fn from(selection: BoardComponentReplacementPlanSelectionSpec) -> Self {
        Self {
            component: selection.uuid,
            package: selection.package_uuid,
            part: selection.part_uuid,
            value: selection.value,
        }
    }
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
    let batch = serde_json::from_str(&std::fs::read_to_string(batch_path)?)?;
    let mut model = ProjectResolver::new(root).resolve()?;
    let batch = guard_proposal_batch(&model, batch)?;
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

pub(crate) fn propose_native_project_board_component_replacement(
    root: &Path,
    component_uuid: Uuid,
    package_uuid: Option<Uuid>,
    part_uuid: Option<Uuid>,
    value: Option<String>,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    propose_native_project_board_component_replacements(
        root,
        vec![BoardComponentReplacementSpec {
            component: component_uuid,
            package: package_uuid,
            part: part_uuid,
            value,
        }],
        proposal_id,
        rationale,
    )
}

pub(crate) fn propose_native_project_board_component_replacements(
    root: &Path,
    replacements: Vec<BoardComponentReplacementSpec>,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    if replacements.is_empty() {
        bail!("board component replacement proposal requires at least one replacement");
    }
    let (project, mut model) = load_native_project_with_resolved_board_and_model(root)?;
    let mut edits = Vec::new();
    let mut seen_components = BTreeSet::new();
    let mut component_ids = Vec::new();
    for replacement in replacements {
        if !seen_components.insert(replacement.component) {
            bail!(
                "board component replacement proposal repeats component {}",
                replacement.component
            );
        }
        append_board_component_replacement_edits(
            root,
            &project.board,
            replacement,
            &mut edits,
            &mut component_ids,
        )?;
    }

    if edits.is_empty() {
        bail!("board component replacement proposal is a no-op for all components");
    }
    let batch = build_board_component_replacement_proposal_batch(
        &model,
        WriteProvenance::new(
            "datum-eda-cli",
            cli_commit_source()?,
            "propose board component replacement",
        ),
        edits,
    )?;
    let proposal = create_draft_proposal_from_batch(
        &mut model,
        root,
        ProposalCreateRequest {
            proposal_id,
            batch,
            rationale: rationale.map(str::to_string).unwrap_or_else(|| {
                if component_ids.len() == 1 {
                    format!("Review board component {} replacement", component_ids[0])
                } else {
                    format!(
                        "Review {} board component replacements",
                        component_ids.len()
                    )
                }
            }),
            source: ProposalSource::Cli,
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
        },
    )?;
    let validation = validate_proposal_in_model(&model, proposal.proposal_id, &proposal);
    Ok(NativeProjectProposalCreateView {
        contract: "proposal_create_v1",
        action: "propose_board_component_replacement",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        proposal_id: proposal.proposal_id,
        proposal,
        validation,
    })
}

pub(crate) fn propose_native_project_board_component_replacement_plan(
    root: &Path,
    selections: Vec<BoardComponentReplacementPlanSelectionSpec>,
    proposal_id: Option<Uuid>,
    rationale: Option<&str>,
) -> Result<NativeProjectProposalCreateView> {
    let replacements = selections
        .into_iter()
        .map(BoardComponentReplacementSpec::from)
        .collect();
    propose_native_project_board_component_replacements(root, replacements, proposal_id, rationale)
}

fn append_board_component_replacement_edits(
    root: &Path,
    board: &NativeBoardRoot,
    replacement: BoardComponentReplacementSpec,
    edits: &mut Vec<(Uuid, BoardPackageEdit)>,
    component_ids: &mut Vec<Uuid>,
) -> Result<()> {
    if replacement.package.is_none() && replacement.part.is_none() && replacement.value.is_none() {
        bail!(
            "board component replacement proposal for {} requires package, part, or value",
            replacement.component
        );
    }
    let component_uuid = replacement.component;
    let key = component_uuid.to_string();
    let entry = board.packages.get(&key).cloned().ok_or_else(|| {
        anyhow::anyhow!("board component not found in native project: {component_uuid}")
    })?;
    let mut component: PlacedPackage = serde_json::from_value(entry)
        .with_context(|| format!("failed to parse board component {component_uuid}"))?;
    let initial_len = edits.len();

    if let Some(part_id) = replacement.part
        && part_id != component.part
    {
        edits.push((component_uuid, BoardPackageEdit::Part { part_id }));
        component.part = part_id;
    }

    if let Some(package_ref_id) = replacement.package
        && package_ref_id != component.package
    {
        let previous_materialized =
            current_board_component_materialization_payload(root, component_uuid)?;
        component.package = package_ref_id;
        let materialized = board_package_materialization_payload_for_component(root, &component)?;
        edits.push((
            component_uuid,
            BoardPackageEdit::Package {
                package_ref_id,
                previous_materialized,
                materialized,
            },
        ));
    }

    if let Some(value) = replacement.value
        && value != component.value
    {
        edits.push((component_uuid, BoardPackageEdit::Value { value }));
    }

    if edits.len() == initial_len {
        bail!("board component replacement proposal is a no-op for component {component_uuid}");
    }
    component_ids.push(component_uuid);
    Ok(())
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
    let preview = preview_proposal_diff_journaled(&model, root, proposal_id)?;
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
        )?
        .into_iter()
        .map(proposal_render_delta_view)
        .collect(),
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

fn proposal_render_delta_view(delta: ProposalRenderDelta) -> NativeProjectProposalRenderDeltaView {
    NativeProjectProposalRenderDeltaView {
        delta_kind: delta.delta_kind,
        object_id: delta.object_id,
        primitive_kind: delta.primitive_kind,
        layer_id: delta.layer_id,
        end_layer_id: delta.end_layer_id,
        width_nm: delta.width_nm,
        drill_nm: delta.drill_nm,
        diameter_nm: delta.diameter_nm,
        path: delta
            .path
            .into_iter()
            .map(|point| NativeProjectProposalRenderPointView {
                x: point.x,
                y: point.y,
            })
            .collect(),
    }
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

pub(super) fn validate_proposal_in_model(
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
