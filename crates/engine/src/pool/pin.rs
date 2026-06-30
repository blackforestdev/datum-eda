use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pin {
    pub uuid: Uuid,
    pub name: String,
    pub direction: PinDirection,
    pub swap_group: u32,
    pub alternates: Vec<AlternateName>,
}

pub type PinDirection = LibraryPinElectricalType;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LibraryPinElectricalType {
    Input,
    Output,
    Bidirectional,
    Passive,
    PowerIn,
    PowerOut,
    OpenCollector,
    OpenEmitter,
    TriState,
    NoConnect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlternateName {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    pub uuid: Uuid,
    pub name: String,
    pub manufacturer: String,
    pub pins: std::collections::HashMap<Uuid, Pin>,
    pub tags: HashSet<String>,
}
