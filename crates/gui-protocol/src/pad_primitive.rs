//! Pad scene transport and optional canonical native process geometry.
use crate::{PadCenter, RectNm};
pub use eda_engine::pool::process::EffectiveProcessAperture;
pub use eda_engine::substrate::component_pad_source::process::PlacedProcessGeometry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PadPrimitive {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_process: Option<PlacedProcessGeometry>,
    pub object_id: String,
    pub object_kind: String,
    pub source_object_uuid: String,
    pub pad_uuid: String,
    pub component_uuid: String,
    pub net_uuid: Option<String>,
    pub layer_id: String,
    #[serde(default)]
    pub copper_layer_ids: Vec<String>,
    pub center: PadCenter,
    pub bounds: RectNm,
    pub shape_kind: String,
    #[serde(default = "default_roundrect_rratio_ppm")]
    pub roundrect_rratio_ppm: u32,
    #[serde(default)]
    pub mask_layer_ids: Vec<String>,
    #[serde(default)]
    pub paste_layer_ids: Vec<String>,
    #[serde(default)]
    pub solder_mask_margin_nm: i64,
    #[serde(default)]
    pub solder_paste_margin_nm: i64,
    #[serde(default)]
    pub solder_paste_margin_ratio_ppm: i32,
    #[serde(default)]
    pub drill_nm: Option<i64>,
    #[serde(default)]
    pub rotation_degrees: f32,
}

pub(crate) fn default_roundrect_rratio_ppm() -> u32 {
    250_000
}
