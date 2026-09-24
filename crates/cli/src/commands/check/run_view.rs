use super::command_project_check_finding_identity::{
    check_finding_evidence, check_finding_fingerprint, check_finding_import_key,
    check_finding_rule_revision, check_finding_standards_basis,
    check_finding_standards_basis_detail, is_standards_profile_finding, standards_basis_detail,
};
use super::command_project_check_targets::{check_primary_target, check_related_targets};
use anyhow::{Result, bail};
use eda_engine::api::{CheckCodeCount, CheckReport, CheckStatus, CheckSummary};
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::rules::ast::RuleType;
use eda_engine::schematic::{
    CheckDeviation, CheckDomain, CheckWaiver, DeviationApprovalStatus, WaiverTarget,
};
use eda_engine::substrate::{
    CHECK_RUN_SCHEMA_VERSION, CheckFinding, CheckRun, CheckRunCoverageEntry, CheckRunProfileBasis,
    ModelRevision, StandardsBasis,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectCheckRunView {
    pub(crate) contract: &'static str,
    pub(crate) persisted: bool,
    pub(crate) check_run_id: Uuid,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) profile_id: &'static str,
    pub(crate) status: CheckStatus,
    pub(crate) summary: CheckSummary,
    pub(crate) finding_count: usize,
    pub(crate) findings: Vec<NativeProjectCheckFindingView>,
    pub(crate) proposal_refs: Vec<String>,
    pub(crate) proposal_links: Vec<NativeProjectCheckProposalLinkView>,
    pub(crate) profile_basis: CheckRunProfileBasis,
    pub(crate) coverage: Vec<CheckRunCoverageEntry>,
    pub(crate) raw_report: CheckReport,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectCheckFindingView {
    pub(crate) finding_id: Uuid,
    pub(crate) index: usize,
    pub(crate) source: &'static str,
    pub(crate) code: String,
    pub(crate) severity: String,
    pub(crate) fingerprint: String,
    pub(crate) domain: String,
    pub(crate) rule_id: String,
    pub(crate) standards_basis: Option<String>,
    pub(crate) standards_basis_detail: Option<StandardsBasis>,
    pub(crate) rule_revision: Option<String>,
    pub(crate) import_key: Option<String>,
    pub(crate) status: String,
    pub(crate) primary_target: serde_json::Value,
    pub(crate) related_targets: Vec<serde_json::Value>,
    pub(crate) message: String,
    pub(crate) explanation: String,
    pub(crate) suggested_next_action: Option<String>,
    pub(crate) evidence: Vec<serde_json::Value>,
    pub(crate) payload: serde_json::Value,
    pub(crate) proposal_refs: Vec<String>,
    pub(crate) proposal_links: Vec<NativeProjectCheckProposalLinkView>,
    pub(crate) waiver_refs: Vec<String>,
    pub(crate) deviation_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectCheckProposalLinkView {
    pub(crate) proposal_id: String,
    pub(crate) status: String,
    pub(crate) source: String,
    pub(crate) rationale: String,
    pub(crate) prepared_against: String,
    pub(crate) checks_run: Vec<String>,
    pub(crate) finding_fingerprints: Vec<String>,
    pub(crate) matched_fingerprint: Option<String>,
    pub(crate) prepared_against_current_model: bool,
    pub(crate) can_apply: bool,
    pub(crate) blocker_codes: Vec<String>,
    pub(crate) command_templates: NativeProjectCheckProposalCommandTemplates,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectCheckProposalCommandTemplates {
    pub(crate) show: String,
    pub(crate) preview: String,
    pub(crate) validate: String,
    pub(crate) accept_apply: String,
    pub(crate) apply: String,
    pub(crate) defer: String,
    pub(crate) reject: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectCheckProfilesView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) default_profile_id: &'static str,
    pub(crate) profile_count: usize,
    pub(crate) profiles: Vec<NativeProjectCheckProfileView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectCheckProfileView {
    pub(crate) profile_id: &'static str,
    pub(crate) name: &'static str,
    pub(crate) status: &'static str,
    pub(crate) domains: Vec<&'static str>,
    pub(crate) description: &'static str,
    pub(crate) selection_supported: bool,
}

#[path = "profile_coverage.rs"]
mod profile_coverage;
pub(crate) use profile_coverage::*;

pub(crate) fn native_check_run_to_substrate(
    project_id: &Uuid,
    view: &NativeProjectCheckRunView,
) -> Result<CheckRun> {
    let status = serde_json::to_value(view.status)?
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    Ok(CheckRun {
        schema_version: CHECK_RUN_SCHEMA_VERSION,
        check_run_id: view.check_run_id,
        project_id: *project_id,
        model_revision: ModelRevision(view.model_revision.clone()),
        profile_id: view.profile_id.to_string(),
        status,
        summary: serde_json::to_value(&view.summary)?,
        finding_count: view.finding_count,
        findings: view
            .findings
            .iter()
            .map(|finding| CheckFinding {
                finding_id: finding.finding_id,
                index: finding.index,
                source: finding.source.to_string(),
                code: finding.code.clone(),
                severity: finding.severity.clone(),
                fingerprint: finding.fingerprint.clone(),
                domain: finding.domain.clone(),
                rule_id: finding.rule_id.clone(),
                standards_basis: finding.standards_basis.clone(),
                standards_basis_detail: finding.standards_basis_detail.clone(),
                rule_revision: finding.rule_revision.clone(),
                import_key: finding.import_key.clone(),
                status: finding.status.clone(),
                primary_target: finding.primary_target.clone(),
                related_targets: finding.related_targets.clone(),
                message: finding.message.clone(),
                explanation: finding.explanation.clone(),
                suggested_next_action: finding.suggested_next_action.clone(),
                evidence: finding.evidence.clone(),
                payload: finding.payload.clone(),
                proposal_refs: finding.proposal_refs.clone(),
                proposal_links: finding
                    .proposal_links
                    .iter()
                    .filter_map(|link| serde_json::to_value(link).ok())
                    .collect(),
                waiver_refs: finding
                    .waiver_refs
                    .iter()
                    .filter_map(|value| Uuid::parse_str(value).ok())
                    .collect(),
                deviation_refs: finding
                    .deviation_refs
                    .iter()
                    .filter_map(|value| Uuid::parse_str(value).ok())
                    .collect(),
            })
            .collect(),
        proposal_refs: view.proposal_refs.clone(),
        proposal_links: view
            .proposal_links
            .iter()
            .filter_map(|link| serde_json::to_value(link).ok())
            .collect(),
        profile_basis: view.profile_basis.clone(),
        coverage: view.coverage.clone(),
        raw_report: serde_json::to_value(&view.raw_report)?,
    })
}

pub(crate) fn append_finding_values(
    project_id: &Uuid,
    model_revision: &str,
    source: &'static str,
    report_value: &serde_json::Value,
    findings: &mut Vec<NativeProjectCheckFindingView>,
) -> Result<()> {
    let key = match source {
        "diagnostic" => "diagnostics",
        "erc" => "erc",
        "drc" => "drc",
        _ => return Ok(()),
    };
    let Some(values) = report_value.get(key).and_then(serde_json::Value::as_array) else {
        return Ok(());
    };
    for value in values {
        let index = findings.len();
        let code = value
            .get("code")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(source)
            .to_string();
        let severity = value
            .get("severity")
            .or_else(|| value.get("status"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let material = format!(
            "datum-eda:check-finding:{model_revision}:{source}:{index}:{}",
            to_json_deterministic(value)?
        );
        let finding_id = Uuid::new_v5(project_id, material.as_bytes());
        let domain = finding_domain(source, &code);
        let rule_id = code.clone();
        let primary_target = check_primary_target(source, value);
        let standards_basis = check_finding_standards_basis(&code).map(str::to_string);
        let standards_basis_detail = check_finding_standards_basis_detail(&code);
        let rule_revision = check_finding_rule_revision(&code).map(str::to_string);
        let import_key = check_finding_import_key(value);
        let related_targets = check_related_targets(&primary_target, value);
        let message = check_finding_message(&code, value);
        let explanation = check_finding_explanation(&code, &message, value);
        let suggested_next_action = check_finding_suggested_next_action(&code, value);
        let fingerprint = if let Some(fingerprint) = value
            .get("fingerprint")
            .and_then(serde_json::Value::as_str)
            .filter(|fingerprint| fingerprint.starts_with("sha256:"))
        {
            fingerprint.to_string()
        } else {
            check_finding_fingerprint(
                &domain,
                &rule_id,
                standards_basis.as_deref(),
                rule_revision.as_deref(),
                import_key.as_deref(),
                &primary_target,
                value,
            )?
        };
        let status = if value
            .get("waived")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
        {
            "waived"
        } else {
            "active"
        };
        let evidence = check_finding_evidence(&code, value, &primary_target);
        findings.push(NativeProjectCheckFindingView {
            finding_id,
            index,
            source,
            code,
            severity,
            fingerprint,
            domain,
            rule_id,
            standards_basis,
            standards_basis_detail,
            rule_revision,
            import_key,
            status: status.to_string(),
            primary_target,
            related_targets,
            message,
            explanation,
            suggested_next_action,
            evidence,
            payload: value.clone(),
            proposal_refs: Vec::new(),
            proposal_links: Vec::new(),
            waiver_refs: Vec::new(),
            deviation_refs: Vec::new(),
        });
    }
    Ok(())
}

pub(crate) fn apply_fingerprint_waivers(
    findings: &mut [NativeProjectCheckFindingView],
    waivers: &[CheckWaiver],
) {
    for finding in findings {
        for waiver in waivers {
            if waiver_matches_finding(waiver, finding) {
                finding.status = "waived".to_string();
                let waiver_ref = waiver.uuid.to_string();
                if !finding.waiver_refs.iter().any(|value| value == &waiver_ref) {
                    finding.waiver_refs.push(waiver_ref);
                }
            }
        }
        finding.waiver_refs.sort();
        finding.waiver_refs.dedup();
    }
}

pub(crate) fn apply_accepted_deviations(
    findings: &mut [NativeProjectCheckFindingView],
    deviations: &[CheckDeviation],
) {
    for finding in findings {
        for deviation in deviations {
            if deviation_matches_finding(deviation, finding) {
                finding.status = "accepted_deviation".to_string();
                let deviation_ref = deviation.uuid.to_string();
                if !finding
                    .deviation_refs
                    .iter()
                    .any(|value| value == &deviation_ref)
                {
                    finding.deviation_refs.push(deviation_ref);
                }
            }
        }
        finding.deviation_refs.sort();
        finding.deviation_refs.dedup();
    }
}

fn waiver_matches_finding(waiver: &CheckWaiver, finding: &NativeProjectCheckFindingView) -> bool {
    if !waiver_domain_matches_finding(waiver.domain.clone(), finding) {
        return false;
    }
    match &waiver.target {
        WaiverTarget::Fingerprint(fingerprint) => fingerprint == &finding.fingerprint,
        _ => false,
    }
}

fn deviation_matches_finding(
    deviation: &CheckDeviation,
    finding: &NativeProjectCheckFindingView,
) -> bool {
    if deviation.approval_status != DeviationApprovalStatus::Accepted {
        return false;
    }
    if !waiver_domain_matches_finding(deviation.domain.clone(), finding) {
        return false;
    }
    match &deviation.target {
        WaiverTarget::Fingerprint(fingerprint) => fingerprint == &finding.fingerprint,
        _ => false,
    }
}

fn waiver_domain_matches_finding(
    domain: CheckDomain,
    finding: &NativeProjectCheckFindingView,
) -> bool {
    matches!(
        (domain, finding.domain.as_str()),
        (CheckDomain::ERC, "erc")
            | (CheckDomain::DRC, "drc")
            | (CheckDomain::Standards, "standards")
    )
}

pub(crate) fn summarize_check_run_findings(
    findings: &[NativeProjectCheckFindingView],
) -> CheckSummary {
    let mut by_code = BTreeMap::<String, usize>::new();
    let mut errors = 0usize;
    let mut warnings = 0usize;
    let mut infos = 0usize;
    let mut waived = 0usize;
    for finding in findings {
        *by_code.entry(finding.code.clone()).or_default() += 1;
        if finding.status == "waived" {
            waived += 1;
            continue;
        }
        if finding.status == "accepted_deviation" {
            continue;
        }
        match finding.severity.as_str() {
            "error" => errors += 1,
            "warning" => warnings += 1,
            "info" => infos += 1,
            _ => infos += 1,
        }
    }
    let status = if errors > 0 {
        CheckStatus::Error
    } else if warnings > 0 {
        CheckStatus::Warning
    } else if infos > 0 {
        CheckStatus::Info
    } else {
        CheckStatus::Ok
    };
    CheckSummary {
        status,
        errors,
        warnings,
        infos,
        waived,
        by_code: by_code
            .into_iter()
            .map(|(code, count)| CheckCodeCount { code, count })
            .collect(),
    }
}

pub(crate) fn finding_domain(source: &str, code: &str) -> String {
    if is_standards_profile_finding(code) {
        return "standards".to_string();
    }
    match source {
        "erc" => "erc",
        "drc" => "drc",
        "artifact" => "manufacturing",
        "zone_fill" => "drc",
        "diagnostic" => "relationships",
        _ => source,
    }
    .to_string()
}

pub(crate) fn check_finding_message(code: &str, payload: &serde_json::Value) -> String {
    payload
        .get("message")
        .or_else(|| payload.get("description"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or(code)
        .to_string()
}

pub(crate) fn check_finding_explanation(
    code: &str,
    message: &str,
    payload: &serde_json::Value,
) -> String {
    if let Some(explanation) = payload
        .get("explanation")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        return explanation.to_string();
    }
    let observed = payload
        .get("observed")
        .or_else(|| payload.get("actual"))
        .and_then(serde_json::Value::as_str);
    let expected = payload
        .get("expected")
        .or_else(|| payload.get("required"))
        .and_then(serde_json::Value::as_str);
    match (observed, expected) {
        (Some(observed), Some(expected)) => {
            format!("{message} Rule {code} observed {observed}, expected {expected}.")
        }
        _ => format!("{message} Rule {code} produced this finding from the recorded evidence."),
    }
}

pub(crate) fn check_finding_suggested_next_action(
    code: &str,
    payload: &serde_json::Value,
) -> Option<String> {
    if let Some(action) = payload
        .get("suggested_next_action")
        .or_else(|| payload.get("suggestion"))
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        return Some(action.to_string());
    }
    Some(match code {
        "zone_fill_unfilled" => {
            "Fill or explicitly waive the zone before generating production copper.".to_string()
        }
        "zone_fill_stale" => {
            "Regenerate zone fills or explicitly waive the stale evidence before production output."
                .to_string()
        }
        "zone_fill_unsupported" => {
            "Review the zone fill limitation, simplify the zone, or explicitly waive before production output."
                .to_string()
        }
        "artifact_validation_invalid" => {
            "Regenerate or validate the referenced artifact before release.".to_string()
        }
        "pad_process_aperture_inherited_from_copper"
        | "pad_mask_expansion_missing" | "pad_mask_expansion_below_rule"
        | "pad_paste_reduction_missing" | "pad_paste_reduction_below_rule"
        | "track_width_below_min" | "via_hole_out_of_range" | "via_annular_below_min" => {
            "Run datum-eda check repair-standards to create reviewed repair proposals.".to_string()
        }
        _ => "Inspect the primary target, then fix, waive, or accept the finding through the check workflow.".to_string(),
    })
}
