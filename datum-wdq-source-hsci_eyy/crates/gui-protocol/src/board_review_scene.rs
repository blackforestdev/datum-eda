//! Serialized board-review scene envelope and pad-expansion setup.

use serde::{Deserialize, Serialize};

use crate::{
    BoardGraphicPrimitive, BoardTextGeometryPrimitive, BoardTextPrimitive, ComponentBounds,
    ComponentGraphicPrimitive, ComponentTextPrimitive, GlyphMeshAssetPrimitive, NetDisplayEntry,
    OutlinePolyline, PadPrimitive, ProposalOverlayPrimitive, ReviewPrimitive, SceneBounds,
    SceneLayer, TrackPrimitive, UnroutedPrimitive, ViaPrimitive, ZonePrimitive,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardReviewSceneV1 {
    pub kind: String,
    pub version: u32,
    pub scene_id: String,
    pub project_uuid: String,
    pub project_name: String,
    pub board_uuid: String,
    pub board_name: String,
    pub units: String,
    pub source_revision: String,
    #[serde(default)]
    pub pad_expansion_setup: ScenePadExpansionSetup,
    pub bounds: SceneBounds,
    pub layers: Vec<SceneLayer>,
    pub outline: Vec<OutlinePolyline>,
    pub components: Vec<ComponentBounds>,
    #[serde(default)]
    pub component_graphics: Vec<ComponentGraphicPrimitive>,
    #[serde(default)]
    pub component_texts: Vec<ComponentTextPrimitive>,
    pub pads: Vec<PadPrimitive>,
    pub tracks: Vec<TrackPrimitive>,
    pub vias: Vec<ViaPrimitive>,
    pub zones: Vec<ZonePrimitive>,
    #[serde(default)]
    pub board_graphics: Vec<BoardGraphicPrimitive>,
    #[serde(default)]
    pub board_texts: Vec<BoardTextPrimitive>,
    #[serde(default)]
    pub board_text_geometries: Vec<BoardTextGeometryPrimitive>,
    #[serde(default)]
    pub glyph_mesh_assets: Vec<GlyphMeshAssetPrimitive>,
    #[serde(default)]
    pub unrouted_primitives: Vec<UnroutedPrimitive>,
    #[serde(default)]
    pub net_display: Vec<NetDisplayEntry>,
    pub proposal_overlay_primitives: Vec<ProposalOverlayPrimitive>,
    pub review_primitives: Vec<ReviewPrimitive>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ScenePadExpansionSetup {
    #[serde(default)]
    pub pad_to_mask_clearance_nm: i64,
    #[serde(default)]
    pub pad_to_paste_clearance_nm: i64,
    #[serde(default)]
    pub pad_to_paste_ratio_ppm: i32,
    #[serde(default)]
    pub solder_mask_min_width_nm: i64,
}
