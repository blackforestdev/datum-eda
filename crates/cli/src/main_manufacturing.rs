use serde::Serialize;

use super::NativeProjectGerberPlanArtifactView;

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
