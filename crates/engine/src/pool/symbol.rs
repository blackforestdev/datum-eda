use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ir::geometry::Point;

use super::Primitive;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    pub uuid: Uuid,
    pub name: String,
    pub unit: Uuid,
    #[serde(default)]
    pub fields: Vec<LibrarySymbolField>,
    #[serde(default)]
    pub default_refdes_prefix: Option<String>,
    #[serde(default)]
    pub style_profile_assertions: Vec<String>,
    #[serde(default)]
    pub standards_basis: Option<String>,
    #[serde(default)]
    pub check_state: Option<LibraryCheckState>,
    #[serde(default)]
    pub provenance: Option<LibraryObjectProvenance>,
    #[serde(default)]
    pub drawings: Vec<Primitive>,
    #[serde(default)]
    pub pin_anchors: Vec<SymbolPinAnchor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibrarySymbolField {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub position: Option<Point>,
    #[serde(default)]
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryCheckState {
    pub status: LibraryCheckStatus,
    #[serde(default)]
    pub checked_at: Option<String>,
    #[serde(default)]
    pub checked_by: Option<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LibraryCheckStatus {
    #[default]
    Unchecked,
    Passed,
    Failed,
    NeedsReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LibraryObjectProvenance {
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub source_hash: Option<String>,
    #[serde(default)]
    pub reviewed_by: Option<String>,
    #[serde(default)]
    pub reviewed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolPinAnchor {
    pub pin: Uuid,
    pub position: Point,
    #[serde(default)]
    pub orientation: SymbolPinOrientation,
    #[serde(default)]
    pub length_nm: Option<i64>,
    #[serde(default)]
    pub decoration: SymbolPinDecoration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SymbolPinOrientation {
    Left,
    #[default]
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SymbolPinDecoration {
    #[default]
    None,
    Inverted,
    Clock,
    InvertedClock,
}
