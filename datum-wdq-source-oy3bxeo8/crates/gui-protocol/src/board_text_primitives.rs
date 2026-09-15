use crate::PointNm;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardTextPrimitive {
    pub object_id: String,
    pub object_kind: String,
    pub text_uuid: String,
    pub text: String,
    pub layer_id: String,
    pub position: PointNm,
    pub rotation_degrees: i32,
    pub height_nm: i64,
    pub stroke_width_nm: i64,
    pub render_intent: String,
    pub family: String,
    pub style: String,
    #[serde(default)]
    pub style_class: Option<String>,
    pub h_align: String,
    pub v_align: String,
    pub mirrored: bool,
    pub keep_upright: bool,
    pub line_spacing_ratio_ppm: i32,
    pub bold: bool,
    pub italic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardTextGeometryPrimitive {
    pub object_id: String,
    pub object_kind: String,
    pub text_uuid: String,
    pub layer_id: String,
    #[serde(default)]
    pub world_transform_nm: Option<Affine2DFixedPrimitive>,
    #[serde(default)]
    pub block_bbox_em_nm: Option<MeshRectEmPrimitive>,
    #[serde(default)]
    pub glyphs: Vec<TextGlyphInstancePrimitive>,
    #[serde(default)]
    pub fills: Vec<BoardTextFillPrimitive>,
    #[serde(default)]
    pub strokes: Vec<BoardTextStrokePrimitive>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GlyphMeshHandlePrimitive {
    pub font_id: u32,
    pub glyph_id: u32,
    pub tolerance_class: u8,
    pub epoch: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlyphMeshAssetPrimitive {
    pub handle: GlyphMeshHandlePrimitive,
    pub vertices: Vec<MeshVertexEmPrimitive>,
    pub indices: Vec<u32>,
    pub bbox_em_nm: MeshRectEmPrimitive,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextGlyphInstancePrimitive {
    pub glyph_handle: GlyphMeshHandlePrimitive,
    pub origin_em_nm_x: i64,
    pub origin_em_nm_y: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct MeshVertexEmPrimitive {
    pub x_em_nm: i64,
    pub y_em_nm: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct MeshRectEmPrimitive {
    pub min_x_em_nm: i64,
    pub min_y_em_nm: i64,
    pub max_x_em_nm: i64,
    pub max_y_em_nm: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Affine2DFixedPrimitive {
    pub m11_ppm: i64,
    pub m12_ppm: i64,
    pub m21_ppm: i64,
    pub m22_ppm: i64,
    pub tx_nm: i64,
    pub ty_nm: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardTextFillPrimitive {
    pub outer: Vec<PointNm>,
    #[serde(default)]
    pub holes: Vec<Vec<PointNm>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardTextStrokePrimitive {
    pub from: PointNm,
    pub to: PointNm,
    pub width_nm: i64,
}
