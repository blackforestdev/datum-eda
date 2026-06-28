use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use eda_engine::substrate::{
    CommitProvenance, CommitSource, DesignModel, OUTPUT_JOB_RUN_SCHEMA_VERSION, Operation,
    OperationBatch, OutputJobLogEntry, OutputJobLogLevel, OutputJobRun, OutputJobRunStatus,
    ProjectResolver,
};
use uuid::Uuid;

use super::super::command_project_artifacts::NativeProjectArtifactGenerateView;
use super::next_output_job_run_sequence;

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
    let previous_output_job_run = model.output_job_runs.get(&run.run_id).map(|previous| {
        serde_json::to_value(previous).expect("output job run serialization must succeed")
    });
    model.commit_journaled(
        root,
        OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "datum-eda-cli".to_string(),
                source: CommitSource::Cli,
                reason: "record output job run generated evidence".to_string(),
            },
            operations: vec![Operation::SetOutputJobRun {
                run_id: run.run_id,
                previous_output_job_run,
                output_job_run: serde_json::to_value(run)
                    .expect("output job run serialization must succeed"),
            }],
        },
    )?;
    Ok(output_job_run_path(root, run.run_id))
}

pub(super) fn failed_output_job_run(
    model: &DesignModel,
    output_job: Uuid,
    include: &str,
    error: &str,
) -> OutputJobRun {
    let run_sequence = next_output_job_run_sequence(model, output_job);
    let material = format!(
        "datum-eda:output-job-run:failed:{include}:{}:{}:{run_sequence}:{error}",
        output_job, model.model_revision.0
    );
    OutputJobRun {
        schema_version: OUTPUT_JOB_RUN_SCHEMA_VERSION,
        run_id: Uuid::new_v5(&model.project.project_id, material.as_bytes()),
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
    let material = format!(
        "datum-eda:output-job-run:aggregate:{include}:{}:{}:{run_sequence}:{}",
        output_job, model.model_revision.0, artifact_report.generated_count
    );
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
        run_id: Uuid::new_v5(&model.project.project_id, material.as_bytes()),
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
