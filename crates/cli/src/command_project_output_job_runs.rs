use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use eda_engine::api::native_write::output_jobs::{
    build_set_output_job_run, derive_output_job_run_id,
};
use eda_engine::api::native_write::commit_prepared;
use eda_engine::substrate::{
    DesignModel, OUTPUT_JOB_RUN_SCHEMA_VERSION, OutputJobLogEntry, OutputJobLogLevel,
    OutputJobRun, OutputJobRunStatus, ProjectResolver,
};
use uuid::Uuid;

use super::super::command_project_artifacts::NativeProjectArtifactGenerateView;
use super::{cli_provenance, next_output_job_run_sequence};

pub(super) fn existing_generated_output_job_run(
    root: &Path,
    artifact_report: &NativeProjectArtifactGenerateView,
) -> Option<(OutputJobRun, PathBuf)> {
    artifact_report.generated.iter().find_map(|entry| {
        let run: OutputJobRun = entry
            .report
            .get("output_job_run")
            .and_then(|value| serde_json::from_value(value.clone()).ok())?;
        let path = root
            .join(".datum/output_job_runs")
            .join(format!("{}.json", run.run_id));
        Some((run, path))
    })
}

pub(super) fn persist_successful_output_job_run(
    root: &Path,
    output_job: Uuid,
    include: &str,
    artifact_report: &NativeProjectArtifactGenerateView,
) -> Result<(OutputJobRun, PathBuf)> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let run = successful_output_job_run(&model, output_job, include, artifact_report);
    let path = persist_output_job_run_journaled(root, &mut model, &run)
        .context("failed to persist output job run")?;
    Ok((run, path))
}

pub(super) fn persist_output_job_run_journaled(
    root: &Path,
    model: &mut DesignModel,
    run: &OutputJobRun,
) -> Result<PathBuf> {
    let prepared = build_set_output_job_run(
        model,
        cli_provenance("record output job run generated evidence"),
        run,
    )?;
    commit_prepared(model, root, prepared)?;
    Ok(output_job_run_path(root, run.run_id))
}

pub(super) fn failed_output_job_run(
    model: &DesignModel,
    output_job: Uuid,
    include: &str,
    error: &str,
) -> OutputJobRun {
    let run_sequence = next_output_job_run_sequence(model, output_job);
    OutputJobRun {
        schema_version: OUTPUT_JOB_RUN_SCHEMA_VERSION,
        run_id: derive_output_job_run_id(
            &model.project.project_id,
            &[
                "failed".to_string(),
                include.to_string(),
                output_job.to_string(),
                model.model_revision.0.clone(),
                run_sequence.to_string(),
                error.to_string(),
            ],
        ),
        output_job,
        run_sequence,
        project_id: model.project.project_id,
        model_revision: model.model_revision.clone(),
        status: OutputJobRunStatus::Failed,
        artifact_id: None,
        exit_code: Some(1),
        provenance: None,
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Error,
            message: error.to_string(),
        }],
    }
}

fn output_job_run_path(root: &Path, run_id: Uuid) -> PathBuf {
    root.join(format!(".datum/output_job_runs/{run_id}.json"))
}

fn successful_output_job_run(
    model: &DesignModel,
    output_job: Uuid,
    include: &str,
    artifact_report: &NativeProjectArtifactGenerateView,
) -> OutputJobRun {
    let run_sequence = next_output_job_run_sequence(model, output_job);
    let artifact_id = if artifact_report.generated.len() == 1 {
        Some(artifact_report.generated[0].artifact_id)
    } else {
        None
    };
    let mut log = vec![OutputJobLogEntry {
        sequence: 1,
        level: OutputJobLogLevel::Info,
        message: format!(
            "generated output job with {} artifact scopes",
            artifact_report.generated_count
        ),
    }];
    for entry in &artifact_report.generated {
        log.push(OutputJobLogEntry {
            sequence: log.len() as u64 + 1,
            level: OutputJobLogLevel::Info,
            message: format!(
                "generated {} artifact {} with {} files",
                entry.include, entry.artifact_id, entry.file_count
            ),
        });
    }
    OutputJobRun {
        schema_version: OUTPUT_JOB_RUN_SCHEMA_VERSION,
        run_id: derive_output_job_run_id(
            &model.project.project_id,
            &[
                "aggregate".to_string(),
                include.to_string(),
                output_job.to_string(),
                model.model_revision.0.clone(),
                run_sequence.to_string(),
                artifact_report.generated_count.to_string(),
            ],
        ),
        output_job,
        run_sequence,
        project_id: model.project.project_id,
        model_revision: model.model_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
        artifact_id,
        exit_code: Some(0),
        provenance: None,
        log,
    }
}
