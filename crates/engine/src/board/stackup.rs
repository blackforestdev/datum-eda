use crate::ir::geometry::LayerId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stackup {
    pub layers: Vec<StackupLayer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackupLayer {
    pub id: LayerId,
    pub name: String,
    pub layer_type: StackupLayerType,
    pub thickness_nm: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StackupLayerType {
    Copper,
    Dielectric,
    SolderMask,
    Silkscreen,
    Paste,
    Mechanical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackupInfo {
    pub layers: Vec<StackupLayer>,
}
