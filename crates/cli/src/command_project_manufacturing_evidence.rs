use std::path::{Path, PathBuf};

use anyhow::Result;
use eda_engine::api::native_write::artifacts::build_artifact_evidence;
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::substrate::{ArtifactMetadata, CommitSource, DesignModel, OutputJobRun};
use uuid::Uuid;

pub(crate) fn commit_manufacturing_set_evidence(
    root: &Path,
    model: &mut DesignModel,
    artifact_metadata: &ArtifactMetadata,
    output_job_run: Option<&OutputJobRun>,
) -> Result<(PathBuf, PathBuf)> {
    let prepared = build_artifact_evidence(
        model,
        WriteProvenance::new(
            "datum-eda-cli",
            CommitSource::Cli,
            "record manufacturing set generated evidence",
        ),
        artifact_metadata,
        None,
        output_job_run,
    )?;
    commit_prepared(model, root, prepared)?;
    let output_run_path = output_job_run
        .map(|run| output_job_run_path(root, run.run_id))
        .unwrap_or_default();
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
