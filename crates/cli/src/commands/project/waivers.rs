//! Thin CLI callers for check-finding dispositions (waive / accept-deviation).
//!
//! All operation authoring lives in the engine facade
//! (`eda_engine::api::native_write::waivers`); this file only validates the
//! finding against the current check run, builds the typed request, and
//! renders the view.

use std::path::Path;

use anyhow::{Result, bail};
use eda_engine::api::native_write::WriteProvenance;
use eda_engine::api::native_write::waivers::{
    CreateSchematicDeviationRequest, CreateSchematicWaiverRequest,
    create_schematic_deviation_and_commit, create_schematic_waiver_and_commit,
};
use eda_engine::schematic::CheckDomain;
use serde::Serialize;
use uuid::Uuid;

use super::{load_native_project_with_resolved_board_and_model, query_native_project_check_run};

use crate::cli_commit_source;

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
    let (waiver_id, report) = create_schematic_waiver_and_commit(
        &mut model,
        root,
        WriteProvenance::new(
            "datum-eda-cli",
            cli_commit_source()?,
            format!("waive check finding {fingerprint}"),
        ),
        &CreateSchematicWaiverRequest {
            schematic_id: project.schematic.uuid,
            domain,
            fingerprint: fingerprint.to_string(),
            rationale: rationale.to_string(),
            created_by,
        },
    )?;
    Ok(NativeProjectWaiveFindingView {
        contract: "project_waive_finding_v1",
        action: "waive_finding",
        project_id: model.project.project_id.to_string(),
        waiver_id,
        fingerprint: fingerprint.to_string(),
        domain: finding.domain.clone(),
        before_model_revision: report.transaction.before_model_revision.0.clone(),
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
    let (deviation_id, report) = create_schematic_deviation_and_commit(
        &mut model,
        root,
        WriteProvenance::new(
            "datum-eda-cli",
            cli_commit_source()?,
            format!("accept check finding deviation {fingerprint}"),
        ),
        &CreateSchematicDeviationRequest {
            schematic_id: project.schematic.uuid,
            domain,
            fingerprint: fingerprint.to_string(),
            rationale: rationale.to_string(),
            accepted_by,
        },
    )?;
    Ok(NativeProjectAcceptDeviationView {
        contract: "project_accept_deviation_v1",
        action: "accept_deviation",
        project_id: model.project.project_id.to_string(),
        deviation_id,
        fingerprint: fingerprint.to_string(),
        domain: finding.domain.clone(),
        before_model_revision: report.transaction.before_model_revision.0.clone(),
        after_model_revision: report.transaction.after_model_revision.0.clone(),
        transaction_id: report.transaction.transaction_id,
        journal_len: report.journal_len,
        status: "applied",
    })
}
