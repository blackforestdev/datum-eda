use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::artifact::{
    OUTPUT_JOB_RUN_SCHEMA_VERSION, OutputJob, OutputJobLogEntry, OutputJobRun,
    OutputJobRunProvenance, OutputJobRunStatus,
};
use super::artifact_run::{ARTIFACT_RUN_SCHEMA_VERSION, ArtifactRun};
use super::{ArtifactMetadata, DesignModel, EngineError, Operation, ResolveDiagnostic};

pub(super) fn validate_output_job_run(run: &OutputJobRun) -> Result<(), String> {
    if run.schema_version != OUTPUT_JOB_RUN_SCHEMA_VERSION {
        return Err(format!(
            "unsupported output job run schema_version {}; supported {}",
            run.schema_version, OUTPUT_JOB_RUN_SCHEMA_VERSION
        ));
    }
    validate_run_sequence(run.run_sequence)?;
    validate_status_exit_code(run.status, run.exit_code)?;
    if run.status == OutputJobRunStatus::Running && run.artifact_id.is_some() {
        return Err("running output job run must not reference an artifact".to_string());
    }
    validate_run_provenance(run.provenance.as_ref())?;
    validate_log_entries(&run.log)
}

pub(super) fn validate_artifact_run(run: &ArtifactRun) -> Result<(), String> {
    if run.schema_version != ARTIFACT_RUN_SCHEMA_VERSION {
        return Err(format!(
            "unsupported artifact run schema_version {}; supported {}",
            run.schema_version, ARTIFACT_RUN_SCHEMA_VERSION
        ));
    }
    validate_run_sequence(run.run_sequence)?;
    validate_status_exit_code(run.status, run.exit_code)?;
    validate_run_provenance(run.provenance.as_ref())?;
    validate_log_entries(&run.log)
}

pub(super) fn validate_run_evidence_batch_links(
    model: &DesignModel,
    operations: &[Operation],
) -> Result<(), EngineError> {
    let mut output_jobs = model.output_jobs.clone();
    let mut artifact_metadata = model.artifact_metadata.clone();
    let mut output_job_runs = model.output_job_runs.clone();
    let mut artifact_runs = model.artifact_runs.clone();

    for operation in operations {
        match operation {
            Operation::CreateOutputJob {
                output_job_id,
                output_job,
            }
            | Operation::SetOutputJob {
                output_job_id,
                output_job,
                ..
            } => {
                let job = serde_json::from_value::<OutputJob>(output_job.clone())?;
                if job.id != *output_job_id {
                    return Err(EngineError::Validation(format!(
                        "output job payload id {} does not match operation output_job_id {}",
                        job.id, output_job_id
                    )));
                }
                output_jobs.insert(*output_job_id, job);
            }
            Operation::DeleteOutputJob { output_job_id, .. } => {
                output_jobs.remove(output_job_id);
            }
            Operation::SetArtifactMetadata {
                artifact_id,
                artifact_metadata: metadata,
                ..
            } => {
                let metadata = serde_json::from_value::<ArtifactMetadata>(metadata.clone())?;
                if metadata.artifact_id != *artifact_id {
                    return Err(EngineError::Validation(format!(
                        "artifact metadata payload id {} does not match operation artifact_id {}",
                        metadata.artifact_id, artifact_id
                    )));
                }
                artifact_metadata.insert(*artifact_id, metadata);
            }
            Operation::DeleteArtifactMetadata { artifact_id, .. } => {
                artifact_metadata.remove(artifact_id);
            }
            Operation::SetOutputJobRun {
                run_id,
                output_job_run,
                ..
            } => {
                let run = serde_json::from_value::<OutputJobRun>(output_job_run.clone())?;
                if run.run_id != *run_id {
                    return Err(EngineError::Validation(format!(
                        "output job run payload id {} does not match operation run_id {}",
                        run.run_id, run_id
                    )));
                }
                output_job_runs.insert(*run_id, run);
            }
            Operation::DeleteOutputJobRun { run_id, .. } => {
                output_job_runs.remove(run_id);
            }
            Operation::SetArtifactRun {
                run_id,
                artifact_run,
                ..
            } => {
                let run = serde_json::from_value::<ArtifactRun>(artifact_run.clone())?;
                if run.run_id != *run_id {
                    return Err(EngineError::Validation(format!(
                        "artifact run payload id {} does not match operation run_id {}",
                        run.run_id, run_id
                    )));
                }
                artifact_runs.insert(*run_id, run);
            }
            Operation::DeleteArtifactRun { run_id, .. } => {
                artifact_runs.remove(run_id);
            }
            _ => {}
        }
    }

    validate_output_job_run_links(
        model.project.project_id,
        &output_jobs,
        &artifact_metadata,
        &output_job_runs,
    )?;
    validate_artifact_run_links(model.project.project_id, &artifact_metadata, &artifact_runs)
}

fn validate_output_job_run_links(
    project_id: uuid::Uuid,
    output_jobs: &BTreeMap<uuid::Uuid, OutputJob>,
    artifact_metadata: &BTreeMap<uuid::Uuid, ArtifactMetadata>,
    output_job_runs: &BTreeMap<uuid::Uuid, OutputJobRun>,
) -> Result<(), EngineError> {
    let mut output_sequences = BTreeMap::new();
    for (run_id, run) in output_job_runs {
        if run.project_id != project_id {
            return Err(EngineError::Validation(format!(
                "output job run project_id does not match project: {run_id}"
            )));
        }
        if !output_jobs.contains_key(&run.output_job) {
            return Err(EngineError::Validation(format!(
                "output job run references missing output job: {run_id}"
            )));
        }
        if let Some(artifact_id) = run.artifact_id {
            let Some(artifact) = artifact_metadata.get(&artifact_id) else {
                return Err(EngineError::Validation(format!(
                    "output job run references missing artifact metadata: {run_id}"
                )));
            };
            if artifact.project_id != run.project_id {
                return Err(EngineError::Validation(format!(
                    "output job run artifact project_id does not match run: {run_id}"
                )));
            }
            if artifact.model_revision != run.model_revision {
                return Err(EngineError::Validation(format!(
                    "output job run artifact model_revision does not match run: {run_id}"
                )));
            }
        }
        let key = (run.output_job, run.run_sequence);
        if let Some(previous_id) = output_sequences.insert(key, *run_id) {
            return Err(EngineError::Validation(format!(
                "duplicate output job run_sequence for output job: {previous_id}, {run_id}"
            )));
        }
    }
    Ok(())
}

fn validate_artifact_run_links(
    project_id: uuid::Uuid,
    artifact_metadata: &BTreeMap<uuid::Uuid, ArtifactMetadata>,
    artifact_runs: &BTreeMap<uuid::Uuid, ArtifactRun>,
) -> Result<(), EngineError> {
    let mut artifact_sequences = BTreeMap::new();
    for (run_id, run) in artifact_runs {
        if run.project_id != project_id {
            return Err(EngineError::Validation(format!(
                "artifact run project_id does not match project: {run_id}"
            )));
        }
        let Some(artifact) = artifact_metadata.get(&run.artifact_id) else {
            return Err(EngineError::Validation(format!(
                "artifact run references missing artifact metadata: {run_id}"
            )));
        };
        if artifact.project_id != run.project_id {
            return Err(EngineError::Validation(format!(
                "artifact run artifact project_id does not match run: {run_id}"
            )));
        }
        if artifact.model_revision != run.model_revision {
            return Err(EngineError::Validation(format!(
                "artifact run artifact model_revision does not match run: {run_id}"
            )));
        }
        let key = (run.artifact_id, run.run_sequence);
        if let Some(previous_id) = artifact_sequences.insert(key, *run_id) {
            return Err(EngineError::Validation(format!(
                "duplicate artifact run_sequence for artifact: {previous_id}, {run_id}"
            )));
        }
    }
    Ok(())
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

fn validate_run_provenance(provenance: Option<&OutputJobRunProvenance>) -> Result<(), String> {
    let Some(provenance) = provenance else {
        return Ok(());
    };
    if provenance
        .terminal_session_id
        .as_ref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err("run evidence terminal_session_id must not be blank".to_string());
    }
    validate_optional_path(
        "run evidence terminal_context_path",
        provenance.terminal_context_path.as_deref(),
    )?;
    validate_optional_path(
        "run evidence project_root",
        provenance.project_root.as_deref(),
    )?;
    if provenance
        .source_revision
        .as_ref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err("run evidence source_revision must not be blank".to_string());
    }
    Ok(())
}

fn validate_optional_path(label: &str, path: Option<&Path>) -> Result<(), String> {
    if path.is_some_and(|path| path.as_os_str().is_empty()) {
        return Err(format!("{label} must not be empty"));
    }
    Ok(())
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
