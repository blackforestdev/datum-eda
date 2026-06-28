use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::generated_evidence::{persist_generated_evidence, validate_filename_uuid};
use super::{
    ModelRevision, ResolveDiagnostic, SourceShardKind, SourceShardRef, read_json_value,
    source_shard_ref_builders::source_shard_ref_for_bytes,
};

pub const CHECK_RUN_SCHEMA_VERSION: u64 = 1;

fn default_check_run_schema_version() -> u64 {
    CHECK_RUN_SCHEMA_VERSION
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckFinding {
    pub finding_id: Uuid,
    pub index: usize,
    pub source: String,
    pub code: String,
    pub severity: String,
    #[serde(default)]
    pub fingerprint: String,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub rule_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standards_basis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standards_basis_detail: Option<StandardsBasis>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import_key: Option<String>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub primary_target: serde_json::Value,
    #[serde(default)]
    pub related_targets: Vec<serde_json::Value>,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub explanation: String,
    #[serde(default)]
    pub suggested_next_action: Option<String>,
    #[serde(default)]
    pub evidence: Vec<serde_json::Value>,
    pub payload: serde_json::Value,
    pub proposal_refs: Vec<String>,
    #[serde(default)]
    pub proposal_links: Vec<serde_json::Value>,
    #[serde(default)]
    pub waiver_refs: Vec<Uuid>,
    #[serde(default)]
    pub deviation_refs: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckRun {
    #[serde(default = "default_check_run_schema_version")]
    pub schema_version: u64,
    pub check_run_id: Uuid,
    pub project_id: Uuid,
    pub model_revision: ModelRevision,
    pub profile_id: String,
    pub status: String,
    pub summary: serde_json::Value,
    pub finding_count: usize,
    pub findings: Vec<CheckFinding>,
    pub proposal_refs: Vec<String>,
    #[serde(default)]
    pub proposal_links: Vec<serde_json::Value>,
    #[serde(default)]
    pub profile_basis: CheckRunProfileBasis,
    #[serde(default)]
    pub coverage: Vec<CheckRunCoverageEntry>,
    pub raw_report: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CheckRunProfileBasis {
    #[serde(default)]
    pub profile_id: String,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub standards_basis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standards_basis_detail: Option<StandardsBasis>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckRunCoverageEntry {
    pub domain: String,
    pub rule_id: String,
    pub status: String,
    pub target_scope: String,
    #[serde(default)]
    pub basis_id: Option<String>,
    #[serde(default)]
    pub rule_revision: Option<String>,
    #[serde(default)]
    pub standards_basis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standards_basis_detail: Option<StandardsBasis>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandardsBasis {
    pub basis_id: String,
    pub registry_entry_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision_or_profile: Option<String>,
    pub selected_by: String,
    pub selection_scope: String,
    pub basis_kind: String,
    pub disposition: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uncertainty: Option<String>,
    pub provenance: String,
}

pub const PROCESS_APERTURE_STANDARDS_BASIS_ID: &str = "datum.process_aperture_and_geometry.current";
pub const ZONE_FILL_HONESTY_STANDARDS_BASIS_ID: &str = "datum.zone_fill_honesty.current";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandardsBasisRegistryEntry {
    pub basis_id: &'static str,
    pub registry_entry_ref: &'static str,
    pub revision_or_profile: Option<&'static str>,
    pub selected_by: &'static str,
    pub selection_scope: &'static str,
    pub basis_kind: &'static str,
    pub disposition: &'static str,
    pub provenance: &'static str,
}

pub const CHECK_RUN_STANDARDS_BASIS_REGISTRY: &[StandardsBasisRegistryEntry] = &[
    StandardsBasisRegistryEntry {
        basis_id: PROCESS_APERTURE_STANDARDS_BASIS_ID,
        registry_entry_ref: "datum.registry.standards.process_aperture_and_geometry",
        revision_or_profile: Some("current"),
        selected_by: "datum.check.profile",
        selection_scope: "board_pads_tracks_vias",
        basis_kind: "process_aperture_geometry",
        disposition: "declared",
        provenance: "datum-eda check standards basis registry v1",
    },
    StandardsBasisRegistryEntry {
        basis_id: ZONE_FILL_HONESTY_STANDARDS_BASIS_ID,
        registry_entry_ref: "datum.registry.standards.zone_fill_honesty",
        revision_or_profile: Some("current"),
        selected_by: "datum.check.profile",
        selection_scope: "board_zones",
        basis_kind: "zone_fill_honesty",
        disposition: "declared",
        provenance: "datum-eda check standards basis registry v1",
    },
];

pub fn standards_basis_registry_entry(
    basis_id: &str,
) -> Option<&'static StandardsBasisRegistryEntry> {
    CHECK_RUN_STANDARDS_BASIS_REGISTRY
        .iter()
        .find(|entry| entry.basis_id == basis_id)
}

pub fn standards_basis_for_id(basis_id: &str) -> Option<StandardsBasis> {
    let entry = standards_basis_registry_entry(basis_id)?;
    Some(StandardsBasis {
        basis_id: entry.basis_id.to_string(),
        registry_entry_ref: entry.registry_entry_ref.to_string(),
        revision_or_profile: entry.revision_or_profile.map(str::to_string),
        selected_by: entry.selected_by.to_string(),
        selection_scope: entry.selection_scope.to_string(),
        basis_kind: entry.basis_kind.to_string(),
        disposition: entry.disposition.to_string(),
        evidence_refs: Vec::new(),
        uncertainty: None,
        provenance: entry.provenance.to_string(),
    })
}

pub fn standards_basis_id_for_check_code(code: &str) -> Option<&'static str> {
    match code {
        "pad_process_aperture_inherited_from_copper"
        | "pad_process_aperture_inconsistent_with_peer_footprint"
        | "pad_mask_expansion_missing"
        | "pad_mask_expansion_below_rule"
        | "pad_paste_reduction_missing"
        | "pad_paste_reduction_below_rule"
        | "track_width_below_min"
        | "via_hole_out_of_range"
        | "via_annular_below_min" => Some(PROCESS_APERTURE_STANDARDS_BASIS_ID),
        "zone_fill_unfilled" | "zone_fill_stale" | "zone_fill_unsupported" => {
            Some(ZONE_FILL_HONESTY_STANDARDS_BASIS_ID)
        }
        _ => None,
    }
}

#[allow(dead_code)]
pub(super) fn persist_check_run(
    project_root: &Path,
    run: &CheckRun,
) -> Result<PathBuf, crate::error::EngineError> {
    validate_check_run(run).map_err(crate::error::EngineError::Validation)?;
    persist_generated_evidence(project_root, ".datum/check_runs", &run.check_run_id, run)
}

pub(super) fn read_check_run_shards(
    project_root: &Path,
) -> (
    Vec<SourceShardRef>,
    BTreeMap<Uuid, CheckRun>,
    Vec<ResolveDiagnostic>,
) {
    let run_dir = project_root.join(".datum/check_runs");
    let mut shards = Vec::new();
    let mut runs = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let Ok(entries) = std::fs::read_dir(&run_dir) else {
        return (shards, runs, diagnostics);
    };
    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();

    for path in paths {
        let Some(filename) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let relative_path = format!(".datum/check_runs/{filename}");
        let path = project_root.join(&relative_path);
        match read_check_run_shard(path, relative_path) {
            Ok((shard, run)) => {
                runs.insert(run.check_run_id, run);
                shards.push(shard);
            }
            Err(error) => diagnostics.push(error),
        }
    }

    (shards, runs, diagnostics)
}

fn read_check_run_shard(
    path: PathBuf,
    relative_path: String,
) -> Result<(SourceShardRef, CheckRun), ResolveDiagnostic> {
    let bytes = std::fs::read(&path).map_err(|error| ResolveDiagnostic {
        code: "missing_check_run".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let value = read_json_value(&path).map_err(|error| ResolveDiagnostic {
        code: "invalid_check_run".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let schema_version = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64);
    let shard = source_shard_ref_for_bytes(
        SourceShardKind::CheckRun,
        path,
        relative_path,
        schema_version,
        &bytes,
        "invalid_check_run",
    )?;
    let run = serde_json::from_value::<CheckRun>(value).map_err(|error| ResolveDiagnostic {
        code: "invalid_check_run".to_string(),
        message: error.to_string(),
        path: Some(shard.path.clone()),
    })?;
    validate_filename_uuid(&shard.path, run.check_run_id, "invalid_check_run")?;
    validate_check_run(&run).map_err(|message| ResolveDiagnostic {
        code: "invalid_check_run".to_string(),
        message,
        path: Some(shard.path.clone()),
    })?;
    Ok((shard, run))
}

pub(super) fn validate_check_run(run: &CheckRun) -> Result<(), String> {
    if run.schema_version != CHECK_RUN_SCHEMA_VERSION {
        return Err(format!(
            "unsupported check run schema_version {}; supported {}",
            run.schema_version, CHECK_RUN_SCHEMA_VERSION
        ));
    }
    if run.profile_id.trim().is_empty() {
        return Err("check run profile_id must not be blank".to_string());
    }
    let status = run.status.to_ascii_lowercase();
    if !matches!(
        status.as_str(),
        "ok" | "info" | "warning" | "error" | "failed"
    ) {
        return Err(format!("unsupported check run status {}", run.status));
    }
    if run.finding_count != run.findings.len() {
        return Err(format!(
            "check run finding_count {} does not match findings length {}",
            run.finding_count,
            run.findings.len()
        ));
    }
    for (index, coverage) in run.coverage.iter().enumerate() {
        validate_check_run_coverage(coverage, index)?;
    }
    if let Some(detail) = &run.profile_basis.standards_basis_detail {
        validate_standards_basis(detail, "check run profile_basis standards_basis_detail")?;
    }
    for (expected_index, finding) in run.findings.iter().enumerate() {
        validate_check_finding(finding, expected_index)?;
    }
    Ok(())
}

fn validate_check_run_coverage(
    coverage: &CheckRunCoverageEntry,
    index: usize,
) -> Result<(), String> {
    if coverage.domain.trim().is_empty() {
        return Err(format!(
            "check run coverage {index} domain must not be blank"
        ));
    }
    if coverage.rule_id.trim().is_empty() {
        return Err(format!(
            "check run coverage {index} rule_id must not be blank"
        ));
    }
    if coverage.target_scope.trim().is_empty() {
        return Err(format!(
            "check run coverage {index} target_scope must not be blank"
        ));
    }
    match coverage.status.as_str() {
        "evaluated" | "filtered_by_profile" | "not_implemented" | "not_applicable" => Ok(()),
        other => Err(format!(
            "check run coverage {index} has unsupported status {other}"
        )),
    }?;
    if let Some(detail) = &coverage.standards_basis_detail {
        validate_standards_basis(
            detail,
            &format!("check run coverage {index} standards_basis_detail"),
        )?;
    }
    Ok(())
}

fn validate_check_finding(finding: &CheckFinding, expected_index: usize) -> Result<(), String> {
    if finding.index != expected_index {
        return Err(format!(
            "check finding index {} does not match expected index {}",
            finding.index, expected_index
        ));
    }
    if finding.source.trim().is_empty() {
        return Err(format!(
            "check finding {expected_index} source must not be blank"
        ));
    }
    if finding.code.trim().is_empty() {
        return Err(format!(
            "check finding {expected_index} code must not be blank"
        ));
    }
    let severity = finding.severity.to_ascii_lowercase();
    if !matches!(severity.as_str(), "error" | "warning" | "info") {
        return Err(format!(
            "check finding {expected_index} has unsupported severity {}",
            finding.severity
        ));
    }
    if !is_sha256_fingerprint(&finding.fingerprint) {
        return Err(format!(
            "check finding {expected_index} fingerprint must be a sha256:<64 lowercase hex> value"
        ));
    }
    if finding.domain.trim().is_empty() {
        return Err(format!(
            "check finding {expected_index} domain must not be blank"
        ));
    }
    if finding.rule_id.trim().is_empty() {
        return Err(format!(
            "check finding {expected_index} rule_id must not be blank"
        ));
    }
    if let Some(detail) = &finding.standards_basis_detail {
        validate_standards_basis(
            detail,
            &format!("check finding {expected_index} standards_basis_detail"),
        )?;
    }
    let status = finding.status.to_ascii_lowercase();
    if !matches!(status.as_str(), "active" | "waived" | "accepted_deviation") {
        return Err(format!(
            "check finding {expected_index} has unsupported status {}",
            finding.status
        ));
    }
    validate_check_target(&finding.primary_target, expected_index, "primary_target")?;
    for (target_index, target) in finding.related_targets.iter().enumerate() {
        validate_check_target(
            target,
            expected_index,
            &format!("related_targets[{target_index}]"),
        )?;
    }
    Ok(())
}

fn validate_standards_basis(basis: &StandardsBasis, label: &str) -> Result<(), String> {
    if basis.basis_id.trim().is_empty() {
        return Err(format!("{label}.basis_id must not be blank"));
    }
    if basis.registry_entry_ref.trim().is_empty() {
        return Err(format!("{label}.registry_entry_ref must not be blank"));
    }
    if basis.selected_by.trim().is_empty() {
        return Err(format!("{label}.selected_by must not be blank"));
    }
    if basis.selection_scope.trim().is_empty() {
        return Err(format!("{label}.selection_scope must not be blank"));
    }
    if basis.basis_kind.trim().is_empty() {
        return Err(format!("{label}.basis_kind must not be blank"));
    }
    match basis.disposition.as_str() {
        "declared" | "inferred" | "user_selected" | "imported" | "unknown" => Ok(()),
        other => Err(format!("{label}.disposition has unsupported value {other}")),
    }?;
    if basis.provenance.trim().is_empty() {
        return Err(format!("{label}.provenance must not be blank"));
    }
    if let Some(entry) = standards_basis_registry_entry(&basis.basis_id) {
        validate_registered_standards_basis_field(
            label,
            "registry_entry_ref",
            &basis.registry_entry_ref,
            entry.registry_entry_ref,
        )?;
        if basis.revision_or_profile.as_deref() != entry.revision_or_profile {
            return Err(format!(
                "{label}.revision_or_profile must match registry value for {}",
                basis.basis_id
            ));
        }
        validate_registered_standards_basis_field(
            label,
            "selected_by",
            &basis.selected_by,
            entry.selected_by,
        )?;
        validate_registered_standards_basis_field(
            label,
            "selection_scope",
            &basis.selection_scope,
            entry.selection_scope,
        )?;
        validate_registered_standards_basis_field(
            label,
            "basis_kind",
            &basis.basis_kind,
            entry.basis_kind,
        )?;
        validate_registered_standards_basis_field(
            label,
            "disposition",
            &basis.disposition,
            entry.disposition,
        )?;
        validate_registered_standards_basis_field(
            label,
            "provenance",
            &basis.provenance,
            entry.provenance,
        )?;
    }
    Ok(())
}

fn validate_registered_standards_basis_field(
    label: &str,
    field: &str,
    actual: &str,
    expected: &str,
) -> Result<(), String> {
    if actual == expected {
        return Ok(());
    }
    Err(format!(
        "{label}.{field} must match registry value {expected}"
    ))
}

fn validate_check_target(
    target: &serde_json::Value,
    finding_index: usize,
    label: &str,
) -> Result<(), String> {
    let Some(object) = target.as_object() else {
        return Err(format!(
            "check finding {finding_index} {label} must be a typed target object"
        ));
    };
    let kind = object
        .get("kind")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .trim();
    if kind.is_empty() || kind == "unknown" {
        return Err(format!(
            "check finding {finding_index} {label}.kind must be a concrete target kind"
        ));
    }
    let id = object
        .get("id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .trim();
    if id.is_empty() {
        return Err(format!(
            "check finding {finding_index} {label}.id must not be blank"
        ));
    }
    Ok(())
}

fn is_sha256_fingerprint(value: &str) -> bool {
    let Some(digest) = value.strip_prefix("sha256:") else {
        return false;
    };
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}
