// commands/board/views_mutations.rs — CLI view structs and text renderers for
// board mutation reports (moved from main.rs in the Wave 2 board lane).

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardKeepoutMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) keepout_uuid: String,
    pub(crate) kind: String,
    pub(crate) layer_count: usize,
    pub(crate) vertex_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardOutlineMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) vertex_count: usize,
    pub(crate) closed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardStackupMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) layer_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardNetMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) net_uuid: String,
    pub(crate) name: String,
    pub(crate) class_uuid: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardTrackMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) track_uuid: String,
    pub(crate) net_uuid: String,
    pub(crate) from_x_nm: i64,
    pub(crate) from_y_nm: i64,
    pub(crate) to_x_nm: i64,
    pub(crate) to_y_nm: i64,
    pub(crate) width_nm: i64,
    pub(crate) layer: i32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardViaMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) via_uuid: String,
    pub(crate) net_uuid: String,
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
    pub(crate) drill_nm: i64,
    pub(crate) diameter_nm: i64,
    pub(crate) from_layer: i32,
    pub(crate) to_layer: i32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardZoneMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) zone_uuid: String,
    pub(crate) net_uuid: String,
    pub(crate) layer: i32,
    pub(crate) priority: u32,
    pub(crate) thermal_relief: bool,
    pub(crate) thermal_gap_nm: i64,
    pub(crate) thermal_spoke_width_nm: i64,
    pub(crate) vertex_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardPadMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) pad_uuid: String,
    pub(crate) package_uuid: String,
    pub(crate) name: String,
    pub(crate) net_uuid: Option<String>,
    pub(crate) x_nm: i64,
    pub(crate) y_nm: i64,
    pub(crate) layer: i32,
    pub(crate) shape: String,
    pub(crate) diameter_nm: i64,
    pub(crate) width_nm: i64,
    pub(crate) height_nm: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardNetClassMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) net_class_uuid: String,
    pub(crate) name: String,
    pub(crate) clearance_nm: i64,
    pub(crate) track_width_nm: i64,
    pub(crate) via_drill_nm: i64,
    pub(crate) via_diameter_nm: i64,
    pub(crate) diffpair_width_nm: i64,
    pub(crate) diffpair_gap_nm: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardDimensionMutationReportView {
    pub(crate) action: String,
    pub(crate) project_root: String,
    pub(crate) board_path: String,
    pub(crate) dimension_uuid: String,
    pub(crate) from_x_nm: i64,
    pub(crate) from_y_nm: i64,
    pub(crate) to_x_nm: i64,
    pub(crate) to_y_nm: i64,
    pub(crate) layer: i32,
    pub(crate) text: Option<String>,
}

pub(crate) fn render_native_project_board_keepout_mutation_text(
    report: &NativeProjectBoardKeepoutMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("keepout_uuid: {}", report.keepout_uuid),
        format!("kind: {}", report.kind),
        format!("layer_count: {}", report.layer_count),
        format!("vertex_count: {}", report.vertex_count),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_board_outline_mutation_text(
    report: &NativeProjectBoardOutlineMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("vertex_count: {}", report.vertex_count),
        format!("closed: {}", report.closed),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_board_stackup_mutation_text(
    report: &NativeProjectBoardStackupMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("layer_count: {}", report.layer_count),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_board_net_mutation_text(
    report: &NativeProjectBoardNetMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("net_uuid: {}", report.net_uuid),
        format!("name: {}", report.name),
        format!("class_uuid: {}", report.class_uuid),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_board_track_mutation_text(
    report: &NativeProjectBoardTrackMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("track_uuid: {}", report.track_uuid),
        format!("net_uuid: {}", report.net_uuid),
        format!("from_x_nm: {}", report.from_x_nm),
        format!("from_y_nm: {}", report.from_y_nm),
        format!("to_x_nm: {}", report.to_x_nm),
        format!("to_y_nm: {}", report.to_y_nm),
        format!("width_nm: {}", report.width_nm),
        format!("layer: {}", report.layer),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_board_via_mutation_text(
    report: &NativeProjectBoardViaMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("via_uuid: {}", report.via_uuid),
        format!("net_uuid: {}", report.net_uuid),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
        format!("drill_nm: {}", report.drill_nm),
        format!("diameter_nm: {}", report.diameter_nm),
        format!("from_layer: {}", report.from_layer),
        format!("to_layer: {}", report.to_layer),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_board_zone_mutation_text(
    report: &NativeProjectBoardZoneMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("zone_uuid: {}", report.zone_uuid),
        format!("net_uuid: {}", report.net_uuid),
        format!("layer: {}", report.layer),
        format!("priority: {}", report.priority),
        format!("thermal_relief: {}", report.thermal_relief),
        format!("thermal_gap_nm: {}", report.thermal_gap_nm),
        format!("thermal_spoke_width_nm: {}", report.thermal_spoke_width_nm),
        format!("vertex_count: {}", report.vertex_count),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_board_pad_mutation_text(
    report: &NativeProjectBoardPadMutationReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("pad_uuid: {}", report.pad_uuid),
        format!("package_uuid: {}", report.package_uuid),
        format!("name: {}", report.name),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
        format!("layer: {}", report.layer),
        format!("shape: {}", report.shape),
        format!("diameter_nm: {}", report.diameter_nm),
        format!("width_nm: {}", report.width_nm),
        format!("height_nm: {}", report.height_nm),
    ];
    if let Some(net_uuid) = &report.net_uuid {
        lines.push(format!("net_uuid: {}", net_uuid));
    }
    lines.join("\n")
}

pub(crate) fn render_native_project_board_net_class_mutation_text(
    report: &NativeProjectBoardNetClassMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("net_class_uuid: {}", report.net_class_uuid),
        format!("name: {}", report.name),
        format!("clearance_nm: {}", report.clearance_nm),
        format!("track_width_nm: {}", report.track_width_nm),
        format!("via_drill_nm: {}", report.via_drill_nm),
        format!("via_diameter_nm: {}", report.via_diameter_nm),
        format!("diffpair_width_nm: {}", report.diffpair_width_nm),
        format!("diffpair_gap_nm: {}", report.diffpair_gap_nm),
    ]
    .join("\n")
}

pub(crate) fn render_native_project_board_dimension_mutation_text(
    report: &NativeProjectBoardDimensionMutationReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("dimension_uuid: {}", report.dimension_uuid),
        format!("from_x_nm: {}", report.from_x_nm),
        format!("from_y_nm: {}", report.from_y_nm),
        format!("to_x_nm: {}", report.to_x_nm),
        format!("to_y_nm: {}", report.to_y_nm),
        format!("layer: {}", report.layer),
    ];
    if let Some(text) = &report.text {
        lines.push(format!("text: {}", text));
    }
    lines.join("\n")
}
