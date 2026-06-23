use std::path::Path;

use anyhow::{Result, anyhow};
use eda_engine::substrate::{ArtifactKind, ProjectResolver, persist_artifact_metadata};

use crate::NativeProjectManufacturingManifestEntryView;

use super::super::super::command_project_manufacturing::{
    NativeProjectManufacturingScope, project_manufacturing_projection,
};
use super::{
    NativeProjectArtifactGenerateEntryView, artifact_file_from_output, generated_artifact_metadata,
};
use super::{
    artifact_production_projection_from_view, export_native_project_drill,
    export_native_project_excellon_drill, find_native_project_output_job_for_scope,
    load_native_project_with_resolved_board, persist_generic_artifact_run,
    persist_generic_output_job_run, sanitize_export_prefix,
};

pub(super) fn generate_drill_artifact(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
    persist_output_runs: bool,
) -> Result<NativeProjectArtifactGenerateEntryView> {
    std::fs::create_dir_all(output_dir)?;
    let project = load_native_project_with_resolved_board(root)?;
    let prefix = sanitize_export_prefix(prefix_override.unwrap_or(&project.board.name));
    let model = ProjectResolver::new(root).resolve()?;
    let scope = NativeProjectManufacturingScope {
        prefix: prefix.clone(),
        variant: None,
        output_job_id: None,
        manufacturing_plan_id: None,
        board_or_panel: project.board.uuid,
        panel_projection: None,
        include: vec![ArtifactKind::Drill],
    };
    let projection = project_manufacturing_projection(root, &scope)?;
    let drill_csv_name = projection_file_name(&projection.entries, "drill_csv")?;
    let excellon_name = projection_file_name(&projection.entries, "excellon_drill")?;
    let drill_csv_report = export_native_project_drill(root, &output_dir.join(&drill_csv_name))?;
    let excellon_report =
        export_native_project_excellon_drill(root, &output_dir.join(&excellon_name))?;
    let files = vec![
        artifact_file_from_output(output_dir, &drill_csv_name)?,
        artifact_file_from_output(output_dir, &excellon_name)?,
    ];
    let output_job = find_native_project_output_job_for_scope(&model, &prefix, ArtifactKind::Drill);
    let artifact_metadata = generated_artifact_metadata(
        &model,
        &prefix,
        ArtifactKind::Drill,
        "drill",
        output_job,
        output_dir,
        files,
        vec![artifact_production_projection_from_view(
            excellon_report.production_projection.clone(),
        )],
    );
    let artifact_manifest_path = persist_artifact_metadata(root, &artifact_metadata)?;
    let output_job_run = persist_generic_output_job_run(
        root,
        "drill",
        &model,
        &artifact_metadata,
        persist_output_runs,
    )?;
    let artifact_run = if output_job_run.is_none() {
        Some(persist_generic_artifact_run(
            root,
            "drill",
            &model,
            &artifact_metadata,
        )?)
    } else {
        None
    };
    Ok(NativeProjectArtifactGenerateEntryView {
        include: "drill".to_string(),
        artifact_id: artifact_metadata.artifact_id,
        kind: "drill".to_string(),
        model_revision: artifact_metadata.model_revision.0.clone(),
        file_count: artifact_metadata.files.len(),
        artifact_manifest_path: artifact_manifest_path.display().to_string(),
        output_job_run: output_job_run.as_ref().map(|(run, _)| run.clone()),
        output_job_run_path: output_job_run
            .as_ref()
            .map(|(_, path)| path.display().to_string()),
        artifact_run: artifact_run.as_ref().map(|(run, _)| run.clone()),
        artifact_run_path: artifact_run
            .as_ref()
            .map(|(_, path)| path.display().to_string()),
        report: serde_json::json!({
            "action": "generate_drill_artifact",
            "artifact_metadata": artifact_metadata,
            "output_job_run": output_job_run.as_ref().map(|(run, _)| run),
            "output_job_run_path": output_job_run
                .as_ref()
                .map(|(_, path)| path.display().to_string()),
            "artifact_run": artifact_run.as_ref().map(|(run, _)| run),
            "artifact_run_path": artifact_run
                .as_ref()
                .map(|(_, path)| path.display().to_string()),
            "projection": projection,
            "source_reports": {
                "drill_csv": drill_csv_report,
                "excellon_drill": excellon_report,
            },
        }),
    })
}

fn projection_file_name(
    entries: &[NativeProjectManufacturingManifestEntryView],
    kind: &str,
) -> Result<String> {
    entries
        .iter()
        .find(|entry| entry.kind == kind)
        .map(|entry| entry.filename.clone())
        .ok_or_else(|| anyhow!("manufacturing projection missing {kind} entry"))
}
