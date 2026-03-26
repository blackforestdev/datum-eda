use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod api_types;
pub use api_types::*;
mod check_summary;
use check_summary::{drc_suggestion, erc_suggestion, summarize_diagnostics, summarize_schematic_checks};
mod ops_helpers;
use ops_helpers::*;
mod persistence_helpers;
use persistence_helpers::{
    deterministic_net_class_uuid, net_class_sidecar_payload, persist_net_class_sidecar,
    persist_package_assignment_sidecar, persist_part_assignment_sidecar, persist_rule_sidecar,
};
mod project_surface;
mod query_surface;
mod save_kicad;
mod write_ops;

use crate::board::{Airwire, Board, BoardNetInfo, BoardSummary, ComponentInfo, NetClass, StackupInfo};
use crate::connectivity;
use crate::drc::{self, DrcReport};
use crate::erc::{self, ErcConfig, ErcFinding};
use crate::error::EngineError;
use crate::import::{
    ImportKind, ImportReport, detect_import_kind, eagle, ids_sidecar, kicad,
    net_classes_sidecar, package_assignments_sidecar, part_assignments_sidecar, rules_sidecar,
};
use crate::ir::units::nm_to_mm;
use crate::pool::{PartSummary, Pool, PoolIndex};
use crate::rules::ast::Rule;
use crate::rules::ast::RuleType;
use crate::rules::validate::validate_rule;
use crate::schematic::{
    BusEntryInfo, BusInfo, CheckWaiver, ConnectivityDiagnosticInfo, HierarchyInfo, LabelInfo,
    NoConnectInfo, PortInfo, Schematic, SchematicNetInfo, SchematicSummary, SheetSummary,
    SymbolFieldInfo, SymbolInfo,
};

/// Public engine facade.
///
/// This is the only surface CLI, daemon, tests, and future GUI should
/// depend on. Internal modules remain implementation details behind it.
/// The full method set will grow milestone-by-milestone per
/// `specs/ENGINE_SPEC.md`.
pub struct Engine {
    pool: Pool,
    pool_index: PoolIndex,
    design: Option<Design>,
    imported_source: Option<ImportedDesignSource>,
    undo_stack: Vec<TransactionRecord>,
    redo_stack: Vec<TransactionRecord>,
    undo_depth: usize,
    redo_depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Design {
    pub board: Option<Board>,
    pub schematic: Option<Schematic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImportedDesignSource {
    kind: ImportKind,
    source_path: std::path::PathBuf,
    original_contents: String,
    loaded_rule_sidecar: bool,
    loaded_package_assignment_sidecar: bool,
    loaded_part_assignment_sidecar: bool,
    loaded_net_class_sidecar: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TransactionRecord {
    DeleteTrack {
        track: crate::board::Track,
    },
    DeleteVia {
        via: crate::board::Via,
    },
    DeleteComponent {
        package: crate::board::PlacedPackage,
        pads: Vec<crate::board::PlacedPad>,
    },
    MoveComponent {
        before: crate::board::PlacedPackage,
        after: crate::board::PlacedPackage,
        before_pads: Vec<(uuid::Uuid, crate::ir::geometry::Point)>,
        after_pads: Vec<(uuid::Uuid, crate::ir::geometry::Point)>,
    },
    RotateComponent {
        before: crate::board::PlacedPackage,
        after: crate::board::PlacedPackage,
        before_pads: Vec<(uuid::Uuid, crate::ir::geometry::Point)>,
        after_pads: Vec<(uuid::Uuid, crate::ir::geometry::Point)>,
    },
    SetValue {
        before: crate::board::PlacedPackage,
        after: crate::board::PlacedPackage,
    },
    SetReference {
        before: crate::board::PlacedPackage,
        after: crate::board::PlacedPackage,
    },
    AssignPart {
        before: crate::board::PlacedPackage,
        after: crate::board::PlacedPackage,
        before_pads: Vec<crate::board::PlacedPad>,
        after_pads: Vec<crate::board::PlacedPad>,
    },
    SetPackage {
        before: crate::board::PlacedPackage,
        after: crate::board::PlacedPackage,
        before_pads: Vec<crate::board::PlacedPad>,
        after_pads: Vec<crate::board::PlacedPad>,
    },
    SetNetClass {
        before_net: crate::board::Net,
        after_net: crate::board::Net,
        previous_class: Option<crate::board::NetClass>,
        current_class: crate::board::NetClass,
    },
    SetDesignRule {
        previous: Option<Rule>,
        current: Rule,
    },
    Batch {
        description: String,
        records: Vec<TransactionRecord>,
    },
}

impl Engine {
    /// Create an empty engine instance. Pool loading and project import are
    /// added as milestone work; this constructor intentionally stays simple.
    pub fn new() -> Result<Self, EngineError> {
        Ok(Self {
            pool: Pool::default(),
            pool_index: PoolIndex::open_in_memory()?,
            design: None,
            imported_source: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            undo_depth: 0,
            redo_depth: 0,
        })
    }

    /// Indicates whether the engine currently holds an in-memory design.
    pub fn has_open_project(&self) -> bool {
        self.design.is_some()
    }

    pub fn can_undo(&self) -> bool {
        self.undo_depth > 0
    }

    pub fn can_redo(&self) -> bool {
        self.redo_depth > 0
    }

    fn merge_pool(&mut self, imported: Pool) {
        self.pool.units.extend(imported.units);
        self.pool.symbols.extend(imported.symbols);
        self.pool.entities.extend(imported.entities);
        self.pool.padstacks.extend(imported.padstacks);
        self.pool.packages.extend(imported.packages);
        self.pool.parts.extend(imported.parts);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    mod goldens;
    mod read_surface;
    mod write_ops;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("testdata/import/kicad")
            .join(name)
    }

    fn unique_temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{}-{}", uuid::Uuid::new_v4(), name))
    }

    fn eagle_fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("testdata/import/eagle")
            .join(name)
    }
}
