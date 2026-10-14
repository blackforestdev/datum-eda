//! Typed board-scene geometry and display payloads shared by GUI consumers.

use crate::{PointNm, RectNm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardGraphicPrimitive {
    pub object_id: String,
    /// Always `"board_graphic"` — coarse selection/filtering vocabulary.
    pub object_kind: String,
    /// Fine-grained shape class: `"line"`, `"arc"`, `"polyline"`, or `"polygon"`.
    pub primitive_kind: String,
    pub source_object_uuid: String,
    pub layer_id: String,
    pub path: Vec<PointNm>,
    #[serde(default)]
    pub holes: Vec<Vec<PointNm>>,
    #[serde(default)]
    pub width_nm: Option<i64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnroutedPrimitive {
    pub object_id: String,
    pub object_kind: String,
    pub source_object_uuid: String,
    pub net_uuid: String,
    pub from_component: String,
    pub from_pin: String,
    pub to_component: String,
    pub to_pin: String,
    pub path: Vec<PointNm>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetDisplayEntry {
    pub net_uuid: String,
    pub net_name: String,
    pub airwire_color_rgb: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentGraphicPrimitive {
    pub graphic_id: String,
    pub component_uuid: String,
    pub layer_id: Option<String>,
    pub primitive_kind: String,
    pub render_role: String,
    pub width_nm: Option<i64>,
    pub closed: bool,
    pub path: Vec<PointNm>,
    #[serde(default)]
    pub holes: Vec<Vec<PointNm>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentTextPrimitive {
    pub text_id: String,
    pub component_uuid: String,
    pub layer_id: Option<String>,
    pub render_role: String,
    pub text: String,
    pub position: PointNm,
    pub rotation_degrees: f32,
    pub height_nm: i64,
    #[serde(default)]
    pub face_name: Option<String>,
    #[serde(default)]
    pub stroke_width_nm: Option<i64>,
    #[serde(default)]
    pub cached_polygons: Vec<Vec<PointNm>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneBounds {
    pub min_x: i64,
    pub min_y: i64,
    pub max_x: i64,
    pub max_y: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneLayer {
    pub layer_id: String,
    pub name: String,
    pub kind: String,
    pub render_order: u32,
    pub visible_by_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutlinePolyline {
    pub object_id: String,
    pub object_kind: String,
    pub source_object_uuid: String,
    #[serde(default = "default_outline_layer_id")]
    pub layer_id: String,
    pub path: Vec<PointNm>,
}

fn default_outline_layer_id() -> String {
    // Scene-level layer key for Edge.Cuts under the `L{n}` convention used by
    // `scene.layers` and the layer-visibility map. KiCad's Edge.Cuts is
    // canonically layer id 44; we use that as the default so existing fixture
    // JSON round-trips to a key that matches the visibility map.
    "L44".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentBounds {
    /// Existing authored Package lock; schematic symbols have no lock in S5A.
    #[serde(default)]
    pub locked: bool,
    pub object_id: String,
    pub object_kind: String,
    pub source_object_uuid: String,
    pub component_uuid: String,
    pub reference: String,
    pub value: Option<String>,
    pub placement_layer: String,
    pub position: PointNm,
    pub rotation_degrees: f32,
    pub bounds: RectNm,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackPrimitive {
    pub object_id: String,
    pub object_kind: String,
    pub source_object_uuid: String,
    pub track_uuid: String,
    pub net_uuid: Option<String>,
    pub layer_id: String,
    pub width_nm: i64,
    pub path: Vec<PointNm>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ViaPrimitive {
    pub object_id: String,
    pub object_kind: String,
    pub source_object_uuid: String,
    pub via_uuid: String,
    pub net_uuid: Option<String>,
    pub position: PointNm,
    pub drill_nm: i64,
    pub diameter_nm: i64,
    pub start_layer_id: String,
    pub end_layer_id: String,
}
