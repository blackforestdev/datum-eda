use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::{
    NativeProjectGerberSetArtifactView, NativeProjectGerberSetComparisonView,
    NativeProjectGerberSetExportView, NativeProjectGerberSetValidationView,
};
use anyhow::{Context, Result};
use eda_engine::board::StackupLayer;
use eda_engine::substrate::{
    ARTIFACT_METADATA_SCHEMA_VERSION, ArtifactFile, ArtifactKind, ArtifactMetadata,
    ArtifactProductionProjection, ArtifactValidationState, DesignModel,
    OUTPUT_JOB_RUN_SCHEMA_VERSION, OutputJobLogEntry, OutputJobLogLevel, OutputJobRun,
    OutputJobRunStatus, PanelProjection, ProjectResolver,
};
use uuid::Uuid;

#[path = "../check/gate.rs"]
mod command_project_direct_export_gate;
#[path = "evidence.rs"]
mod command_project_gerber_evidence;
#[path = "panel.rs"]
mod command_project_gerber_panel;
#[path = "../project/output_log.rs"]
mod command_project_output_log;

use self::command_project_direct_export_gate::ensure_release_check_gate_clear;
use self::command_project_gerber_evidence::{artifact_metadata_path, commit_gerber_set_evidence};
pub(crate) use self::command_project_gerber_panel::panelize_rs274x_gerber;
use self::command_project_gerber_panel::{
    panel_gerber_production_projection, panelize_and_rewrite_rs274x_gerber_file,
};
pub(crate) use self::command_project_output_log::{
    append_production_projection_log_entries, terminal_origin_log_entries_from,
    terminal_origin_provenance_from,
};
use crate::{
    NativeProjectGerberPlanArtifactView, NativeProjectGerberPlanComparisonView,
    NativeProjectGerberPlanView, StackupLayerType, compare_native_project_gerber_copper_layer,
    compare_native_project_gerber_mechanical_layer, compare_native_project_gerber_outline,
    compare_native_project_gerber_paste_layer, compare_native_project_gerber_silkscreen_layer,
    compare_native_project_gerber_soldermask_layer, compute_source_hash_bytes,
    ensure_native_project_gerber_set_output_job, export_native_project_gerber_copper_layer,
    export_native_project_gerber_mechanical_layer, export_native_project_gerber_outline,
    export_native_project_gerber_paste_layer, export_native_project_gerber_silkscreen_layer,
    export_native_project_gerber_soldermask_layer, load_native_project_with_resolved_board,
    next_output_job_run_sequence, validate_native_project_gerber_copper_layer,
    validate_native_project_gerber_mechanical_layer, validate_native_project_gerber_outline,
    validate_native_project_gerber_paste_layer, validate_native_project_gerber_silkscreen_layer,
    validate_native_project_gerber_soldermask_layer,
};

pub(crate) fn plan_native_project_gerber_export(
    root: &Path,
    prefix_override: Option<&str>,
) -> Result<NativeProjectGerberPlanView> {
    let project = load_native_project_with_resolved_board(root)?;
    let mut layers = project
        .board
        .stackup
        .layers
        .iter()
        .map(|value| {
            serde_json::from_value::<StackupLayer>(value.clone())
                .context("failed to parse board stackup layer")
        })
        .collect::<Result<Vec<_>>>()?;
    layers.sort_by(|a, b| a.id.cmp(&b.id).then_with(|| a.name.cmp(&b.name)));

    let prefix = sanitize_export_prefix(prefix_override.unwrap_or(&project.board.name));
    let mut artifacts = vec![NativeProjectGerberPlanArtifactView {
        kind: "outline".to_string(),
        layer_id: None,
        layer_name: None,
        filename: format!("{prefix}-outline.gbr"),
    }];

    let mut copper_layers = 0;
    let mut soldermask_layers = 0;
    let mut silkscreen_layers = 0;
    let mut paste_layers = 0;
    let mut mechanical_layers = 0;

    for layer in layers {
        let (kind, suffix, count_ref) = match layer.layer_type {
            StackupLayerType::Copper => ("copper", "copper", &mut copper_layers),
            StackupLayerType::SolderMask => ("soldermask", "mask", &mut soldermask_layers),
            StackupLayerType::Silkscreen => ("silkscreen", "silk", &mut silkscreen_layers),
            StackupLayerType::Paste => ("paste", "paste", &mut paste_layers),
            StackupLayerType::Mechanical => ("mechanical", "mech", &mut mechanical_layers),
            StackupLayerType::Dielectric => continue,
        };
        *count_ref += 1;
        let layer_slug = sanitize_export_prefix(&layer.name);
        artifacts.push(NativeProjectGerberPlanArtifactView {
            kind: kind.to_string(),
            layer_id: Some(layer.id),
            layer_name: Some(layer.name.clone()),
            filename: format!("{prefix}-l{}-{layer_slug}-{suffix}.gbr", layer.id),
        });
    }

    Ok(NativeProjectGerberPlanView {
        action: "plan_gerber_export".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        prefix,
        outline_vertex_count: project.board.outline.vertices.len(),
        outline_closed: project.board.outline.closed,
        copper_layers,
        soldermask_layers,
        silkscreen_layers,
        paste_layers,
        mechanical_layers,
        artifacts,
    })
}

pub(crate) fn export_native_project_gerber_set(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
) -> Result<NativeProjectGerberSetExportView> {
    let plan = plan_native_project_gerber_export(root, prefix_override)?;
    export_native_project_gerber_set_from_plan(root, output_dir, plan, true, None, None)
}

pub(crate) fn export_native_project_gerber_set_without_output_run(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
) -> Result<NativeProjectGerberSetExportView> {
    let plan = plan_native_project_gerber_export(root, prefix_override)?;
    export_native_project_gerber_set_from_plan(root, output_dir, plan, false, None, None)
}

pub(crate) fn export_native_project_gerber_set_without_output_run_for_output_job(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
    output_job: Uuid,
) -> Result<NativeProjectGerberSetExportView> {
    let plan = plan_native_project_gerber_export(root, prefix_override)?;
    export_native_project_gerber_set_from_plan(
        root,
        output_dir,
        plan,
        false,
        None,
        Some(output_job),
    )
}

pub(crate) fn export_native_project_gerber_set_from_plan(
    root: &Path,
    output_dir: &Path,
    plan: NativeProjectGerberPlanView,
    persist_output_run: bool,
    panel_projection: Option<&PanelProjection>,
    output_job_override: Option<Uuid>,
) -> Result<NativeProjectGerberSetExportView> {
    if persist_output_run {
        ensure_release_check_gate_clear(root)?;
    }
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;
    let output_job = if persist_output_run {
        let (output_job, _, _, _) = ensure_native_project_gerber_set_output_job(
            root,
            &plan.prefix,
            None,
            None,
            None,
            None,
        )?;
        Some(output_job)
    } else {
        None
    };
    let output_job_id = output_job
        .as_ref()
        .map(|job| job.id)
        .or(output_job_override);
    let mut model = ProjectResolver::new(root).resolve()?;
    let mut artifacts = Vec::new();
    let mut artifact_files = Vec::new();
    let mut production_projections = Vec::new();

    for artifact in plan.artifacts {
        let output_path = output_dir.join(&artifact.filename);
        match (artifact.kind.as_str(), artifact.layer_id) {
            ("outline", None) => {
                export_native_project_gerber_outline(root, &output_path)?;
            }
            ("copper", Some(layer)) => {
                let report = export_native_project_gerber_copper_layer(root, layer, &output_path)?;
                if panel_projection.is_none() {
                    production_projections.push(artifact_production_projection_from_view(
                        report.production_projection,
                    ));
                }
            }
            ("soldermask", Some(layer)) => {
                export_native_project_gerber_soldermask_layer(root, layer, &output_path)?;
            }
            ("silkscreen", Some(layer)) => {
                export_native_project_gerber_silkscreen_layer(root, layer, &output_path)?;
            }
            ("paste", Some(layer)) => {
                export_native_project_gerber_paste_layer(root, layer, &output_path)?;
            }
            ("mechanical", Some(layer)) => {
                export_native_project_gerber_mechanical_layer(root, layer, &output_path)?;
            }
            _ => anyhow::bail!("unsupported Gerber export plan artifact: {}", artifact.kind),
        }
        if let Some(panel_projection) = panel_projection {
            let panel_gerber =
                panelize_and_rewrite_rs274x_gerber_file(root, &output_path, panel_projection)?;
            if artifact.kind == "copper" {
                production_projections.push(panel_gerber_production_projection(
                    &model,
                    panel_gerber.as_bytes(),
                ));
            }
        }
        let bytes = std::fs::read(&output_path)
            .with_context(|| format!("failed to read {}", output_path.display()))?;
        let sha256 = compute_source_hash_bytes(&bytes);
        artifact_files.push(ArtifactFile {
            path: PathBuf::from(&artifact.filename),
            sha256: sha256.clone(),
        });
        artifacts.push(NativeProjectGerberSetArtifactView {
            kind: artifact.kind,
            layer_id: artifact.layer_id,
            layer_name: artifact.layer_name,
            filename: artifact.filename,
            output_path: output_path.display().to_string(),
            sha256,
        });
    }
    let artifact_metadata = gerber_set_artifact_metadata(
        &model,
        &plan.prefix,
        output_job_id,
        output_dir,
        artifact_files,
        production_projections,
    );
    let (output_job_run, output_job_run_path) = if persist_output_run {
        let output_job = output_job.expect("persisted Gerber export should have an OutputJob");
        let run = gerber_set_output_job_run(
            &model,
            output_job.id,
            artifact_metadata.artifact_id,
            artifacts.len(),
            &artifact_metadata.production_projections,
        );
        let (_, path) =
            commit_gerber_set_evidence(root, &mut model, &artifact_metadata, Some(&run))
                .context("failed to persist Gerber set generated evidence")?;
        (Some(run), Some(path.display().to_string()))
    } else {
        commit_gerber_set_evidence(root, &mut model, &artifact_metadata, None)
            .context("failed to persist Gerber set artifact metadata")?;
        (None, None)
    };
    let artifact_manifest_path = artifact_metadata_path(root, artifact_metadata.artifact_id);

    Ok(NativeProjectGerberSetExportView {
        action: "export_gerber_set".to_string(),
        project_root: plan.project_root,
        output_dir: output_dir.display().to_string(),
        prefix: plan.prefix,
        written_count: artifacts.len(),
        artifact_manifest_path: artifact_manifest_path.display().to_string(),
        artifact_metadata,
        output_job_run,
        output_job_run_path,
        artifacts,
    })
}

pub(crate) fn compare_native_project_gerber_export_plan(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
) -> Result<NativeProjectGerberPlanComparisonView> {
    let plan = plan_native_project_gerber_export(root, prefix_override)?;
    let expected = plan
        .artifacts
        .iter()
        .map(|artifact| artifact.filename.clone())
        .collect::<BTreeSet<_>>();

    let mut present = BTreeSet::new();
    for entry in std::fs::read_dir(output_dir)
        .with_context(|| format!("failed to read {}", output_dir.display()))?
    {
        let entry = entry.with_context(|| format!("failed to read {}", output_dir.display()))?;
        if entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", entry.path().display()))?
            .is_file()
        {
            present.insert(entry.file_name().to_string_lossy().into_owned());
        }
    }

    let matched = expected.intersection(&present).cloned().collect::<Vec<_>>();
    let missing = expected.difference(&present).cloned().collect::<Vec<_>>();
    let extra = present.difference(&expected).cloned().collect::<Vec<_>>();

    Ok(NativeProjectGerberPlanComparisonView {
        action: "compare_gerber_export_plan".to_string(),
        project_root: plan.project_root,
        output_dir: output_dir.display().to_string(),
        prefix: plan.prefix,
        expected_count: expected.len(),
        present_count: present.len(),
        missing_count: missing.len(),
        extra_count: extra.len(),
        matched,
        missing,
        extra,
    })
}

pub(crate) fn validate_native_project_gerber_set(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
) -> Result<NativeProjectGerberSetValidationView> {
    let plan = plan_native_project_gerber_export(root, prefix_override)?;
    let expected = plan
        .artifacts
        .iter()
        .map(|artifact| artifact.filename.clone())
        .collect::<BTreeSet<_>>();

    let mut present = BTreeSet::new();
    for entry in std::fs::read_dir(output_dir)
        .with_context(|| format!("failed to read {}", output_dir.display()))?
    {
        let entry = entry.with_context(|| format!("failed to read {}", output_dir.display()))?;
        if entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", entry.path().display()))?
            .is_file()
        {
            present.insert(entry.file_name().to_string_lossy().into_owned());
        }
    }

    let mut matched = Vec::new();
    let mut missing = Vec::new();
    let mut mismatched = Vec::new();

    for artifact in &plan.artifacts {
        let gerber_path = output_dir.join(&artifact.filename);
        if !gerber_path.is_file() {
            missing.push(artifact.filename.clone());
            continue;
        }
        let is_match = match (artifact.kind.as_str(), artifact.layer_id) {
            ("outline", None) => {
                validate_native_project_gerber_outline(root, &gerber_path)?.matches_expected
            }
            ("copper", Some(layer)) => {
                validate_native_project_gerber_copper_layer(root, layer, &gerber_path)?
                    .matches_expected
            }
            ("soldermask", Some(layer)) => {
                validate_native_project_gerber_soldermask_layer(root, layer, &gerber_path)?
                    .matches_expected
            }
            ("silkscreen", Some(layer)) => {
                validate_native_project_gerber_silkscreen_layer(root, layer, &gerber_path)?
                    .matches_expected
            }
            ("paste", Some(layer)) => {
                validate_native_project_gerber_paste_layer(root, layer, &gerber_path)?
                    .matches_expected
            }
            ("mechanical", Some(layer)) => {
                validate_native_project_gerber_mechanical_layer(root, layer, &gerber_path)?
                    .matches_expected
            }
            _ => anyhow::bail!("unsupported Gerber export plan artifact: {}", artifact.kind),
        };
        if is_match {
            matched.push(artifact.filename.clone());
        } else {
            mismatched.push(artifact.filename.clone());
        }
    }

    let extra = present.difference(&expected).cloned().collect::<Vec<_>>();
    let artifact_validation = update_gerber_set_artifact_validation(
        root,
        output_dir,
        &plan,
        missing.is_empty() && mismatched.is_empty() && extra.is_empty(),
    )?;

    Ok(NativeProjectGerberSetValidationView {
        action: "validate_gerber_set".to_string(),
        project_root: plan.project_root,
        output_dir: output_dir.display().to_string(),
        prefix: plan.prefix,
        expected_count: expected.len(),
        matched_count: matched.len(),
        missing_count: missing.len(),
        mismatched_count: mismatched.len(),
        extra_count: extra.len(),
        artifact_id: artifact_validation
            .as_ref()
            .map(|validation| validation.artifact_id.to_string()),
        artifact_manifest_path: artifact_validation
            .as_ref()
            .map(|validation| validation.manifest_path.display().to_string()),
        artifact_validation_state: artifact_validation
            .as_ref()
            .map(|validation| validation.validation_state.to_string()),
        artifact_file_hash_mismatch_count: artifact_validation
            .as_ref()
            .map(|validation| validation.file_hash_mismatch_count)
            .unwrap_or(0),
        matched,
        missing,
        mismatched,
        extra,
    })
}

pub(crate) fn compare_native_project_gerber_set(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
) -> Result<NativeProjectGerberSetComparisonView> {
    let plan = plan_native_project_gerber_export(root, prefix_override)?;
    let expected = plan
        .artifacts
        .iter()
        .map(|artifact| artifact.filename.clone())
        .collect::<BTreeSet<_>>();

    let mut present = BTreeSet::new();
    for entry in std::fs::read_dir(output_dir)
        .with_context(|| format!("failed to read {}", output_dir.display()))?
    {
        let entry = entry.with_context(|| format!("failed to read {}", output_dir.display()))?;
        if entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", entry.path().display()))?
            .is_file()
        {
            present.insert(entry.file_name().to_string_lossy().into_owned());
        }
    }

    let mut matched = Vec::new();
    let mut missing = Vec::new();
    let mut mismatched = Vec::new();

    for artifact in &plan.artifacts {
        let gerber_path = output_dir.join(&artifact.filename);
        if !gerber_path.is_file() {
            missing.push(artifact.filename.clone());
            continue;
        }
        let is_match = match (artifact.kind.as_str(), artifact.layer_id) {
            ("outline", None) => {
                let report = compare_native_project_gerber_outline(root, &gerber_path)?;
                report.missing_count == 0 && report.extra_count == 0
            }
            ("copper", Some(layer)) => {
                let report = compare_native_project_gerber_copper_layer(root, layer, &gerber_path)?;
                report.missing_count == 0 && report.extra_count == 0
            }
            ("soldermask", Some(layer)) => {
                let report =
                    compare_native_project_gerber_soldermask_layer(root, layer, &gerber_path)?;
                report.missing_count == 0 && report.extra_count == 0
            }
            ("silkscreen", Some(layer)) => {
                let report =
                    compare_native_project_gerber_silkscreen_layer(root, layer, &gerber_path)?;
                report.missing_count == 0 && report.extra_count == 0
            }
            ("paste", Some(layer)) => {
                let report = compare_native_project_gerber_paste_layer(root, layer, &gerber_path)?;
                report.missing_count == 0 && report.extra_count == 0
            }
            ("mechanical", Some(layer)) => {
                let report =
                    compare_native_project_gerber_mechanical_layer(root, layer, &gerber_path)?;
                report.missing_count == 0 && report.extra_count == 0
            }
            _ => anyhow::bail!("unsupported Gerber export plan artifact: {}", artifact.kind),
        };
        if is_match {
            matched.push(artifact.filename.clone());
        } else {
            mismatched.push(artifact.filename.clone());
        }
    }

    let extra = present.difference(&expected).cloned().collect::<Vec<_>>();

    Ok(NativeProjectGerberSetComparisonView {
        action: "compare_gerber_set".to_string(),
        project_root: plan.project_root,
        output_dir: output_dir.display().to_string(),
        prefix: plan.prefix,
        expected_count: expected.len(),
        matched_count: matched.len(),
        missing_count: missing.len(),
        mismatched_count: mismatched.len(),
        extra_count: extra.len(),
        matched,
        missing,
        mismatched,
        extra,
    })
}

pub(crate) fn sanitize_export_prefix(value: &str) -> String {
    let mut prefix = String::new();
    let mut last_dash = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            prefix.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            prefix.push('-');
            last_dash = true;
        }
    }
    prefix.trim_matches('-').to_string()
}

fn gerber_set_artifact_metadata(
    model: &DesignModel,
    prefix: &str,
    output_job: Option<Uuid>,
    output_dir: &Path,
    files: Vec<ArtifactFile>,
    production_projections: Vec<ArtifactProductionProjection>,
) -> ArtifactMetadata {
    let mut material = format!(
        "datum-eda:artifact:gerber-set:{}:{}:{}:{}",
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
        kind: ArtifactKind::GerberSet,
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

pub(crate) fn artifact_production_projection_from_view(
    view: crate::NativeProjectProductionProjectionView,
) -> ArtifactProductionProjection {
    ArtifactProductionProjection {
        projection_kind: view.projection_kind,
        projection_contract: view.projection_contract,
        model_revision: eda_engine::substrate::ModelRevision(view.model_revision),
        byte_count: view.byte_count,
        sha256: view.sha256,
    }
}

fn gerber_set_output_job_run(
    model: &DesignModel,
    output_job: Uuid,
    artifact_id: Uuid,
    artifact_count: usize,
    production_projections: &[ArtifactProductionProjection],
) -> OutputJobRun {
    let run_sequence = next_output_job_run_sequence(model, output_job);
    let material = format!(
        "datum-eda:output-job-run:gerber-set:{}:{}:{}:{run_sequence}",
        output_job, model.model_revision.0, artifact_id
    );
    let env = std::env::vars().collect::<std::collections::BTreeMap<_, _>>();
    let mut log = vec![OutputJobLogEntry {
        sequence: 1,
        level: OutputJobLogLevel::Info,
        message: format!("generated Gerber set with {artifact_count} artifact files"),
    }];
    log.extend(terminal_origin_log_entries_from(&env, 2));
    append_production_projection_log_entries(&mut log, production_projections);
    OutputJobRun {
        schema_version: OUTPUT_JOB_RUN_SCHEMA_VERSION,
        run_id: Uuid::new_v5(&model.project.project_id, material.as_bytes()),
        output_job,
        run_sequence,
        project_id: model.project.project_id,
        model_revision: model.model_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
        artifact_id: Some(artifact_id),
        exit_code: Some(0),
        provenance: terminal_origin_provenance_from(&env),
        log,
    }
}

struct GerberSetArtifactValidation {
    artifact_id: Uuid,
    manifest_path: PathBuf,
    validation_state: &'static str,
    file_hash_mismatch_count: usize,
}

fn update_gerber_set_artifact_validation(
    root: &Path,
    output_dir: &Path,
    plan: &NativeProjectGerberPlanView,
    geometry_matches: bool,
) -> Result<Option<GerberSetArtifactValidation>> {
    let mut model = ProjectResolver::new(root).resolve()?;
    let expected_files = plan
        .artifacts
        .iter()
        .map(|artifact| PathBuf::from(&artifact.filename))
        .collect::<BTreeSet<_>>();
    let Some(mut metadata) = model
        .artifact_metadata
        .values()
        .find(|metadata| {
            metadata.kind == ArtifactKind::GerberSet
                && metadata.model_revision == model.model_revision
                && metadata
                    .files
                    .iter()
                    .map(|file| file.path.clone())
                    .collect::<BTreeSet<_>>()
                    == expected_files
        })
        .cloned()
    else {
        return Ok(None);
    };

    let mut file_hash_mismatch_count = 0usize;
    for file in &metadata.files {
        let path = output_dir.join(&file.path);
        let Ok(bytes) = std::fs::read(&path) else {
            file_hash_mismatch_count += 1;
            continue;
        };
        if compute_source_hash_bytes(&bytes) != file.sha256 {
            file_hash_mismatch_count += 1;
        }
    }
    metadata.validation_state = if geometry_matches && file_hash_mismatch_count == 0 {
        ArtifactValidationState::Valid
    } else {
        ArtifactValidationState::Invalid
    };
    let manifest_path = commit_gerber_set_evidence(root, &mut model, &metadata, None)
        .context("failed to persist artifact metadata")?
        .0;
    let validation_state = match metadata.validation_state {
        ArtifactValidationState::NotValidated => "not_validated",
        ArtifactValidationState::Valid => "valid",
        ArtifactValidationState::Invalid => "invalid",
    };
    Ok(Some(GerberSetArtifactValidation {
        artifact_id: metadata.artifact_id,
        manifest_path,
        validation_state,
        file_hash_mismatch_count,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use eda_engine::substrate::OutputJobRunLauncher;

    #[test]
    fn terminal_origin_log_entries_capture_gui_terminal_context() {
        let env = [
            (
                "DATUM_PROJECT_ROOT".to_string(),
                "/tmp/datum/project".to_string(),
            ),
            (
                "DATUM_SOURCE_REVISION".to_string(),
                "source-revision-test".to_string(),
            ),
            (
                "DATUM_TERMINAL_CONTEXT".to_string(),
                "/tmp/datum/context.json".to_string(),
            ),
            (
                "DATUM_TERMINAL_SESSION_ID".to_string(),
                "terminal-test".to_string(),
            ),
        ]
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
        let entries = terminal_origin_log_entries_from(&env, 7);
        let provenance = terminal_origin_provenance_from(&env).expect("terminal provenance");
        assert_eq!(provenance.launcher, OutputJobRunLauncher::GuiTerminal);
        assert_eq!(
            provenance.terminal_session_id.as_deref(),
            Some("terminal-test")
        );
        assert_eq!(
            provenance.terminal_context_path,
            Some(PathBuf::from("/tmp/datum/context.json"))
        );
        assert_eq!(
            provenance.project_root,
            Some(PathBuf::from("/tmp/datum/project"))
        );
        assert_eq!(
            provenance.source_revision.as_deref(),
            Some("source-revision-test")
        );
        let non_terminal_env = [
            (
                "DATUM_PROJECT_ROOT".to_string(),
                "/tmp/datum/project".to_string(),
            ),
            (
                "DATUM_SOURCE_REVISION".to_string(),
                "source-revision-test".to_string(),
            ),
        ]
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(terminal_origin_provenance_from(&non_terminal_env), None);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].sequence, 7);
        assert_eq!(
            entries[0].message,
            "launched from Datum terminal session terminal-test"
        );
        assert_eq!(entries[1].sequence, 8);
        assert_eq!(
            entries[1].message,
            "Datum terminal context /tmp/datum/context.json"
        );
    }
}
