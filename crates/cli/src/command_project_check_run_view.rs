use anyhow::{Result, bail};
use eda_engine::api::{CheckCodeCount, CheckReport, CheckStatus, CheckSummary};
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::rules::ast::RuleType;
use eda_engine::schematic::{
    CheckDeviation, CheckDomain, CheckWaiver, DeviationApprovalStatus, WaiverTarget,
};
use eda_engine::substrate::{
    CheckFinding, CheckRun, CheckRunCoverageEntry, CheckRunProfileBasis, ModelRevision,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;
use uuid::Uuid;

use super::command_project_check_finding_identity::{
    check_finding_evidence, check_finding_fingerprint, check_finding_import_key,
    check_finding_rule_revision, check_finding_standards_basis, is_standards_profile_finding,
};

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

pub(crate) const NATIVE_COMBINED_CHECK_PROFILE: &str = "native-combined";
const ERC_CHECK_PROFILE: &str = "erc";
const DRC_CHECK_PROFILE: &str = "drc";
const STANDARDS_CHECK_PROFILE: &str = "standards";
const MANUFACTURING_CHECK_PROFILE: &str = "manufacturing";
const RELEASE_CHECK_PROFILE: &str = "release";

pub(crate) fn query_native_project_check_profiles(
    root: &Path,
) -> Result<NativeProjectCheckProfilesView> {
    let model = eda_engine::substrate::ProjectResolver::new(root).resolve()?;
    let profiles = native_check_profile_descriptors();
    Ok(NativeProjectCheckProfilesView {
        contract: "check_profiles_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        default_profile_id: "native-combined",
        profile_count: profiles.len(),
        profiles,
    })
}

pub(crate) fn resolve_native_project_check_profile(profile: Option<&str>) -> Result<&'static str> {
    match profile.unwrap_or(NATIVE_COMBINED_CHECK_PROFILE) {
        NATIVE_COMBINED_CHECK_PROFILE => Ok(NATIVE_COMBINED_CHECK_PROFILE),
        ERC_CHECK_PROFILE => Ok(ERC_CHECK_PROFILE),
        DRC_CHECK_PROFILE => Ok(DRC_CHECK_PROFILE),
        STANDARDS_CHECK_PROFILE => Ok(STANDARDS_CHECK_PROFILE),
        MANUFACTURING_CHECK_PROFILE => Ok(MANUFACTURING_CHECK_PROFILE),
        RELEASE_CHECK_PROFILE => Ok(RELEASE_CHECK_PROFILE),
        profile => bail!(
            "unsupported check profile {profile}; current native-project surface supports native-combined, erc, drc, standards, manufacturing, release"
        ),
    }
}

pub(crate) fn filter_check_run_findings_for_profile(
    profile_id: &str,
    findings: &mut Vec<NativeProjectCheckFindingView>,
) {
    findings.retain(|finding| match profile_id {
        NATIVE_COMBINED_CHECK_PROFILE | RELEASE_CHECK_PROFILE => true,
        ERC_CHECK_PROFILE => finding.domain == "erc",
        DRC_CHECK_PROFILE => finding.domain == "drc",
        MANUFACTURING_CHECK_PROFILE => finding.domain == "manufacturing",
        STANDARDS_CHECK_PROFILE => finding.domain == "standards",
        _ => true,
    });
    for (index, finding) in findings.iter_mut().enumerate() {
        finding.index = index;
    }
}

pub(crate) fn check_profile_includes_relationships(profile_id: &str) -> bool {
    is_combined_profile(profile_id)
}

pub(crate) fn check_profile_includes_erc(profile_id: &str) -> bool {
    is_combined_profile(profile_id) || profile_id == ERC_CHECK_PROFILE
}

pub(crate) fn check_profile_includes_artifacts(profile_id: &str) -> bool {
    is_combined_profile(profile_id) || profile_id == MANUFACTURING_CHECK_PROFILE
}

pub(crate) fn check_profile_includes_zone_fills(profile_id: &str) -> bool {
    is_combined_profile(profile_id) || profile_id == STANDARDS_CHECK_PROFILE
}

pub(crate) fn check_profile_drc_rules(profile_id: &str) -> &'static [RuleType] {
    match profile_id {
        NATIVE_COMBINED_CHECK_PROFILE | RELEASE_CHECK_PROFILE | DRC_CHECK_PROFILE => &[
            RuleType::Connectivity,
            RuleType::ClearanceCopper,
            RuleType::TrackWidth,
            RuleType::ViaHole,
            RuleType::ViaAnnularRing,
            RuleType::SilkClearance,
            RuleType::ProcessAperture,
        ],
        STANDARDS_CHECK_PROFILE => &[
            RuleType::TrackWidth,
            RuleType::ViaHole,
            RuleType::ViaAnnularRing,
            RuleType::ProcessAperture,
        ],
        _ => &[],
    }
}

fn is_combined_profile(profile_id: &str) -> bool {
    [NATIVE_COMBINED_CHECK_PROFILE, RELEASE_CHECK_PROFILE].contains(&profile_id)
}

pub(crate) fn profile_basis_for_check_run(profile_id: &str) -> CheckRunProfileBasis {
    let profile = native_check_profile_descriptor(profile_id);
    CheckRunProfileBasis {
        profile_id: profile.profile_id.to_string(),
        domains: profile
            .domains
            .iter()
            .map(|domain| domain.to_string())
            .collect(),
        description: profile.description.to_string(),
        standards_basis: (profile.profile_id == STANDARDS_CHECK_PROFILE)
            .then_some("datum.process_aperture_and_geometry.current".to_string()),
    }
}

pub(crate) fn check_run_coverage_for_profile(profile_id: &str) -> Vec<CheckRunCoverageEntry> {
    [
        coverage_entry(
            profile_id,
            "relationships",
            "resolver_diagnostics",
            "project",
            None,
        ),
        coverage_entry(
            profile_id,
            "erc",
            "schematic_connectivity",
            "schematic",
            None,
        ),
        coverage_entry(profile_id, "drc", "board_geometry", "board", None),
        coverage_entry(
            profile_id,
            "standards",
            "zone_fill_state",
            "board_zones",
            Some("datum.zone_fill_honesty.current"),
        ),
        coverage_entry(
            profile_id,
            "standards",
            "process_aperture_policy",
            "board_pads_tracks_vias",
            Some("datum.process_aperture_and_geometry.current"),
        ),
        coverage_entry(
            profile_id,
            "manufacturing",
            "artifact_validation",
            "generated_artifacts",
            None,
        ),
        not_implemented_entry(
            "drc",
            "clearance_solver",
            "board_copper",
            Some("datum.layout.clearance.future"),
        ),
        not_implemented_entry("drc", "silkscreen_clearance", "board_silkscreen", None),
        not_implemented_entry(
            "erc",
            "hierarchical_power_intent",
            "schematic_hierarchy",
            None,
        ),
    ]
    .into_iter()
    .collect()
}

fn native_check_profile_descriptor(profile_id: &str) -> NativeProjectCheckProfileView {
    native_check_profile_descriptors()
        .into_iter()
        .find(|profile| profile.profile_id == profile_id)
        .unwrap_or(NativeProjectCheckProfileView {
            profile_id: NATIVE_COMBINED_CHECK_PROFILE,
            name: "Native Combined",
            status: "current_default",
            domains: vec!["relationships", "erc", "drc", "standards", "manufacturing"],
            description: "Current deterministic native-project profile combining resolver diagnostics, ERC, DRC, standards, and artifact validation findings.",
            selection_supported: true,
        })
}

fn native_check_profile_descriptors() -> Vec<NativeProjectCheckProfileView> {
    vec![
        NativeProjectCheckProfileView {
            profile_id: "native-combined",
            name: "Native Combined",
            status: "current_default",
            domains: vec!["relationships", "erc", "drc", "standards", "manufacturing"],
            description: "Current deterministic native-project profile combining resolver diagnostics, ERC, DRC, standards, and artifact validation findings.",
            selection_supported: true,
        },
        NativeProjectCheckProfileView {
            profile_id: "erc",
            name: "ERC",
            status: "supported",
            domains: vec!["erc"],
            description: "Electrical-rule focused profile over native schematic connectivity findings.",
            selection_supported: true,
        },
        NativeProjectCheckProfileView {
            profile_id: "drc",
            name: "DRC",
            status: "supported",
            domains: vec!["drc"],
            description: "Physical-rule focused profile over board geometry findings.",
            selection_supported: true,
        },
        NativeProjectCheckProfileView {
            profile_id: "standards",
            name: "Standards",
            status: "supported",
            domains: vec!["standards"],
            description: "Standards-focused profile for process-aperture, track-width, via-geometry, and ZoneFill honesty findings.",
            selection_supported: true,
        },
        NativeProjectCheckProfileView {
            profile_id: "manufacturing",
            name: "Manufacturing",
            status: "supported",
            domains: vec!["manufacturing"],
            description: "Manufacturing-evidence profile for generated artifact validation findings.",
            selection_supported: true,
        },
        NativeProjectCheckProfileView {
            profile_id: "release",
            name: "Release",
            status: "supported",
            domains: vec!["relationships", "erc", "drc", "standards", "manufacturing"],
            description: "Release-gate profile over all currently deterministic native CheckRun domains.",
            selection_supported: true,
        },
    ]
}

fn coverage_entry(
    profile_id: &str,
    domain: &str,
    rule_id: &str,
    target_scope: &str,
    standards_basis: Option<&str>,
) -> CheckRunCoverageEntry {
    CheckRunCoverageEntry {
        domain: domain.to_string(),
        rule_id: rule_id.to_string(),
        status: coverage_status_for_profile(profile_id, domain, rule_id).to_string(),
        target_scope: target_scope.to_string(),
        basis_id: Some(format!("datum.check.coverage.{domain}.{rule_id}.v1")),
        rule_revision: Some("v1".to_string()),
        standards_basis: standards_basis.map(str::to_string),
    }
}

fn not_implemented_entry(
    domain: &str,
    rule_id: &str,
    target_scope: &str,
    basis_id: Option<&str>,
) -> CheckRunCoverageEntry {
    CheckRunCoverageEntry {
        domain: domain.to_string(),
        rule_id: rule_id.to_string(),
        status: "not_implemented".to_string(),
        target_scope: target_scope.to_string(),
        basis_id: basis_id.map(str::to_string),
        rule_revision: None,
        standards_basis: None,
    }
}

fn coverage_status_for_profile(profile_id: &str, domain: &str, rule_id: &str) -> &'static str {
    match profile_id {
        NATIVE_COMBINED_CHECK_PROFILE | RELEASE_CHECK_PROFILE => "evaluated",
        ERC_CHECK_PROFILE => {
            if domain == "erc" {
                "evaluated"
            } else {
                "filtered_by_profile"
            }
        }
        DRC_CHECK_PROFILE => {
            if domain == "drc" {
                "evaluated"
            } else {
                "filtered_by_profile"
            }
        }
        MANUFACTURING_CHECK_PROFILE => {
            if domain == "manufacturing" {
                "evaluated"
            } else {
                "filtered_by_profile"
            }
        }
        STANDARDS_CHECK_PROFILE => {
            if domain == "standards"
                && matches!(rule_id, "process_aperture_policy" | "zone_fill_state")
            {
                "evaluated"
            } else {
                "filtered_by_profile"
            }
        }
        _ => "filtered_by_profile",
    }
}

pub(crate) fn native_check_run_to_substrate(
    project_id: &Uuid,
    view: &NativeProjectCheckRunView,
) -> Result<CheckRun> {
    let summary = serde_json::to_value(&view.summary)?;
    let raw_report = serde_json::to_value(&view.raw_report)?;
    let status = serde_json::to_value(view.status)?
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    Ok(CheckRun {
        check_run_id: view.check_run_id,
        project_id: *project_id,
        model_revision: ModelRevision(view.model_revision.clone()),
        profile_id: view.profile_id.to_string(),
        status,
        summary,
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
        raw_report,
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

pub(crate) fn check_primary_target(source: &str, payload: &serde_json::Value) -> serde_json::Value {
    if let Some(value) = first_object_id(payload) {
        return serde_json::json!({
            "kind": "object_uuid",
            "id": value,
        });
    }
    for key in [
        "object_id",
        "object_uuid",
        "component_uuid",
        "symbol_uuid",
        "pin_uuid",
        "pad_uuid",
        "pad_id",
        "zone_id",
        "artifact_id",
        "net_uuid",
        "uuid",
    ] {
        if let Some(value) = payload.get(key) {
            return serde_json::json!({
                "kind": key,
                "id": value,
            });
        }
    }
    serde_json::json!({
        "kind": source,
        "id": "unknown",
    })
}

pub(crate) fn check_related_targets(
    primary_target: &serde_json::Value,
    payload: &serde_json::Value,
) -> Vec<serde_json::Value> {
    let Some(objects) = payload.get("objects").and_then(serde_json::Value::as_array) else {
        return Vec::new();
    };
    objects
        .iter()
        .filter_map(serde_json::Value::as_str)
        .map(|value| {
            serde_json::json!({
                "kind": "object_uuid",
                "id": value,
            })
        })
        .filter(|target| target != primary_target)
        .collect()
}

fn first_object_id(payload: &serde_json::Value) -> Option<&str> {
    payload
        .get("objects")
        .and_then(serde_json::Value::as_array)
        .and_then(|objects| objects.first())
        .and_then(serde_json::Value::as_str)
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
        "pad_mask_expansion_missing" | "pad_mask_expansion_below_rule"
        | "pad_paste_reduction_missing" | "pad_paste_reduction_below_rule"
        | "track_width_below_min" | "via_hole_out_of_range" | "via_annular_below_min" => {
            "Run datum-eda check repair-standards to create reviewed repair proposals.".to_string()
        }
        _ => "Inspect the primary target, then fix, waive, or accept the finding through the check workflow.".to_string(),
    })
}
