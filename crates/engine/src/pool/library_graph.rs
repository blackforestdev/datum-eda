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
    pub registration_diagnostics: Vec<LibraryGraphDiagnostic>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LibraryGraphValidationTier {
    Registration,
    Shape,
    Dependency,
}

impl LibraryGraphValidationTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Registration => "registration",
            Self::Shape => "shape",
            Self::Dependency => "dependency",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LibraryGraphValidationSummary {
    pub diagnostics: usize,
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
    pub by_tier: BTreeMap<&'static str, usize>,
    pub by_code: BTreeMap<&'static str, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryGraphValidationReport {
    pub valid: bool,
    pub diagnostics: Vec<LibraryGraphDiagnostic>,
    pub summary: LibraryGraphValidationSummary,
    pub legacy_pin_pad_map_migration: LibraryGraphLegacyPinPadMapMigrationReport,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LibraryGraphLegacyPinPadMapMigrationReport {
    pub parts: usize,
    pub rows: usize,
    pub migratable_rows: usize,
    pub shadowed_rows: usize,
    pub blocked_rows: usize,
    pub migratable_subjects: Vec<String>,
    pub shadowed_subjects: Vec<String>,
    pub blocked_subjects: Vec<String>,
}

impl LibraryGraphDiagnostic {
    pub fn tier(&self) -> LibraryGraphValidationTier {
        match self.code {
            "duplicate_uuid" | "unknown_pool_kind" => LibraryGraphValidationTier::Registration,
            "invalid_uuid" | "invalid_uuid_key" | "missing_required_field" => {
                LibraryGraphValidationTier::Shape
            }
            _ => LibraryGraphValidationTier::Dependency,
        }
    }
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
                message: format!(
                    "pool object UUID {object_id} already appeared at {previous}; later object shadows the earlier registration"
                ),
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
        self.registration_diagnostics
            .extend(diagnostics.iter().cloned());
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

    pub fn validation_report(&self) -> LibraryGraphValidationReport {
        let mut diagnostics = self.registration_diagnostics.clone();
        diagnostics.extend(self.dependency_diagnostics());
        let legacy_pin_pad_map_migration = self.legacy_pin_pad_map_migration_report();
        diagnostics.extend(legacy_pin_pad_map_migration.diagnostics());
        let summary = LibraryGraphValidationSummary::from_diagnostics(&diagnostics);
        LibraryGraphValidationReport {
            valid: summary.errors == 0,
            diagnostics,
            summary,
            legacy_pin_pad_map_migration,
        }
    }
}

impl LibraryGraphValidationSummary {
    pub fn from_diagnostics(diagnostics: &[LibraryGraphDiagnostic]) -> Self {
        let mut summary = Self {
            diagnostics: diagnostics.len(),
            ..Self::default()
        };
        for diagnostic in diagnostics {
            match diagnostic.severity {
                "error" => summary.errors += 1,
                "warning" => summary.warnings += 1,
                "info" => summary.infos += 1,
                _ => {}
            }
            *summary
                .by_tier
                .entry(diagnostic.tier().as_str())
                .or_insert(0) += 1;
            *summary.by_code.entry(diagnostic.code).or_insert(0) += 1;
        }
        summary
    }
}
