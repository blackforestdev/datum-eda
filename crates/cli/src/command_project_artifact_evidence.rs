use std::path::{Path, PathBuf};

use anyhow::Result;
use eda_engine::substrate::{
    ArtifactMetadata, ArtifactRun, CommitProvenance, CommitSource, DesignModel, Operation,
    OperationBatch, OutputJobRun,
};
use uuid::Uuid;

use super::super::command_project_artifact_runs::generic_artifact_run;

pub(crate) fn commit_unlinked_artifact_evidence(
    root: &Path,
    scope: &str,
    model: &mut DesignModel,
    artifact_metadata: &ArtifactMetadata,
) -> Result<(PathBuf, ArtifactRun, PathBuf)> {
    let artifact_run = generic_artifact_run(scope, model, artifact_metadata);
    let previous_artifact_metadata = model
        .artifact_metadata
        .get(&artifact_metadata.artifact_id)
        .map(|artifact| {
            serde_json::to_value(artifact).expect("artifact metadata serialization must succeed")
        });
    let previous_artifact_run = model
        .artifact_runs
        .get(&artifact_run.run_id)
        .map(|run| serde_json::to_value(run).expect("artifact run serialization must succeed"));
    model.commit_journaled(
        root,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "datum-eda-cli".to_string(),
                source: CommitSource::Cli,
                reason: format!("generate unlinked {scope} artifact evidence"),
            },
            operations: vec![
                Operation::SetArtifactMetadata {
                    artifact_id: artifact_metadata.artifact_id,
                    previous_artifact_metadata,
                    artifact_metadata: serde_json::to_value(artifact_metadata)
                        .expect("artifact metadata serialization must succeed"),
                },
                Operation::SetArtifactRun {
                    run_id: artifact_run.run_id,
                    previous_artifact_run,
                    artifact_run: serde_json::to_value(&artifact_run)
                        .expect("artifact run serialization must succeed"),
                },
            ],
        },
    )?;
    let manifest_path = artifact_metadata_path(root, artifact_metadata.artifact_id);
    let run_path = artifact_run_path(root, artifact_run.run_id);
    Ok((manifest_path, artifact_run, run_path))
}

pub(crate) fn commit_artifact_metadata_evidence(
    root: &Path,
    model: &mut DesignModel,
    artifact_metadata: &ArtifactMetadata,
) -> Result<PathBuf> {
    let previous_artifact_metadata = model
        .artifact_metadata
        .get(&artifact_metadata.artifact_id)
        .map(|artifact| {
            serde_json::to_value(artifact).expect("artifact metadata serialization must succeed")
        });
    model.commit_journaled(
        root,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "datum-eda-cli".to_string(),
                source: CommitSource::Cli,
                reason: "record generated artifact metadata".to_string(),
            },
            operations: vec![Operation::SetArtifactMetadata {
                artifact_id: artifact_metadata.artifact_id,
                previous_artifact_metadata,
                artifact_metadata: serde_json::to_value(artifact_metadata)
                    .expect("artifact metadata serialization must succeed"),
            }],
        },
    )?;
    Ok(artifact_metadata_path(root, artifact_metadata.artifact_id))
}

pub(crate) fn commit_linked_artifact_output_job_evidence(
    root: &Path,
    model: &mut DesignModel,
    artifact_metadata: &ArtifactMetadata,
    output_job_run: &OutputJobRun,
) -> Result<(PathBuf, PathBuf)> {
    let previous_artifact_metadata = model
        .artifact_metadata
        .get(&artifact_metadata.artifact_id)
        .map(|artifact| {
            serde_json::to_value(artifact).expect("artifact metadata serialization must succeed")
        });
    let previous_output_job_run = model
        .output_job_runs
        .get(&output_job_run.run_id)
        .map(|run| serde_json::to_value(run).expect("output job run serialization must succeed"));
    model.commit_journaled(
        root,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "datum-eda-cli".to_string(),
                source: CommitSource::Cli,
                reason: "record linked artifact output-job evidence".to_string(),
            },
            operations: vec![
                Operation::SetArtifactMetadata {
                    artifact_id: artifact_metadata.artifact_id,
                    previous_artifact_metadata,
                    artifact_metadata: serde_json::to_value(artifact_metadata)
                        .expect("artifact metadata serialization must succeed"),
                },
                Operation::SetOutputJobRun {
                    run_id: output_job_run.run_id,
                    previous_output_job_run,
                    output_job_run: serde_json::to_value(output_job_run)
                        .expect("output job run serialization must succeed"),
                },
            ],
        },
    )?;
    Ok((
        artifact_metadata_path(root, artifact_metadata.artifact_id),
        output_job_run_path(root, output_job_run.run_id),
    ))
}

fn artifact_metadata_path(root: &Path, artifact_id: Uuid) -> PathBuf {
    root.join(format!(".datum/artifacts/{artifact_id}.json"))
}

fn artifact_run_path(root: &Path, run_id: Uuid) -> PathBuf {
    root.join(format!(".datum/artifact_runs/{run_id}.json"))
}

fn output_job_run_path(root: &Path, run_id: Uuid) -> PathBuf {
    root.join(format!(".datum/output_job_runs/{run_id}.json"))
}
