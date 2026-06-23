use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    DomainObject, ModelRevision, ObjectId, ObjectRevision, ResolveDiagnostic,
    SourceShardDirtyState, SourceShardKind, SourceShardRef,
    artifact_validation::validate_artifact_metadata, read_json_value,
    run_evidence_validation::validate_output_job_run, sha256_hex, source_shard_authority_for_kind,
};

use super::generated_evidence::{persist_generated_evidence, validate_filename_uuid};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    GerberSet,
    ManufacturingSet,
    Bom,
    Pnp,
    Drill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactValidationState {
    NotValidated,
    Valid,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputJobRunStatus {
    Running,
    Succeeded,
    Failed,
    Canceled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputJobLogLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputJob {
    pub id: ObjectId,
    pub name: String,
    pub include: Vec<ArtifactKind>,
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub output_dir: Option<PathBuf>,
    pub board_or_panel: ObjectId,
    pub variant: Option<ObjectId>,
    pub manufacturing_plan: Option<ObjectId>,
    pub object_revision: ObjectRevision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManufacturingPlan {
    pub id: ObjectId,
    pub name: String,
    pub board_or_panel: ObjectId,
    pub variant: Option<ObjectId>,
    pub prefix: String,
    pub object_revision: ObjectRevision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelBoardInstance {
    pub board: ObjectId,
    pub x_nm: i64,
    pub y_nm: i64,
    pub rotation_deg: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelProjection {
    pub id: ObjectId,
    pub name: String,
    pub board_instances: Vec<PanelBoardInstance>,
    pub object_revision: ObjectRevision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputJobLogEntry {
    pub sequence: u64,
    pub level: OutputJobLogLevel,
    pub message: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputJobRunLauncher {
    GuiTerminal,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputJobRunProvenance {
    pub launcher: OutputJobRunLauncher,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_context_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_root: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_revision: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputJobRun {
    pub run_id: Uuid,
    pub output_job: ObjectId,
    #[serde(default)]
    pub run_sequence: u64,
    pub project_id: Uuid,
    pub model_revision: ModelRevision,
    pub status: OutputJobRunStatus,
    pub artifact_id: Option<Uuid>,
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<OutputJobRunProvenance>,
    pub log: Vec<OutputJobLogEntry>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactFile {
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactProductionProjection {
    pub projection_kind: String,
    pub projection_contract: String,
    pub model_revision: ModelRevision,
    pub byte_count: usize,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub artifact_id: Uuid,
    pub kind: ArtifactKind,
    pub project_id: Uuid,
    pub model_revision: ModelRevision,
    pub output_job: Option<ObjectId>,
    pub variant: Option<ObjectId>,
    pub generator_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_dir: Option<PathBuf>,
    pub files: Vec<ArtifactFile>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub production_projections: Vec<ArtifactProductionProjection>,
    pub validation_state: ArtifactValidationState,
}

pub fn persist_artifact_metadata(
    project_root: &Path,
    metadata: &ArtifactMetadata,
) -> Result<PathBuf, crate::error::EngineError> {
    validate_artifact_metadata(metadata).map_err(crate::error::EngineError::Validation)?;
    persist_generated_evidence(
        project_root,
        ".datum/artifacts",
        &metadata.artifact_id,
        metadata,
    )
}

pub fn persist_output_job_run(
    project_root: &Path,
    run: &OutputJobRun,
) -> Result<PathBuf, crate::error::EngineError> {
    validate_output_job_run(run).map_err(crate::error::EngineError::Validation)?;
    persist_generated_evidence(project_root, ".datum/output_job_runs", &run.run_id, run)
}

pub(super) fn read_output_job_run_shards(
    project_root: &Path,
) -> (
    Vec<SourceShardRef>,
    BTreeMap<Uuid, OutputJobRun>,
    Vec<ResolveDiagnostic>,
) {
    let run_dir = project_root.join(".datum/output_job_runs");
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
        let relative_path = format!(".datum/output_job_runs/{filename}");
        let path = project_root.join(&relative_path);
        match read_output_job_run_shard(path, relative_path) {
            Ok((shard, run)) => {
                runs.insert(run.run_id, run);
                shards.push(shard);
            }
            Err(error) => diagnostics.push(error),
        }
    }

    (shards, runs, diagnostics)
}

pub(super) fn read_artifact_metadata_shards(
    project_root: &Path,
) -> (
    Vec<SourceShardRef>,
    BTreeMap<Uuid, ArtifactMetadata>,
    Vec<ResolveDiagnostic>,
) {
    let artifact_dir = project_root.join(".datum/artifacts");
    let mut shards = Vec::new();
    let mut artifacts = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let Ok(entries) = std::fs::read_dir(&artifact_dir) else {
        return (shards, artifacts, diagnostics);
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
        let relative_path = format!(".datum/artifacts/{filename}");
        let path = project_root.join(&relative_path);
        match read_artifact_metadata_shard(path, relative_path) {
            Ok((shard, metadata)) => {
                artifacts.insert(metadata.artifact_id, metadata);
                shards.push(shard);
            }
            Err(error) => diagnostics.push(error),
        }
    }

    (shards, artifacts, diagnostics)
}

pub(super) fn read_output_job_shards(
    project_root: &Path,
) -> (
    Vec<SourceShardRef>,
    BTreeMap<ObjectId, OutputJob>,
    Vec<ResolveDiagnostic>,
) {
    let job_dir = project_root.join(".datum/output_jobs");
    let mut shards = Vec::new();
    let mut jobs = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let Ok(entries) = std::fs::read_dir(&job_dir) else {
        return (shards, jobs, diagnostics);
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
        let relative_path = format!(".datum/output_jobs/{filename}");
        let path = project_root.join(&relative_path);
        match read_output_job_shard(path, relative_path) {
            Ok((shard, job)) => {
                jobs.insert(job.id, job);
                shards.push(shard);
            }
            Err(error) => diagnostics.push(error),
        }
    }

    (shards, jobs, diagnostics)
}

pub(super) fn read_manufacturing_plan_shards(
    project_root: &Path,
) -> (
    Vec<SourceShardRef>,
    BTreeMap<ObjectId, ManufacturingPlan>,
    Vec<ResolveDiagnostic>,
) {
    let plan_dir = project_root.join(".datum/manufacturing_plans");
    let mut shards = Vec::new();
    let mut plans = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let Ok(entries) = std::fs::read_dir(&plan_dir) else {
        return (shards, plans, diagnostics);
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
        let relative_path = format!(".datum/manufacturing_plans/{filename}");
        let path = project_root.join(&relative_path);
        match read_manufacturing_plan_shard(path, relative_path) {
            Ok((shard, plan)) => {
                plans.insert(plan.id, plan);
                shards.push(shard);
            }
            Err(error) => diagnostics.push(error),
        }
    }

    (shards, plans, diagnostics)
}

pub(super) fn read_panel_projection_shards(
    project_root: &Path,
) -> (
    Vec<SourceShardRef>,
    BTreeMap<ObjectId, PanelProjection>,
    Vec<ResolveDiagnostic>,
) {
    let panel_dir = project_root.join(".datum/panel_projections");
    let mut shards = Vec::new();
    let mut panels = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let Ok(entries) = std::fs::read_dir(&panel_dir) else {
        return (shards, panels, diagnostics);
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
        let relative_path = format!(".datum/panel_projections/{filename}");
        let path = project_root.join(&relative_path);
        match read_panel_projection_shard(path, relative_path) {
            Ok((shard, panel)) => {
                panels.insert(panel.id, panel);
                shards.push(shard);
            }
            Err(error) => diagnostics.push(error),
        }
    }

    (shards, panels, diagnostics)
}

pub(super) fn insert_output_job_objects(
    shards: &[SourceShardRef],
    jobs: &BTreeMap<ObjectId, OutputJob>,
    objects: &mut BTreeMap<ObjectId, DomainObject>,
) {
    for (job_id, job) in jobs {
        if let Some(shard) = shards.iter().find(|shard| {
            shard.path.file_stem().and_then(|value| value.to_str()) == Some(&job_id.to_string())
        }) {
            objects.insert(
                *job_id,
                DomainObject {
                    object_id: *job_id,
                    object_revision: job.object_revision,
                    source_shard_id: shard.shard_id,
                    domain: "output".to_string(),
                    kind: "output_job".to_string(),
                },
            );
        }
    }
}

pub(super) fn insert_manufacturing_plan_objects(
    shards: &[SourceShardRef],
    plans: &BTreeMap<ObjectId, ManufacturingPlan>,
    objects: &mut BTreeMap<ObjectId, DomainObject>,
) {
    for (plan_id, plan) in plans {
        if let Some(shard) = shards.iter().find(|shard| {
            shard.path.file_stem().and_then(|value| value.to_str()) == Some(&plan_id.to_string())
        }) {
            objects.insert(
                *plan_id,
                DomainObject {
                    object_id: *plan_id,
                    object_revision: plan.object_revision,
                    source_shard_id: shard.shard_id,
                    domain: "manufacturing".to_string(),
                    kind: "manufacturing_plan".to_string(),
                },
            );
        }
    }
}

pub(super) fn insert_panel_projection_objects(
    shards: &[SourceShardRef],
    panels: &BTreeMap<ObjectId, PanelProjection>,
    objects: &mut BTreeMap<ObjectId, DomainObject>,
) {
    for (panel_id, panel) in panels {
        if let Some(shard) = shards.iter().find(|shard| {
            shard.path.file_stem().and_then(|value| value.to_str()) == Some(&panel_id.to_string())
        }) {
            objects.insert(
                *panel_id,
                DomainObject {
                    object_id: *panel_id,
                    object_revision: panel.object_revision,
                    source_shard_id: shard.shard_id,
                    domain: "manufacturing".to_string(),
                    kind: "panel_projection".to_string(),
                },
            );
        }
    }
}

fn read_output_job_run_shard(
    path: PathBuf,
    relative_path: String,
) -> Result<(SourceShardRef, OutputJobRun), ResolveDiagnostic> {
    let bytes = std::fs::read(&path).map_err(|error| ResolveDiagnostic {
        code: "missing_output_job_run".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let value = read_json_value(&path).map_err(|error| ResolveDiagnostic {
        code: "invalid_output_job_run".to_string(),
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
        kind: SourceShardKind::OutputJobRun,
        path,
        relative_path,
        authority: source_shard_authority_for_kind(&SourceShardKind::OutputJobRun),
        dirty_state: SourceShardDirtyState::Clean,
        schema_version,
        content_hash: sha256_hex(&bytes),
    };
    let run = serde_json::from_value::<OutputJobRun>(value).map_err(|error| ResolveDiagnostic {
        code: "invalid_output_job_run".to_string(),
        message: error.to_string(),
        path: Some(shard.path.clone()),
    })?;
    validate_filename_uuid(&shard.path, run.run_id, "invalid_output_job_run")?;
    validate_output_job_run(&run).map_err(|message| ResolveDiagnostic {
        code: "invalid_output_job_run".to_string(),
        message,
        path: Some(shard.path.clone()),
    })?;
    Ok((shard, run))
}

fn read_manufacturing_plan_shard(
    path: PathBuf,
    relative_path: String,
) -> Result<(SourceShardRef, ManufacturingPlan), ResolveDiagnostic> {
    let bytes = std::fs::read(&path).map_err(|error| ResolveDiagnostic {
        code: "missing_manufacturing_plan".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let value = read_json_value(&path).map_err(|error| ResolveDiagnostic {
        code: "invalid_manufacturing_plan".to_string(),
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
        kind: SourceShardKind::ManufacturingPlan,
        path,
        relative_path,
        authority: source_shard_authority_for_kind(&SourceShardKind::ManufacturingPlan),
        dirty_state: SourceShardDirtyState::Clean,
        schema_version,
        content_hash: sha256_hex(&bytes),
    };
    let plan =
        serde_json::from_value::<ManufacturingPlan>(value).map_err(|error| ResolveDiagnostic {
            code: "invalid_manufacturing_plan".to_string(),
            message: error.to_string(),
            path: Some(shard.path.clone()),
        })?;
    Ok((shard, plan))
}

fn read_panel_projection_shard(
    path: PathBuf,
    relative_path: String,
) -> Result<(SourceShardRef, PanelProjection), ResolveDiagnostic> {
    let bytes = std::fs::read(&path).map_err(|error| ResolveDiagnostic {
        code: "missing_panel_projection".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let value = read_json_value(&path).map_err(|error| ResolveDiagnostic {
        code: "invalid_panel_projection".to_string(),
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
        kind: SourceShardKind::PanelProjection,
        path,
        relative_path,
        authority: source_shard_authority_for_kind(&SourceShardKind::PanelProjection),
        dirty_state: SourceShardDirtyState::Clean,
        schema_version,
        content_hash: sha256_hex(&bytes),
    };
    let panel =
        serde_json::from_value::<PanelProjection>(value).map_err(|error| ResolveDiagnostic {
            code: "invalid_panel_projection".to_string(),
            message: error.to_string(),
            path: Some(shard.path.clone()),
        })?;
    Ok((shard, panel))
}

fn read_artifact_metadata_shard(
    path: PathBuf,
    relative_path: String,
) -> Result<(SourceShardRef, ArtifactMetadata), ResolveDiagnostic> {
    let bytes = std::fs::read(&path).map_err(|error| ResolveDiagnostic {
        code: "missing_artifact_metadata".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let value = read_json_value(&path).map_err(|error| ResolveDiagnostic {
        code: "invalid_artifact_metadata".to_string(),
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
        kind: SourceShardKind::ArtifactMetadata,
        path,
        relative_path,
        authority: source_shard_authority_for_kind(&SourceShardKind::ArtifactMetadata),
        dirty_state: SourceShardDirtyState::Clean,
        schema_version,
        content_hash: sha256_hex(&bytes),
    };
    let metadata =
        serde_json::from_value::<ArtifactMetadata>(value).map_err(|error| ResolveDiagnostic {
            code: "invalid_artifact_metadata".to_string(),
            message: error.to_string(),
            path: Some(shard.path.clone()),
        })?;
    validate_filename_uuid(
        &shard.path,
        metadata.artifact_id,
        "invalid_artifact_metadata",
    )?;
    validate_artifact_metadata(&metadata).map_err(|message| ResolveDiagnostic {
        code: "invalid_artifact_metadata".to_string(),
        message,
        path: Some(shard.path.clone()),
    })?;
    Ok((shard, metadata))
}

fn read_output_job_shard(
    path: PathBuf,
    relative_path: String,
) -> Result<(SourceShardRef, OutputJob), ResolveDiagnostic> {
    let bytes = std::fs::read(&path).map_err(|error| ResolveDiagnostic {
        code: "missing_output_job".to_string(),
        message: error.to_string(),
        path: Some(path.clone()),
    })?;
    let value = read_json_value(&path).map_err(|error| ResolveDiagnostic {
        code: "invalid_output_job".to_string(),
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
        kind: SourceShardKind::OutputJob,
        path,
        relative_path,
        authority: source_shard_authority_for_kind(&SourceShardKind::OutputJob),
        dirty_state: SourceShardDirtyState::Clean,
        schema_version,
        content_hash: sha256_hex(&bytes),
    };
    let job = serde_json::from_value::<OutputJob>(value).map_err(|error| ResolveDiagnostic {
        code: "invalid_output_job".to_string(),
        message: error.to_string(),
        path: Some(shard.path.clone()),
    })?;
    Ok((shard, job))
}
