use crate::ir::geometry::{LayerId, Point, Polygon};
use crate::text::{
    TextFamilyId, TextFamilySource, TextHAlign, TextRenderIntent, TextStyleId, TextVAlign,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacedPackage {
    pub uuid: Uuid,
    pub part: Uuid,
    pub package: Uuid,
    pub reference: String,
    pub value: String,
    pub position: Point,
    pub rotation: i32,
    pub layer: LayerId,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    pub uuid: Uuid,
    pub net: Uuid,
    pub from: Point,
    pub to: Point,
    pub width: i64,
    pub layer: LayerId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Via {
    pub uuid: Uuid,
    pub net: Uuid,
    pub position: Point,
    pub drill: i64,
    pub diameter: i64,
    pub from_layer: LayerId,
    pub to_layer: LayerId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zone {
    pub uuid: Uuid,
    pub net: Uuid,
    pub polygon: Polygon,
    pub layer: LayerId,
    pub priority: u32,
    pub thermal_relief: bool,
    pub thermal_gap: i64,
    pub thermal_spoke_width: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Net {
    pub uuid: Uuid,
    pub name: String,
    pub class: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub controlled_impedance: Option<ImpedanceSpec>,
}

impl Net {
    pub fn new(uuid: Uuid, name: impl Into<String>, class: Uuid) -> Self {
        Self {
            uuid,
            name: name.into(),
            class,
            controlled_impedance: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpedanceSpec {
    pub target_ohms: serde_json::Number,
    pub tolerance_pct: serde_json::Number,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub controlled_dielectric: Option<LayerId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetClass {
    pub uuid: Uuid,
    pub name: String,
    pub clearance: i64,
    pub track_width: i64,
    pub via_drill: i64,
    pub via_diameter: i64,
    pub diffpair_width: i64,
    pub diffpair_gap: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Keepout {
    pub uuid: Uuid,
    pub polygon: Polygon,
    pub layers: Vec<LayerId>,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dimension {
    pub uuid: Uuid,
    pub from: Point,
    pub to: Point,
    #[serde(default = "super::dimension::default_dimension_layer")]
    pub layer: LayerId,
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardText {
    pub uuid: Uuid,
    pub text: String,
    pub position: Point,
    pub rotation: i32,
    pub layer: LayerId,
    #[serde(default)]
    pub render_intent: TextRenderIntent,
    #[serde(default)]
    pub family: TextFamilyId,
    #[serde(default)]
    pub family_source: TextFamilySource,
    #[serde(default)]
    pub style: TextStyleId,
    #[serde(default = "super::text::default_board_text_height_nm")]
    pub height_nm: i64,
    #[serde(default = "super::text::default_board_text_stroke_width_nm")]
    pub stroke_width_nm: i64,
    #[serde(default = "default_board_text_h_align")]
    pub h_align: TextHAlign,
    #[serde(default = "default_board_text_v_align")]
    pub v_align: TextVAlign,
    #[serde(default)]
    pub mirrored: bool,
    #[serde(default)]
    pub keep_upright: bool,
    #[serde(default = "default_board_text_line_spacing_ratio_ppm")]
    pub line_spacing_ratio_ppm: i32,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub style_class: Option<String>,
}

fn default_board_text_h_align() -> TextHAlign {
    TextHAlign::Left
}

fn default_board_text_v_align() -> TextVAlign {
    TextVAlign::Bottom
}

fn default_board_text_line_spacing_ratio_ppm() -> i32 {
    1_000_000
}
