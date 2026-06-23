use std::collections::{BTreeMap, BTreeSet};

use super::artifact::{OutputJobLogEntry, OutputJobRun, OutputJobRunStatus};
use super::artifact_run::ArtifactRun;
use super::{DesignModel, ResolveDiagnostic};

pub(super) fn validate_output_job_run(run: &OutputJobRun) -> Result<(), String> {
    validate_run_sequence(run.run_sequence)?;
    validate_status_exit_code(run.status, run.exit_code)?;
    if run.status == OutputJobRunStatus::Running && run.artifact_id.is_some() {
        return Err("running output job run must not reference an artifact".to_string());
    }
    validate_log_entries(&run.log)
}

pub(super) fn validate_artifact_run(run: &ArtifactRun) -> Result<(), String> {
    validate_run_sequence(run.run_sequence)?;
    validate_status_exit_code(run.status, run.exit_code)?;
    validate_log_entries(&run.log)
}

pub(super) fn validate_run_evidence_links(model: &mut DesignModel) {
    let mut invalid_output_runs = BTreeSet::new();
    let mut output_sequences = BTreeMap::new();
    for (run_id, run) in &model.output_job_runs {
        if run.project_id != model.project.project_id {
            mark_invalid_output_run(
                &mut model.diagnostics,
                &mut invalid_output_runs,
                *run_id,
                "output job run project_id does not match project",
            );
            continue;
        }
        if !model.output_jobs.contains_key(&run.output_job) {
            mark_invalid_output_run(
                &mut model.diagnostics,
                &mut invalid_output_runs,
                *run_id,
                "output job run references missing output job",
            );
            continue;
        }
        if let Some(artifact_id) = run.artifact_id {
            let Some(artifact) = model.artifact_metadata.get(&artifact_id) else {
                mark_invalid_output_run(
                    &mut model.diagnostics,
                    &mut invalid_output_runs,
                    *run_id,
                    "output job run references missing artifact metadata",
                );
                continue;
            };
            if artifact.project_id != run.project_id {
                mark_invalid_output_run(
                    &mut model.diagnostics,
                    &mut invalid_output_runs,
                    *run_id,
                    "output job run artifact project_id does not match run",
                );
                continue;
            }
            if artifact.model_revision != run.model_revision {
                mark_invalid_output_run(
                    &mut model.diagnostics,
                    &mut invalid_output_runs,
                    *run_id,
                    "output job run artifact model_revision does not match run",
                );
                continue;
            }
        }
        let key = (run.output_job, run.run_sequence);
        if let Some(previous_id) = output_sequences.insert(key, *run_id) {
            mark_invalid_output_run(
                &mut model.diagnostics,
                &mut invalid_output_runs,
                previous_id,
                "duplicate output job run_sequence for output job",
            );
            mark_invalid_output_run(
                &mut model.diagnostics,
                &mut invalid_output_runs,
                *run_id,
                "duplicate output job run_sequence for output job",
            );
        }
    }
    for run_id in invalid_output_runs {
        model.output_job_runs.remove(&run_id);
    }

    let mut invalid_artifact_runs = BTreeSet::new();
    let mut artifact_sequences = BTreeMap::new();
    for (run_id, run) in &model.artifact_runs {
        if run.project_id != model.project.project_id {
            mark_invalid_artifact_run(
                &mut model.diagnostics,
                &mut invalid_artifact_runs,
                *run_id,
                "artifact run project_id does not match project",
            );
            continue;
        }
        let Some(artifact) = model.artifact_metadata.get(&run.artifact_id) else {
            mark_invalid_artifact_run(
                &mut model.diagnostics,
                &mut invalid_artifact_runs,
                *run_id,
                "artifact run references missing artifact metadata",
            );
            continue;
        };
        if artifact.project_id != run.project_id {
            mark_invalid_artifact_run(
                &mut model.diagnostics,
                &mut invalid_artifact_runs,
                *run_id,
                "artifact run artifact project_id does not match run",
            );
            continue;
        }
        if artifact.model_revision != run.model_revision {
            mark_invalid_artifact_run(
                &mut model.diagnostics,
                &mut invalid_artifact_runs,
                *run_id,
                "artifact run artifact model_revision does not match run",
            );
            continue;
        }
        let key = (run.artifact_id, run.run_sequence);
        if let Some(previous_id) = artifact_sequences.insert(key, *run_id) {
            mark_invalid_artifact_run(
                &mut model.diagnostics,
                &mut invalid_artifact_runs,
                previous_id,
                "duplicate artifact run_sequence for artifact",
            );
            mark_invalid_artifact_run(
                &mut model.diagnostics,
                &mut invalid_artifact_runs,
                *run_id,
                "duplicate artifact run_sequence for artifact",
            );
        }
    }
    for run_id in invalid_artifact_runs {
        model.artifact_runs.remove(&run_id);
    }
}

fn mark_invalid_output_run(
    diagnostics: &mut Vec<ResolveDiagnostic>,
    invalid_runs: &mut BTreeSet<uuid::Uuid>,
    run_id: uuid::Uuid,
    message: &str,
) {
    if invalid_runs.insert(run_id) {
        diagnostics.push(ResolveDiagnostic {
            code: "invalid_output_job_run".to_string(),
            message: format!("{message}: {run_id}"),
            path: None,
        });
    }
}

fn mark_invalid_artifact_run(
    diagnostics: &mut Vec<ResolveDiagnostic>,
    invalid_runs: &mut BTreeSet<uuid::Uuid>,
    run_id: uuid::Uuid,
    message: &str,
) {
    if invalid_runs.insert(run_id) {
        diagnostics.push(ResolveDiagnostic {
            code: "invalid_artifact_run".to_string(),
            message: format!("{message}: {run_id}"),
            path: None,
        });
    }
}

fn validate_run_sequence(run_sequence: u64) -> Result<(), String> {
    if run_sequence == 0 {
        return Err("run_sequence must be greater than zero".to_string());
    }
    Ok(())
}

fn validate_status_exit_code(
    status: OutputJobRunStatus,
    exit_code: Option<i32>,
) -> Result<(), String> {
    match (status, exit_code) {
        (OutputJobRunStatus::Running, None) => Ok(()),
        (OutputJobRunStatus::Running, Some(_)) => {
            Err("running run evidence must not have an exit_code".to_string())
        }
        (OutputJobRunStatus::Succeeded, Some(0)) => Ok(()),
        (OutputJobRunStatus::Succeeded, Some(other)) => Err(format!(
            "succeeded run evidence must have exit_code 0, found {other}"
        )),
        (OutputJobRunStatus::Succeeded, None) => {
            Err("succeeded run evidence must have an exit_code".to_string())
        }
        (OutputJobRunStatus::Failed | OutputJobRunStatus::Canceled, Some(_)) => Ok(()),
        (OutputJobRunStatus::Failed | OutputJobRunStatus::Canceled, None) => {
            Err(format!("{status:?} run evidence must have an exit_code"))
        }
    }
}

fn validate_log_entries(log: &[OutputJobLogEntry]) -> Result<(), String> {
    if log.is_empty() {
        return Err("run evidence log must contain at least one entry".to_string());
    }
    for (index, entry) in log.iter().enumerate() {
        let expected_sequence = index as u64 + 1;
        if entry.sequence != expected_sequence {
            return Err(format!(
                "run evidence log entry sequence {} does not match expected {}",
                entry.sequence, expected_sequence
            ));
        }
        if entry.message.trim().is_empty() {
            return Err(format!(
                "run evidence log entry {expected_sequence} message must not be blank"
            ));
        }
    }
    Ok(())
}
