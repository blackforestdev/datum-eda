use crate::ir::geometry::{LayerId, Point};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[rustfmt::skip]
pub enum PadShape { Circle, Rect, Oval, RoundRect }

#[rustfmt::skip]
impl Default for PadShape { fn default() -> Self { Self::Circle } }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PadExpansionSetup {
    #[serde(default)]
    pub pad_to_mask_clearance_nm: i64,
    #[serde(default)]
    pub pad_to_paste_clearance_nm: i64,
    #[serde(default)]
    pub pad_to_paste_ratio_ppm: i32,
    #[serde(default)]
    pub solder_mask_min_width_nm: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PadAperture {
    Circle { diameter_nm: i64 },
    Rect { width_nm: i64, height_nm: i64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacedPad {
    pub uuid: Uuid,
    pub package: Uuid,
    pub name: String,
    pub net: Option<Uuid>,
    pub position: Point,
    pub layer: LayerId,
    #[serde(default)]
    pub copper_layers: Vec<LayerId>,
    #[serde(default)]
    pub shape: PadShape,
    #[serde(default)]
    pub diameter: i64,
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64,
    #[serde(default)]
    pub drill: i64,
    #[serde(default)]
    pub rotation: i32,
    #[serde(default = "default_roundrect_rratio_ppm")]
    pub roundrect_rratio_ppm: u32,
    #[serde(default)]
    pub mask_layers: Vec<LayerId>,
    #[serde(default)]
    pub paste_layers: Vec<LayerId>,
    #[serde(default)]
    pub solder_mask_margin_nm: i64,
    #[serde(default)]
    pub solder_paste_margin_nm: i64,
    #[serde(default)]
    pub solder_paste_margin_ratio_ppm: i32,
}

fn default_roundrect_rratio_ppm() -> u32 {
    250_000
}

impl PlacedPad {
    pub fn aperture(&self) -> PadAperture {
        match self.shape {
            PadShape::Circle => PadAperture::Circle {
                diameter_nm: self.diameter,
            },
            PadShape::Rect | PadShape::Oval | PadShape::RoundRect => PadAperture::Rect {
                width_nm: self.width,
                height_nm: self.height,
            },
        }
    }
}
