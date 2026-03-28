use crate::ir::geometry::{LayerId, Point};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[rustfmt::skip]
pub enum PadShape { Circle, Rect }

#[rustfmt::skip]
impl Default for PadShape { fn default() -> Self { Self::Circle } }

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
    pub shape: PadShape,
    #[serde(default)]
    pub diameter: i64,
    #[serde(default)]
    pub width: i64,
    #[serde(default)]
    pub height: i64,
}

impl PlacedPad {
    pub fn aperture(&self) -> PadAperture {
        match self.shape {
            PadShape::Circle => PadAperture::Circle {
                diameter_nm: self.diameter,
            },
            PadShape::Rect => PadAperture::Rect {
                width_nm: self.width,
                height_nm: self.height,
            },
        }
    }
}
