use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ir::geometry::Polygon;

use super::{ModelRef, Pad, Primitive};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Footprint {
    pub uuid: Uuid,
    pub name: String,
    pub package: Uuid,
    #[serde(default)]
    pub pads: HashMap<Uuid, Pad>,
    pub courtyard: Polygon,
    #[serde(default)]
    pub silkscreen: Vec<Primitive>,
    #[serde(default)]
    pub fab: Vec<Primitive>,
    #[serde(default)]
    pub assembly: Vec<Primitive>,
    #[serde(default)]
    pub mechanical: Vec<Primitive>,
    #[serde(default)]
    pub models_3d: Vec<ModelRef>,
    #[serde(default)]
    pub standards_basis: Option<String>,
    #[serde(default)]
    pub process_aperture_policy: Option<String>,
    #[serde(default)]
    pub tags: HashSet<String>,
}
