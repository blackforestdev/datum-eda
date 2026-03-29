use crate::ir::geometry::{LayerId, Point};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoardNetInfo {
    pub uuid: Uuid,
    pub name: String,
    pub class: String,
    pub pins: Vec<NetPinRef>,
    pub tracks: usize,
    pub vias: usize,
    pub zones: usize,
    pub routed_length_nm: i64,
    pub routed_pct: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub uuid: Uuid,
    pub package_uuid: Uuid,
    pub reference: String,
    pub value: String,
    pub position: Point,
    pub rotation: i32,
    pub layer: LayerId,
    pub locked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetPinRef {
    pub component: String,
    pub pin: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Airwire {
    pub net: Uuid,
    pub net_name: String,
    pub from: NetPinRef,
    pub to: NetPinRef,
    pub distance_nm: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardSummary {
    pub name: String,
    pub layer_count: usize,
    pub component_count: usize,
    pub net_count: usize,
}
