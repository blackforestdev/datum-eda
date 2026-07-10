use crate::*;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use anyhow::{Result, anyhow};
use eda_engine::substrate::{
    ARTIFACT_METADATA_SCHEMA_VERSION, ArtifactFile, ArtifactKind, ArtifactMetadata,
    ArtifactProductionProjection, ArtifactRun, ArtifactValidationState, DesignModel, OutputJobRun,
    ProjectResolver,
};
use serde::Serialize;
use uuid::Uuid;

#[path = "drill.rs"]
mod command_project_artifact_drill;
#[path = "evidence.rs"]
mod command_project_artifact_evidence;
#[path = "include.rs"]
mod command_project_artifact_include;
#[path = "latest.rs"]
mod command_project_artifact_latest;
#[path = "output_runs.rs"]
mod command_project_artifact_output_runs;
#[path = "preview.rs"]
mod command_project_artifact_preview;
// Shared check-gate helper #[path]-included into several sibling command modules by design.
#[allow(clippy::duplicate_mod)]
#[path = "../check/gate.rs"]
mod command_project_check_gate;
use super::runs::compare_artifact_runs;
pub(super) use command_project_artifact_evidence::{
    commit_artifact_metadata_evidence, commit_linked_artifact_output_job_evidence,
    commit_unlinked_artifact_evidence,
};
use command_project_artifact_include::parse_artifact_generate_include;
use command_project_artifact_latest::latest_artifact_id;
use command_project_artifact_output_runs::generic_output_job_run;
use command_project_artifact_preview::{
    MAX_PREVIEW_PRIMITIVES, NativeProjectArtifactPreviewPrimitive, inspect_artifact_preview_file,
};

use self::command_project_check_gate::ensure_release_check_gate_clear;
use crate::commands::gerber::plan::{
    artifact_production_projection_from_view, sanitize_export_prefix,
};
use crate::{
    compute_source_hash_bytes, export_native_project_bom, export_native_project_drill,
    export_native_project_excellon_drill, export_native_project_gerber_set,
    export_native_project_gerber_set_without_output_run,
    export_native_project_gerber_set_without_output_run_for_output_job,
    export_native_project_manufacturing_set,
    export_native_project_manufacturing_set_without_output_run, export_native_project_pnp,
    find_native_project_output_job_for_scope, load_native_project_with_resolved_board,
};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectArtifactsView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) artifact_count: usize,
    pub(crate) artifact_run_count: usize,
    pub(crate) output_job_run_count: usize,
    pub(crate) latest_artifact_id: Option<Uuid>,
    pub(crate) latest_artifact_run_id: Option<Uuid>,
    pub(crate) latest_output_job_run_id: Option<Uuid>,
    pub(crate) artifacts: Vec<ArtifactMetadata>,
    pub(crate) artifact_runs: Vec<ArtifactRun>,
    pub(crate) output_job_runs: Vec<OutputJobRun>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectArtifactView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) artifact: ArtifactMetadata,
    pub(crate) run_count: usize,
    pub(crate) latest_run: Option<ArtifactRun>,
    pub(crate) runs: Vec<ArtifactRun>,
    pub(crate) output_job_run_count: usize,
    pub(crate) latest_output_job_run: Option<OutputJobRun>,
    pub(crate) output_job_runs: Vec<OutputJobRun>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectArtifactFilesView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) artifact_id: Uuid,
    pub(crate) kind: ArtifactKind,
    pub(crate) output_dir: Option<PathBuf>,
    pub(crate) artifact_model_revision: String,
    pub(crate) validation_state: ArtifactValidationState,
    pub(crate) file_count: usize,
    pub(crate) files: Vec<ArtifactFile>,
    pub(crate) production_projection_count: usize,
    pub(crate) production_projections: Vec<ArtifactProductionProjection>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectArtifactPreviewView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) artifact_id: Uuid,
    pub(crate) kind: ArtifactKind,
    pub(crate) output_dir: String,
    pub(crate) file: PathBuf,
    pub(crate) file_path: String,
    pub(crate) expected_sha256: String,
    pub(crate) actual_sha256: String,
    pub(crate) hash_matches_metadata: bool,
    pub(crate) preview_kind: String,
    pub(crate) preview_available: bool,
    pub(crate) primitive_count: usize,
    pub(crate) primitive_limit: usize,
    pub(crate) primitives: Vec<NativeProjectArtifactPreviewPrimitive>,
    pub(crate) inspection: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectArtifactComparisonView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) before_artifact_id: Uuid,
    pub(crate) after_artifact_id: Uuid,
    pub(crate) equivalent: bool,
    pub(crate) kind_equal: bool,
    pub(crate) model_revision_equal: bool,
    pub(crate) validation_state_equal: bool,
    pub(crate) files_equal: bool,
    pub(crate) production_projections_equal: bool,
    pub(crate) before_file_count: usize,
    pub(crate) after_file_count: usize,
    pub(crate) added_files: Vec<PathBuf>,
    pub(crate) removed_files: Vec<PathBuf>,
    pub(crate) changed_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectArtifactGenerateView {
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) project_root: String,
    pub(crate) output_dir: String,
    pub(crate) include: Vec<String>,
    pub(crate) generated_count: usize,
    pub(crate) generated: Vec<NativeProjectArtifactGenerateEntryView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectArtifactGenerateEntryView {
    pub(crate) include: String,
    pub(crate) artifact_id: Uuid,
    pub(crate) kind: String,
    pub(crate) model_revision: String,
    pub(crate) file_count: usize,
    pub(crate) artifact_manifest_path: String,
    pub(crate) output_job_run: Option<OutputJobRun>,
    pub(crate) output_job_run_path: Option<String>,
    pub(crate) artifact_run: Option<ArtifactRun>,
    pub(crate) artifact_run_path: Option<String>,
    pub(crate) report: serde_json::Value,
}

pub(crate) fn generate_native_project_artifacts(
    root: &Path,
    output_dir: &Path,
    include: &str,
    prefix: Option<&str>,
    variant: Option<Uuid>,
    output_job_id: Option<Uuid>,
    persist_output_runs: bool,
) -> Result<NativeProjectArtifactGenerateView> {
    if persist_output_runs {
        ensure_release_check_gate_clear(root)?;
    }
    let include = parse_artifact_generate_include(include)?;
    let mut generated = Vec::new();
    for scope in &include {
        match scope.as_str() {
            "gerber-set" => {
                let report = if persist_output_runs {
                    export_native_project_gerber_set(root, output_dir, prefix)?
                } else if let Some(output_job_id) = output_job_id {
                    export_native_project_gerber_set_without_output_run_for_output_job(
                        root,
                        output_dir,
                        prefix,
                        output_job_id,
                    )?
                } else {
                    export_native_project_gerber_set_without_output_run(root, output_dir, prefix)?
                };
                generated.push(generated_entry(
                    scope,
                    "gerber_set",
                    &report.artifact_metadata,
                    &report.artifact_manifest_path,
                    &report,
                )?);
            }
            "manufacturing-set" => {
                let report = if persist_output_runs {
                    export_native_project_manufacturing_set(
                        root,
                        output_dir,
                        prefix,
                        variant,
                        None,
                        output_job_id,
                        None,
                    )?
                } else {
                    export_native_project_manufacturing_set_without_output_run(
                        root,
                        output_dir,
                        prefix,
                        variant,
                        None,
                        output_job_id,
                        None,
                    )?
                };
                generated.push(generated_entry(
                    scope,
                    "manufacturing_set",
                    &report.artifact_metadata,
                    &report.artifact_manifest_path,
                    &report,
                )?);
            }
            "bom" => {
                generated.push(generate_single_file_artifact(
                    root,
                    output_dir,
                    prefix,
                    "bom",
                    ArtifactKind::Bom,
                    variant,
                    persist_output_runs,
                )?);
            }
            "pnp" => {
                generated.push(generate_single_file_artifact(
                    root,
                    output_dir,
                    prefix,
                    "pnp",
                    ArtifactKind::Pnp,
                    variant,
                    persist_output_runs,
                )?);
            }
            "drill" => {
                generated.push(command_project_artifact_drill::generate_drill_artifact(
                    root,
                    output_dir,
                    prefix,
                    persist_output_runs,
                )?);
            }
            _ => unreachable!("include parser returned unsupported scope"),
        }
    }
    Ok(NativeProjectArtifactGenerateView {
        contract: "artifact_generate_v1",
        action: "generate_artifacts",
        project_root: root.display().to_string(),
        output_dir: output_dir.display().to_string(),
        generated_count: generated.len(),
        include,
        generated,
    })
}

fn generated_entry<T: Serialize>(
    scope: &str,
    kind: &str,
    metadata: &ArtifactMetadata,
    artifact_manifest_path: &str,
    report: &T,
) -> Result<NativeProjectArtifactGenerateEntryView> {
    let report = serde_json::to_value(report)?;
    let output_job_run = optional_report_object::<OutputJobRun>(&report, "output_job_run")?;
    let output_job_run_path = optional_report_string(&report, "output_job_run_path");
    let artifact_run = optional_report_object::<ArtifactRun>(&report, "artifact_run")?;
    let artifact_run_path = optional_report_string(&report, "artifact_run_path");
    Ok(NativeProjectArtifactGenerateEntryView {
        include: scope.to_string(),
        artifact_id: metadata.artifact_id,
        kind: kind.to_string(),
        model_revision: metadata.model_revision.0.clone(),
        file_count: metadata.files.len(),
        artifact_manifest_path: artifact_manifest_path.to_string(),
        output_job_run,
        output_job_run_path,
        artifact_run,
        artifact_run_path,
        report,
    })
}
fn optional_report_object<T: serde::de::DeserializeOwned>(
    report: &serde_json::Value,
    key: &str,
) -> Result<Option<T>> {
    match report.get(key) {
        Some(value) if !value.is_null() => Ok(Some(serde_json::from_value(value.clone())?)),
        _ => Ok(None),
    }
}
fn optional_report_string(report: &serde_json::Value, key: &str) -> Option<String> {
    report
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}
fn generate_single_file_artifact(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
    scope: &str,
    kind: ArtifactKind,
    variant: Option<Uuid>,
    persist_output_runs: bool,
) -> Result<NativeProjectArtifactGenerateEntryView> {
    std::fs::create_dir_all(output_dir)?;
    let project = load_native_project_with_resolved_board(root)?;
    let prefix = sanitize_export_prefix(prefix_override.unwrap_or(&project.board.name));
    let file_name = format!("{prefix}-{scope}.csv");
    let output_path = output_dir.join(&file_name);
    let report = match scope {
        "bom" => serde_json::to_value(export_native_project_bom(root, &output_path, variant)?)?,
        "pnp" => serde_json::to_value(export_native_project_pnp(root, &output_path, variant)?)?,
        _ => unreachable!("single file artifact scope is not supported"),
    };
    let mut model = ProjectResolver::new(root).resolve()?;
    let files = vec![artifact_file_from_output(output_dir, &file_name)?];
    let output_job = find_native_project_output_job_for_scope(&model, &prefix, kind);
    let artifact_metadata = generated_artifact_metadata(
        &model,
        &prefix,
        kind,
        scope,
        output_job,
        output_dir,
        files,
        Vec::new(),
    );
    let output_job_run = if persist_output_runs {
        artifact_metadata
            .output_job
            .map(|output_job| generic_output_job_run(&model, output_job, scope, &artifact_metadata))
    } else {
        None
    };
    let (artifact_manifest_path, output_job_run_path, artifact_run) = if let Some(run) =
        &output_job_run
    {
        let (manifest_path, run_path) =
            commit_linked_artifact_output_job_evidence(root, &mut model, &artifact_metadata, run)?;
        (manifest_path, Some(run_path), None)
    } else {
        let (manifest_path, run, run_path) =
            commit_unlinked_artifact_evidence(root, scope, &mut model, &artifact_metadata)?;
        (manifest_path, None, Some((run, run_path)))
    };
    Ok(NativeProjectArtifactGenerateEntryView {
        include: scope.to_string(),
        artifact_id: artifact_metadata.artifact_id,
        kind: artifact_kind_name(kind).to_string(),
        model_revision: artifact_metadata.model_revision.0.clone(),
        file_count: artifact_metadata.files.len(),
        artifact_manifest_path: artifact_manifest_path.display().to_string(),
        output_job_run: output_job_run.clone(),
        output_job_run_path: output_job_run_path
            .as_ref()
            .map(|path| path.display().to_string()),
        artifact_run: artifact_run.as_ref().map(|(run, _)| run.clone()),
        artifact_run_path: artifact_run
            .as_ref()
            .map(|(_, path)| path.display().to_string()),
        report: serde_json::json!({
            "action": format!("generate_{scope}_artifact"),
            "artifact_metadata": artifact_metadata,
            "output_job_run": output_job_run.as_ref(),
            "output_job_run_path": output_job_run_path
                .as_ref()
                .map(|path| path.display().to_string()),
            "artifact_run": artifact_run.as_ref().map(|(run, _)| run),
            "artifact_run_path": artifact_run
                .as_ref()
                .map(|(_, path)| path.display().to_string()),
            "source_report": report,
        }),
    })
}

fn artifact_file_from_output(output_dir: &Path, file_name: &str) -> Result<ArtifactFile> {
    let output_path = output_dir.join(file_name);
    let bytes = std::fs::read(&output_path)
        .map_err(|error| anyhow!("failed to read {}: {error}", output_path.display()))?;
    Ok(ArtifactFile {
        path: PathBuf::from(file_name),
        sha256: compute_source_hash_bytes(&bytes),
    })
}

// CLI command handler threads individually parsed flag values.
#[allow(clippy::too_many_arguments)]
fn generated_artifact_metadata(
    model: &DesignModel,
    prefix: &str,
    kind: ArtifactKind,
    scope: &str,
    output_job: Option<Uuid>,
    output_dir: &Path,
    files: Vec<ArtifactFile>,
    production_projections: Vec<ArtifactProductionProjection>,
) -> ArtifactMetadata {
    let mut material = format!(
        "datum-eda:artifact:{scope}:{}:{}:{}:{}",
        model.project.project_id,
        model.model_revision.0,
        env!("CARGO_PKG_VERSION"),
        prefix
    );
    for file in &files {
        material.push('|');
        material.push_str(&file.path.to_string_lossy());
        material.push('=');
        material.push_str(&file.sha256);
    }
    ArtifactMetadata {
        schema_version: ARTIFACT_METADATA_SCHEMA_VERSION,
        artifact_id: Uuid::new_v5(&model.project.project_id, material.as_bytes()),
        kind,
        project_id: model.project.project_id,
        model_revision: model.model_revision.clone(),
        output_job,
        variant: None,
        generator_version: env!("CARGO_PKG_VERSION").to_string(),
        output_dir: Some(output_dir.to_path_buf()),
        files,
        production_projections,
        validation_state: ArtifactValidationState::NotValidated,
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

pub(crate) fn query_native_project_artifacts(root: &Path) -> Result<NativeProjectArtifactsView> {
    let model = ProjectResolver::new(root).resolve()?;
    let artifacts = model
        .artifact_metadata
        .values()
        .cloned()
        .collect::<Vec<_>>();
    let mut artifact_runs = model.artifact_runs.values().cloned().collect::<Vec<_>>();
    artifact_runs.sort_by(compare_artifact_runs);
    let mut output_job_runs = model
        .output_job_runs
        .values()
        .filter(|run| run.artifact_id.is_some())
        .cloned()
        .collect::<Vec<_>>();
    output_job_runs.sort_by(compare_linked_output_job_runs);
    let latest_artifact_id = latest_artifact_id(&model, &artifacts);
    let latest_artifact_run_id = latest_artifact_run_id(&artifact_runs);
    let latest_output_job_run_id = latest_output_job_run_id(&output_job_runs);
    Ok(NativeProjectArtifactsView {
        contract: "artifact_metadata_list_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        artifact_count: artifacts.len(),
        artifact_run_count: artifact_runs.len(),
        output_job_run_count: output_job_runs.len(),
        latest_artifact_id,
        latest_artifact_run_id,
        latest_output_job_run_id,
        artifacts,
        artifact_runs,
        output_job_runs,
    })
}

fn latest_artifact_run_id(runs: &[ArtifactRun]) -> Option<Uuid> {
    runs.iter()
        .max_by(|a, b| {
            a.run_sequence
                .cmp(&b.run_sequence)
                .then_with(|| a.run_id.cmp(&b.run_id))
        })
        .map(|run| run.run_id)
}

fn latest_output_job_run_id(runs: &[OutputJobRun]) -> Option<Uuid> {
    runs.iter()
        .max_by(|a, b| {
            a.run_sequence
                .cmp(&b.run_sequence)
                .then_with(|| a.run_id.cmp(&b.run_id))
        })
        .map(|run| run.run_id)
}

pub(crate) fn query_native_project_artifact(
    root: &Path,
    artifact_id: Uuid,
) -> Result<NativeProjectArtifactView> {
    let model = ProjectResolver::new(root).resolve()?;
    let artifact = model
        .artifact_metadata
        .get(&artifact_id)
        .cloned()
        .ok_or_else(|| anyhow!("artifact metadata not found: {artifact_id}"))?;
    let mut runs = model
        .artifact_runs
        .values()
        .filter(|run| run.artifact_id == artifact_id)
        .cloned()
        .collect::<Vec<_>>();
    runs.sort_by(compare_artifact_runs);
    let latest_run = runs.last().cloned();
    let mut output_job_runs = model
        .output_job_runs
        .values()
        .filter(|run| run.artifact_id == Some(artifact_id))
        .cloned()
        .collect::<Vec<_>>();
    output_job_runs.sort_by(compare_linked_output_job_runs);
    let latest_output_job_run = output_job_runs.last().cloned();
    Ok(NativeProjectArtifactView {
        contract: "artifact_metadata_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        artifact,
        run_count: runs.len(),
        latest_run,
        runs,
        output_job_run_count: output_job_runs.len(),
        latest_output_job_run,
        output_job_runs,
    })
}

fn compare_linked_output_job_runs(a: &OutputJobRun, b: &OutputJobRun) -> std::cmp::Ordering {
    a.output_job
        .cmp(&b.output_job)
        .then_with(|| a.run_sequence.cmp(&b.run_sequence))
        .then_with(|| a.run_id.cmp(&b.run_id))
}

pub(crate) fn query_native_project_artifact_files(
    root: &Path,
    artifact_id: Uuid,
) -> Result<NativeProjectArtifactFilesView> {
    let model = ProjectResolver::new(root).resolve()?;
    let artifact = model
        .artifact_metadata
        .get(&artifact_id)
        .cloned()
        .ok_or_else(|| anyhow!("artifact metadata not found: {artifact_id}"))?;
    Ok(NativeProjectArtifactFilesView {
        contract: "artifact_files_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        artifact_id,
        kind: artifact.kind,
        output_dir: artifact.output_dir,
        artifact_model_revision: artifact.model_revision.0,
        validation_state: artifact.validation_state,
        file_count: artifact.files.len(),
        files: artifact.files,
        production_projection_count: artifact.production_projections.len(),
        production_projections: artifact.production_projections,
    })
}

pub(crate) fn preview_native_project_artifact_file(
    root: &Path,
    artifact_id: Uuid,
    artifact_dir: Option<&Path>,
    file: &Path,
) -> Result<NativeProjectArtifactPreviewView> {
    if !artifact_file_path_is_safe(file) {
        return Err(anyhow!(
            "artifact preview file must be a safe relative path"
        ));
    }
    let model = ProjectResolver::new(root).resolve()?;
    let artifact = model
        .artifact_metadata
        .get(&artifact_id)
        .cloned()
        .ok_or_else(|| anyhow!("artifact metadata not found: {artifact_id}"))?;
    let artifact_file = artifact
        .files
        .iter()
        .find(|candidate| candidate.path == file)
        .cloned()
        .ok_or_else(|| anyhow!("artifact file not found in metadata: {}", file.display()))?;
    let artifact_dir = artifact_dir
        .map(Path::to_path_buf)
        .or_else(|| artifact.output_dir.clone())
        .ok_or_else(|| {
            anyhow!(
                "artifact preview requires --artifact-dir because artifact metadata has no output_dir"
            )
        })?;
    let effective_artifact_dir = if artifact_dir.is_absolute() {
        artifact_dir
    } else {
        root.join(artifact_dir)
    };
    let file_path = effective_artifact_dir.join(file);
    let bytes = std::fs::read(&file_path)
        .map_err(|error| anyhow!("failed to read {}: {error}", file_path.display()))?;
    let actual_sha256 = compute_source_hash_bytes(&bytes);
    let hash_matches_metadata = actual_sha256 == artifact_file.sha256;
    let (preview_kind, preview_available, inspection, primitives) =
        inspect_artifact_preview_file(&file_path, &bytes)?;
    let primitive_count = primitives.len();
    Ok(NativeProjectArtifactPreviewView {
        contract: "artifact_file_preview_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        artifact_id,
        kind: artifact.kind,
        output_dir: effective_artifact_dir.display().to_string(),
        file: artifact_file.path,
        file_path: file_path.display().to_string(),
        expected_sha256: artifact_file.sha256,
        actual_sha256,
        hash_matches_metadata,
        preview_kind,
        preview_available,
        primitive_count,
        primitive_limit: MAX_PREVIEW_PRIMITIVES,
        primitives,
        inspection,
    })
}

pub(crate) fn compare_native_project_artifacts(
    root: &Path,
    before_artifact_id: Uuid,
    after_artifact_id: Uuid,
) -> Result<NativeProjectArtifactComparisonView> {
    let model = ProjectResolver::new(root).resolve()?;
    let before = model
        .artifact_metadata
        .get(&before_artifact_id)
        .ok_or_else(|| anyhow!("artifact metadata not found: {before_artifact_id}"))?;
    let after = model
        .artifact_metadata
        .get(&after_artifact_id)
        .ok_or_else(|| anyhow!("artifact metadata not found: {after_artifact_id}"))?;
    let before_files = artifact_file_hashes(before);
    let after_files = artifact_file_hashes(after);
    let added_files = after_files
        .keys()
        .filter(|path| !before_files.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    let removed_files = before_files
        .keys()
        .filter(|path| !after_files.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    let changed_files = before_files
        .iter()
        .filter(|(path, before_hash)| {
            after_files
                .get(*path)
                .is_some_and(|after_hash| after_hash != *before_hash)
        })
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    let kind_equal = before.kind == after.kind;
    let model_revision_equal = before.model_revision == after.model_revision;
    let validation_state_equal = before.validation_state == after.validation_state;
    let files_equal = before_files == after_files;
    let production_projections_equal =
        before.production_projections == after.production_projections;
    Ok(NativeProjectArtifactComparisonView {
        contract: "artifact_metadata_compare_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        before_artifact_id,
        after_artifact_id,
        equivalent: kind_equal
            && model_revision_equal
            && validation_state_equal
            && files_equal
            && production_projections_equal,
        kind_equal,
        model_revision_equal,
        validation_state_equal,
        files_equal,
        production_projections_equal,
        before_file_count: before.files.len(),
        after_file_count: after.files.len(),
        added_files,
        removed_files,
        changed_files,
    })
}

fn artifact_file_hashes(artifact: &ArtifactMetadata) -> BTreeMap<PathBuf, String> {
    artifact
        .files
        .iter()
        .map(|file| (file.path.clone(), file.sha256.clone()))
        .collect()
}

fn artifact_file_path_is_safe(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

// Phase 5: exec-layer dissolution — variant run() impls (the former
// command_exec destructure-and-forward glue, now inherent methods on the
// clap args structs).

impl ArtifactGenerateArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            output_dir,
            include,
            prefix,
            output_job,
        } = self;
        if let Some(output_job) = output_job {
            let report = run_native_project_output_job(&path, output_job, output_dir.as_deref())?;
            let exit_code = report.exit_code;
            return Ok((render_output(format, &report), exit_code));
        }
        let output_dir = output_dir.as_deref().ok_or_else(|| {
            anyhow::anyhow!(
                "artifact generate requires --output-dir unless --output-job is provided"
            )
        })?;
        let include = include.as_deref().ok_or_else(|| {
            anyhow::anyhow!("artifact generate requires --include unless --output-job is provided")
        })?;
        Ok((
            render_output(
                format,
                &generate_native_project_artifacts(
                    &path,
                    output_dir,
                    include,
                    prefix.as_deref(),
                    None,
                    None,
                    true,
                )?,
            ),
            0,
        ))
    }
}

impl ArtifactStartOutputJobRunArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, output_job } = self;
        Ok((
            render_output(
                format,
                &start_native_project_output_job_run(&path, output_job)?,
            ),
            0,
        ))
    }
}

impl ArtifactCancelOutputJobRunArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, run } = self;
        Ok((
            render_output(format, &cancel_native_project_output_job_run(&path, run)?),
            0,
        ))
    }
}

impl ArtifactListArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path } = self;
        Ok((
            render_output(format, &query_native_project_artifacts(&path)?),
            0,
        ))
    }
}

impl ArtifactShowArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, artifact } = self;
        Ok((
            render_output(format, &query_native_project_artifact(&path, artifact)?),
            0,
        ))
    }
}

impl ArtifactFilesArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, artifact } = self;
        Ok((
            render_output(
                format,
                &query_native_project_artifact_files(&path, artifact)?,
            ),
            0,
        ))
    }
}

impl ArtifactPreviewArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            artifact,
            artifact_dir,
            file,
        } = self;
        Ok((
            render_output(
                format,
                &preview_native_project_artifact_file(
                    &path,
                    artifact,
                    artifact_dir.as_deref(),
                    &file,
                )?,
            ),
            0,
        ))
    }
}

impl ArtifactCompareArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            before,
            after,
        } = self;
        Ok((
            render_output(
                format,
                &compare_native_project_artifacts(&path, before, after)?,
            ),
            0,
        ))
    }
}

impl ArtifactValidateArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, artifact } = self;
        let report = validate_native_project_artifact(&path, artifact)?;
        let exit_code = if report.valid { 0 } else { 1 };
        Ok((render_output(format, &report), exit_code))
    }
}
