use eda_engine::substrate::DesignModel;
use serde_json::{Map, Value};
use std::collections::BTreeSet;

pub(super) fn update_production_visibility(object: &mut Map<String, Value>, model: &DesignModel) {
    let existing_previous_artifact_id = object
        .get("previous_artifact_id")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let mut artifact_ids = BTreeSet::new();
    let mut artifact_file_paths = BTreeSet::new();
    for (artifact_id, artifact) in &model.artifact_metadata {
        artifact_ids.insert(artifact_id.to_string());
        for file in &artifact.files {
            artifact_file_paths.insert(file.path.display().to_string());
        }
    }
    for run in model.artifact_runs.values() {
        artifact_ids.insert(run.artifact_id.to_string());
    }
    let latest_output_job_run = model.output_job_runs.values().max_by(|a, b| {
        a.run_sequence
            .cmp(&b.run_sequence)
            .then_with(|| a.run_id.cmp(&b.run_id))
    });
    let latest_artifact = model.artifact_metadata.values().max_by(|a, b| {
        a.model_revision
            .0
            .cmp(&b.model_revision.0)
            .then_with(|| a.artifact_id.cmp(&b.artifact_id))
    });
    let previous_artifact = latest_artifact.and_then(|latest| {
        model
            .artifact_metadata
            .values()
            .filter(|artifact| artifact.artifact_id != latest.artifact_id)
            .max_by(|a, b| {
                a.model_revision
                    .0
                    .cmp(&b.model_revision.0)
                    .then_with(|| a.artifact_id.cmp(&b.artifact_id))
            })
    });
    let latest_artifact_run = model.artifact_runs.values().max_by(|a, b| {
        a.run_sequence
            .cmp(&b.run_sequence)
            .then_with(|| a.run_id.cmp(&b.run_id))
    });

    object.insert(
        "visible_artifact_ids".to_string(),
        Value::Array(artifact_ids.into_iter().map(Value::String).collect()),
    );
    object.insert(
        "visible_output_job_ids".to_string(),
        Value::Array(
            model
                .output_jobs
                .keys()
                .map(|id| Value::String(id.to_string()))
                .collect(),
        ),
    );
    object.insert(
        "visible_artifact_file_paths".to_string(),
        Value::Array(artifact_file_paths.into_iter().map(Value::String).collect()),
    );
    object.insert(
        "latest_output_job_id".to_string(),
        latest_output_job_run
            .map(|run| Value::String(run.output_job.to_string()))
            .unwrap_or(Value::Null),
    );
    object.insert(
        "latest_output_job_run_id".to_string(),
        latest_output_job_run
            .map(|run| Value::String(run.run_id.to_string()))
            .unwrap_or(Value::Null),
    );
    object.insert(
        "latest_output_job_artifact_id".to_string(),
        latest_output_job_run
            .and_then(|run| run.artifact_id)
            .map(|artifact_id| Value::String(artifact_id.to_string()))
            .unwrap_or(Value::Null),
    );
    object.insert(
        "latest_artifact_id".to_string(),
        latest_artifact
            .map(|artifact| Value::String(artifact.artifact_id.to_string()))
            .unwrap_or(Value::Null),
    );
    object.insert(
        "latest_artifact_run_id".to_string(),
        latest_artifact_run
            .map(|run| Value::String(run.run_id.to_string()))
            .unwrap_or(Value::Null),
    );
    object.insert(
        "previous_artifact_id".to_string(),
        previous_artifact
            .map(|artifact| Value::String(artifact.artifact_id.to_string()))
            .or_else(|| existing_previous_artifact_id.map(Value::String))
            .unwrap_or(Value::Null),
    );
}
