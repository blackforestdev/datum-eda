use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::artifact::{OutputJobLogEntry, OutputJobRunProvenance, OutputJobRunStatus};
use super::generated_evidence::{persist_generated_evidence, validate_filename_uuid};
use super::{
    ModelRevision, ResolveDiagnostic, SourceShardDirtyState, SourceShardKind, SourceShardRef,
    read_json_value, run_evidence_validation::validate_artifact_run, sha256_hex,
    source_shard_authority_for_kind,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRun {
    pub run_id: Uuid,
    pub artifact_id: Uuid,
    #[serde(default)]
    pub run_sequence: u64,
    pub project_id: Uuid,
    pub model_revision: ModelRevision,
    pub status: OutputJobRunStatus,
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<OutputJobRunProvenance>,
    pub log: Vec<OutputJobLogEntry>,
}

pub fn persist_artifact_run(
    project_root: &Path,
    run: &ArtifactRun,
) -> Result<PathBuf, crate::error::EngineError> {
    validate_artifact_run(run).map_err(crate::error::EngineError::Validation)?;
    persist_generated_evidence(project_root, ".datum/artifact_runs", &run.run_id, run)
}

pub(super) fn read_artifact_run_shards(
    project_root: &Path,
) -> (
    Vec<SourceShardRef>,
    BTreeMap<Uuid, ArtifactRun>,
    Vec<ResolveDiagnostic>,
) {
    let run_dir = project_root.join(".datum/artifact_runs");
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
        let relative_path = format!(".datum/artifact_runs/{filename}");
        let path = project_root.join(&relative_path);
        match read_artifact_run_shard(path, relative_path) {
            Ok((shard, run)) => {
                runs.insert(run.run_id, run);
                shards.push(shard);
            }
            Err(error) => diagnostics.push(error),
        }
    }

    (shards, runs, diagnostics)
}

fn read_artifact_run_shard(
    path: PathBuf,
    relative_path: String,
) -> Result<(SourceShardRef, ArtifactRun), ResolveDiagnostic> {
    let bytes = std::fs::read(&path).map_err(|error| ResolveDiagnostic {
        code: "missing_artifact_run".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let value = read_json_value(&path).map_err(|error| ResolveDiagnostic {
        code: "invalid_artifact_run".to_string(),
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
        kind: SourceShardKind::ArtifactRun,
        path,
        relative_path,
        authority: source_shard_authority_for_kind(&SourceShardKind::ArtifactRun),
        dirty_state: SourceShardDirtyState::Clean,
        schema_version,
        content_hash: sha256_hex(&bytes),
    };
    let run = serde_json::from_value::<ArtifactRun>(value).map_err(|error| ResolveDiagnostic {
        code: "invalid_artifact_run".to_string(),
        message: error.to_string(),
        path: Some(shard.path.clone()),
    })?;
    validate_filename_uuid(&shard.path, run.run_id, "invalid_artifact_run")?;
    validate_artifact_run(&run).map_err(|message| ResolveDiagnostic {
        code: "invalid_artifact_run".to_string(),
        message,
        path: Some(shard.path.clone()),
    })?;
    Ok((shard, run))
}
