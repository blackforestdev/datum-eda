use eda_engine::substrate::{
    ArtifactMetadata, DesignModel, OUTPUT_JOB_RUN_SCHEMA_VERSION, OutputJobLogEntry,
    OutputJobLogLevel, OutputJobRun, OutputJobRunStatus,
};
use uuid::Uuid;

use super::super::command_project_gerber_plan::{
    append_production_projection_log_entries, terminal_origin_log_entries_from,
    terminal_origin_provenance_from,
};
use super::super::command_project_output_jobs::next_output_job_run_sequence;

pub(super) fn generic_output_job_run(
    model: &DesignModel,
    output_job: Uuid,
    scope: &str,
    artifact: &ArtifactMetadata,
) -> OutputJobRun {
    let run_sequence = next_output_job_run_sequence(model, output_job);
    let material = format!(
        "datum-eda:output-job-run:{scope}:{}:{}:{}:{run_sequence}",
        output_job, model.model_revision.0, artifact.artifact_id
    );
    let env = std::env::vars().collect::<std::collections::BTreeMap<_, _>>();
    let mut log = vec![OutputJobLogEntry {
        sequence: 1,
        level: OutputJobLogLevel::Info,
        message: format!(
            "generated {scope} artifact with {} artifact files",
            artifact.files.len()
        ),
    }];
    log.extend(terminal_origin_log_entries_from(&env, 2));
    append_production_projection_log_entries(&mut log, &artifact.production_projections);
    OutputJobRun {
        schema_version: OUTPUT_JOB_RUN_SCHEMA_VERSION,
        run_id: Uuid::new_v5(&model.project.project_id, material.as_bytes()),
        output_job,
        run_sequence,
        project_id: model.project.project_id,
        model_revision: model.model_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
        artifact_id: Some(artifact.artifact_id),
        exit_code: Some(0),
        provenance: terminal_origin_provenance_from(&env),
        log,
    }
}
