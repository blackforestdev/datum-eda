//! Existing authored presentation DTO vocabulary shared by renderer owners.
pub(crate) use crate::world_primitives::{
    push_convex_polygon_fill, push_world_quad, push_world_rect_nm, push_world_triangle,
};
pub(crate) use datum_gui_protocol::{
    Affine2DFixedPrimitive, BoardGraphicPrimitive, BoardReviewSceneV1, BoardTextGeometryPrimitive,
    BoardTextPrimitive, ComponentGraphicPrimitive, ComponentTextPrimitive, GlyphMeshAssetPrimitive,
    GlyphMeshHandlePrimitive, PointNm, ProposalOverlayPrimitive, ReviewWorkspaceState,
    SelectionTarget, UnroutedPrimitive, WorkspaceTool,
};
pub(crate) use eda_engine::board::BoardText;
pub(crate) use eda_engine::export::render_silkscreen_text_strokes;
pub(crate) use eda_engine::ir::geometry::{LayerId, Point};
pub(crate) use uuid::Uuid;

pub(crate) use crate::retained::{
    push_retained_board_graphic_batches, push_retained_scene_geometry,
};
pub(crate) use crate::world_polygon::{push_world_polygon_fill, push_world_polygon_fill_contours};

#[cfg(test)]
pub(crate) use crate::retained::push_retained_board_text_geometry_batches;
