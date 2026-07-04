use std::path::Path;

use anyhow::{Result, bail};
use eda_engine::api::CheckStatus;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ReleaseCheckGateView {
    pub(crate) check_run_id: Uuid,
    pub(crate) profile_id: &'static str,
    pub(crate) status: CheckStatus,
    pub(crate) finding_count: usize,
    pub(crate) active_error_count: usize,
    pub(crate) active_error_codes: Vec<String>,
}

pub(crate) fn release_check_gate(root: &Path) -> Result<ReleaseCheckGateView> {
    let check_run = crate::query_native_project_check_run_with_profile(root, Some("release"))?;
    let active_error_findings = check_run
        .findings
        .iter()
        .filter(|finding| finding.status == "active" && finding.severity == "error")
        .collect::<Vec<_>>();
    let active_error_codes = active_error_findings
        .iter()
        .map(|finding| finding.code.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    Ok(ReleaseCheckGateView {
        check_run_id: check_run.check_run_id,
        profile_id: check_run.profile_id,
        status: check_run.status,
        finding_count: check_run.finding_count,
        active_error_count: active_error_findings.len(),
        active_error_codes,
    })
}

pub(crate) fn ensure_release_check_gate_clear(root: &Path) -> Result<ReleaseCheckGateView> {
    let gate = release_check_gate(root)?;
    if gate.active_error_count > 0 {
        bail!("{}", release_check_gate_error(&gate));
    }
    Ok(gate)
}

pub(crate) fn release_check_gate_error(gate: &ReleaseCheckGateView) -> String {
    format!(
        "release check gate failed: {} active error code(s) [{}] in check run {}",
        gate.active_error_count,
        gate.active_error_codes.join(","),
        gate.check_run_id
    )
}
