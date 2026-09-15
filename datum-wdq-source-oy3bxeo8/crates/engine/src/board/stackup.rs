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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dielectric_constant: Option<serde_json::Number>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loss_tangent: Option<serde_json::Number>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copper_weight_oz: Option<serde_json::Number>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roughness_um: Option<serde_json::Number>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material_name: Option<String>,
}

impl StackupLayer {
    pub fn new(
        id: LayerId,
        name: impl Into<String>,
        layer_type: StackupLayerType,
        thickness_nm: i64,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            layer_type,
            thickness_nm,
            dielectric_constant: None,
            loss_tangent: None,
            copper_weight_oz: None,
            roughness_um: None,
            material_name: None,
        }
    }
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
