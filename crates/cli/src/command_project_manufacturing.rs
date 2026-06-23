use std::path::Path;

use anyhow::{Context, Result};
use eda_engine::substrate::{
    ArtifactKind, ArtifactProductionProjection, DesignModel, OutputJobLogEntry, OutputJobLogLevel,
    OutputJobRun, OutputJobRunStatus, ProjectResolver, persist_artifact_metadata,
    persist_output_job_run,
};
use std::collections::BTreeSet;
use uuid::Uuid;

#[path = "command_project_check_gate.rs"]
mod command_project_direct_export_gate;
#[path = "command_project_manufacturing_match.rs"]
mod command_project_manufacturing_match;
#[path = "command_project_manufacturing_panel_projection.rs"]
mod command_project_manufacturing_panel_projection;
#[path = "command_project_manufacturing_scope.rs"]
mod command_project_manufacturing_scope;

use crate::{
    NativeProjectManufacturingComparisonView, NativeProjectManufacturingExportView,
    NativeProjectManufacturingInspectionEntryView, NativeProjectManufacturingInspectionView,
    NativeProjectManufacturingManifestView, NativeProjectManufacturingReportView,
    NativeProjectManufacturingValidationView,
};

use self::command_project_direct_export_gate::ensure_release_check_gate_clear;
pub(crate) use self::command_project_manufacturing_scope::{
    NativeProjectManufacturingScope,
    native_project_manufacturing_projection as project_manufacturing_projection,
};
use self::command_project_manufacturing_scope::{
    manufacturing_set_artifact_metadata, native_project_manufacturing_projection,
    push_manufacturing_artifact_view, resolve_native_project_manufacturing_scope,
    update_manufacturing_set_artifact_validation,
};
use super::command_project_gerber_plan::{
    append_production_projection_log_entries, artifact_production_projection_from_view,
    terminal_origin_log_entries_from, terminal_origin_provenance_from,
};
use super::{
    NativeProjectGerberPlanArtifactView, ensure_native_project_gerber_set_output_job,
    ensure_native_project_manufacturing_set_output_job, export_native_project_bom,
    export_native_project_drill, export_native_project_excellon_drill,
    export_native_project_gerber_set_from_plan, export_native_project_panel_drill,
    export_native_project_panel_excellon_drill, export_native_project_panel_pnp,
    export_native_project_pnp, load_native_project_with_resolved_board,
    next_output_job_run_sequence, query_native_project_board_components,
    query_native_project_board_vias, report_native_project_drill_hole_classes,
};
use command_project_manufacturing_match::manufacturing_artifact_matches;
use command_project_manufacturing_panel_projection::{
    panel_drill_csv_production_projection, panel_pnp_production_projection,
};

pub(crate) fn report_native_project_manufacturing(
    root: &Path,
    prefix_override: Option<&str>,
) -> Result<NativeProjectManufacturingReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let component_count = query_native_project_board_components(root)?.len();
    let via_count = query_native_project_board_vias(root)?.len();
    let scope =
        resolve_native_project_manufacturing_scope(root, prefix_override, None, None, None, None)?;
    let projection = native_project_manufacturing_projection(root, &scope)?;
    let gerber_plan = projection
        .gerber_plan
        .context("manufacturing report projection missing Gerber plan")?;
    let drill_report = report_native_project_drill_hole_classes(root)?;
    let gerber_artifacts = gerber_plan
        .artifacts
        .into_iter()
        .map(|artifact| NativeProjectGerberPlanArtifactView {
            kind: artifact.kind,
            layer_id: artifact.layer_id,
            layer_name: artifact.layer_name,
            filename: artifact.filename,
        })
        .collect::<Vec<_>>();

    Ok(NativeProjectManufacturingReportView {
        action: "report_manufacturing".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        prefix: gerber_plan.prefix,
        bom_component_count: component_count,
        pnp_component_count: component_count,
        drill_csv_row_count: via_count,
        excellon_via_count: drill_report.via_count,
        excellon_component_pad_count: drill_report.component_pad_count,
        excellon_hit_count: drill_report.hit_count,
        drill_hole_class_count: drill_report.class_count,
        gerber_artifact_count: gerber_artifacts.len(),
        gerber_artifacts,
    })
}

pub(crate) fn manifest_native_project_manufacturing_set(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
    include_override: Option<&str>,
    output_job_id: Option<Uuid>,
    output_job_name: Option<&str>,
) -> Result<NativeProjectManufacturingManifestView> {
    let project = load_native_project_with_resolved_board(root)?;
    let scope = resolve_native_project_manufacturing_scope(
        root,
        prefix_override,
        None,
        include_override,
        output_job_id,
        output_job_name,
    )?;
    let projection = native_project_manufacturing_projection(root, &scope)?;
    Ok(NativeProjectManufacturingManifestView {
        action: "manifest_manufacturing_set".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        output_dir: output_dir.display().to_string(),
        prefix: scope.prefix,
        expected_count: projection.entries.len(),
        entries: projection.entries,
    })
}

pub(crate) fn inspect_native_project_manufacturing_set(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
    include_override: Option<&str>,
    output_job_id: Option<Uuid>,
    output_job_name: Option<&str>,
) -> Result<NativeProjectManufacturingInspectionView> {
    let project = load_native_project_with_resolved_board(root)?;
    let scope = resolve_native_project_manufacturing_scope(
        root,
        prefix_override,
        None,
        include_override,
        output_job_id,
        output_job_name,
    )?;
    let projection = native_project_manufacturing_projection(root, &scope)?;
    let entries = projection.entries;
    let expected = entries
        .iter()
        .map(|entry| entry.filename.clone())
        .collect::<Vec<_>>();
    let expected_set = expected.iter().cloned().collect::<BTreeSet<_>>();

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

    let inspection_entries = entries
        .into_iter()
        .map(|entry| NativeProjectManufacturingInspectionEntryView {
            present: present.contains(&entry.filename),
            kind: entry.kind,
            filename: entry.filename,
            contract: entry.contract,
        })
        .collect::<Vec<_>>();
    let present_count = inspection_entries
        .iter()
        .filter(|entry| entry.present)
        .count();
    let missing_count = inspection_entries.len() - present_count;
    let extra = present
        .into_iter()
        .filter(|filename| !expected_set.contains(filename))
        .collect::<Vec<_>>();

    Ok(NativeProjectManufacturingInspectionView {
        action: "inspect_manufacturing_set".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        output_dir: output_dir.display().to_string(),
        prefix: scope.prefix,
        expected_count: inspection_entries.len(),
        present_count,
        missing_count,
        extra_count: extra.len(),
        entries: inspection_entries,
        extra,
    })
}

pub(crate) fn export_native_project_manufacturing_set(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
    variant: Option<Uuid>,
    include_override: Option<&str>,
    output_job_id: Option<Uuid>,
    output_job_name: Option<&str>,
) -> Result<NativeProjectManufacturingExportView> {
    export_native_project_manufacturing_set_with_output_run(
        root,
        output_dir,
        prefix_override,
        variant,
        include_override,
        output_job_id,
        output_job_name,
        true,
    )
}

pub(crate) fn export_native_project_manufacturing_set_without_output_run(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
    variant: Option<Uuid>,
    include_override: Option<&str>,
    output_job_id: Option<Uuid>,
    output_job_name: Option<&str>,
) -> Result<NativeProjectManufacturingExportView> {
    export_native_project_manufacturing_set_with_output_run(
        root,
        output_dir,
        prefix_override,
        variant,
        include_override,
        output_job_id,
        output_job_name,
        false,
    )
}

fn export_native_project_manufacturing_set_with_output_run(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
    variant: Option<Uuid>,
    include_override: Option<&str>,
    output_job_id: Option<Uuid>,
    output_job_name: Option<&str>,
    persist_output_run: bool,
) -> Result<NativeProjectManufacturingExportView> {
    if persist_output_run {
        ensure_release_check_gate_clear(root)?;
    }
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;

    let project = load_native_project_with_resolved_board(root)?;
    let scope = resolve_native_project_manufacturing_scope(
        root,
        prefix_override,
        variant,
        include_override,
        output_job_id,
        output_job_name,
    )?;
    let projection = native_project_manufacturing_projection(root, &scope)?;
    let output_job_id = if let Some(output_job_id) = scope.output_job_id {
        output_job_id
    } else {
        let (output_job, _, _, _) = ensure_native_project_manufacturing_set_output_job(
            root,
            &scope.prefix,
            None,
            None,
            None,
            scope.variant,
        )?;
        output_job.id
    };
    if scope.include.contains(&ArtifactKind::ManufacturingSet)
        || scope.include.contains(&ArtifactKind::GerberSet)
    {
        ensure_native_project_gerber_set_output_job(root, &scope.prefix, None, None, None, None)?;
    }
    let model = ProjectResolver::new(root).resolve()?;

    let mut artifacts = Vec::new();
    let mut artifact_files = Vec::new();
    let mut production_projections = Vec::new();
    let mut bom_row_count = 0;
    let mut pnp_row_count = 0;
    let mut drill_csv_row_count = 0;
    let mut excellon_hit_count = 0;
    if projection.entries.iter().any(|entry| entry.kind == "bom") {
        let file_name = format!("{}-bom.csv", scope.prefix);
        bom_row_count =
            export_native_project_bom(root, &output_dir.join(&file_name), scope.variant)?.rows;
        push_manufacturing_artifact_view(
            output_dir,
            &mut artifacts,
            &mut artifact_files,
            "bom",
            &file_name,
        )?;
    }
    if projection.entries.iter().any(|entry| entry.kind == "pnp") {
        let file_name = format!("{}-pnp.csv", scope.prefix);
        pnp_row_count = if let Some(panel_projection) = &scope.panel_projection {
            let report = export_native_project_panel_pnp(
                root,
                &output_dir.join(&file_name),
                scope.variant,
                panel_projection,
            )?;
            production_projections.push(panel_pnp_production_projection(
                &model,
                scope.manufacturing_plan_id,
                scope.board_or_panel,
                panel_projection,
                report.rows,
            )?);
            report.rows
        } else {
            export_native_project_pnp(root, &output_dir.join(&file_name), scope.variant)?.rows
        };
        push_manufacturing_artifact_view(
            output_dir,
            &mut artifacts,
            &mut artifact_files,
            "pnp",
            &file_name,
        )?;
    }
    if projection
        .entries
        .iter()
        .any(|entry| entry.kind == "drill_csv" || entry.kind == "excellon_drill")
    {
        let drill_csv_name = format!("{}-drill.csv", scope.prefix);
        let excellon_name = format!("{}-drill.drl", scope.prefix);
        let (drill_report, excellon_report) =
            if let Some(panel_projection) = &scope.panel_projection {
                (
                    export_native_project_panel_drill(
                        root,
                        &output_dir.join(&drill_csv_name),
                        panel_projection,
                    )?,
                    export_native_project_panel_excellon_drill(
                        root,
                        &output_dir.join(&excellon_name),
                        panel_projection,
                    )?,
                )
            } else {
                (
                    export_native_project_drill(root, &output_dir.join(&drill_csv_name))?,
                    export_native_project_excellon_drill(root, &output_dir.join(&excellon_name))?,
                )
            };
        drill_csv_row_count = drill_report.rows;
        if let Some(panel_projection) = &scope.panel_projection {
            production_projections.push(panel_drill_csv_production_projection(
                &model,
                scope.manufacturing_plan_id,
                scope.board_or_panel,
                panel_projection,
                drill_report.rows,
            )?);
        }
        excellon_hit_count = excellon_report.hit_count;
        production_projections.push(artifact_production_projection_from_view(
            excellon_report.production_projection,
        ));
        push_manufacturing_artifact_view(
            output_dir,
            &mut artifacts,
            &mut artifact_files,
            "drill_csv",
            &drill_csv_name,
        )?;
        push_manufacturing_artifact_view(
            output_dir,
            &mut artifacts,
            &mut artifact_files,
            "excellon_drill",
            &excellon_name,
        )?;
    }
    if let Some(gerber_plan) = projection.gerber_plan {
        let gerber_report = export_native_project_gerber_set_from_plan(
            root,
            output_dir,
            gerber_plan,
            false,
            scope.panel_projection.as_ref(),
        )?;
        production_projections.extend(
            gerber_report
                .artifact_metadata
                .production_projections
                .clone(),
        );
        for artifact in gerber_report.artifacts {
            push_manufacturing_artifact_view(
                output_dir,
                &mut artifacts,
                &mut artifact_files,
                &format!("gerber_{}", artifact.kind),
                &artifact.filename,
            )?;
        }
    }
    let artifact_metadata = manufacturing_set_artifact_metadata(
        &model,
        &scope.prefix,
        Some(output_job_id),
        scope.variant,
        output_dir,
        artifact_files,
        production_projections,
    );
    let artifact_manifest_path = persist_artifact_metadata(root, &artifact_metadata)
        .context("failed to persist manufacturing artifact metadata")?;
    let (output_job_run, output_job_run_path) = if persist_output_run {
        let run = manufacturing_set_output_job_run(
            &model,
            output_job_id,
            artifact_metadata.artifact_id,
            artifacts.len(),
            &artifact_metadata.production_projections,
        );
        let path = persist_output_job_run(root, &run)
            .context("failed to persist manufacturing output job run")?;
        (Some(run), Some(path.display().to_string()))
    } else {
        (None, None)
    };

    Ok(NativeProjectManufacturingExportView {
        action: "export_manufacturing_set".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        output_dir: output_dir.display().to_string(),
        prefix: scope.prefix,
        bom_row_count,
        pnp_row_count,
        drill_csv_row_count,
        excellon_hit_count,
        gerber_artifact_count: artifacts
            .iter()
            .filter(|artifact| artifact.kind.starts_with("gerber_"))
            .count(),
        written_count: artifacts.len(),
        artifact_manifest_path: artifact_manifest_path.display().to_string(),
        artifact_metadata,
        output_job_run,
        output_job_run_path,
        artifacts,
    })
}

pub(crate) fn validate_native_project_manufacturing_set(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
    variant: Option<Uuid>,
    include_override: Option<&str>,
    output_job_id: Option<Uuid>,
    output_job_name: Option<&str>,
) -> Result<NativeProjectManufacturingValidationView> {
    let project = load_native_project_with_resolved_board(root)?;
    let scope = resolve_native_project_manufacturing_scope(
        root,
        prefix_override,
        variant,
        include_override,
        output_job_id,
        output_job_name,
    )?;
    let projection = native_project_manufacturing_projection(root, &scope)?;
    let gerber_plan = projection.gerber_plan;
    let entries = projection.entries;
    let expected = entries
        .iter()
        .map(|entry| entry.filename.clone())
        .collect::<Vec<_>>();
    let expected_set = expected.iter().cloned().collect::<BTreeSet<_>>();

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

    for filename in &expected {
        let artifact_path = output_dir.join(filename);
        if !artifact_path.is_file() {
            missing.push(filename.clone());
            continue;
        }
        let is_match = manufacturing_artifact_matches(
            root,
            &artifact_path,
            filename,
            scope.variant,
            scope.panel_projection.as_ref(),
            gerber_plan.as_ref(),
        )?;
        if is_match {
            matched.push(filename.clone());
        } else {
            mismatched.push(filename.clone());
        }
    }

    let extra = present
        .difference(&expected_set)
        .cloned()
        .collect::<Vec<_>>();
    let artifact_validation = update_manufacturing_set_artifact_validation(
        root,
        output_dir,
        &expected,
        missing.is_empty() && mismatched.is_empty() && extra.is_empty(),
    )?;

    Ok(NativeProjectManufacturingValidationView {
        action: "validate_manufacturing_set".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        output_dir: output_dir.display().to_string(),
        prefix: scope.prefix,
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

pub(crate) fn compare_native_project_manufacturing_set(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
    variant: Option<Uuid>,
    include_override: Option<&str>,
    output_job_id: Option<Uuid>,
    output_job_name: Option<&str>,
) -> Result<NativeProjectManufacturingComparisonView> {
    let project = load_native_project_with_resolved_board(root)?;
    let scope = resolve_native_project_manufacturing_scope(
        root,
        prefix_override,
        variant,
        include_override,
        output_job_id,
        output_job_name,
    )?;
    let projection = native_project_manufacturing_projection(root, &scope)?;
    let entries = projection.entries;
    let expected = entries
        .iter()
        .map(|entry| entry.filename.clone())
        .collect::<Vec<_>>();
    let expected_set = expected.iter().cloned().collect::<BTreeSet<_>>();

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

    for filename in &expected {
        let artifact_path = output_dir.join(filename);
        if !artifact_path.is_file() {
            missing.push(filename.clone());
            continue;
        }
        let is_match = manufacturing_artifact_matches(
            root,
            &artifact_path,
            filename,
            scope.variant,
            scope.panel_projection.as_ref(),
            projection.gerber_plan.as_ref(),
        )?;
        if is_match {
            matched.push(filename.clone());
        } else {
            mismatched.push(filename.clone());
        }
    }

    let extra = present
        .difference(&expected_set)
        .cloned()
        .collect::<Vec<_>>();

    Ok(NativeProjectManufacturingComparisonView {
        action: "compare_manufacturing_set".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        output_dir: output_dir.display().to_string(),
        prefix: scope.prefix,
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

fn manufacturing_set_output_job_run(
    model: &DesignModel,
    output_job: Uuid,
    artifact_id: Uuid,
    artifact_count: usize,
    production_projections: &[ArtifactProductionProjection],
) -> OutputJobRun {
    let run_sequence = next_output_job_run_sequence(model, output_job);
    let material = format!(
        "datum-eda:output-job-run:manufacturing-set:{}:{}:{}:{run_sequence}",
        output_job, model.model_revision.0, artifact_id
    );
    let env = std::env::vars().collect::<std::collections::BTreeMap<_, _>>();
    let mut log = vec![OutputJobLogEntry {
        sequence: 1,
        level: OutputJobLogLevel::Info,
        message: format!("generated manufacturing set with {artifact_count} artifact files"),
    }];
    log.extend(terminal_origin_log_entries_from(&env, 2));
    append_production_projection_log_entries(&mut log, production_projections);
    OutputJobRun {
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
