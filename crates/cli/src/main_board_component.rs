use eda_engine::pool::ModelRef;
use serde::Serialize;

use super::{
    NativeComponentMechanicalArc, NativeComponentMechanicalCircle, NativeComponentMechanicalLine,
    NativeComponentMechanicalPolygon, NativeComponentMechanicalPolyline,
    NativeComponentMechanicalText, NativeComponentPad, NativeComponentSilkscreenArc,
    NativeComponentSilkscreenCircle, NativeComponentSilkscreenLine,
    NativeComponentSilkscreenPolygon, NativeComponentSilkscreenPolyline,
    NativeComponentSilkscreenText,
};

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
    pub(crate) has_persisted_component_pads: bool,
    pub(crate) persisted_component_pad_count: usize,
    pub(crate) has_persisted_component_models_3d: bool,
    pub(crate) persisted_component_model_3d_count: usize,
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
    pub(crate) has_persisted_component_pads: bool,
    pub(crate) persisted_component_pad_count: usize,
    pub(crate) has_persisted_component_models_3d: bool,
    pub(crate) persisted_component_model_3d_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardComponentQueryPointView {
    pub(crate) x: i64,
    pub(crate) y: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardComponentModels3dView {
    pub(crate) component_uuid: String,
    pub(crate) model_count: usize,
    pub(crate) models: Vec<ModelRef>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardComponentPadsView {
    pub(crate) component_uuid: String,
    pub(crate) pad_count: usize,
    pub(crate) pads: Vec<NativeComponentPad>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardComponentSilkscreenView {
    pub(crate) component_uuid: String,
    pub(crate) text_count: usize,
    pub(crate) line_count: usize,
    pub(crate) arc_count: usize,
    pub(crate) circle_count: usize,
    pub(crate) polygon_count: usize,
    pub(crate) polyline_count: usize,
    pub(crate) texts: Vec<NativeComponentSilkscreenText>,
    pub(crate) lines: Vec<NativeComponentSilkscreenLine>,
    pub(crate) arcs: Vec<NativeComponentSilkscreenArc>,
    pub(crate) circles: Vec<NativeComponentSilkscreenCircle>,
    pub(crate) polygons: Vec<NativeComponentSilkscreenPolygon>,
    pub(crate) polylines: Vec<NativeComponentSilkscreenPolyline>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NativeProjectBoardComponentMechanicalView {
    pub(crate) component_uuid: String,
    pub(crate) text_count: usize,
    pub(crate) line_count: usize,
    pub(crate) arc_count: usize,
    pub(crate) circle_count: usize,
    pub(crate) polygon_count: usize,
    pub(crate) polyline_count: usize,
    pub(crate) texts: Vec<NativeComponentMechanicalText>,
    pub(crate) lines: Vec<NativeComponentMechanicalLine>,
    pub(crate) arcs: Vec<NativeComponentMechanicalArc>,
    pub(crate) circles: Vec<NativeComponentMechanicalCircle>,
    pub(crate) polygons: Vec<NativeComponentMechanicalPolygon>,
    pub(crate) polylines: Vec<NativeComponentMechanicalPolyline>,
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
        format!(
            "has_persisted_component_pads: {}",
            report.has_persisted_component_pads
        ),
        format!(
            "persisted_component_pad_count: {}",
            report.persisted_component_pad_count
        ),
        format!(
            "has_persisted_component_models_3d: {}",
            report.has_persisted_component_models_3d
        ),
        format!(
            "persisted_component_model_3d_count: {}",
            report.persisted_component_model_3d_count
        ),
    ]
    .join("\n")
}
