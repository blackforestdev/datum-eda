//! Finding dispositions create reviewable drafts; existing Proposal review/apply
//! owns explicit acceptance. No CLI alias privately applies the disposition.
use super::{load_native_project_with_resolved_board_and_model, query_native_project_check_run};
use crate::cli_commit_source;
use anyhow::{Result, bail};
use eda_engine::api::native_write::{WriteProvenance, waivers::*};
use eda_engine::schematic::CheckDomain;
use eda_engine::substrate::{
    CommitSource, ProposalCreateRequest, ProposalSource, create_draft_proposal_from_batch,
};
use serde::Serialize;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectDispositionProposalView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) waiver_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) deviation_id: Option<Uuid>,
    pub(crate) fingerprint: String,
    pub(crate) domain: String,
    pub(crate) model_revision: String,
    pub(crate) proposal_id: Uuid,
    pub(crate) status: &'static str,
    pub(crate) next_step: &'static str,
}

#[derive(Clone, Copy)]
enum Disposition {
    Waiver,
    Deviation,
}

pub(crate) fn waive_native_project_finding(
    root: &Path,
    fingerprint: &str,
    rationale: &str,
    created_by: Option<String>,
) -> Result<NativeProjectDispositionProposalView> {
    propose(
        root,
        fingerprint,
        rationale,
        created_by,
        Disposition::Waiver,
    )
}
pub(crate) fn accept_native_project_deviation(
    root: &Path,
    fingerprint: &str,
    rationale: &str,
    accepted_by: Option<String>,
) -> Result<NativeProjectDispositionProposalView> {
    propose(
        root,
        fingerprint,
        rationale,
        accepted_by,
        Disposition::Deviation,
    )
}

fn propose(
    root: &Path,
    fingerprint: &str,
    rationale: &str,
    actor: Option<String>,
    kind: Disposition,
) -> Result<NativeProjectDispositionProposalView> {
    if rationale.trim().is_empty() {
        bail!("A nonblank disposition rationale is required");
    }
    let check_run = query_native_project_check_run(root)?;
    let Some(finding) = check_run
        .findings
        .iter()
        .find(|f| f.fingerprint == fingerprint)
    else {
        bail!("check finding fingerprint {fingerprint} not found in current check run");
    };
    let (contract, action, already) = match kind {
        Disposition::Waiver => ("project_waive_finding_v2", "waive_finding", "waived"),
        Disposition::Deviation => (
            "project_accept_deviation_v2",
            "accept_deviation",
            "accepted_deviation",
        ),
    };
    if finding.status == already {
        bail!("check finding fingerprint {fingerprint} is already {already}");
    }
    let domain = match finding.domain.as_str() {
        "erc" => CheckDomain::ERC,
        "drc" => CheckDomain::DRC,
        "standards" => CheckDomain::Standards,
        other => bail!("fingerprint disposition authoring for domain `{other}` is not implemented"),
    };
    let (project, mut model) = load_native_project_with_resolved_board_and_model(root)?;
    if model.model_revision.0 != check_run.model_revision {
        bail!(
            "Project changed after checking the finding; run the command again on the current revision"
        );
    }
    let source = cli_commit_source()?;
    let provenance =
        WriteProvenance::new("datum-eda-cli", source, format!("{action} {fingerprint}"));
    let (id, write) = match kind {
        Disposition::Waiver => {
            let prepared = build_create_schematic_waiver(
                &model,
                provenance,
                &CreateSchematicWaiverRequest {
                    schematic_id: project.schematic.uuid,
                    domain,
                    fingerprint: fingerprint.into(),
                    rationale: rationale.into(),
                    created_by: actor,
                },
            )?;
            (prepared.waiver_id, prepared.write)
        }
        Disposition::Deviation => {
            let prepared = build_create_schematic_deviation(
                &model,
                provenance,
                &CreateSchematicDeviationRequest {
                    schematic_id: project.schematic.uuid,
                    domain,
                    fingerprint: fingerprint.into(),
                    rationale: rationale.into(),
                    accepted_by: actor,
                },
            )?;
            (prepared.deviation_id, prepared.write)
        }
    };
    // A read-only check query need not have persisted a CheckRun. Never invent
    // a durable evidence link; retain the qualified fingerprint and revision.
    let checks_run = model
        .check_runs
        .contains_key(&check_run.check_run_id)
        .then_some(check_run.check_run_id)
        .into_iter()
        .collect();
    let proposal = create_draft_proposal_from_batch(
        &mut model,
        root,
        ProposalCreateRequest {
            proposal_id: None,
            batch: write.batch,
            rationale: rationale.into(),
            source: match source {
                CommitSource::Manual => ProposalSource::Manual,
                CommitSource::Tool => ProposalSource::Tool,
                CommitSource::Assistant => ProposalSource::Assistant,
                CommitSource::Cli | CommitSource::Test => ProposalSource::Cli,
            },
            checks_run,
            finding_fingerprints: vec![fingerprint.into()],
        },
    )?;
    Ok(NativeProjectDispositionProposalView {
        contract,
        action,
        project_id: model.project.project_id.to_string(),
        waiver_id: matches!(kind, Disposition::Waiver).then_some(id),
        deviation_id: matches!(kind, Disposition::Deviation).then_some(id),
        fingerprint: fingerprint.into(),
        domain: finding.domain.clone(),
        model_revision: model.model_revision.0,
        proposal_id: proposal.proposal_id,
        status: "draft",
        next_step: "Use proposal preview and proposal review --status accepted, then proposal apply; no disposition has been applied yet",
    })
}
