use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardComponentMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) component_uuid: String,
    pub(crate) part_uuid: String,
    pub(crate) package_uuid: String,
    pub(crate) reference: String,
    pub(crate) value: String,
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
    pub(crate) rotation_deg: i32,
    pub(crate) layer: i32,
    pub(crate) locked: bool,
    pub(crate) has_persisted_component_silkscreen: bool,
    pub(crate) persisted_component_silkscreen_text_count: usize,
    pub(crate) persisted_component_silkscreen_line_count: usize,
    pub(crate) persisted_component_silkscreen_arc_count: usize,
    pub(crate) persisted_component_silkscreen_circle_count: usize,
    pub(crate) persisted_component_silkscreen_polygon_count: usize,
    pub(crate) persisted_component_silkscreen_polyline_count: usize,
    pub(crate) has_persisted_component_mechanical: bool,
    pub(crate) persisted_component_mechanical_text_count: usize,
    pub(crate) persisted_component_mechanical_line_count: usize,
    pub(crate) persisted_component_mechanical_arc_count: usize,
    pub(crate) persisted_component_mechanical_circle_count: usize,
    pub(crate) persisted_component_mechanical_polygon_count: usize,
    pub(crate) persisted_component_mechanical_polyline_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardComponentQueryView {
    pub(crate) uuid: String,
    pub(crate) part: String,
    pub(crate) package: String,
    pub(crate) reference: String,
    pub(crate) value: String,
    pub(crate) position: NativeProjectBoardComponentQueryPointView,
    pub(crate) rotation: i32,
    pub(crate) layer: i32,
    pub(crate) locked: bool,
    pub(crate) has_persisted_component_silkscreen: bool,
    pub(crate) persisted_component_silkscreen_text_count: usize,
    pub(crate) persisted_component_silkscreen_line_count: usize,
    pub(crate) persisted_component_silkscreen_arc_count: usize,
    pub(crate) persisted_component_silkscreen_circle_count: usize,
    pub(crate) persisted_component_silkscreen_polygon_count: usize,
    pub(crate) persisted_component_silkscreen_polyline_count: usize,
    pub(crate) has_persisted_component_mechanical: bool,
    pub(crate) persisted_component_mechanical_text_count: usize,
    pub(crate) persisted_component_mechanical_line_count: usize,
    pub(crate) persisted_component_mechanical_arc_count: usize,
    pub(crate) persisted_component_mechanical_circle_count: usize,
    pub(crate) persisted_component_mechanical_polygon_count: usize,
    pub(crate) persisted_component_mechanical_polyline_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardComponentQueryPointView {
    pub(crate) x: i64,
    pub(crate) y: i64,
}

pub(crate) fn render_native_project_board_component_mutation_text(
    report: &NativeProjectBoardComponentMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("component_uuid: {}", report.component_uuid),
        format!("part_uuid: {}", report.part_uuid),
        format!("package_uuid: {}", report.package_uuid),
        format!("reference: {}", report.reference),
        format!("value: {}", report.value),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
        format!("rotation_deg: {}", report.rotation_deg),
        format!("layer: {}", report.layer),
        format!("locked: {}", report.locked),
        format!(
            "has_persisted_component_silkscreen: {}",
            report.has_persisted_component_silkscreen
        ),
        format!(
            "persisted_component_silkscreen_text_count: {}",
            report.persisted_component_silkscreen_text_count
        ),
        format!(
            "persisted_component_silkscreen_line_count: {}",
            report.persisted_component_silkscreen_line_count
        ),
        format!(
            "persisted_component_silkscreen_arc_count: {}",
            report.persisted_component_silkscreen_arc_count
        ),
        format!(
            "persisted_component_silkscreen_circle_count: {}",
            report.persisted_component_silkscreen_circle_count
        ),
        format!(
            "persisted_component_silkscreen_polygon_count: {}",
            report.persisted_component_silkscreen_polygon_count
        ),
        format!(
            "persisted_component_silkscreen_polyline_count: {}",
            report.persisted_component_silkscreen_polyline_count
        ),
        format!(
            "has_persisted_component_mechanical: {}",
            report.has_persisted_component_mechanical
        ),
        format!(
            "persisted_component_mechanical_text_count: {}",
            report.persisted_component_mechanical_text_count
        ),
        format!(
            "persisted_component_mechanical_line_count: {}",
            report.persisted_component_mechanical_line_count
        ),
        format!(
            "persisted_component_mechanical_arc_count: {}",
            report.persisted_component_mechanical_arc_count
        ),
        format!(
            "persisted_component_mechanical_circle_count: {}",
            report.persisted_component_mechanical_circle_count
        ),
        format!(
            "persisted_component_mechanical_polygon_count: {}",
            report.persisted_component_mechanical_polygon_count
        ),
        format!(
            "persisted_component_mechanical_polyline_count: {}",
            report.persisted_component_mechanical_polyline_count
        ),
    ]
    .join("\n")
}
