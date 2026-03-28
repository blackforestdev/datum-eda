use serde::Serialize;

use super::NativeProjectGerberPlanArtifactView;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectManufacturingArtifactView {
    pub(crate) kind: String,
    pub(crate) output_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectManufacturingReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) prefix: String,
    pub(crate) bom_component_count: usize,
    pub(crate) pnp_component_count: usize,
    pub(crate) drill_csv_row_count: usize,
    pub(crate) excellon_via_count: usize,
    pub(crate) excellon_component_pad_count: usize,
    pub(crate) excellon_hit_count: usize,
    pub(crate) drill_hole_class_count: usize,
    pub(crate) gerber_artifact_count: usize,
    pub(crate) gerber_artifacts: Vec<NativeProjectGerberPlanArtifactView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectManufacturingExportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) output_dir: String,
    pub(crate) prefix: String,
    pub(crate) bom_row_count: usize,
    pub(crate) pnp_row_count: usize,
    pub(crate) drill_csv_row_count: usize,
    pub(crate) excellon_hit_count: usize,
    pub(crate) gerber_artifact_count: usize,
    pub(crate) written_count: usize,
    pub(crate) artifacts: Vec<NativeProjectManufacturingArtifactView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectManufacturingValidationView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) output_dir: String,
    pub(crate) prefix: String,
    pub(crate) expected_count: usize,
    pub(crate) matched_count: usize,
    pub(crate) missing_count: usize,
    pub(crate) mismatched_count: usize,
    pub(crate) extra_count: usize,
    pub(crate) matched: Vec<String>,
    pub(crate) missing: Vec<String>,
    pub(crate) mismatched: Vec<String>,
    pub(crate) extra: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectManufacturingComparisonView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) output_dir: String,
    pub(crate) prefix: String,
    pub(crate) expected_count: usize,
    pub(crate) matched_count: usize,
    pub(crate) missing_count: usize,
    pub(crate) mismatched_count: usize,
    pub(crate) extra_count: usize,
    pub(crate) matched: Vec<String>,
    pub(crate) missing: Vec<String>,
    pub(crate) mismatched: Vec<String>,
    pub(crate) extra: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectManufacturingManifestEntryView {
    pub(crate) kind: String,
    pub(crate) filename: String,
    pub(crate) contract: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectManufacturingManifestView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) output_dir: String,
    pub(crate) prefix: String,
    pub(crate) expected_count: usize,
    pub(crate) entries: Vec<NativeProjectManufacturingManifestEntryView>,
}

pub(crate) fn render_native_project_manufacturing_report_text(
    report: &NativeProjectManufacturingReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("prefix: {}", report.prefix),
        format!("bom_component_count: {}", report.bom_component_count),
        format!("pnp_component_count: {}", report.pnp_component_count),
        format!("drill_csv_row_count: {}", report.drill_csv_row_count),
        format!("excellon_via_count: {}", report.excellon_via_count),
        format!(
            "excellon_component_pad_count: {}",
            report.excellon_component_pad_count
        ),
        format!("excellon_hit_count: {}", report.excellon_hit_count),
        format!("drill_hole_class_count: {}", report.drill_hole_class_count),
        format!("gerber_artifact_count: {}", report.gerber_artifact_count),
    ];
    if !report.gerber_artifacts.is_empty() {
        lines.push("gerber_artifacts:".to_string());
        for artifact in &report.gerber_artifacts {
            match (&artifact.layer_id, &artifact.layer_name) {
                (Some(layer_id), Some(layer_name)) => lines.push(format!(
                    "  - {}:{}:{}:{}",
                    artifact.kind, layer_id, layer_name, artifact.filename
                )),
                _ => lines.push(format!("  - {}:{}", artifact.kind, artifact.filename)),
            }
        }
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_manufacturing_export_text(
    report: &NativeProjectManufacturingExportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("output_dir: {}", report.output_dir),
        format!("prefix: {}", report.prefix),
        format!("bom_row_count: {}", report.bom_row_count),
        format!("pnp_row_count: {}", report.pnp_row_count),
        format!("drill_csv_row_count: {}", report.drill_csv_row_count),
        format!("excellon_hit_count: {}", report.excellon_hit_count),
        format!("gerber_artifact_count: {}", report.gerber_artifact_count),
        format!("written_count: {}", report.written_count),
    ];
    if !report.artifacts.is_empty() {
        lines.push("artifacts:".to_string());
        for artifact in &report.artifacts {
            lines.push(format!("  - {}:{}", artifact.kind, artifact.output_path));
        }
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_manufacturing_validation_text(
    report: &NativeProjectManufacturingValidationView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("output_dir: {}", report.output_dir),
        format!("prefix: {}", report.prefix),
        format!("expected_count: {}", report.expected_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("mismatched_count: {}", report.mismatched_count),
        format!("extra_count: {}", report.extra_count),
    ];
    if !report.matched.is_empty() {
        lines.push("matched:".to_string());
        for entry in &report.matched {
            lines.push(format!("  {entry}"));
        }
    }
    if !report.missing.is_empty() {
        lines.push("missing:".to_string());
        for entry in &report.missing {
            lines.push(format!("  {entry}"));
        }
    }
    if !report.mismatched.is_empty() {
        lines.push("mismatched:".to_string());
        for entry in &report.mismatched {
            lines.push(format!("  {entry}"));
        }
    }
    if !report.extra.is_empty() {
        lines.push("extra:".to_string());
        for entry in &report.extra {
            lines.push(format!("  {entry}"));
        }
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_manufacturing_comparison_text(
    report: &NativeProjectManufacturingComparisonView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("output_dir: {}", report.output_dir),
        format!("prefix: {}", report.prefix),
        format!("expected_count: {}", report.expected_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("mismatched_count: {}", report.mismatched_count),
        format!("extra_count: {}", report.extra_count),
    ];
    if !report.matched.is_empty() {
        lines.push("matched:".to_string());
        for entry in &report.matched {
            lines.push(format!("  {entry}"));
        }
    }
    if !report.missing.is_empty() {
        lines.push("missing:".to_string());
        for entry in &report.missing {
            lines.push(format!("  {entry}"));
        }
    }
    if !report.mismatched.is_empty() {
        lines.push("mismatched:".to_string());
        for entry in &report.mismatched {
            lines.push(format!("  {entry}"));
        }
    }
    if !report.extra.is_empty() {
        lines.push("extra:".to_string());
        for entry in &report.extra {
            lines.push(format!("  {entry}"));
        }
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_manufacturing_manifest_text(
    report: &NativeProjectManufacturingManifestView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("output_dir: {}", report.output_dir),
        format!("prefix: {}", report.prefix),
        format!("expected_count: {}", report.expected_count),
    ];
    if !report.entries.is_empty() {
        lines.push("entries:".to_string());
        for entry in &report.entries {
            lines.push(format!(
                "  - {}:{}:{}",
                entry.kind, entry.contract, entry.filename
            ));
        }
    }
    lines.join("\n")
}
