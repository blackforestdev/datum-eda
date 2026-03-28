use serde::Serialize;

use super::{NativeProjectGerberGeometryEntryView, append_gerber_geometry_entries};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberMechanicalExportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) gerber_path: String,
    pub(crate) layer: i32,
    pub(crate) keepout_count: usize,
    pub(crate) dimension_count: usize,
    pub(crate) board_text_count: usize,
    pub(crate) component_text_count: usize,
    pub(crate) component_polygon_count: usize,
    pub(crate) component_stroke_count: usize,
    pub(crate) component_polyline_count: usize,
    pub(crate) component_circle_count: usize,
    pub(crate) component_arc_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberMechanicalValidationView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) gerber_path: String,
    pub(crate) layer: i32,
    pub(crate) matches_expected: bool,
    pub(crate) expected_bytes: usize,
    pub(crate) actual_bytes: usize,
    pub(crate) keepout_count: usize,
    pub(crate) dimension_count: usize,
    pub(crate) board_text_count: usize,
    pub(crate) component_text_count: usize,
    pub(crate) component_polygon_count: usize,
    pub(crate) component_stroke_count: usize,
    pub(crate) component_polyline_count: usize,
    pub(crate) component_circle_count: usize,
    pub(crate) component_arc_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectGerberMechanicalComparisonView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) gerber_path: String,
    pub(crate) layer: i32,
    pub(crate) expected_keepout_count: usize,
    pub(crate) expected_dimension_count: usize,
    pub(crate) expected_board_text_count: usize,
    pub(crate) expected_component_text_count: usize,
    pub(crate) expected_component_polygon_count: usize,
    pub(crate) expected_component_stroke_count: usize,
    pub(crate) expected_component_polyline_count: usize,
    pub(crate) expected_component_circle_count: usize,
    pub(crate) expected_component_arc_count: usize,
    pub(crate) actual_geometry_count: usize,
    pub(crate) matched_count: usize,
    pub(crate) missing_count: usize,
    pub(crate) extra_count: usize,
    pub(crate) matched: Vec<NativeProjectGerberGeometryEntryView>,
    pub(crate) missing: Vec<NativeProjectGerberGeometryEntryView>,
    pub(crate) extra: Vec<NativeProjectGerberGeometryEntryView>,
}

pub(crate) fn render_native_project_gerber_mechanical_export_text(
    report: &NativeProjectGerberMechanicalExportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("keepout_count: {}", report.keepout_count),
        format!("dimension_count: {}", report.dimension_count),
        format!("board_text_count: {}", report.board_text_count),
        format!("component_text_count: {}", report.component_text_count),
        format!(
            "component_polygon_count: {}",
            report.component_polygon_count
        ),
        format!("component_stroke_count: {}", report.component_stroke_count),
        format!(
            "component_polyline_count: {}",
            report.component_polyline_count
        ),
        format!("component_circle_count: {}", report.component_circle_count),
        format!("component_arc_count: {}", report.component_arc_count),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_gerber_mechanical_validation_text(
    report: &NativeProjectGerberMechanicalValidationView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("matches_expected: {}", report.matches_expected),
        format!("expected_bytes: {}", report.expected_bytes),
        format!("actual_bytes: {}", report.actual_bytes),
        format!("keepout_count: {}", report.keepout_count),
        format!("dimension_count: {}", report.dimension_count),
        format!("board_text_count: {}", report.board_text_count),
        format!("component_text_count: {}", report.component_text_count),
        format!(
            "component_polygon_count: {}",
            report.component_polygon_count
        ),
        format!("component_stroke_count: {}", report.component_stroke_count),
        format!(
            "component_polyline_count: {}",
            report.component_polyline_count
        ),
        format!("component_circle_count: {}", report.component_circle_count),
        format!("component_arc_count: {}", report.component_arc_count),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_gerber_mechanical_comparison_text(
    report: &NativeProjectGerberMechanicalComparisonView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("expected_keepout_count: {}", report.expected_keepout_count),
        format!(
            "expected_dimension_count: {}",
            report.expected_dimension_count
        ),
        format!(
            "expected_board_text_count: {}",
            report.expected_board_text_count
        ),
        format!(
            "expected_component_text_count: {}",
            report.expected_component_text_count
        ),
        format!(
            "expected_component_polygon_count: {}",
            report.expected_component_polygon_count
        ),
        format!(
            "expected_component_stroke_count: {}",
            report.expected_component_stroke_count
        ),
        format!(
            "expected_component_polyline_count: {}",
            report.expected_component_polyline_count
        ),
        format!(
            "expected_component_circle_count: {}",
            report.expected_component_circle_count
        ),
        format!(
            "expected_component_arc_count: {}",
            report.expected_component_arc_count
        ),
        format!("actual_geometry_count: {}", report.actual_geometry_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("extra_count: {}", report.extra_count),
    ];
    append_gerber_geometry_entries(&mut lines, "matched", &report.matched);
    append_gerber_geometry_entries(&mut lines, "missing", &report.missing);
    append_gerber_geometry_entries(&mut lines, "extra", &report.extra);
    lines.join("\n")
}
