use std::path::{Path, PathBuf};

use anyhow::Result;
use eda_engine::substrate::{
    ArtifactMetadata, CommitProvenance, CommitSource, DesignModel, Operation, OperationBatch,
    OutputJobRun,
};
use uuid::Uuid;

pub(crate) fn commit_manufacturing_set_evidence(
    root: &Path,
    model: &mut DesignModel,
    artifact_metadata: &ArtifactMetadata,
    output_job_run: Option<&OutputJobRun>,
) -> Result<(PathBuf, PathBuf)> {
    let previous_artifact_metadata = model
        .artifact_metadata
        .get(&artifact_metadata.artifact_id)
        .map(|artifact| {
            serde_json::to_value(artifact).expect("artifact metadata serialization must succeed")
        });
    let mut operations = vec![Operation::SetArtifactMetadata {
        artifact_id: artifact_metadata.artifact_id,
        previous_artifact_metadata,
        artifact_metadata: serde_json::to_value(artifact_metadata)
            .expect("artifact metadata serialization must succeed"),
    }];
    let output_run_path = if let Some(run) = output_job_run {
        let previous_output_job_run = model.output_job_runs.get(&run.run_id).map(|previous| {
            serde_json::to_value(previous).expect("output job run serialization must succeed")
        });
        operations.push(Operation::SetOutputJobRun {
            run_id: run.run_id,
            previous_output_job_run,
            output_job_run: serde_json::to_value(run)
                .expect("output job run serialization must succeed"),
        });
        output_job_run_path(root, run.run_id)
    } else {
        PathBuf::new()
    };
    model.commit_journaled(
        root,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "datum-eda-cli".to_string(),
                source: CommitSource::Cli,
                reason: "record manufacturing set generated evidence".to_string(),
            },
            operations,
        },
    )?;
    Ok((
        artifact_metadata_path(root, artifact_metadata.artifact_id),
        output_run_path,
    ))
}

pub(crate) fn artifact_metadata_path(root: &Path, artifact_id: Uuid) -> PathBuf {
    root.join(format!(".datum/artifacts/{artifact_id}.json"))
}

fn output_job_run_path(root: &Path, run_id: Uuid) -> PathBuf {
    root.join(format!(".datum/output_job_runs/{run_id}.json"))
}
