use std::collections::BTreeMap;

use uuid::Uuid;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LibraryGraph {
    pub units: BTreeMap<Uuid, serde_json::Value>,
    pub symbols: BTreeMap<Uuid, serde_json::Value>,
    pub entities: BTreeMap<Uuid, serde_json::Value>,
    pub parts: BTreeMap<Uuid, serde_json::Value>,
    pub packages: BTreeMap<Uuid, serde_json::Value>,
    pub footprints: BTreeMap<Uuid, serde_json::Value>,
    pub padstacks: BTreeMap<Uuid, serde_json::Value>,
    pub pin_pad_maps: BTreeMap<Uuid, serde_json::Value>,
    pub model_blobs: BTreeMap<String, LibraryModelBlob>,
    pub seen: BTreeMap<Uuid, String>,
    pub subjects: BTreeMap<Uuid, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryModelBlob {
    pub model_uuid: Uuid,
}
