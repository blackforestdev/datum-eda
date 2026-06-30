use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ir::geometry::LayerId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Padstack {
    pub uuid: Uuid,
    pub name: String,
    #[serde(default)]
    pub aperture: Option<PadstackAperture>,
    #[serde(default)]
    pub drill_nm: Option<i64>,
    #[serde(default)]
    pub plated: Option<bool>,
    #[serde(default)]
    pub layer_span: PadstackLayerSpan,
    #[serde(default)]
    pub mask_policy: PadstackMaskPolicy,
    #[serde(default)]
    pub paste_policy: PadstackPastePolicy,
    #[serde(default)]
    pub annular_ring_nm: Option<i64>,
    #[serde(default)]
    pub thermal: Option<PadstackThermal>,
    #[serde(default)]
    pub antipad: Option<PadstackAntipad>,
}

impl Padstack {
    pub fn new(uuid: Uuid, name: String) -> Self {
        Self {
            uuid,
            name,
            aperture: None,
            drill_nm: None,
            plated: None,
            layer_span: PadstackLayerSpan::default(),
            mask_policy: PadstackMaskPolicy::default(),
            paste_policy: PadstackPastePolicy::default(),
            annular_ring_nm: None,
            thermal: None,
            antipad: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PadstackAperture {
    Circle { diameter_nm: i64 },
    Rect { width_nm: i64, height_nm: i64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PadstackLayerSpan {
    #[default]
    PadLayer,
    Through,
    Blind {
        start_layer: LayerId,
        end_layer: LayerId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PadstackMaskPolicy {
    #[default]
    Inherit,
    Exposed,
    Tented,
    ExpansionNm(i64),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PadstackPastePolicy {
    #[default]
    Inherit,
    None,
    Aperture,
    ExpansionNm(i64),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PadstackThermal {
    #[serde(default)]
    pub spoke_count: Option<u8>,
    #[serde(default)]
    pub spoke_width_nm: Option<i64>,
    #[serde(default)]
    pub gap_nm: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PadstackAntipad {
    #[serde(default)]
    pub clearance_nm: Option<i64>,
    #[serde(default)]
    pub aperture: Option<PadstackAperture>,
}
