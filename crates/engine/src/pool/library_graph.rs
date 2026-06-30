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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryGraphDiagnostic {
    pub severity: &'static str,
    pub code: &'static str,
    pub subject: String,
    pub message: String,
}

impl LibraryGraph {
    pub fn dependency_diagnostics(&self) -> Vec<LibraryGraphDiagnostic> {
        let mut diagnostics = Vec::new();
        self.validate_symbol_refs(&mut diagnostics);
        self.validate_entity_refs(&mut diagnostics);
        self.validate_part_refs(&mut diagnostics);
        self.validate_package_refs(&mut diagnostics);
        self.validate_footprint_refs(&mut diagnostics);
        self.validate_pin_pad_map_refs(&mut diagnostics);
        diagnostics
    }
}
