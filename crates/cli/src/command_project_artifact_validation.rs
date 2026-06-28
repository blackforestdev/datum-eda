use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use eda_engine::substrate::{
    ArtifactKind, ArtifactMetadata, ArtifactValidationState, ProjectResolver,
};
use serde::Serialize;
use uuid::Uuid;

use super::command_project_artifacts::commit_artifact_metadata_evidence;
use super::{
    compare_native_project_bom, compare_native_project_excellon_drill, compare_native_project_pnp,
    compute_source_hash_bytes, render_expected_native_project_drill_csv,
};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectArtifactValidationView {
    pub(crate) contract: &'static str,
    pub(crate) project_id: String,
    pub(crate) model_revision: String,
    pub(crate) artifact_id: Uuid,
    pub(crate) valid: bool,
    pub(crate) validation_state: String,
    pub(crate) artifact_manifest_path: Option<String>,
    pub(crate) project_id_matches: bool,
    pub(crate) model_revision_current: bool,
    pub(crate) output_dir_available: bool,
    pub(crate) file_count: usize,
    pub(crate) unsafe_file_paths: Vec<PathBuf>,
    pub(crate) invalid_file_hashes: Vec<PathBuf>,
    pub(crate) file_hash_mismatches: Vec<PathBuf>,
    pub(crate) semantic_mismatches: Vec<PathBuf>,
    pub(crate) invalid_projection_hash_count: usize,
}

pub(crate) fn validate_native_project_artifact(
    root: &Path,
    artifact_id: Uuid,
) -> Result<NativeProjectArtifactValidationView> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let mut artifact = model
        .artifact_metadata
        .get(&artifact_id)
        .cloned()
        .ok_or_else(|| anyhow!("artifact metadata not found: {artifact_id}"))?;
    let unsafe_file_paths = artifact
        .files
        .iter()
        .filter(|file| !artifact_file_path_is_safe(&file.path))
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let invalid_file_hashes = artifact
        .files
        .iter()
        .filter(|file| !file.sha256.starts_with("sha256:"))
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let invalid_projection_hash_count = artifact
        .production_projections
        .iter()
        .filter(|projection| !projection.sha256.starts_with("sha256:"))
        .count();
    let project_id_matches = artifact.project_id == model.project.project_id;
    let model_revision_current = artifact.model_revision == model.model_revision;
    let (output_dir_available, file_hash_mismatches) =
        artifact_file_hash_mismatches(root, &artifact)?;
    let semantic_mismatches = artifact_semantic_mismatches(root, &artifact)?;
    let valid = project_id_matches
        && model_revision_current
        && unsafe_file_paths.is_empty()
        && invalid_file_hashes.is_empty()
        && file_hash_mismatches.is_empty()
        && semantic_mismatches.is_empty()
        && invalid_projection_hash_count == 0;
    let artifact_manifest_path = if artifact_validation_should_persist(artifact.kind) {
        artifact.validation_state = if valid {
            ArtifactValidationState::Valid
        } else {
            ArtifactValidationState::Invalid
        };
        Some(
            commit_artifact_metadata_evidence(root, &mut model, &artifact)
                .context("failed to persist artifact validation state")?
                .display()
                .to_string(),
        )
    } else {
        None
    };
    Ok(NativeProjectArtifactValidationView {
        contract: "artifact_metadata_validation_v1",
        project_id: model.project.project_id.to_string(),
        model_revision: model.model_revision.0,
        artifact_id,
        valid,
        validation_state: artifact_validation_state_label(artifact.validation_state).to_string(),
        artifact_manifest_path,
        project_id_matches,
        model_revision_current,
        output_dir_available,
        file_count: artifact.files.len(),
        unsafe_file_paths,
        invalid_file_hashes,
        file_hash_mismatches,
        semantic_mismatches,
        invalid_projection_hash_count,
    })
}

fn artifact_file_hash_mismatches(
    root: &Path,
    artifact: &ArtifactMetadata,
) -> Result<(bool, Vec<PathBuf>)> {
    let Some(output_dir) = artifact.output_dir.as_ref() else {
        return Ok((
            false,
            artifact
                .files
                .iter()
                .map(|file| file.path.clone())
                .collect(),
        ));
    };
    let effective_output_dir = if output_dir.is_absolute() {
        output_dir.clone()
    } else {
        root.join(output_dir)
    };
    let mut mismatches = Vec::new();
    for file in &artifact.files {
        if !artifact_file_path_is_safe(&file.path) {
            mismatches.push(file.path.clone());
            continue;
        }
        let file_path = effective_output_dir.join(&file.path);
        let Ok(bytes) = std::fs::read(&file_path) else {
            mismatches.push(file.path.clone());
            continue;
        };
        if compute_source_hash_bytes(&bytes) != file.sha256 {
            mismatches.push(file.path.clone());
        }
    }
    Ok((true, mismatches))
}

fn artifact_semantic_mismatches(root: &Path, artifact: &ArtifactMetadata) -> Result<Vec<PathBuf>> {
    match artifact.kind {
        ArtifactKind::Bom | ArtifactKind::Pnp | ArtifactKind::Drill => {}
        ArtifactKind::GerberSet | ArtifactKind::ManufacturingSet => return Ok(Vec::new()),
    }
    let Some(output_dir) = artifact.output_dir.as_ref() else {
        return Ok(artifact
            .files
            .iter()
            .map(|file| file.path.clone())
            .collect());
    };
    let effective_output_dir = if output_dir.is_absolute() {
        output_dir.clone()
    } else {
        root.join(output_dir)
    };
    let mut mismatches = Vec::new();
    for file in &artifact.files {
        if !artifact_file_path_is_safe(&file.path) {
            mismatches.push(file.path.clone());
            continue;
        }
        let artifact_path = effective_output_dir.join(&file.path);
        let matches = match artifact.kind {
            ArtifactKind::Bom if file.path.to_string_lossy().ends_with("-bom.csv") => {
                let report = compare_native_project_bom(root, &artifact_path, None)?;
                report.missing_count == 0 && report.extra_count == 0 && report.drift_count == 0
            }
            ArtifactKind::Pnp if file.path.to_string_lossy().ends_with("-pnp.csv") => {
                let report = compare_native_project_pnp(root, &artifact_path, None)?;
                report.missing_count == 0 && report.extra_count == 0 && report.drift_count == 0
            }
            ArtifactKind::Drill if file.path.to_string_lossy().ends_with("-drill.csv") => {
                let expected_csv = render_expected_native_project_drill_csv(root)?;
                let actual_csv = std::fs::read_to_string(&artifact_path)
                    .with_context(|| format!("failed to read {}", artifact_path.display()))?;
                actual_csv == expected_csv
            }
            ArtifactKind::Drill if file.path.to_string_lossy().ends_with("-drill.drl") => {
                let report = compare_native_project_excellon_drill(root, &artifact_path)?;
                report.missing_count == 0 && report.extra_count == 0 && report.hit_drift_count == 0
            }
            _ => false,
        };
        if !matches {
            mismatches.push(file.path.clone());
        }
    }
    Ok(mismatches)
}

fn artifact_file_path_is_safe(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn artifact_validation_should_persist(kind: ArtifactKind) -> bool {
    matches!(
        kind,
        ArtifactKind::Bom | ArtifactKind::Pnp | ArtifactKind::Drill
    )
}

fn artifact_validation_state_label(state: ArtifactValidationState) -> &'static str {
    match state {
        ArtifactValidationState::NotValidated => "not_validated",
        ArtifactValidationState::Valid => "valid",
        ArtifactValidationState::Invalid => "invalid",
    }
}
