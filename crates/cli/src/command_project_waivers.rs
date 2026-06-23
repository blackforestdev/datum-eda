use std::path::Path;

use anyhow::{Result, bail};
use eda_engine::schematic::{
    CheckDeviation, CheckDomain, CheckWaiver, DeviationApprovalStatus, WaiverTarget,
};
use eda_engine::substrate::{CommitProvenance, CommitSource, Operation, OperationBatch};
use serde::Serialize;
use uuid::Uuid;

use super::{load_native_project_with_resolved_board_and_model, query_native_project_check_run};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectWaiveFindingView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) waiver_id: Uuid,
    pub(crate) fingerprint: String,
    pub(crate) domain: String,
    pub(crate) before_model_revision: String,
    pub(crate) after_model_revision: String,
    pub(crate) transaction_id: Uuid,
    pub(crate) journal_len: usize,
    pub(crate) status: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectAcceptDeviationView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_id: String,
    pub(crate) deviation_id: Uuid,
    pub(crate) fingerprint: String,
    pub(crate) domain: String,
    pub(crate) before_model_revision: String,
    pub(crate) after_model_revision: String,
    pub(crate) transaction_id: Uuid,
    pub(crate) journal_len: usize,
    pub(crate) status: &'static str,
}

pub(crate) fn waive_native_project_finding(
    root: &Path,
    fingerprint: &str,
    rationale: &str,
    created_by: Option<String>,
) -> Result<NativeProjectWaiveFindingView> {
    let check_run = query_native_project_check_run(root)?;
    let Some(finding) = check_run
        .findings
        .iter()
        .find(|finding| finding.fingerprint == fingerprint)
    else {
        bail!("check finding fingerprint {fingerprint} not found in current check run");
    };
    if finding.status == "waived" {
        bail!("check finding fingerprint {fingerprint} is already waived");
    }
    let domain = match finding.domain.as_str() {
        "erc" => CheckDomain::ERC,
        "drc" => CheckDomain::DRC,
        "standards" => CheckDomain::Standards,
        other => bail!("fingerprint waiver authoring for domain `{other}` is not implemented"),
    };
    let (project, mut model) = load_native_project_with_resolved_board_and_model(root)?;
    let waiver_id = Uuid::new_v5(
        &model.project.project_id,
        format!(
            "datum-eda:schematic-waiver:{}:{}:{}",
            model.model_revision.0, fingerprint, rationale
        )
        .as_bytes(),
    );
    let waiver = CheckWaiver {
        uuid: waiver_id,
        domain: domain.clone(),
        target: WaiverTarget::Fingerprint(fingerprint.to_string()),
        rationale: rationale.to_string(),
        created_by,
    };
    let waiver_payload = serde_json::to_value(&waiver)?;
    let before_model_revision = model.model_revision.0.clone();
    let report = model.commit_journaled(
        root,
        OperationBatch {
            batch_id: Uuid::new_v5(&model.project.project_id, waiver_id.as_bytes()),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "datum-eda-cli".to_string(),
                source: CommitSource::Cli,
                reason: format!("waive check finding {fingerprint}"),
            },
            operations: vec![Operation::CreateSchematicWaiver {
                schematic_id: project.schematic.uuid,
                waiver_id,
                waiver: waiver_payload,
            }],
        },
    )?;
    Ok(NativeProjectWaiveFindingView {
        contract: "project_waive_finding_v1",
        action: "waive_finding",
        project_id: model.project.project_id.to_string(),
        waiver_id,
        fingerprint: fingerprint.to_string(),
        domain: finding.domain.clone(),
        before_model_revision,
        after_model_revision: report.transaction.after_model_revision.0.clone(),
        transaction_id: report.transaction.transaction_id,
        journal_len: report.journal_len,
        status: "applied",
    })
}

pub(crate) fn accept_native_project_deviation(
    root: &Path,
    fingerprint: &str,
    rationale: &str,
    accepted_by: Option<String>,
) -> Result<NativeProjectAcceptDeviationView> {
    let check_run = query_native_project_check_run(root)?;
    let Some(finding) = check_run
        .findings
        .iter()
        .find(|finding| finding.fingerprint == fingerprint)
    else {
        bail!("check finding fingerprint {fingerprint} not found in current check run");
    };
    if finding.status == "accepted_deviation" {
        bail!("check finding fingerprint {fingerprint} is already accepted as a deviation");
    }
    let domain = match finding.domain.as_str() {
        "erc" => CheckDomain::ERC,
        "drc" => CheckDomain::DRC,
        "standards" => CheckDomain::Standards,
        other => bail!("fingerprint deviation authoring for domain `{other}` is not implemented"),
    };
    let (project, mut model) = load_native_project_with_resolved_board_and_model(root)?;
    let deviation_id = Uuid::new_v5(
        &model.project.project_id,
        format!(
            "datum-eda:schematic-deviation:{}:{}:{}",
            model.model_revision.0, fingerprint, rationale
        )
        .as_bytes(),
    );
    let deviation = CheckDeviation {
        uuid: deviation_id,
        domain: domain.clone(),
        target: WaiverTarget::Fingerprint(fingerprint.to_string()),
        rationale: rationale.to_string(),
        accepted_by,
        approval_status: DeviationApprovalStatus::Accepted,
    };
    let deviation_payload = serde_json::to_value(&deviation)?;
    let before_model_revision = model.model_revision.0.clone();
    let report = model.commit_journaled(
        root,
        OperationBatch {
            batch_id: Uuid::new_v5(&model.project.project_id, deviation_id.as_bytes()),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "datum-eda-cli".to_string(),
                source: CommitSource::Cli,
                reason: format!("accept check finding deviation {fingerprint}"),
            },
            operations: vec![Operation::CreateSchematicDeviation {
                schematic_id: project.schematic.uuid,
                deviation_id,
                deviation: deviation_payload,
            }],
        },
    )?;
    Ok(NativeProjectAcceptDeviationView {
        contract: "project_accept_deviation_v1",
        action: "accept_deviation",
        project_id: model.project.project_id.to_string(),
        deviation_id,
        fingerprint: fingerprint.to_string(),
        domain: finding.domain.clone(),
        before_model_revision,
        after_model_revision: report.transaction.after_model_revision.0.clone(),
        transaction_id: report.transaction.transaction_id,
        journal_len: report.journal_len,
        status: "applied",
    })
}
