use std::path::Path;

use anyhow::{Context, Result};
use std::collections::BTreeSet;

use crate::{
    NativeProjectManufacturingArtifactView, NativeProjectManufacturingComparisonView,
    NativeProjectManufacturingExportView, NativeProjectManufacturingManifestEntryView,
    NativeProjectManufacturingManifestView, NativeProjectManufacturingReportView,
    NativeProjectManufacturingValidationView,
};

use super::command_project_gerber_plan::sanitize_export_prefix;
use super::{
    NativeProjectGerberPlanArtifactView, compare_native_project_bom,
    compare_native_project_excellon_drill, compare_native_project_gerber_set,
    compare_native_project_pnp, export_native_project_bom, export_native_project_drill,
    export_native_project_excellon_drill, export_native_project_gerber_set,
    export_native_project_pnp, load_native_project, plan_native_project_gerber_export,
    query_native_project_board_components, query_native_project_board_vias,
    render_expected_native_project_drill_csv, report_native_project_drill_hole_classes,
    validate_native_project_excellon_drill, validate_native_project_gerber_copper_layer,
    validate_native_project_gerber_mechanical_layer, validate_native_project_gerber_outline,
    validate_native_project_gerber_paste_layer, validate_native_project_gerber_silkscreen_layer,
    validate_native_project_gerber_soldermask_layer,
};

pub(crate) fn report_native_project_manufacturing(
    root: &Path,
    prefix_override: Option<&str>,
) -> Result<NativeProjectManufacturingReportView> {
    let project = load_native_project(root)?;
    let component_count = query_native_project_board_components(root)?.len();
    let via_count = query_native_project_board_vias(root)?.len();
    let gerber_plan = plan_native_project_gerber_export(root, prefix_override)?;
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

fn native_project_manufacturing_manifest_entries(
    root: &Path,
    prefix_override: Option<&str>,
) -> Result<(String, Vec<NativeProjectManufacturingManifestEntryView>)> {
    let project = load_native_project(root)?;
    let prefix = sanitize_export_prefix(prefix_override.unwrap_or(&project.board.name));
    let gerber_plan = plan_native_project_gerber_export(root, Some(&prefix))?;

    let mut entries = vec![
        NativeProjectManufacturingManifestEntryView {
            kind: "bom".to_string(),
            filename: format!("{prefix}-bom.csv"),
            contract: "semantic".to_string(),
        },
        NativeProjectManufacturingManifestEntryView {
            kind: "pnp".to_string(),
            filename: format!("{prefix}-pnp.csv"),
            contract: "semantic".to_string(),
        },
        NativeProjectManufacturingManifestEntryView {
            kind: "drill_csv".to_string(),
            filename: format!("{prefix}-drill.csv"),
            contract: "strict".to_string(),
        },
        NativeProjectManufacturingManifestEntryView {
            kind: "excellon_drill".to_string(),
            filename: format!("{prefix}-drill.drl"),
            contract: "semantic".to_string(),
        },
    ];
    entries.extend(gerber_plan.artifacts.into_iter().map(|artifact| {
        NativeProjectManufacturingManifestEntryView {
            kind: format!("gerber_{}", artifact.kind),
            filename: artifact.filename,
            contract: "semantic".to_string(),
        }
    }));
    Ok((prefix, entries))
}

pub(crate) fn manifest_native_project_manufacturing_set(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
) -> Result<NativeProjectManufacturingManifestView> {
    let project = load_native_project(root)?;
    let (prefix, entries) = native_project_manufacturing_manifest_entries(root, prefix_override)?;
    Ok(NativeProjectManufacturingManifestView {
        action: "manifest_manufacturing_set".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        output_dir: output_dir.display().to_string(),
        prefix,
        expected_count: entries.len(),
        entries,
    })
}

pub(crate) fn export_native_project_manufacturing_set(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
) -> Result<NativeProjectManufacturingExportView> {
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;

    let project = load_native_project(root)?;
    let prefix = sanitize_export_prefix(prefix_override.unwrap_or(&project.board.name));
    let bom_path = output_dir.join(format!("{prefix}-bom.csv"));
    let pnp_path = output_dir.join(format!("{prefix}-pnp.csv"));
    let drill_csv_path = output_dir.join(format!("{prefix}-drill.csv"));
    let excellon_path = output_dir.join(format!("{prefix}-drill.drl"));

    let bom_report = export_native_project_bom(root, &bom_path)?;
    let pnp_report = export_native_project_pnp(root, &pnp_path)?;
    let drill_csv_report = export_native_project_drill(root, &drill_csv_path)?;
    let excellon_report = export_native_project_excellon_drill(root, &excellon_path)?;
    let gerber_report = export_native_project_gerber_set(root, output_dir, Some(&prefix))?;

    let mut artifacts = vec![
        NativeProjectManufacturingArtifactView {
            kind: "bom".to_string(),
            output_path: bom_path.display().to_string(),
        },
        NativeProjectManufacturingArtifactView {
            kind: "pnp".to_string(),
            output_path: pnp_path.display().to_string(),
        },
        NativeProjectManufacturingArtifactView {
            kind: "drill_csv".to_string(),
            output_path: drill_csv_path.display().to_string(),
        },
        NativeProjectManufacturingArtifactView {
            kind: "excellon_drill".to_string(),
            output_path: excellon_path.display().to_string(),
        },
    ];
    artifacts.extend(gerber_report.artifacts.into_iter().map(|artifact| {
        NativeProjectManufacturingArtifactView {
            kind: format!("gerber_{}", artifact.kind),
            output_path: artifact.output_path,
        }
    }));

    Ok(NativeProjectManufacturingExportView {
        action: "export_manufacturing_set".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        output_dir: output_dir.display().to_string(),
        prefix,
        bom_row_count: bom_report.rows,
        pnp_row_count: pnp_report.rows,
        drill_csv_row_count: drill_csv_report.rows,
        excellon_hit_count: excellon_report.hit_count,
        gerber_artifact_count: artifacts
            .iter()
            .filter(|artifact| artifact.kind.starts_with("gerber_"))
            .count(),
        written_count: artifacts.len(),
        artifacts,
    })
}

pub(crate) fn validate_native_project_manufacturing_set(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
) -> Result<NativeProjectManufacturingValidationView> {
    let project = load_native_project(root)?;
    let (prefix, entries) = native_project_manufacturing_manifest_entries(root, prefix_override)?;
    let gerber_plan = plan_native_project_gerber_export(root, Some(&prefix))?;
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
        let is_match = if filename.ends_with("-bom.csv") {
            let report = compare_native_project_bom(root, &artifact_path)?;
            report.missing_count == 0 && report.extra_count == 0 && report.drift_count == 0
        } else if filename.ends_with("-pnp.csv") {
            let report = compare_native_project_pnp(root, &artifact_path)?;
            report.missing_count == 0 && report.extra_count == 0 && report.drift_count == 0
        } else if filename.ends_with("-drill.csv") {
            let expected_csv = render_expected_native_project_drill_csv(root)?;
            let actual_csv = std::fs::read_to_string(&artifact_path)
                .with_context(|| format!("failed to read {}", artifact_path.display()))?;
            actual_csv == expected_csv
        } else if filename.ends_with("-drill.drl") {
            validate_native_project_excellon_drill(root, &artifact_path)?.matches_expected
        } else {
            let gerber_artifact = gerber_plan
                .artifacts
                .iter()
                .find(|artifact| artifact.filename == *filename)
                .expect("expected Gerber artifact should be present in plan");
            match (gerber_artifact.kind.as_str(), gerber_artifact.layer_id) {
                ("outline", None) => {
                    validate_native_project_gerber_outline(root, &artifact_path)?.matches_expected
                }
                ("copper", Some(layer)) => {
                    validate_native_project_gerber_copper_layer(root, layer, &artifact_path)?
                        .matches_expected
                }
                ("soldermask", Some(layer)) => {
                    validate_native_project_gerber_soldermask_layer(root, layer, &artifact_path)?
                        .matches_expected
                }
                ("silkscreen", Some(layer)) => {
                    validate_native_project_gerber_silkscreen_layer(root, layer, &artifact_path)?
                        .matches_expected
                }
                ("paste", Some(layer)) => {
                    validate_native_project_gerber_paste_layer(root, layer, &artifact_path)?
                        .matches_expected
                }
                ("mechanical", Some(layer)) => {
                    validate_native_project_gerber_mechanical_layer(root, layer, &artifact_path)?
                        .matches_expected
                }
                _ => anyhow::bail!("unsupported manufacturing Gerber artifact: {filename}"),
            }
        };
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

    Ok(NativeProjectManufacturingValidationView {
        action: "validate_manufacturing_set".to_string(),
        project_root: project.root.display().to_string(),
        board_path: project.board_path.display().to_string(),
        output_dir: output_dir.display().to_string(),
        prefix,
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

pub(crate) fn compare_native_project_manufacturing_set(
    root: &Path,
    output_dir: &Path,
    prefix_override: Option<&str>,
) -> Result<NativeProjectManufacturingComparisonView> {
    let project = load_native_project(root)?;
    let (prefix, entries) = native_project_manufacturing_manifest_entries(root, prefix_override)?;
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

    let gerber_compare = compare_native_project_gerber_set(root, output_dir, Some(&prefix))?;
    let gerber_mismatched = gerber_compare
        .mismatched
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();

    for filename in &expected {
        let artifact_path = output_dir.join(filename);
        if !artifact_path.is_file() {
            missing.push(filename.clone());
            continue;
        }
        let is_match = if filename.ends_with("-bom.csv") {
            let report = compare_native_project_bom(root, &artifact_path)?;
            report.missing_count == 0 && report.extra_count == 0 && report.drift_count == 0
        } else if filename.ends_with("-pnp.csv") {
            let report = compare_native_project_pnp(root, &artifact_path)?;
            report.missing_count == 0 && report.extra_count == 0 && report.drift_count == 0
        } else if filename.ends_with("-drill.csv") {
            let expected_csv = render_expected_native_project_drill_csv(root)?;
            let actual_csv = std::fs::read_to_string(&artifact_path)
                .with_context(|| format!("failed to read {}", artifact_path.display()))?;
            actual_csv == expected_csv
        } else if filename.ends_with("-drill.drl") {
            let report = compare_native_project_excellon_drill(root, &artifact_path)?;
            report.missing_count == 0 && report.extra_count == 0 && report.hit_drift_count == 0
        } else {
            !gerber_mismatched.contains(filename)
        };
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
        prefix,
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
