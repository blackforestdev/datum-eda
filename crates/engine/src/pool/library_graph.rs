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
    pub fn insert_pool_object(
        &mut self,
        kind: &str,
        object_id: Uuid,
        subject: impl Into<String>,
        value: serde_json::Value,
    ) -> Vec<LibraryGraphDiagnostic> {
        let subject = subject.into();
        let mut diagnostics = Vec::new();
        if let Some(previous) = self.seen.insert(object_id, subject.clone()) {
            diagnostics.push(LibraryGraphDiagnostic {
                severity: "error",
                code: "duplicate_uuid",
                subject: subject.clone(),
                message: format!("pool object UUID {object_id} already appeared at {previous}"),
            });
        }
        self.subjects.insert(object_id, subject);
        match kind {
            "units" => {
                self.units.insert(object_id, value);
            }
            "symbols" => {
                self.symbols.insert(object_id, value);
            }
            "entities" => {
                self.entities.insert(object_id, value);
            }
            "parts" => {
                self.parts.insert(object_id, value);
            }
            "packages" => {
                self.packages.insert(object_id, value);
            }
            "footprints" => {
                self.footprints.insert(object_id, value);
            }
            "padstacks" => {
                self.padstacks.insert(object_id, value);
            }
            "pin_pad_maps" => {
                self.pin_pad_maps.insert(object_id, value);
            }
            _ => diagnostics.push(LibraryGraphDiagnostic {
                severity: "error",
                code: "unknown_pool_kind",
                subject: object_id.to_string(),
                message: format!("pool object kind `{kind}` is not supported by LibraryGraph"),
            }),
        }
        diagnostics
    }

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
