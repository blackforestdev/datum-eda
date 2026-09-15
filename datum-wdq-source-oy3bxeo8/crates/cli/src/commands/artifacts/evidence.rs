use std::path::{Path, PathBuf};

use anyhow::Result;
use eda_engine::api::native_write::artifacts::build_artifact_evidence;
use eda_engine::api::native_write::{WriteProvenance, commit_prepared};
use eda_engine::substrate::{ArtifactMetadata, ArtifactRun, DesignModel, OutputJobRun};
use uuid::Uuid;

use super::super::runs::generic_artifact_run;

use crate::cli_commit_source;

fn evidence_provenance(reason: impl Into<String>) -> Result<WriteProvenance> {
    Ok(WriteProvenance::new(
        "datum-eda-cli",
        cli_commit_source()?,
        reason,
    ))
}

pub(crate) fn commit_unlinked_artifact_evidence(
    root: &Path,
    scope: &str,
    model: &mut DesignModel,
    artifact_metadata: &ArtifactMetadata,
) -> Result<(PathBuf, ArtifactRun, PathBuf)> {
    let artifact_run = generic_artifact_run(scope, model, artifact_metadata);
    let prepared = build_artifact_evidence(
        model,
        evidence_provenance(format!("generate unlinked {scope} artifact evidence"))?,
        artifact_metadata,
        Some(&artifact_run),
        None,
    )?;
    commit_prepared(model, root, prepared)?;
    let manifest_path = artifact_metadata_path(root, artifact_metadata.artifact_id);
    let run_path = artifact_run_path(root, artifact_run.run_id);
    Ok((manifest_path, artifact_run, run_path))
}

pub(crate) fn commit_artifact_metadata_evidence(
    root: &Path,
    model: &mut DesignModel,
    artifact_metadata: &ArtifactMetadata,
) -> Result<PathBuf> {
    let prepared = build_artifact_evidence(
        model,
        evidence_provenance("record generated artifact metadata")?,
        artifact_metadata,
        None,
        None,
    )?;
    commit_prepared(model, root, prepared)?;
    Ok(artifact_metadata_path(root, artifact_metadata.artifact_id))
}

pub(crate) fn commit_linked_artifact_output_job_evidence(
    root: &Path,
    model: &mut DesignModel,
    artifact_metadata: &ArtifactMetadata,
    output_job_run: &OutputJobRun,
) -> Result<(PathBuf, PathBuf)> {
    let prepared = build_artifact_evidence(
        model,
        evidence_provenance("record linked artifact output-job evidence")?,
        artifact_metadata,
        None,
        Some(output_job_run),
    )?;
    commit_prepared(model, root, prepared)?;
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
