use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::generated_evidence::{persist_generated_evidence, validate_filename_uuid};
use super::{
    ModelRevision, ResolveDiagnostic, SourceShardDirtyState, SourceShardKind, SourceShardRef,
    read_json_value, sha256_hex, source_shard_authority_for_kind,
};

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
}

pub fn persist_check_run(
    project_root: &Path,
    run: &CheckRun,
) -> Result<PathBuf, crate::error::EngineError> {
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
    let shard = SourceShardRef {
        shard_id: Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("datum-eda:source-shard:{relative_path}").as_bytes(),
        ),
        kind: SourceShardKind::CheckRun,
        path,
        relative_path,
        authority: source_shard_authority_for_kind(&SourceShardKind::CheckRun),
        dirty_state: SourceShardDirtyState::Clean,
        schema_version,
        content_hash: sha256_hex(&bytes),
    };
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

fn validate_check_run(run: &CheckRun) -> Result<(), String> {
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
    }
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
    let status = finding.status.to_ascii_lowercase();
    if !matches!(status.as_str(), "active" | "waived" | "accepted_deviation") {
        return Err(format!(
            "check finding {expected_index} has unsupported status {}",
            finding.status
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
