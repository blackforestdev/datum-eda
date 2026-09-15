use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ir::geometry::Polygon;

use super::{ModelRef, Pad, Primitive};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PackageBodyDimensions {
    #[serde(default)]
    pub x_nm: Option<i64>,
    #[serde(default)]
    pub y_nm: Option<i64>,
    #[serde(default)]
    pub z_nm: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageTerminal {
    pub uuid: Uuid,
    pub name: String,
    #[serde(default)]
    pub role: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    pub uuid: Uuid,
    pub name: String,
    #[serde(default)]
    pub package_family: Option<String>,
    #[serde(default)]
    pub package_code: Option<String>,
    #[serde(default)]
    pub mounting_type: Option<String>,
    #[serde(default)]
    pub body_dimensions: Option<PackageBodyDimensions>,
    #[serde(default)]
    pub terminals: HashMap<Uuid, PackageTerminal>,
    #[serde(default)]
    pub pads: HashMap<Uuid, Pad>,
    #[serde(default = "empty_polygon")]
    pub courtyard: Polygon,
    #[serde(default)]
    pub silkscreen: Vec<Primitive>,
    #[serde(default)]
    pub models_3d: Vec<ModelRef>,
    #[serde(default)]
    pub body_height_nm: Option<i64>,
    #[serde(default)]
    pub body_height_mounted_nm: Option<i64>,
    #[serde(default)]
    pub tags: HashSet<String>,
}

fn empty_polygon() -> Polygon {
    Polygon::new(Vec::new())
}
