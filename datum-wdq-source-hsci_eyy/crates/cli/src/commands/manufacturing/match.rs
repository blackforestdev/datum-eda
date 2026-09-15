use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use eda_engine::substrate::PanelProjection;
use uuid::Uuid;

use crate::NativeProjectGerberPlanView;

use super::super::{
    compare_native_project_bom, compare_native_project_pnp,
    export_native_project_gerber_copper_layer, export_native_project_gerber_mechanical_layer,
    export_native_project_gerber_outline, export_native_project_gerber_paste_layer,
    export_native_project_gerber_silkscreen_layer, export_native_project_gerber_soldermask_layer,
    panelize_rs274x_gerber, render_expected_native_project_drill_csv,
    render_expected_native_project_panel_drill_csv,
    render_expected_native_project_panel_excellon_drill,
    render_expected_native_project_panel_pnp_csv, validate_native_project_excellon_drill,
    validate_native_project_gerber_copper_layer, validate_native_project_gerber_mechanical_layer,
    validate_native_project_gerber_outline, validate_native_project_gerber_paste_layer,
    validate_native_project_gerber_silkscreen_layer,
    validate_native_project_gerber_soldermask_layer,
};

pub(super) fn manufacturing_artifact_matches(
    root: &Path,
    artifact_path: &Path,
    filename: &str,
    variant: Option<Uuid>,
    panel_projection: Option<&PanelProjection>,
    gerber_plan: Option<&NativeProjectGerberPlanView>,
) -> Result<bool> {
    if filename.ends_with("-bom.csv") {
        let report = compare_native_project_bom(root, artifact_path, variant)?;
        return Ok(report.missing_count == 0 && report.extra_count == 0 && report.drift_count == 0);
    }
    if filename.ends_with("-pnp.csv") {
        if let Some(panel_projection) = panel_projection {
            let expected =
                render_expected_native_project_panel_pnp_csv(root, variant, panel_projection)?;
            let actual = std::fs::read_to_string(artifact_path)
                .with_context(|| format!("failed to read {}", artifact_path.display()))?;
            return Ok(actual == expected);
        }
        let report = compare_native_project_pnp(root, artifact_path, variant)?;
        return Ok(report.missing_count == 0 && report.extra_count == 0 && report.drift_count == 0);
    }
    if filename.ends_with("-drill.csv") {
        let expected_csv = if let Some(panel_projection) = panel_projection {
            render_expected_native_project_panel_drill_csv(root, panel_projection)?
        } else {
            render_expected_native_project_drill_csv(root)?
        };
        let actual_csv = std::fs::read_to_string(artifact_path)
            .with_context(|| format!("failed to read {}", artifact_path.display()))?;
        return Ok(actual_csv == expected_csv);
    }
    if filename.ends_with("-drill.drl") {
        if let Some(panel_projection) = panel_projection {
            let expected =
                render_expected_native_project_panel_excellon_drill(root, panel_projection)?;
            let actual = std::fs::read_to_string(artifact_path)
                .with_context(|| format!("failed to read {}", artifact_path.display()))?;
            return Ok(actual == expected);
        }
        return Ok(validate_native_project_excellon_drill(root, artifact_path)?.matches_expected);
    }
    let gerber_artifact = gerber_plan
        .context("manufacturing projection missing Gerber plan")?
        .artifacts
        .iter()
        .find(|artifact| artifact.filename == filename)
        .with_context(|| format!("expected Gerber artifact missing from projection: {filename}"))?;
    if let Some(panel_projection) = panel_projection {
        let expected = expected_panel_gerber(
            root,
            gerber_artifact.kind.as_str(),
            gerber_artifact.layer_id,
            panel_projection,
        )?;
        let actual = std::fs::read_to_string(artifact_path)
            .with_context(|| format!("failed to read {}", artifact_path.display()))?;
        return Ok(actual == expected);
    }
    match (gerber_artifact.kind.as_str(), gerber_artifact.layer_id) {
        ("outline", None) => {
            Ok(validate_native_project_gerber_outline(root, artifact_path)?.matches_expected)
        }
        ("copper", Some(layer)) => {
            Ok(
                validate_native_project_gerber_copper_layer(root, layer, artifact_path)?
                    .matches_expected,
            )
        }
        ("soldermask", Some(layer)) => {
            Ok(
                validate_native_project_gerber_soldermask_layer(root, layer, artifact_path)?
                    .matches_expected,
            )
        }
        ("silkscreen", Some(layer)) => {
            Ok(
                validate_native_project_gerber_silkscreen_layer(root, layer, artifact_path)?
                    .matches_expected,
            )
        }
        ("paste", Some(layer)) => {
            Ok(
                validate_native_project_gerber_paste_layer(root, layer, artifact_path)?
                    .matches_expected,
            )
        }
        ("mechanical", Some(layer)) => {
            Ok(
                validate_native_project_gerber_mechanical_layer(root, layer, artifact_path)?
                    .matches_expected,
            )
        }
        _ => anyhow::bail!("unsupported manufacturing Gerber artifact: {filename}"),
    }
}

fn expected_panel_gerber(
    root: &Path,
    kind: &str,
    layer_id: Option<i32>,
    panel_projection: &PanelProjection,
) -> Result<String> {
    let temp_path = panel_gerber_temp_path(kind, layer_id);
    match (kind, layer_id) {
        ("outline", None) => export_native_project_gerber_outline(root, &temp_path).map(|_| ()),
        ("copper", Some(layer)) => {
            export_native_project_gerber_copper_layer(root, layer, &temp_path).map(|_| ())
        }
        ("soldermask", Some(layer)) => {
            export_native_project_gerber_soldermask_layer(root, layer, &temp_path).map(|_| ())
        }
        ("silkscreen", Some(layer)) => {
            export_native_project_gerber_silkscreen_layer(root, layer, &temp_path).map(|_| ())
        }
        ("paste", Some(layer)) => {
            export_native_project_gerber_paste_layer(root, layer, &temp_path).map(|_| ())
        }
        ("mechanical", Some(layer)) => {
            export_native_project_gerber_mechanical_layer(root, layer, &temp_path).map(|_| ())
        }
        _ => anyhow::bail!("unsupported manufacturing Gerber artifact: {kind}"),
    }?;
    let base = std::fs::read_to_string(&temp_path)
        .with_context(|| format!("failed to read {}", temp_path.display()))?;
    let _ = std::fs::remove_file(&temp_path);
    panelize_rs274x_gerber(root, &base, panel_projection)
}

fn panel_gerber_temp_path(kind: &str, layer_id: Option<i32>) -> PathBuf {
    std::env::temp_dir().join(format!(
        "datum-panel-gerber-{}-{}-{}.gbr",
        kind,
        layer_id
            .map(|layer| layer.to_string())
            .unwrap_or_else(|| "none".to_string()),
        Uuid::new_v4()
    ))
}
