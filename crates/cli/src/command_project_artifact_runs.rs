use std::path::{Path, PathBuf};

use anyhow::Result;
use eda_engine::substrate::{
    ArtifactKind, ArtifactMetadata, ArtifactRun, DesignModel, OutputJobLogEntry, OutputJobLogLevel,
    OutputJobRunStatus, persist_artifact_run,
};
use uuid::Uuid;

use super::command_project_gerber_plan::{
    append_production_projection_log_entries, terminal_origin_log_entries_from,
    terminal_origin_provenance_from,
};

pub(crate) fn persist_generic_artifact_run(
    root: &Path,
    scope: &str,
    model: &DesignModel,
    artifact: &ArtifactMetadata,
) -> Result<(ArtifactRun, PathBuf)> {
    let run = generic_artifact_run(model, scope, artifact);
    let path = persist_artifact_run(root, &run)?;
    Ok((run, path))
}

pub(crate) fn compare_artifact_runs(a: &ArtifactRun, b: &ArtifactRun) -> std::cmp::Ordering {
    a.run_sequence
        .cmp(&b.run_sequence)
        .then_with(|| a.run_id.cmp(&b.run_id))
}

fn next_artifact_run_sequence(model: &DesignModel, artifact_id: Uuid) -> u64 {
    model
        .artifact_runs
        .values()
        .filter(|run| run.artifact_id == artifact_id)
        .map(|run| run.run_sequence)
        .max()
        .unwrap_or(0)
        + 1
}

fn generic_artifact_run(
    model: &DesignModel,
    scope: &str,
    artifact: &ArtifactMetadata,
) -> ArtifactRun {
    let run_sequence = next_artifact_run_sequence(model, artifact.artifact_id);
    let material = format!(
        "datum-eda:artifact-run:{scope}:{}:{}:{}:{run_sequence}",
        artifact.artifact_id,
        model.model_revision.0,
        artifact_kind_name(artifact.kind)
    );
    let env = std::env::vars().collect::<std::collections::BTreeMap<_, _>>();
    let mut log = vec![OutputJobLogEntry {
        sequence: 1,
        level: OutputJobLogLevel::Info,
        message: format!(
            "generated unlinked {scope} artifact with {} artifact files",
            artifact.files.len()
        ),
    }];
    log.extend(terminal_origin_log_entries_from(&env, 2));
    append_production_projection_log_entries(&mut log, &artifact.production_projections);
    ArtifactRun {
        run_id: Uuid::new_v5(&model.project.project_id, material.as_bytes()),
        artifact_id: artifact.artifact_id,
        run_sequence,
        project_id: model.project.project_id,
        model_revision: model.model_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
        exit_code: Some(0),
        provenance: terminal_origin_provenance_from(&env),
        log,
    }
}

fn artifact_kind_name(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::GerberSet => "gerber_set",
        ArtifactKind::ManufacturingSet => "manufacturing_set",
        ArtifactKind::Bom => "bom",
        ArtifactKind::Pnp => "pnp",
        ArtifactKind::Drill => "drill",
    }
}
