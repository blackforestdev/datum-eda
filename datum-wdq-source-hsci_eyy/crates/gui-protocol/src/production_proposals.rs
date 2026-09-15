use std::collections::BTreeMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{PointNm, ProductionStatus, run_cli_json_owned};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductionProposalSummary {
    pub proposal_id: String,
    pub status: String,
    pub source: String,
    pub rationale: String,
    pub operation_count: usize,
    pub can_apply: Option<bool>,
    pub blocker_codes: Vec<String>,
    pub preview: Option<ProductionProposalPreviewSummary>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductionProposalPreviewSummary {
    pub prepared_against: String,
    pub preview_after_model_revision: String,
    pub created_count: usize,
    pub modified_count: usize,
    pub deleted_count: usize,
    pub affected_object_count: usize,
    pub affected_objects: Vec<String>,
    pub render_deltas: Vec<ProductionProposalRenderDeltaSummary>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProductionProposalRenderDeltaSummary {
    pub delta_kind: String,
    pub object_id: String,
    pub primitive_kind: String,
    pub layer_id: String,
    pub end_layer_id: Option<String>,
    pub width_nm: i64,
    pub drill_nm: Option<i64>,
    pub diameter_nm: Option<i64>,
    pub path: Vec<PointNm>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
pub(super) struct ProposalsPayload {
    pub(super) proposal_count: usize,
    #[serde(default)]
    proposals: BTreeMap<String, ProposalPayload>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ProposalPayload {
    status: String,
    #[serde(default)]
    source: String,
    #[serde(default)]
    rationale: String,
    #[serde(default)]
    batch: ProposalBatchPayload,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
struct ProposalBatchPayload {
    #[serde(default)]
    operations: Vec<Value>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ProposalValidationPayload {
    can_apply: bool,
    #[serde(default)]
    blocker_codes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ProposalPreviewPayload {
    prepared_against: String,
    preview_after_model_revision: String,
    #[serde(default)]
    affected_objects: Vec<String>,
    #[serde(default)]
    diff: ProposalPreviewDiffPayload,
    #[serde(default)]
    render_deltas: Vec<ProposalPreviewRenderDeltaPayload>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
struct ProposalPreviewDiffPayload {
    #[serde(default)]
    created: Vec<String>,
    #[serde(default)]
    modified: Vec<String>,
    #[serde(default)]
    deleted: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ProposalPreviewRenderDeltaPayload {
    delta_kind: String,
    object_id: String,
    primitive_kind: String,
    layer_id: String,
    #[serde(default)]
    end_layer_id: Option<String>,
    width_nm: i64,
    #[serde(default)]
    drill_nm: Option<i64>,
    #[serde(default)]
    diameter_nm: Option<i64>,
    #[serde(default)]
    path: Vec<PointNm>,
}

pub fn production_status_from_proposals_json(payload: &str) -> Result<ProductionStatus> {
    let payload: ProposalsPayload =
        serde_json::from_str(payload).context("failed to decode proposal list JSON")?;
    Ok(super::production_payloads_to_production_status(
        super::OutputJobsPayload::default(),
        super::ArtifactListPayload::default(),
        payload,
        super::ManufacturingPlansPayload::default(),
        super::PanelProjectionsPayload::default(),
    ))
}

pub(super) fn attach_proposal_validation(
    cli: &[String],
    project_root: &str,
    status: &mut ProductionStatus,
) {
    for proposal in &mut status.proposals {
        let args = vec![
            "proposal".to_string(),
            "validate".to_string(),
            project_root.to_string(),
            "--proposal".to_string(),
            proposal.proposal_id.clone(),
        ];
        let Ok(validation) = run_cli_json_owned::<ProposalValidationPayload>(cli, &args) else {
            continue;
        };
        proposal.can_apply = Some(validation.can_apply);
        proposal.blocker_codes = validation.blocker_codes;

        let args = vec![
            "proposal".to_string(),
            "preview".to_string(),
            project_root.to_string(),
            "--proposal".to_string(),
            proposal.proposal_id.clone(),
        ];
        if let Ok(preview) = run_cli_json_owned::<ProposalPreviewPayload>(cli, &args) {
            proposal.preview = Some(ProductionProposalPreviewSummary {
                prepared_against: preview.prepared_against,
                preview_after_model_revision: preview.preview_after_model_revision,
                created_count: preview.diff.created.len(),
                modified_count: preview.diff.modified.len(),
                deleted_count: preview.diff.deleted.len(),
                affected_object_count: preview.affected_objects.len(),
                affected_objects: preview.affected_objects,
                render_deltas: preview
                    .render_deltas
                    .into_iter()
                    .map(|delta| ProductionProposalRenderDeltaSummary {
                        delta_kind: delta.delta_kind,
                        object_id: delta.object_id,
                        primitive_kind: delta.primitive_kind,
                        layer_id: delta.layer_id,
                        end_layer_id: delta.end_layer_id,
                        width_nm: delta.width_nm,
                        drill_nm: delta.drill_nm,
                        diameter_nm: delta.diameter_nm,
                        path: delta.path,
                    })
                    .collect(),
            });
        }
    }
}

pub(super) fn proposal_summaries(payload: &ProposalsPayload) -> Vec<ProductionProposalSummary> {
    payload
        .proposals
        .iter()
        .map(|(proposal_id, proposal)| ProductionProposalSummary {
            proposal_id: proposal_id.clone(),
            status: proposal.status.clone(),
            source: proposal.source.clone(),
            rationale: proposal.rationale.clone(),
            operation_count: proposal.batch.operations.len(),
            can_apply: None,
            blocker_codes: Vec::new(),
            preview: None,
        })
        .collect()
}
