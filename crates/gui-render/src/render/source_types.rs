//! Existing authored presentation DTO vocabulary shared by renderer owners.
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
