// import module — see specs/ENGINE_SPEC.md

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub mod eagle;
pub mod ids_sidecar;
pub mod kicad;
pub mod net_classes_sidecar;
pub mod package_assignments_sidecar;
pub mod part_assignments_sidecar;
pub mod rules_sidecar;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportKind {
    EagleLibrary,
    KiCadBoard,
    KiCadSchematic,
    KiCadProject,
    EagleBoard,
    EagleSchematic,
}

impl ImportKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EagleLibrary => "eagle_library",
            Self::KiCadBoard => "kicad_board",
            Self::KiCadSchematic => "kicad_schematic",
            Self::KiCadProject => "kicad_project",
            Self::EagleBoard => "eagle_board",
            Self::EagleSchematic => "eagle_schematic",
        }
    }
}

pub fn detect_import_kind(path: &Path) -> Option<ImportKind> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("lbr") => Some(ImportKind::EagleLibrary),
        Some("kicad_pcb") => Some(ImportKind::KiCadBoard),
        Some("kicad_sch") => Some(ImportKind::KiCadSchematic),
        Some("kicad_pro") => Some(ImportKind::KiCadProject),
        Some("brd") => Some(ImportKind::EagleBoard),
        Some("sch") => Some(ImportKind::EagleSchematic),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ImportObjectCounts {
    pub units: usize,
    pub symbols: usize,
    pub entities: usize,
    pub padstacks: usize,
    pub packages: usize,
    pub parts: usize,
}

impl ImportObjectCounts {
    pub fn is_empty(&self) -> bool {
        self.units == 0
            && self.symbols == 0
            && self.entities == 0
            && self.padstacks == 0
            && self.packages == 0
            && self.parts == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportReport {
    pub kind: ImportKind,
    pub source: PathBuf,
    pub counts: ImportObjectCounts,
    pub warnings: Vec<String>,
    pub metadata: BTreeMap<String, String>,
}

impl ImportReport {
    pub fn new(kind: ImportKind, source: impl AsRef<Path>, counts: ImportObjectCounts) -> Self {
        Self {
            kind,
            source: source.as_ref().to_path_buf(),
            counts,
            warnings: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }

    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
#[path = "tests/mod_tests_import.rs"]
mod tests;
