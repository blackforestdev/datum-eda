use std::collections::BTreeSet;
use std::path::Path;

use crate::{
    NativeProjectGerberSetArtifactView, NativeProjectGerberSetComparisonView,
    NativeProjectGerberSetExportView, NativeProjectGerberSetValidationView,
};
use anyhow::{Context, Result};
use eda_engine::board::StackupLayer;

use super::{
    NativeProjectGerberPlanArtifactView, NativeProjectGerberPlanComparisonView,
    NativeProjectGerberPlanView, StackupLayerType, compare_native_project_gerber_copper_layer,
    compare_native_project_gerber_mechanical_layer, compare_native_project_gerber_outline,
    compare_native_project_gerber_paste_layer, compare_native_project_gerber_silkscreen_layer,
    compare_native_project_gerber_soldermask_layer, export_native_project_gerber_copper_layer,
    export_native_project_gerber_mechanical_layer, export_native_project_gerber_outline,
    export_native_project_gerber_paste_layer, export_native_project_gerber_silkscreen_layer,
    export_native_project_gerber_soldermask_layer, load_native_project,
    validate_native_project_gerber_copper_layer, validate_native_project_gerber_mechanical_layer,
    validate_native_project_gerber_outline, validate_native_project_gerber_paste_layer,
    validate_native_project_gerber_silkscreen_layer,
    validate_native_project_gerber_soldermask_layer,
};

pub(crate) fn plan_native_project_gerber_export(
    root: &Path,
    prefix_override: Option<&str>,
) -> Result<NativeProjectGerberPlanView> {
    let project = load_native_project(root)?;
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
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create {}", output_dir.display()))?;
    let plan = plan_native_project_gerber_export(root, prefix_override)?;
    let mut artifacts = Vec::new();

    for artifact in plan.artifacts {
        let output_path = output_dir.join(&artifact.filename);
        match (artifact.kind.as_str(), artifact.layer_id) {
            ("outline", None) => {
                export_native_project_gerber_outline(root, &output_path)?;
            }
            ("copper", Some(layer)) => {
                export_native_project_gerber_copper_layer(root, layer, &output_path)?;
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
        artifacts.push(NativeProjectGerberSetArtifactView {
            kind: artifact.kind,
            layer_id: artifact.layer_id,
            layer_name: artifact.layer_name,
            filename: artifact.filename,
            output_path: output_path.display().to_string(),
        });
    }

    Ok(NativeProjectGerberSetExportView {
        action: "export_gerber_set".to_string(),
        project_root: plan.project_root,
        output_dir: output_dir.display().to_string(),
        prefix: plan.prefix,
        written_count: artifacts.len(),
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
