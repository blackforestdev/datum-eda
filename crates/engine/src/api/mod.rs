use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationRef {
    pub object_type: String,
    pub uuid: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OperationDiff {
    pub created: Vec<OperationRef>,
    pub modified: Vec<OperationRef>,
    pub deleted: Vec<OperationRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationResult {
    pub diff: OperationDiff,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetDesignRuleInput {
    pub rule_type: RuleType,
    pub scope: crate::rules::ast::RuleScope,
    pub parameters: crate::rules::ast::RuleParams,
    pub priority: u32,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveComponentInput {
    pub uuid: uuid::Uuid,
    pub position: crate::ir::geometry::Point,
    pub rotation: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RotateComponentInput {
    pub uuid: uuid::Uuid,
    pub rotation: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetValueInput {
    pub uuid: uuid::Uuid,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetReferenceInput {
    pub uuid: uuid::Uuid,
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignPartInput {
    pub uuid: uuid::Uuid,
    pub part_uuid: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetPackageInput {
    pub uuid: uuid::Uuid,
    pub package_uuid: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetNetClassInput {
    pub net_uuid: uuid::Uuid,
    pub class_name: String,
    pub clearance: i64,
    pub track_width: i64,
    pub via_drill: i64,
    pub via_diameter: i64,
    pub diffpair_width: i64,
    pub diffpair_gap: i64,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub enum CheckReport {
    Board {
        summary: CheckSummary,
        diagnostics: Vec<ConnectivityDiagnosticInfo>,
    },
    Schematic {
        summary: CheckSummary,
        diagnostics: Vec<ConnectivityDiagnosticInfo>,
        erc: Vec<ErcFinding>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Ok,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CheckSummary {
    pub status: CheckStatus,
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
    pub waived: usize,
    pub by_code: Vec<CheckCodeCount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CheckCodeCount {
    pub code: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetlistPin {
    pub component: String,
    pub pin: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetlistNet {
    pub uuid: uuid::Uuid,
    pub name: String,
    pub class: Option<String>,
    pub pins: Vec<NetlistPin>,
    pub routed_pct: Option<f32>,
    pub labels: Option<usize>,
    pub ports: Option<usize>,
    pub sheets: Option<Vec<String>>,
    pub semantic_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartGateDetail {
    pub name: String,
    pub pins: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartEntityDetail {
    pub name: String,
    pub prefix: String,
    pub gates: Vec<PartGateDetail>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartPackageDetail {
    pub uuid: uuid::Uuid,
    pub name: String,
    pub pads: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartLifecycle {
    Active,
    Nrnd,
    Eol,
    Obsolete,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartDetail {
    pub uuid: uuid::Uuid,
    pub mpn: String,
    pub manufacturer: String,
    pub value: String,
    pub description: String,
    pub datasheet: String,
    pub entity: PartEntityDetail,
    pub package: PartPackageDetail,
    pub parametric: BTreeMap<String, String>,
    pub lifecycle: PartLifecycle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackagePadDetail {
    pub name: String,
    pub x_mm: f64,
    pub y_mm: f64,
    pub layer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageCourtyardDetail {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageDetail {
    pub uuid: uuid::Uuid,
    pub name: String,
    pub pads: Vec<PackagePadDetail>,
    pub courtyard_mm: PackageCourtyardDetail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationDomain {
    Erc,
    Drc,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViolationObjectInfo {
    #[serde(rename = "type")]
    pub type_name: String,
    pub uuid: uuid::Uuid,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViolationExplanation {
    pub explanation: String,
    pub rule_detail: String,
    pub objects_involved: Vec<ViolationObjectInfo>,
    pub suggestion: String,
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

    /// M1 import dispatch. Recognizes supported design/library file kinds and
    /// routes them to the matching importer slice.
    pub fn import(&mut self, path: &Path) -> Result<ImportReport, EngineError> {
        match detect_import_kind(path) {
            Some(ImportKind::EagleLibrary) => self.import_eagle_library(path),
            Some(ImportKind::KiCadBoard) => {
                let (mut board, report) = kicad::import_board_document(path)?;
                let original_contents = std::fs::read_to_string(path)?;
                let source_hash =
                    ids_sidecar::compute_source_hash_bytes(original_contents.as_bytes());
                let rule_sidecar_path = rules_sidecar::sidecar_path_for_source(path);
                let loaded_rule_sidecar = if rule_sidecar_path.exists() {
                    match rules_sidecar::read_sidecar(&rule_sidecar_path) {
                        Ok(sidecar) if sidecar.source_hash == source_hash => {
                            board.rules = sidecar.rules;
                            true
                        }
                        Ok(_) => false,
                        Err(_) => false,
                    }
                } else {
                    false
                };
                self.design = Some(Design {
                    board: Some(board),
                    schematic: None,
                });
                let mut loaded_package_assignment_sidecar = false;
                let package_sidecar_path =
                    package_assignments_sidecar::sidecar_path_for_source(path);
                if package_sidecar_path.exists() {
                    match package_assignments_sidecar::read_sidecar(&package_sidecar_path) {
                        Ok(sidecar) if sidecar.source_hash == source_hash => {
                            if let Some(design) = self.design.as_mut()
                                && let Some(board) = design.board.as_mut()
                            {
                                for (component_uuid, package_uuid) in sidecar.assignments {
                                    if let Some(package) = board.packages.get_mut(&component_uuid) {
                                        package.package = package_uuid;
                                    }
                                }
                                loaded_package_assignment_sidecar = true;
                            }
                        }
                        Ok(_) => {}
                        Err(_) => {}
                    }
                }
                let mut loaded_part_assignment_sidecar = false;
                let part_sidecar_path = part_assignments_sidecar::sidecar_path_for_source(path);
                if part_sidecar_path.exists() {
                    match part_assignments_sidecar::read_sidecar(&part_sidecar_path) {
                        Ok(sidecar) if sidecar.source_hash == source_hash => {
                            if let Some(design) = self.design.as_mut()
                                && let Some(board) = design.board.as_mut()
                            {
                                for (component_uuid, part_uuid) in sidecar.assignments {
                                    if let Some(package) = board.packages.get_mut(&component_uuid) {
                                        package.part = part_uuid;
                                    }
                                }
                                loaded_part_assignment_sidecar = true;
                            }
                        }
                        Ok(_) => {}
                        Err(_) => {}
                    }
                }
                let mut loaded_net_class_sidecar = false;
                let net_class_sidecar_path = net_classes_sidecar::sidecar_path_for_source(path);
                if net_class_sidecar_path.exists() {
                    match net_classes_sidecar::read_sidecar(&net_class_sidecar_path) {
                        Ok(sidecar) if sidecar.source_hash == source_hash => {
                            if let Some(design) = self.design.as_mut()
                                && let Some(board) = design.board.as_mut()
                            {
                                for class in sidecar.classes {
                                    board.net_classes.insert(class.uuid, class);
                                }
                                for (net_uuid, class_uuid) in sidecar.assignments {
                                    if board.net_classes.contains_key(&class_uuid)
                                        && let Some(net) = board.nets.get_mut(&net_uuid)
                                    {
                                        net.class = class_uuid;
                                    }
                                }
                                loaded_net_class_sidecar = true;
                            }
                        }
                        Ok(_) => {}
                        Err(_) => {}
                    }
                }
                self.imported_source = Some(ImportedDesignSource {
                    kind: ImportKind::KiCadBoard,
                    source_path: path.to_path_buf(),
                    original_contents,
                    loaded_rule_sidecar,
                    loaded_package_assignment_sidecar,
                    loaded_part_assignment_sidecar,
                    loaded_net_class_sidecar,
                });
                self.undo_stack.clear();
                self.redo_stack.clear();
                self.undo_depth = 0;
                self.redo_depth = 0;
                Ok(report)
            }
            Some(ImportKind::KiCadSchematic) => {
                let (schematic, report) = kicad::import_schematic_document(path)?;
                let original_contents = std::fs::read_to_string(path)?;
                self.design = Some(Design {
                    board: None,
                    schematic: Some(schematic),
                });
                self.imported_source = Some(ImportedDesignSource {
                    kind: ImportKind::KiCadSchematic,
                    source_path: path.to_path_buf(),
                    original_contents,
                    loaded_rule_sidecar: false,
                    loaded_package_assignment_sidecar: false,
                    loaded_part_assignment_sidecar: false,
                    loaded_net_class_sidecar: false,
                });
                self.undo_stack.clear();
                self.redo_stack.clear();
                self.undo_depth = 0;
                self.redo_depth = 0;
                Ok(report)
            }
            Some(ImportKind::KiCadProject) => kicad::import_project_file(path),
            Some(ImportKind::EagleBoard) => eagle::import_board_file(path),
            Some(ImportKind::EagleSchematic) => eagle::import_schematic_file(path),
            None => Err(EngineError::Import(format!(
                "unsupported import path {}; expected .lbr, .kicad_pcb, .kicad_sch, .kicad_pro, .brd, or .sch",
                path.display()
            ))),
        }
    }

    /// M0 Eagle library import into the in-memory pool.
    pub fn import_eagle_library(&mut self, path: &Path) -> Result<ImportReport, EngineError> {
        let (imported, report) = eagle::import_library_file(path)?;
        self.merge_pool(imported);
        self.pool_index.rebuild_from_pool(&self.pool)?;
        Ok(report)
    }

    pub fn search_pool(&self, query: &str) -> Result<Vec<PartSummary>, EngineError> {
        Ok(self.pool_index.search_keyword(query)?)
    }

    pub fn get_part(&self, uuid: &uuid::Uuid) -> Result<PartDetail, EngineError> {
        let part = self.pool.parts.get(uuid).ok_or(EngineError::NotFound {
            object_type: "part",
            uuid: *uuid,
        })?;
        let entity =
            self.pool
                .entities
                .get(&part.entity)
                .ok_or(EngineError::DanglingReference {
                    source_type: "part",
                    source_uuid: part.uuid,
                    target_type: "entity",
                    target_uuid: part.entity,
                })?;
        let package =
            self.pool
                .packages
                .get(&part.package)
                .ok_or(EngineError::DanglingReference {
                    source_type: "part",
                    source_uuid: part.uuid,
                    target_type: "package",
                    target_uuid: part.package,
                })?;

        let mut gates: Vec<_> = entity
            .gates
            .values()
            .map(|gate| {
                let mut pins: Vec<String> = self
                    .pool
                    .units
                    .get(&gate.unit)
                    .map(|unit| unit.pins.values().map(|pin| pin.name.clone()).collect())
                    .unwrap_or_default();
                pins.sort();
                PartGateDetail {
                    name: gate.name.clone(),
                    pins,
                }
            })
            .collect();
        gates.sort_by(|a, b| a.name.cmp(&b.name));

        let mut parametric = BTreeMap::new();
        for (key, value) in &part.parametric {
            parametric.insert(key.clone(), value.clone());
        }

        Ok(PartDetail {
            uuid: part.uuid,
            mpn: part.mpn.clone(),
            manufacturer: part.manufacturer.clone(),
            value: part.value.clone(),
            description: part.description.clone(),
            datasheet: part.datasheet.clone(),
            entity: PartEntityDetail {
                name: entity.name.clone(),
                prefix: entity.prefix.clone(),
                gates,
            },
            package: PartPackageDetail {
                uuid: package.uuid,
                name: package.name.clone(),
                pads: package.pads.len(),
            },
            parametric,
            lifecycle: match part.lifecycle {
                crate::pool::Lifecycle::Active => PartLifecycle::Active,
                crate::pool::Lifecycle::Nrnd => PartLifecycle::Nrnd,
                crate::pool::Lifecycle::Eol => PartLifecycle::Eol,
                crate::pool::Lifecycle::Obsolete => PartLifecycle::Obsolete,
                crate::pool::Lifecycle::Unknown => PartLifecycle::Unknown,
            },
        })
    }

    pub fn get_package(&self, uuid: &uuid::Uuid) -> Result<PackageDetail, EngineError> {
        let package = self.pool.packages.get(uuid).ok_or(EngineError::NotFound {
            object_type: "package",
            uuid: *uuid,
        })?;
        let mut pads: Vec<_> = package
            .pads
            .values()
            .map(|pad| PackagePadDetail {
                name: pad.name.clone(),
                x_mm: nm_to_mm(pad.position.x),
                y_mm: nm_to_mm(pad.position.y),
                layer: pad.layer.to_string(),
            })
            .collect();
        pads.sort_by(|a, b| a.name.cmp(&b.name));
        let courtyard = package
            .courtyard
            .bounding_box()
            .map(|bbox| PackageCourtyardDetail {
                width: nm_to_mm(bbox.width()),
                height: nm_to_mm(bbox.height()),
            })
            .unwrap_or(PackageCourtyardDetail {
                width: 0.0,
                height: 0.0,
            });
        Ok(PackageDetail {
            uuid: package.uuid,
            name: package.name.clone(),
            pads,
            courtyard_mm: courtyard,
        })
    }

    pub fn close_project(&mut self) {
        self.design = None;
        self.imported_source = None;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.undo_depth = 0;
        self.redo_depth = 0;
    }

    pub fn explain_violation(
        &self,
        domain: ViolationDomain,
        index: usize,
    ) -> Result<ViolationExplanation, EngineError> {
        match domain {
            ViolationDomain::Erc => {
                let findings = self.run_erc_prechecks()?;
                let finding = findings.get(index).ok_or_else(|| {
                    EngineError::Validation(format!(
                        "erc finding index {index} is out of range ({} findings)",
                        findings.len()
                    ))
                })?;
                let objects_involved = finding
                    .object_uuids
                    .iter()
                    .enumerate()
                    .map(|(i, uuid)| {
                        let descriptor = finding.objects.get(i);
                        ViolationObjectInfo {
                            type_name: descriptor
                                .map(|obj| obj.kind.to_string())
                                .unwrap_or_else(|| "object".to_string()),
                            uuid: *uuid,
                            description: descriptor
                                .map(|obj| obj.key.clone())
                                .unwrap_or_else(|| uuid.to_string()),
                        }
                    })
                    .collect();
                Ok(ViolationExplanation {
                    explanation: finding.message.clone(),
                    rule_detail: format!("erc {} ({:?})", finding.code, finding.severity),
                    objects_involved,
                    suggestion: erc_suggestion(finding.code).to_string(),
                })
            }
            ViolationDomain::Drc => {
                let report = self.run_drc(&[
                    RuleType::Connectivity,
                    RuleType::ClearanceCopper,
                    RuleType::TrackWidth,
                    RuleType::ViaHole,
                    RuleType::ViaAnnularRing,
                    RuleType::SilkClearance,
                ])?;
                let violation = report.violations.get(index).ok_or_else(|| {
                    EngineError::Validation(format!(
                        "drc violation index {index} is out of range ({} violations)",
                        report.violations.len()
                    ))
                })?;
                let objects_involved = violation
                    .objects
                    .iter()
                    .map(|uuid| ViolationObjectInfo {
                        type_name: "board_object".to_string(),
                        uuid: *uuid,
                        description: uuid.to_string(),
                    })
                    .collect();
                Ok(ViolationExplanation {
                    explanation: violation.message.clone(),
                    rule_detail: format!(
                        "drc {} ({:?}, {:?})",
                        violation.code, violation.rule_type, violation.severity
                    ),
                    objects_involved,
                    suggestion: drc_suggestion(&violation.code).to_string(),
                })
            }
        }
    }

    pub fn get_board_summary(&self) -> Result<BoardSummary, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_ref().ok_or_else(|| EngineError::NotFound {
            object_type: "board",
            uuid: uuid::Uuid::nil(),
        })?;
        Ok(board.summary())
    }

    pub fn get_components(&self) -> Result<Vec<ComponentInfo>, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_ref().ok_or_else(|| EngineError::NotFound {
            object_type: "board",
            uuid: uuid::Uuid::nil(),
        })?;
        Ok(board.components())
    }

    pub fn get_net_info(&self) -> Result<Vec<BoardNetInfo>, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_ref().ok_or_else(|| EngineError::NotFound {
            object_type: "board",
            uuid: uuid::Uuid::nil(),
        })?;
        Ok(board.net_info())
    }

    pub fn get_stackup(&self) -> Result<StackupInfo, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_ref().ok_or_else(|| EngineError::NotFound {
            object_type: "board",
            uuid: uuid::Uuid::nil(),
        })?;
        Ok(board.stackup_info())
    }

    pub fn get_unrouted(&self) -> Result<Vec<Airwire>, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_ref().ok_or_else(|| EngineError::NotFound {
            object_type: "board",
            uuid: uuid::Uuid::nil(),
        })?;
        Ok(board.unrouted())
    }

    pub fn get_schematic_summary(&self) -> Result<SchematicSummary, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "schematic",
                uuid: uuid::Uuid::nil(),
            })?;
        Ok(schematic.summary())
    }

    pub fn get_sheets(&self) -> Result<Vec<SheetSummary>, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "schematic",
                uuid: uuid::Uuid::nil(),
            })?;
        Ok(schematic.sheet_summaries())
    }

    pub fn get_labels(&self, sheet: Option<&uuid::Uuid>) -> Result<Vec<LabelInfo>, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "schematic",
                uuid: uuid::Uuid::nil(),
            })?;
        Ok(schematic.labels(sheet))
    }

    pub fn get_symbols(&self, sheet: Option<&uuid::Uuid>) -> Result<Vec<SymbolInfo>, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "schematic",
                uuid: uuid::Uuid::nil(),
            })?;
        Ok(schematic.symbols(sheet))
    }

    pub fn get_symbol_fields(
        &self,
        symbol_uuid: &uuid::Uuid,
    ) -> Result<Vec<SymbolFieldInfo>, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "schematic",
                uuid: uuid::Uuid::nil(),
            })?;
        for sheet in schematic.sheets.values() {
            if let Some(symbol) = sheet.symbols.get(symbol_uuid) {
                let mut fields: Vec<_> = symbol
                    .fields
                    .iter()
                    .map(|field| SymbolFieldInfo {
                        uuid: field.uuid,
                        symbol: symbol.uuid,
                        key: field.key.clone(),
                        value: field.value.clone(),
                        visible: field.visible,
                        position: field.position,
                    })
                    .collect();
                fields.sort_by(|a, b| a.key.cmp(&b.key).then_with(|| a.uuid.cmp(&b.uuid)));
                return Ok(fields);
            }
        }
        Err(EngineError::NotFound {
            object_type: "symbol",
            uuid: *symbol_uuid,
        })
    }

    pub fn get_ports(&self, sheet: Option<&uuid::Uuid>) -> Result<Vec<PortInfo>, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "schematic",
                uuid: uuid::Uuid::nil(),
            })?;
        Ok(schematic.ports(sheet))
    }

    pub fn get_buses(&self, sheet: Option<&uuid::Uuid>) -> Result<Vec<BusInfo>, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "schematic",
                uuid: uuid::Uuid::nil(),
            })?;
        Ok(schematic.buses(sheet))
    }

    pub fn get_bus_entries(
        &self,
        sheet: Option<&uuid::Uuid>,
    ) -> Result<Vec<BusEntryInfo>, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "schematic",
                uuid: uuid::Uuid::nil(),
            })?;
        Ok(schematic.bus_entries(sheet))
    }

    pub fn get_noconnects(
        &self,
        sheet: Option<&uuid::Uuid>,
    ) -> Result<Vec<NoConnectInfo>, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "schematic",
                uuid: uuid::Uuid::nil(),
            })?;
        Ok(schematic.noconnects(sheet))
    }

    pub fn get_hierarchy(&self) -> Result<HierarchyInfo, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "schematic",
                uuid: uuid::Uuid::nil(),
            })?;
        Ok(schematic.hierarchy())
    }

    pub fn get_schematic_net_info(&self) -> Result<Vec<SchematicNetInfo>, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "schematic",
                uuid: uuid::Uuid::nil(),
            })?;
        Ok(connectivity::schematic_net_info(schematic))
    }

    pub fn get_netlist(&self) -> Result<Vec<NetlistNet>, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        if let Some(board) = design.board.as_ref() {
            return Ok(board
                .net_info()
                .into_iter()
                .map(|net| NetlistNet {
                    uuid: net.uuid,
                    name: net.name,
                    class: Some(net.class),
                    pins: net
                        .pins
                        .into_iter()
                        .map(|pin| NetlistPin {
                            component: pin.component,
                            pin: pin.pin,
                        })
                        .collect(),
                    routed_pct: Some(net.routed_pct),
                    labels: None,
                    ports: None,
                    sheets: None,
                    semantic_class: None,
                })
                .collect());
        }

        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "design",
                uuid: uuid::Uuid::nil(),
            })?;
        Ok(connectivity::schematic_net_info(schematic)
            .into_iter()
            .map(|net| NetlistNet {
                uuid: net.uuid,
                name: net.name,
                class: net.class,
                pins: net
                    .pins
                    .into_iter()
                    .map(|pin| NetlistPin {
                        component: pin.component,
                        pin: pin.pin,
                    })
                    .collect(),
                routed_pct: None,
                labels: Some(net.labels),
                ports: Some(net.ports),
                sheets: Some(net.sheets),
                semantic_class: net.semantic_class,
            })
            .collect())
    }

    pub fn get_connectivity_diagnostics(
        &self,
    ) -> Result<Vec<ConnectivityDiagnosticInfo>, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        if let Some(board) = design.board.as_ref() {
            return Ok(board.diagnostics());
        }
        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "design",
                uuid: uuid::Uuid::nil(),
            })?;
        Ok(connectivity::schematic_diagnostics(schematic))
    }

    pub fn get_check_report(&self) -> Result<CheckReport, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        if let Some(board) = design.board.as_ref() {
            let diagnostics = board.diagnostics();
            return Ok(CheckReport::Board {
                summary: summarize_diagnostics(&diagnostics),
                diagnostics,
            });
        }
        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "design",
                uuid: uuid::Uuid::nil(),
            })?;
        let diagnostics = connectivity::schematic_diagnostics(schematic);
        let erc = erc::run_prechecks(schematic);
        Ok(CheckReport::Schematic {
            summary: summarize_schematic_checks(&diagnostics, &erc),
            diagnostics,
            erc,
        })
    }

    pub fn run_erc_prechecks(&self) -> Result<Vec<ErcFinding>, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "schematic",
                uuid: uuid::Uuid::nil(),
            })?;
        Ok(erc::run_prechecks(schematic))
    }

    pub fn run_erc_prechecks_with_config(
        &self,
        config: &ErcConfig,
    ) -> Result<Vec<ErcFinding>, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "schematic",
                uuid: uuid::Uuid::nil(),
            })?;
        Ok(erc::run_prechecks_with_config(schematic, config))
    }

    pub fn run_erc_prechecks_with_config_and_waivers(
        &self,
        config: &ErcConfig,
        waivers: &[CheckWaiver],
    ) -> Result<Vec<ErcFinding>, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let schematic = design
            .schematic
            .as_ref()
            .ok_or_else(|| EngineError::NotFound {
                object_type: "schematic",
                uuid: uuid::Uuid::nil(),
            })?;

        let mut effective_waivers = schematic.waivers.clone();
        effective_waivers.extend_from_slice(waivers);

        Ok(erc::run_prechecks_with_config_and_waivers(
            schematic,
            config,
            &effective_waivers,
        ))
    }

    pub fn run_drc(&self, rule_types: &[RuleType]) -> Result<DrcReport, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_ref().ok_or_else(|| EngineError::NotFound {
            object_type: "board",
            uuid: uuid::Uuid::nil(),
        })?;
        Ok(drc::run(board, rule_types))
    }

    pub fn get_design_rules(&self) -> Result<Vec<Rule>, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_ref().ok_or_else(|| EngineError::NotFound {
            object_type: "board",
            uuid: uuid::Uuid::nil(),
        })?;
        Ok(board.rules.clone())
    }

    pub fn delete_track(&mut self, uuid: &uuid::Uuid) -> Result<OperationResult, EngineError> {
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "delete_track is currently implemented only for board projects".to_string(),
            )
        })?;
        let track = board.tracks.remove(uuid).ok_or(EngineError::NotFound {
            object_type: "track",
            uuid: *uuid,
        })?;

        self.undo_stack.push(TransactionRecord::DeleteTrack {
            track: track.clone(),
        });
        self.redo_stack.clear();
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = 0;

        Ok(OperationResult {
            diff: OperationDiff {
                created: Vec::new(),
                modified: Vec::new(),
                deleted: vec![OperationRef {
                    object_type: "track".to_string(),
                    uuid: *uuid,
                }],
            },
            description: format!("delete_track {}", uuid),
        })
    }

    pub fn delete_via(&mut self, uuid: &uuid::Uuid) -> Result<OperationResult, EngineError> {
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "delete_via is currently implemented only for board projects".to_string(),
            )
        })?;
        let via = board.vias.remove(uuid).ok_or(EngineError::NotFound {
            object_type: "via",
            uuid: *uuid,
        })?;

        self.undo_stack
            .push(TransactionRecord::DeleteVia { via: via.clone() });
        self.redo_stack.clear();
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = 0;

        Ok(OperationResult {
            diff: OperationDiff {
                created: Vec::new(),
                modified: Vec::new(),
                deleted: vec![OperationRef {
                    object_type: "via".to_string(),
                    uuid: *uuid,
                }],
            },
            description: format!("delete_via {}", uuid),
        })
    }

    pub fn delete_component(&mut self, uuid: &uuid::Uuid) -> Result<OperationResult, EngineError> {
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "delete_component is currently implemented only for board projects".to_string(),
            )
        })?;
        let package = board.packages.remove(uuid).ok_or(EngineError::NotFound {
            object_type: "component",
            uuid: *uuid,
        })?;
        let pad_uuids: Vec<_> = board
            .pads
            .values()
            .filter(|pad| pad.package == *uuid)
            .map(|pad| pad.uuid)
            .collect();
        let mut pads = Vec::with_capacity(pad_uuids.len());
        for pad_uuid in pad_uuids {
            let pad = board.pads.remove(&pad_uuid).ok_or(EngineError::NotFound {
                object_type: "pad",
                uuid: pad_uuid,
            })?;
            pads.push(pad);
        }
        pads.sort_by_key(|pad| pad.uuid);

        self.undo_stack.push(TransactionRecord::DeleteComponent {
            package: package.clone(),
            pads,
        });
        self.redo_stack.clear();
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = 0;

        Ok(OperationResult {
            diff: OperationDiff {
                created: Vec::new(),
                modified: Vec::new(),
                deleted: vec![OperationRef {
                    object_type: "component".to_string(),
                    uuid: *uuid,
                }],
            },
            description: format!("delete_component {}", uuid),
        })
    }

    pub fn move_component(
        &mut self,
        input: MoveComponentInput,
    ) -> Result<OperationResult, EngineError> {
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "move_component is currently implemented only for board projects".to_string(),
            )
        })?;

        let before = board
            .packages
            .get(&input.uuid)
            .cloned()
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: input.uuid,
            })?;
        let after = crate::board::PlacedPackage {
            position: input.position,
            rotation: input.rotation.unwrap_or(before.rotation),
            ..before.clone()
        };
        let (before_pads, after_pads) = apply_package_transform(board, &before, &after)?;

        self.undo_stack.push(TransactionRecord::MoveComponent {
            before: before.clone(),
            after: after.clone(),
            before_pads,
            after_pads,
        });
        self.redo_stack.clear();
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = 0;

        Ok(OperationResult {
            diff: OperationDiff {
                created: Vec::new(),
                modified: vec![OperationRef {
                    object_type: "component".to_string(),
                    uuid: input.uuid,
                }],
                deleted: Vec::new(),
            },
            description: format!("move_component {}", input.uuid),
        })
    }

    pub fn rotate_component(
        &mut self,
        input: RotateComponentInput,
    ) -> Result<OperationResult, EngineError> {
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "rotate_component is currently implemented only for board projects".to_string(),
            )
        })?;

        let before = board
            .packages
            .get(&input.uuid)
            .cloned()
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: input.uuid,
            })?;
        let after = crate::board::PlacedPackage {
            rotation: input.rotation,
            ..before.clone()
        };
        let (before_pads, after_pads) = apply_package_transform(board, &before, &after)?;

        self.undo_stack.push(TransactionRecord::RotateComponent {
            before: before.clone(),
            after: after.clone(),
            before_pads,
            after_pads,
        });
        self.redo_stack.clear();
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = 0;

        Ok(OperationResult {
            diff: OperationDiff {
                created: Vec::new(),
                modified: vec![OperationRef {
                    object_type: "component".to_string(),
                    uuid: input.uuid,
                }],
                deleted: Vec::new(),
            },
            description: format!("rotate_component {}", input.uuid),
        })
    }

    pub fn set_value(&mut self, input: SetValueInput) -> Result<OperationResult, EngineError> {
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "set_value is currently implemented only for board projects".to_string(),
            )
        })?;

        let before = board
            .packages
            .get(&input.uuid)
            .cloned()
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: input.uuid,
            })?;
        let package = board
            .packages
            .get_mut(&input.uuid)
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: input.uuid,
            })?;
        package.value = input.value;
        let after = package.clone();

        self.undo_stack.push(TransactionRecord::SetValue {
            before: before.clone(),
            after: after.clone(),
        });
        self.redo_stack.clear();
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = 0;

        Ok(OperationResult {
            diff: OperationDiff {
                created: Vec::new(),
                modified: vec![OperationRef {
                    object_type: "component".to_string(),
                    uuid: input.uuid,
                }],
                deleted: Vec::new(),
            },
            description: format!("set_value {}", input.uuid),
        })
    }

    pub fn set_reference(
        &mut self,
        input: SetReferenceInput,
    ) -> Result<OperationResult, EngineError> {
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "set_reference is currently implemented only for board projects".to_string(),
            )
        })?;

        let before = board
            .packages
            .get(&input.uuid)
            .cloned()
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: input.uuid,
            })?;
        let package = board
            .packages
            .get_mut(&input.uuid)
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: input.uuid,
            })?;
        package.reference = input.reference;
        let after = package.clone();

        self.undo_stack.push(TransactionRecord::SetReference {
            before: before.clone(),
            after: after.clone(),
        });
        self.redo_stack.clear();
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = 0;

        Ok(OperationResult {
            diff: OperationDiff {
                created: Vec::new(),
                modified: vec![OperationRef {
                    object_type: "component".to_string(),
                    uuid: input.uuid,
                }],
                deleted: Vec::new(),
            },
            description: format!("set_reference {}", input.uuid),
        })
    }

    pub fn assign_part(&mut self, input: AssignPartInput) -> Result<OperationResult, EngineError> {
        let part = self.pool.parts.get(&input.part_uuid).ok_or(EngineError::NotFound {
            object_type: "part",
            uuid: input.part_uuid,
        })?;
        let target_package = self
            .pool
            .packages
            .get(&part.package)
            .ok_or(EngineError::DanglingReference {
                source_type: "part",
                source_uuid: input.part_uuid,
                target_type: "package",
                target_uuid: part.package,
            })?;
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "assign_part is currently implemented only for board projects".to_string(),
            )
        })?;

        let before = board
            .packages
            .get(&input.uuid)
            .cloned()
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: input.uuid,
            })?;
        let before_pads = component_pads(board, input.uuid);
        let package = board
            .packages
            .get_mut(&input.uuid)
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: input.uuid,
            })?;
        package.part = input.part_uuid;
        package.package = part.package;
        package.value = part.value.clone();
        let after = package.clone();
        replace_component_pads_from_pool_package(board, &after, target_package)?;
        let after_pads = component_pads(board, input.uuid);

        self.undo_stack.push(TransactionRecord::AssignPart {
            before: before.clone(),
            after: after.clone(),
            before_pads,
            after_pads,
        });
        self.redo_stack.clear();
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = 0;

        Ok(OperationResult {
            diff: OperationDiff {
                created: Vec::new(),
                modified: vec![OperationRef {
                    object_type: "component".to_string(),
                    uuid: input.uuid,
                }],
                deleted: Vec::new(),
            },
            description: format!("assign_part {}", input.uuid),
        })
    }

    pub fn set_package(
        &mut self,
        input: SetPackageInput,
    ) -> Result<OperationResult, EngineError> {
        let target_package = self
            .pool
            .packages
            .get(&input.package_uuid)
            .ok_or(EngineError::NotFound {
                object_type: "package",
                uuid: input.package_uuid,
            })?;
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "set_package is currently implemented only for board projects".to_string(),
            )
        })?;

        let before = board
            .packages
            .get(&input.uuid)
            .cloned()
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: input.uuid,
            })?;
        let before_pads = component_pads(board, input.uuid);
        let package = board
            .packages
            .get_mut(&input.uuid)
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: input.uuid,
            })?;
        package.package = input.package_uuid;
        if package.part != Uuid::nil()
            && self
                .pool
                .parts
                .get(&package.part)
                .is_some_and(|part| part.package != input.package_uuid)
        {
            package.part = Uuid::nil();
        }
        let after = package.clone();
        replace_component_pads_from_pool_package(board, &after, target_package)?;
        let after_pads = component_pads(board, input.uuid);

        self.undo_stack.push(TransactionRecord::SetPackage {
            before: before.clone(),
            after: after.clone(),
            before_pads,
            after_pads,
        });
        self.redo_stack.clear();
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = 0;

        Ok(OperationResult {
            diff: OperationDiff {
                created: Vec::new(),
                modified: vec![OperationRef {
                    object_type: "component".to_string(),
                    uuid: input.uuid,
                }],
                deleted: Vec::new(),
            },
            description: format!("set_package {}", input.uuid),
        })
    }

    pub fn set_net_class(
        &mut self,
        input: SetNetClassInput,
    ) -> Result<OperationResult, EngineError> {
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "set_net_class is currently implemented only for board projects".to_string(),
            )
        })?;

        let before_net = board
            .nets
            .get(&input.net_uuid)
            .cloned()
            .ok_or(EngineError::NotFound {
                object_type: "net",
                uuid: input.net_uuid,
            })?;
        let target_class_uuid = if before_net.class != Uuid::nil() {
            before_net.class
        } else {
            deterministic_net_class_uuid(input.net_uuid, &input.class_name)
        };
        let previous_class = board.net_classes.get(&target_class_uuid).cloned();
        let current_class = NetClass {
            uuid: target_class_uuid,
            name: input.class_name,
            clearance: input.clearance,
            track_width: input.track_width,
            via_drill: input.via_drill,
            via_diameter: input.via_diameter,
            diffpair_width: input.diffpair_width,
            diffpair_gap: input.diffpair_gap,
        };
        board.net_classes.insert(target_class_uuid, current_class.clone());
        let net = board
            .nets
            .get_mut(&input.net_uuid)
            .ok_or(EngineError::NotFound {
                object_type: "net",
                uuid: input.net_uuid,
            })?;
        net.class = target_class_uuid;
        let after_net = net.clone();

        self.undo_stack.push(TransactionRecord::SetNetClass {
            before_net: before_net.clone(),
            after_net: after_net.clone(),
            previous_class: previous_class.clone(),
            current_class: current_class.clone(),
        });
        self.redo_stack.clear();
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = 0;

        Ok(OperationResult {
            diff: OperationDiff {
                created: if previous_class.is_none() {
                    vec![OperationRef {
                        object_type: "net_class".to_string(),
                        uuid: current_class.uuid,
                    }]
                } else {
                    Vec::new()
                },
                modified: vec![OperationRef {
                    object_type: "net".to_string(),
                    uuid: input.net_uuid,
                }],
                deleted: Vec::new(),
            },
            description: format!("set_net_class {}", input.net_uuid),
        })
    }

    pub fn set_design_rule(
        &mut self,
        input: SetDesignRuleInput,
    ) -> Result<OperationResult, EngineError> {
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "set_design_rule is currently implemented only for board projects".to_string(),
            )
        })?;

        let rule_key = (
            input.name.clone(),
            input.rule_type.clone(),
            input.scope.clone(),
        );
        let existing_index = board.rules.iter().position(|rule| {
            (
                Some(rule.name.clone()),
                rule.rule_type.clone(),
                rule.scope.clone(),
            ) == rule_key
                || (rule.name == default_rule_name(&input.rule_type)
                    && input.name.is_none()
                    && rule.rule_type == input.rule_type
                    && rule.scope == input.scope)
        });

        let rule = Rule {
            uuid: existing_index
                .map(|index| board.rules[index].uuid)
                .unwrap_or_else(uuid::Uuid::new_v4),
            name: input
                .name
                .clone()
                .unwrap_or_else(|| default_rule_name(&input.rule_type)),
            scope: input.scope,
            priority: input.priority,
            enabled: true,
            rule_type: input.rule_type,
            parameters: input.parameters,
        };
        validate_rule(&rule)?;

        let previous = existing_index.map(|index| board.rules[index].clone());
        if let Some(index) = existing_index {
            board.rules[index] = rule.clone();
        } else {
            board.rules.push(rule.clone());
        }
        board.rules.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| a.name.cmp(&b.name))
                .then_with(|| a.uuid.cmp(&b.uuid))
        });

        self.undo_stack.push(TransactionRecord::SetDesignRule {
            previous: previous.clone(),
            current: rule.clone(),
        });
        self.redo_stack.clear();
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = 0;

        Ok(OperationResult {
            diff: OperationDiff {
                created: if previous.is_none() {
                    vec![OperationRef {
                        object_type: "rule".to_string(),
                        uuid: rule.uuid,
                    }]
                } else {
                    Vec::new()
                },
                modified: if previous.is_some() {
                    vec![OperationRef {
                        object_type: "rule".to_string(),
                        uuid: rule.uuid,
                    }]
                } else {
                    Vec::new()
                },
                deleted: Vec::new(),
            },
            description: format!("set_design_rule {}", rule.uuid),
        })
    }

    pub fn undo(&mut self) -> Result<OperationResult, EngineError> {
        let transaction = self.undo_stack.pop().ok_or(EngineError::NothingToUndo)?;
        let result = match &transaction {
            TransactionRecord::DeleteTrack { track } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                board.tracks.insert(track.uuid, track.clone());
                OperationResult {
                    diff: OperationDiff {
                        created: vec![OperationRef {
                            object_type: "track".to_string(),
                            uuid: track.uuid,
                        }],
                        modified: Vec::new(),
                        deleted: Vec::new(),
                    },
                    description: format!("undo delete_track {}", track.uuid),
                }
            }
            TransactionRecord::DeleteVia { via } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                board.vias.insert(via.uuid, via.clone());
                OperationResult {
                    diff: OperationDiff {
                        created: vec![OperationRef {
                            object_type: "via".to_string(),
                            uuid: via.uuid,
                        }],
                        modified: Vec::new(),
                        deleted: Vec::new(),
                    },
                    description: format!("undo delete_via {}", via.uuid),
                }
            }
            TransactionRecord::DeleteComponent { package, pads } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                board.packages.insert(package.uuid, package.clone());
                for pad in pads {
                    board.pads.insert(pad.uuid, pad.clone());
                }
                OperationResult {
                    diff: OperationDiff {
                        created: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: package.uuid,
                        }],
                        modified: Vec::new(),
                        deleted: Vec::new(),
                    },
                    description: format!("undo delete_component {}", package.uuid),
                }
            }
            TransactionRecord::MoveComponent {
                before,
                after,
                before_pads,
                after_pads: _,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                restore_package_transform(board, after.uuid, before.clone(), before_pads)?;
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("undo move_component {}", after.uuid),
                }
            }
            TransactionRecord::RotateComponent {
                before,
                after,
                before_pads,
                after_pads: _,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                restore_package_transform(board, after.uuid, before.clone(), before_pads)?;
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("undo rotate_component {}", after.uuid),
                }
            }
            TransactionRecord::SetDesignRule { previous, current } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                match previous {
                    Some(previous_rule) => {
                        let rule = board
                            .rules
                            .iter_mut()
                            .find(|rule| rule.uuid == current.uuid)
                            .ok_or(EngineError::NotFound {
                                object_type: "rule",
                                uuid: current.uuid,
                            })?;
                        *rule = previous_rule.clone();
                        OperationResult {
                            diff: OperationDiff {
                                created: Vec::new(),
                                modified: vec![OperationRef {
                                    object_type: "rule".to_string(),
                                    uuid: current.uuid,
                                }],
                                deleted: Vec::new(),
                            },
                            description: format!("undo set_design_rule {}", current.uuid),
                        }
                    }
                    None => {
                        let removed = board
                            .rules
                            .iter()
                            .position(|rule| rule.uuid == current.uuid)
                            .ok_or(EngineError::NotFound {
                                object_type: "rule",
                                uuid: current.uuid,
                            })?;
                        board.rules.remove(removed);
                        OperationResult {
                            diff: OperationDiff {
                                created: Vec::new(),
                                modified: Vec::new(),
                                deleted: vec![OperationRef {
                                    object_type: "rule".to_string(),
                                    uuid: current.uuid,
                                }],
                            },
                            description: format!("undo set_design_rule {}", current.uuid),
                        }
                    }
                }
            }
            TransactionRecord::SetValue { before, after } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let package = board
                    .packages
                    .get_mut(&after.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: after.uuid,
                    })?;
                *package = before.clone();
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("undo set_value {}", after.uuid),
                }
            }
            TransactionRecord::SetReference { before, after } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let package = board
                    .packages
                    .get_mut(&after.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: after.uuid,
                    })?;
                *package = before.clone();
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("undo set_reference {}", after.uuid),
                }
            }
            TransactionRecord::AssignPart {
                before,
                after,
                before_pads,
                after_pads: _,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let package = board
                    .packages
                    .get_mut(&after.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: after.uuid,
                    })?;
                *package = before.clone();
                restore_component_pads(board, after.uuid, before_pads);
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("undo assign_part {}", after.uuid),
                }
            }
            TransactionRecord::SetPackage {
                before,
                after,
                before_pads,
                after_pads: _,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let package = board
                    .packages
                    .get_mut(&after.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: after.uuid,
                    })?;
                *package = before.clone();
                restore_component_pads(board, after.uuid, before_pads);
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("undo set_package {}", after.uuid),
                }
            }
            TransactionRecord::SetNetClass {
                before_net,
                after_net: _,
                previous_class,
                current_class,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let net = board
                    .nets
                    .get_mut(&before_net.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "net",
                        uuid: before_net.uuid,
                    })?;
                *net = before_net.clone();
                if let Some(previous_class) = previous_class {
                    board
                        .net_classes
                        .insert(previous_class.uuid, previous_class.clone());
                } else if current_class.uuid != Uuid::nil() {
                    board.net_classes.remove(&current_class.uuid);
                }
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "net".to_string(),
                            uuid: before_net.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("undo set_net_class {}", before_net.uuid),
                }
            }
        };
        self.redo_stack.push(transaction);
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = self.redo_stack.len();
        Ok(result)
    }

    pub fn redo(&mut self) -> Result<OperationResult, EngineError> {
        let transaction = self.redo_stack.pop().ok_or(EngineError::NothingToRedo)?;
        let result = match &transaction {
            TransactionRecord::DeleteTrack { track } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let removed = board
                    .tracks
                    .remove(&track.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "track",
                        uuid: track.uuid,
                    })?;
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: Vec::new(),
                        deleted: vec![OperationRef {
                            object_type: "track".to_string(),
                            uuid: removed.uuid,
                        }],
                    },
                    description: format!("redo delete_track {}", track.uuid),
                }
            }
            TransactionRecord::DeleteVia { via } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let removed = board.vias.remove(&via.uuid).ok_or(EngineError::NotFound {
                    object_type: "via",
                    uuid: via.uuid,
                })?;
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: Vec::new(),
                        deleted: vec![OperationRef {
                            object_type: "via".to_string(),
                            uuid: removed.uuid,
                        }],
                    },
                    description: format!("redo delete_via {}", via.uuid),
                }
            }
            TransactionRecord::DeleteComponent { package, pads } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let removed = board
                    .packages
                    .remove(&package.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: package.uuid,
                    })?;
                for pad in pads {
                    board.pads.remove(&pad.uuid).ok_or(EngineError::NotFound {
                        object_type: "pad",
                        uuid: pad.uuid,
                    })?;
                }
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: Vec::new(),
                        deleted: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: removed.uuid,
                        }],
                    },
                    description: format!("redo delete_component {}", package.uuid),
                }
            }
            TransactionRecord::MoveComponent {
                before: _,
                after,
                before_pads: _,
                after_pads,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                restore_package_transform(board, after.uuid, after.clone(), after_pads)?;
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("redo move_component {}", after.uuid),
                }
            }
            TransactionRecord::RotateComponent {
                before: _,
                after,
                before_pads: _,
                after_pads,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                restore_package_transform(board, after.uuid, after.clone(), after_pads)?;
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("redo rotate_component {}", after.uuid),
                }
            }
            TransactionRecord::SetDesignRule {
                previous: _,
                current,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                if let Some(existing) = board
                    .rules
                    .iter_mut()
                    .find(|rule| rule.uuid == current.uuid)
                {
                    *existing = current.clone();
                    OperationResult {
                        diff: OperationDiff {
                            created: Vec::new(),
                            modified: vec![OperationRef {
                                object_type: "rule".to_string(),
                                uuid: current.uuid,
                            }],
                            deleted: Vec::new(),
                        },
                        description: format!("redo set_design_rule {}", current.uuid),
                    }
                } else {
                    board.rules.push(current.clone());
                    board.rules.sort_by(|a, b| {
                        a.priority
                            .cmp(&b.priority)
                            .then_with(|| a.name.cmp(&b.name))
                            .then_with(|| a.uuid.cmp(&b.uuid))
                    });
                    OperationResult {
                        diff: OperationDiff {
                            created: vec![OperationRef {
                                object_type: "rule".to_string(),
                                uuid: current.uuid,
                            }],
                            modified: Vec::new(),
                            deleted: Vec::new(),
                        },
                        description: format!("redo set_design_rule {}", current.uuid),
                    }
                }
            }
            TransactionRecord::SetValue { before: _, after } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let package = board
                    .packages
                    .get_mut(&after.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: after.uuid,
                    })?;
                *package = after.clone();
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("redo set_value {}", after.uuid),
                }
            }
            TransactionRecord::SetReference { before: _, after } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let package = board
                    .packages
                    .get_mut(&after.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: after.uuid,
                    })?;
                *package = after.clone();
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("redo set_reference {}", after.uuid),
                }
            }
            TransactionRecord::AssignPart {
                before: _,
                after,
                before_pads: _,
                after_pads,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let package = board
                    .packages
                    .get_mut(&after.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: after.uuid,
                    })?;
                *package = after.clone();
                restore_component_pads(board, after.uuid, after_pads);
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("redo assign_part {}", after.uuid),
                }
            }
            TransactionRecord::SetPackage {
                before: _,
                after,
                before_pads: _,
                after_pads,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let package = board
                    .packages
                    .get_mut(&after.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: after.uuid,
                    })?;
                *package = after.clone();
                restore_component_pads(board, after.uuid, after_pads);
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("redo set_package {}", after.uuid),
                }
            }
            TransactionRecord::SetNetClass {
                before_net: _,
                after_net,
                previous_class: _,
                current_class,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                board
                    .net_classes
                    .insert(current_class.uuid, current_class.clone());
                let net = board
                    .nets
                    .get_mut(&after_net.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "net",
                        uuid: after_net.uuid,
                    })?;
                *net = after_net.clone();
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "net".to_string(),
                            uuid: after_net.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("redo set_net_class {}", after_net.uuid),
                }
            }
        };
        self.undo_stack.push(transaction);
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = self.redo_stack.len();
        Ok(result)
    }

    /// M3 save entry point for the current implemented write-back slice.
    ///
    /// Current scope: imported KiCad boards can be written back byte-identically
    /// when unmodified, and can persist the current `delete_track` slice by
    /// removing deleted top-level KiCad `segment` forms from the imported
    /// source text.
    pub fn save(&self, path: &Path) -> Result<(), EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let imported = self.imported_source.as_ref().ok_or_else(|| {
            EngineError::Operation("save requires imported design provenance".to_string())
        })?;

        if design.board.is_none() {
            return Err(EngineError::Operation(
                "save is currently implemented only for imported KiCad boards".to_string(),
            ));
        }
        if imported.kind != ImportKind::KiCadBoard {
            return Err(EngineError::Operation(format!(
                "save is currently implemented only for imported KiCad boards; current design kind is {}",
                imported.kind.as_str()
            )));
        }

        let serialized = serialize_current_kicad_board_slice(
            &imported.original_contents,
            design
                .board
                .as_ref()
                .ok_or_else(|| EngineError::Operation("save requires imported board".to_string()))?,
            &self.pool,
            &self.undo_stack,
        )?;
        std::fs::write(path, serialized.as_bytes())?;
        persist_rule_sidecar(
            path,
            &serialized,
            design
                .board
                .as_ref()
                .map(|board| board.rules.clone())
                .unwrap_or_default(),
            imported.loaded_rule_sidecar,
        )?;
        persist_part_assignment_sidecar(
            path,
            &serialized,
            design
                .board
                .as_ref()
                .map(|board| {
                    board
                        .packages
                        .values()
                        .filter(|package| package.part != uuid::Uuid::nil())
                        .map(|package| (package.uuid, package.part))
                        .collect()
                })
                .unwrap_or_default(),
            imported.loaded_part_assignment_sidecar,
        )?;
        persist_package_assignment_sidecar(
            path,
            &serialized,
            design
                .board
                .as_ref()
                .map(|board| {
                    board
                        .packages
                        .values()
                        .filter(|package| package.package != uuid::Uuid::nil())
                        .map(|package| (package.uuid, package.package))
                        .collect()
                })
                .unwrap_or_default(),
            imported.loaded_package_assignment_sidecar,
        )?;
        persist_net_class_sidecar(
            path,
            &serialized,
            design
                .board
                .as_ref()
                .map(net_class_sidecar_payload)
                .unwrap_or_default(),
            imported.loaded_net_class_sidecar,
        )?;
        Ok(())
    }

    pub fn save_to_original(&self) -> Result<std::path::PathBuf, EngineError> {
        let imported = self.imported_source.as_ref().ok_or_else(|| {
            EngineError::Operation("save requires imported design provenance".to_string())
        })?;
        self.save(&imported.source_path)?;
        Ok(imported.source_path.clone())
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

fn serialize_current_kicad_board_slice(
    original_contents: &str,
    board: &Board,
    pool: &Pool,
    undo_stack: &[TransactionRecord],
) -> Result<String, EngineError> {
    let deleted_tracks = active_deleted_track_uuids(undo_stack);
    let deleted_vias = active_deleted_via_uuids(undo_stack);
    let deleted_components = active_deleted_component_uuids(undo_stack);
    let moved_components = active_moved_components(undo_stack);
    let valued_components = active_set_value_components(undo_stack);
    let referenced_components = active_set_reference_components(undo_stack);
    let assigned_components = active_assigned_part_components(undo_stack);
    let package_rewritten_components = active_package_rewritten_components(board);
    let forms = [
        ("segment", &deleted_tracks),
        ("via", &deleted_vias),
        ("footprint", &deleted_components),
    ];
    let without_removed = if deleted_tracks.is_empty()
        && deleted_vias.is_empty()
        && deleted_components.is_empty()
    {
        original_contents.to_string()
    } else {
        remove_kicad_top_level_forms(original_contents, &forms)?
    };
    let package_rewritten = rewrite_package_footprints(
        &without_removed,
        board,
        pool,
        &package_rewritten_components,
    )?;
    let moved = rewrite_moved_footprints(
        &package_rewritten,
        &filter_component_map(&moved_components, &package_rewritten_components),
    )?;
    let assigned_values = merge_component_value_overrides(&valued_components, &assigned_components);
    let valued = rewrite_value_footprints(
        &moved,
        &filter_component_map(&assigned_values, &package_rewritten_components),
    )?;
    rewrite_reference_footprints(
        &valued,
        &filter_component_map(&referenced_components, &package_rewritten_components),
    )
}

fn active_deleted_track_uuids(undo_stack: &[TransactionRecord]) -> BTreeSet<uuid::Uuid> {
    let mut deleted = BTreeSet::new();
    for transaction in undo_stack {
        if let TransactionRecord::DeleteTrack { track } = transaction {
            deleted.insert(track.uuid);
        }
    }
    deleted
}

fn active_deleted_via_uuids(undo_stack: &[TransactionRecord]) -> BTreeSet<uuid::Uuid> {
    let mut deleted = BTreeSet::new();
    for transaction in undo_stack {
        if let TransactionRecord::DeleteVia { via } = transaction {
            deleted.insert(via.uuid);
        }
    }
    deleted
}

fn active_deleted_component_uuids(undo_stack: &[TransactionRecord]) -> BTreeSet<uuid::Uuid> {
    let mut deleted = BTreeSet::new();
    for transaction in undo_stack {
        if let TransactionRecord::DeleteComponent { package, .. } = transaction {
            deleted.insert(package.uuid);
        }
    }
    deleted
}

fn active_moved_components(
    undo_stack: &[TransactionRecord],
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut moved = BTreeMap::new();
    for transaction in undo_stack {
        match transaction {
            TransactionRecord::MoveComponent { after, .. }
            | TransactionRecord::RotateComponent { after, .. } => {
                moved.insert(after.uuid, after.clone());
            }
            _ => {}
        }
    }
    moved
}

fn active_set_value_components(
    undo_stack: &[TransactionRecord],
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut valued = BTreeMap::new();
    for transaction in undo_stack {
        if let TransactionRecord::SetValue { after, .. } = transaction {
            valued.insert(after.uuid, after.clone());
        }
    }
    valued
}

fn active_set_reference_components(
    undo_stack: &[TransactionRecord],
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut referenced = BTreeMap::new();
    for transaction in undo_stack {
        if let TransactionRecord::SetReference { after, .. } = transaction {
            referenced.insert(after.uuid, after.clone());
        }
    }
    referenced
}

fn active_assigned_part_components(
    undo_stack: &[TransactionRecord],
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut assigned = BTreeMap::new();
    for transaction in undo_stack {
        if let TransactionRecord::AssignPart { after, .. } = transaction {
            assigned.insert(after.uuid, after.clone());
        }
    }
    assigned
}

fn merge_component_value_overrides(
    valued_components: &BTreeMap<uuid::Uuid, crate::board::PlacedPackage>,
    assigned_components: &BTreeMap<uuid::Uuid, crate::board::PlacedPackage>,
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    let mut merged = valued_components.clone();
    for (uuid, package) in assigned_components {
        merged.insert(*uuid, package.clone());
    }
    merged
}

fn active_package_rewritten_components(board: &Board) -> BTreeSet<uuid::Uuid> {
    board.packages
        .values()
        .filter(|package| package.package != uuid::Uuid::nil())
        .map(|package| package.uuid)
        .collect()
}

fn filter_component_map(
    components: &BTreeMap<uuid::Uuid, crate::board::PlacedPackage>,
    exclude: &BTreeSet<uuid::Uuid>,
) -> BTreeMap<uuid::Uuid, crate::board::PlacedPackage> {
    components
        .iter()
        .filter(|(uuid, _)| !exclude.contains(uuid))
        .map(|(uuid, package)| (*uuid, package.clone()))
        .collect()
}

fn remove_kicad_top_level_forms(
    original_contents: &str,
    forms: &[(&str, &BTreeSet<uuid::Uuid>)],
) -> Result<String, EngineError> {
    let removal_ranges = find_kicad_top_level_form_ranges(original_contents, forms)?;
    if removal_ranges.is_empty() {
        return Err(EngineError::Operation(
            "save could not locate deleted KiCad board form in imported source".to_string(),
        ));
    }

    let mut output = String::with_capacity(original_contents.len());
    let mut cursor = 0;
    for (start, end) in removal_ranges {
        if start < cursor || end < start || end > original_contents.len() {
            return Err(EngineError::Operation(
                "save generated invalid KiCad segment removal range".to_string(),
            ));
        }
        output.push_str(&original_contents[cursor..start]);
        cursor = end;
    }
    output.push_str(&original_contents[cursor..]);
    Ok(output)
}

fn find_kicad_top_level_form_ranges(
    original_contents: &str,
    forms: &[(&str, &BTreeSet<uuid::Uuid>)],
) -> Result<Vec<(usize, usize)>, EngineError> {
    let mut ranges = Vec::new();
    let mut found: BTreeMap<&str, BTreeSet<uuid::Uuid>> = forms
        .iter()
        .map(|(name, _)| (*name, BTreeSet::new()))
        .collect();
    let mut depth = 0usize;
    let mut top_level_start = None;

    for (idx, ch) in original_contents.char_indices() {
        match ch {
            '(' => {
                depth += 1;
                if depth == 2 {
                    top_level_start = Some(idx);
                }
            }
            ')' => {
                if depth == 2 {
                    let start = top_level_start.ok_or_else(|| {
                        EngineError::Operation(
                            "save encountered malformed KiCad top-level form boundaries"
                                .to_string(),
                        )
                    })?;
                    let end = idx + ch.len_utf8();
                    let form = &original_contents[start..end];
                    for (form_name, uuids) in forms {
                        if uuids.is_empty() {
                            continue;
                        }
                        if let Some(uuid) = top_level_form_uuid(form, form_name)?
                            && uuids.contains(&uuid)
                        {
                            let line_start = original_contents[..start]
                                .rfind('\n')
                                .map(|pos| pos + 1)
                                .unwrap_or(0);
                            let line_end = original_contents[end..]
                                .find('\n')
                                .map(|offset| end + offset + 1)
                                .unwrap_or(original_contents.len());
                            ranges.push((line_start, line_end));
                            found.entry(form_name).or_default().insert(uuid);
                        }
                    }
                    top_level_start = None;
                }
                depth = depth.checked_sub(1).ok_or_else(|| {
                    EngineError::Operation(
                        "save encountered malformed KiCad board structure".to_string(),
                    )
                })?;
            }
            _ => {}
        }
    }

    for (form_name, expected) in forms {
        if found.get(form_name).unwrap_or(&BTreeSet::new()) != *expected {
            return Err(EngineError::Operation(format!(
                "save could not map deleted KiCad {form_name} UUIDs into imported source; expected {:?}, found {:?}",
                expected,
                found.get(form_name).unwrap_or(&BTreeSet::new())
            )));
        }
    }

    Ok(ranges)
}

fn top_level_form_uuid(form: &str, form_name: &str) -> Result<Option<uuid::Uuid>, EngineError> {
    let trimmed = form.trim_start();
    if !trimmed.starts_with(&format!("({form_name}")) {
        return Ok(None);
    }

    let uuid_marker = "(uuid ";
    let uuid_start = match form.find(uuid_marker) {
        Some(index) => index + uuid_marker.len(),
        None => {
            return Err(EngineError::Operation(format!(
                "save found KiCad {form_name} without UUID in imported source"
            )));
        }
    };
    let uuid_end = form[uuid_start..]
        .find(')')
        .map(|offset| uuid_start + offset)
        .ok_or_else(|| {
            EngineError::Operation(format!(
                "save found unterminated KiCad {form_name} UUID in imported source"
            ))
        })?;
    let uuid = uuid::Uuid::parse_str(form[uuid_start..uuid_end].trim()).map_err(|err| {
        EngineError::Operation(format!(
            "save found invalid KiCad {form_name} UUID in imported source: {err}"
        ))
    })?;
    Ok(Some(uuid))
}

type PadPositions = Vec<(uuid::Uuid, crate::ir::geometry::Point)>;

fn apply_package_transform(
    board: &mut Board,
    before: &crate::board::PlacedPackage,
    after: &crate::board::PlacedPackage,
) -> Result<(PadPositions, PadPositions), EngineError> {
    let package = board
        .packages
        .get_mut(&before.uuid)
        .ok_or(EngineError::NotFound {
            object_type: "component",
            uuid: before.uuid,
        })?;
    *package = after.clone();

    let mut before_pads = Vec::new();
    let mut after_pads = Vec::new();
    for pad in board
        .pads
        .values_mut()
        .filter(|pad| pad.package == before.uuid)
    {
        before_pads.push((pad.uuid, pad.position));
        let local =
            inverse_transform_board_local_point(before.position, before.rotation, pad.position);
        pad.position = transform_board_local_point(after.position, after.rotation, local);
        after_pads.push((pad.uuid, pad.position));
    }
    before_pads.sort_by_key(|(uuid, _)| *uuid);
    after_pads.sort_by_key(|(uuid, _)| *uuid);
    Ok((before_pads, after_pads))
}

fn restore_package_transform(
    board: &mut Board,
    package_uuid: uuid::Uuid,
    package_state: crate::board::PlacedPackage,
    pad_positions: &[(uuid::Uuid, crate::ir::geometry::Point)],
) -> Result<(), EngineError> {
    let package = board
        .packages
        .get_mut(&package_uuid)
        .ok_or(EngineError::NotFound {
            object_type: "component",
            uuid: package_uuid,
        })?;
    *package = package_state;
    for (pad_uuid, position) in pad_positions {
        let pad = board.pads.get_mut(pad_uuid).ok_or(EngineError::NotFound {
            object_type: "pad",
            uuid: *pad_uuid,
        })?;
        pad.position = *position;
    }
    Ok(())
}

fn component_pads(board: &Board, component_uuid: uuid::Uuid) -> Vec<crate::board::PlacedPad> {
    let mut pads: Vec<_> = board
        .pads
        .values()
        .filter(|pad| pad.package == component_uuid)
        .cloned()
        .collect();
    pads.sort_by_key(|pad| pad.uuid);
    pads
}

fn restore_component_pads(
    board: &mut Board,
    component_uuid: uuid::Uuid,
    pads: &[crate::board::PlacedPad],
) {
    let stale_pad_uuids: Vec<_> = board
        .pads
        .values()
        .filter(|pad| pad.package == component_uuid)
        .map(|pad| pad.uuid)
        .collect();
    for pad_uuid in stale_pad_uuids {
        board.pads.remove(&pad_uuid);
    }
    for pad in pads {
        board.pads.insert(pad.uuid, pad.clone());
    }
}

fn replace_component_pads_from_pool_package(
    board: &mut Board,
    component: &crate::board::PlacedPackage,
    package: &crate::pool::Package,
) -> Result<(), EngineError> {
    let net_by_name: BTreeMap<String, Option<uuid::Uuid>> = component_pads(board, component.uuid)
        .into_iter()
        .map(|pad| (pad.name, pad.net))
        .collect();
    let mut regenerated = Vec::new();
    for package_pad in package.pads.values() {
        regenerated.push(crate::board::PlacedPad {
            uuid: deterministic_component_pad_uuid(component.uuid, &package_pad.name),
            package: component.uuid,
            name: package_pad.name.clone(),
            net: net_by_name.get(&package_pad.name).copied().flatten(),
            position: transform_board_local_point(
                component.position,
                component.rotation,
                package_pad.position,
            ),
            layer: package_pad.layer,
        });
    }
    regenerated.sort_by_key(|pad| pad.uuid);
    restore_component_pads(board, component.uuid, &regenerated);
    Ok(())
}

fn deterministic_component_pad_uuid(component_uuid: uuid::Uuid, pad_name: &str) -> uuid::Uuid {
    crate::ir::ids::import_uuid(
        &crate::ir::ids::namespace_kicad(),
        &format!("board/pad/{component_uuid}/{pad_name}"),
    )
}

fn transform_board_local_point(
    origin: crate::ir::geometry::Point,
    rotation_deg: i32,
    local: crate::ir::geometry::Point,
) -> crate::ir::geometry::Point {
    let rotated = match rotation_deg.rem_euclid(360) {
        90 => crate::ir::geometry::Point::new(-local.y, local.x),
        180 => crate::ir::geometry::Point::new(-local.x, -local.y),
        270 => crate::ir::geometry::Point::new(local.y, -local.x),
        _ => local,
    };
    crate::ir::geometry::Point::new(origin.x + rotated.x, origin.y + rotated.y)
}

fn inverse_transform_board_local_point(
    origin: crate::ir::geometry::Point,
    rotation_deg: i32,
    absolute: crate::ir::geometry::Point,
) -> crate::ir::geometry::Point {
    let translated = crate::ir::geometry::Point::new(absolute.x - origin.x, absolute.y - origin.y);
    match rotation_deg.rem_euclid(360) {
        90 => crate::ir::geometry::Point::new(translated.y, -translated.x),
        180 => crate::ir::geometry::Point::new(-translated.x, -translated.y),
        270 => crate::ir::geometry::Point::new(-translated.y, translated.x),
        _ => translated,
    }
}

fn rewrite_moved_footprints(
    contents: &str,
    moved_components: &BTreeMap<uuid::Uuid, crate::board::PlacedPackage>,
) -> Result<String, EngineError> {
    let mut updated = contents.to_string();
    for (uuid, package) in moved_components {
        updated = rewrite_footprint_at_line(&updated, *uuid, package.position, package.rotation)?;
    }
    Ok(updated)
}

fn rewrite_value_footprints(
    contents: &str,
    valued_components: &BTreeMap<uuid::Uuid, crate::board::PlacedPackage>,
) -> Result<String, EngineError> {
    let mut updated = contents.to_string();
    for (uuid, package) in valued_components {
        updated = rewrite_footprint_property_line(&updated, *uuid, "Value", &package.value)?;
    }
    Ok(updated)
}

fn rewrite_reference_footprints(
    contents: &str,
    referenced_components: &BTreeMap<uuid::Uuid, crate::board::PlacedPackage>,
) -> Result<String, EngineError> {
    let mut updated = contents.to_string();
    for (uuid, package) in referenced_components {
        updated =
            rewrite_footprint_property_line(&updated, *uuid, "Reference", &package.reference)?;
    }
    Ok(updated)
}

fn rewrite_package_footprints(
    contents: &str,
    board: &Board,
    pool: &Pool,
    component_uuids: &BTreeSet<uuid::Uuid>,
) -> Result<String, EngineError> {
    if component_uuids.is_empty() {
        return Ok(contents.to_string());
    }
    let net_codes = kicad_net_code_map(contents)?;
    let mut updated = contents.to_string();
    for component_uuid in component_uuids {
        let component = board
            .packages
            .get(component_uuid)
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: *component_uuid,
            })?;
        let package = pool
            .packages
            .get(&component.package)
            .ok_or(EngineError::NotFound {
                object_type: "package",
                uuid: component.package,
            })?;
        let replacement = render_kicad_footprint_block(component, package, board, &net_codes)?;
        updated = replace_kicad_top_level_form(&updated, "footprint", *component_uuid, &replacement)?;
    }
    Ok(updated)
}

fn rewrite_footprint_at_line(
    contents: &str,
    package_uuid: uuid::Uuid,
    position: crate::ir::geometry::Point,
    rotation: i32,
) -> Result<String, EngineError> {
    let lookup = BTreeSet::from([package_uuid]);
    let ranges = find_kicad_top_level_form_ranges(contents, &[("footprint", &lookup)])?;
    let (start, end) = *ranges.first().ok_or_else(|| {
        EngineError::Operation(format!(
            "save could not locate moved component {} in imported KiCad source",
            package_uuid
        ))
    })?;
    let block = &contents[start..end];
    let at_line = block
        .lines()
        .find(|line| line.trim_start().starts_with("(at "))
        .ok_or_else(|| {
            EngineError::Operation(format!(
                "save found moved footprint {} without (at ...) line",
                package_uuid
            ))
        })?;
    let indent_len = at_line.len() - at_line.trim_start().len();
    let replacement = format!(
        "{}(at {} {} {})",
        " ".repeat(indent_len),
        format_mm(position.x),
        format_mm(position.y),
        rotation
    );
    let replaced_block = block.replacen(at_line, &replacement, 1);
    Ok(format!(
        "{}{}{}",
        &contents[..start],
        replaced_block,
        &contents[end..]
    ))
}

fn rewrite_footprint_property_line(
    contents: &str,
    package_uuid: uuid::Uuid,
    property_name: &str,
    property_value: &str,
) -> Result<String, EngineError> {
    let lookup = BTreeSet::from([package_uuid]);
    let ranges = find_kicad_top_level_form_ranges(contents, &[("footprint", &lookup)])?;
    let (start, end) = *ranges.first().ok_or_else(|| {
        EngineError::Operation(format!(
            "save could not locate component {} in imported KiCad source",
            package_uuid
        ))
    })?;
    let block = &contents[start..end];
    let marker = format!("(property \"{property_name}\" ");
    let property_line = block
        .lines()
        .find(|line| line.trim_start().starts_with(&marker))
        .ok_or_else(|| {
            EngineError::Operation(format!(
                "save found footprint {} without {property_name} property line",
                package_uuid
            ))
        })?;
    let indent_len = property_line.len() - property_line.trim_start().len();
    let remainder = property_line
        .trim_start()
        .strip_prefix(&marker)
        .ok_or_else(|| {
            EngineError::Operation(format!(
                "save could not parse {property_name} property line for footprint {}",
                package_uuid
            ))
        })?;
    let tail_start = remainder.find('"').ok_or_else(|| {
        EngineError::Operation(format!(
            "save could not locate property value terminator for footprint {}",
            package_uuid
        ))
    })? + 1;
    let replacement = format!(
        "{}(property \"{}\" \"{}\"{}",
        " ".repeat(indent_len),
        property_name,
        property_value,
        &remainder[tail_start..]
    );
    let replaced_block = block.replacen(property_line, &replacement, 1);
    Ok(format!(
        "{}{}{}",
        &contents[..start],
        replaced_block,
        &contents[end..]
    ))
}

fn replace_kicad_top_level_form(
    contents: &str,
    form_name: &str,
    form_uuid: uuid::Uuid,
    replacement: &str,
) -> Result<String, EngineError> {
    let lookup = BTreeSet::from([form_uuid]);
    let ranges = find_kicad_top_level_form_ranges(contents, &[(form_name, &lookup)])?;
    let (start, end) = *ranges.first().ok_or_else(|| {
        EngineError::Operation(format!(
            "save could not locate {form_name} {} in imported KiCad source",
            form_uuid
        ))
    })?;
    Ok(format!(
        "{}{}{}",
        &contents[..start],
        replacement,
        &contents[end..]
    ))
}

fn render_kicad_footprint_block(
    component: &crate::board::PlacedPackage,
    package: &crate::pool::Package,
    board: &Board,
    net_codes: &BTreeMap<uuid::Uuid, i32>,
) -> Result<String, EngineError> {
    let mut lines = Vec::new();
    lines.push(format!("  (footprint \"{}\"", package.name));
    lines.push(format!(
        "    (layer \"{}\")",
        kicad_layer_name_for_id(component.layer)
    ));
    lines.push(format!("    (uuid {})", component.uuid));
    lines.push(format!(
        "    (at {} {} {})",
        format_mm(component.position.x),
        format_mm(component.position.y),
        component.rotation
    ));
    lines.push(format!(
        "    (property \"Reference\" \"{}\" (at 0 -2 0) (layer \"F.SilkS\"))",
        component.reference
    ));
    lines.push(format!(
        "    (property \"Value\" \"{}\" (at 0 2 0) (layer \"F.Fab\"))",
        component.value
    ));
    let mut pads: Vec<_> = package.pads.values().collect();
    pads.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    for pad in pads {
        lines.push(format!("    (pad \"{}\" smd rect", pad.name));
        lines.push(format!(
            "      (at {} {})",
            format_mm(pad.position.x),
            format_mm(pad.position.y)
        ));
        lines.push("      (size 1 1)".to_string());
        lines.push(format!(
            "      (layers \"{}\" \"F.Paste\" \"F.Mask\")",
            kicad_layer_name_for_id(pad.layer)
        ));
        let pad_state = board
            .pads
            .values()
            .find(|candidate| candidate.package == component.uuid && candidate.name == pad.name);
        if let Some(net_uuid) = pad_state.and_then(|pad| pad.net)
            && let Some(net_code) = net_codes.get(&net_uuid)
        {
            let net_name = board
                .nets
                .get(&net_uuid)
                .map(|net| net.name.as_str())
                .unwrap_or("");
            lines.push(format!("      (net {} \"{}\")", net_code, net_name));
        }
        lines.push("    )".to_string());
    }
    lines.push("  )".to_string());
    Ok(format!("{}\n", lines.join("\n")))
}

fn kicad_net_code_map(contents: &str) -> Result<BTreeMap<uuid::Uuid, i32>, EngineError> {
    let mut net_codes = BTreeMap::new();
    for block in contents
        .lines()
        .collect::<Vec<_>>()
        .join("\n")
        .split("\n  (net ")
    {
        let candidate = if block.starts_with("(kicad_pcb") {
            continue;
        } else {
            format!("  (net {block}")
        };
        if let Some((code, _name)) = parse_simple_net_block(&candidate)
            && code >= 0
        {
            let uuid = deterministic_kicad_net_uuid(code);
            net_codes.insert(uuid, code);
        }
    }
    Ok(net_codes)
}

fn parse_simple_net_block(block: &str) -> Option<(i32, String)> {
    let first = block.lines().next()?.trim_start();
    if !first.starts_with("(net ") {
        return None;
    }
    let after = first.trim_start_matches("(net ").trim_end_matches(')');
    let mut chars = after.chars().peekable();
    let mut code = String::new();
    while let Some(ch) = chars.peek() {
        if ch.is_ascii_whitespace() {
            break;
        }
        code.push(*ch);
        chars.next();
    }
    let code = code.parse::<i32>().ok()?;
    let rest: String = chars.collect();
    let start = rest.find('"')?;
    let rest = &rest[start + 1..];
    let end = rest.find('"')?;
    Some((code, rest[..end].to_string()))
}

fn deterministic_kicad_net_uuid(code: i32) -> uuid::Uuid {
    crate::ir::ids::import_uuid(
        &crate::ir::ids::namespace_kicad(),
        &format!("board/net/{code}"),
    )
}

fn kicad_layer_name_for_id(layer: i32) -> &'static str {
    match layer {
        31 => "B.Cu",
        36 => "B.SilkS",
        37 => "F.SilkS",
        44 => "Edge.Cuts",
        _ => "F.Cu",
    }
}

fn format_mm(nm: i64) -> String {
    let value = nm_to_mm(nm);
    if (value.fract()).abs() < f64::EPSILON {
        format!("{}", value.round() as i64)
    } else {
        let mut text = format!("{value:.6}");
        while text.contains('.') && text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
        text
    }
}

fn summarize_diagnostics(diagnostics: &[ConnectivityDiagnosticInfo]) -> CheckSummary {
    let mut summary = CheckSummary {
        status: CheckStatus::Ok,
        errors: 0,
        warnings: 0,
        infos: 0,
        waived: 0,
        by_code: summarize_diagnostic_codes(diagnostics),
    };

    for diagnostic in diagnostics {
        match diagnostic.severity.as_str() {
            "error" => summary.errors += 1,
            "warning" => summary.warnings += 1,
            _ => summary.infos += 1,
        }
    }

    summary.status = derive_status(summary.errors, summary.warnings, summary.infos);
    summary
}

fn default_rule_name(rule_type: &RuleType) -> String {
    match rule_type {
        RuleType::ClearanceCopper => "clearance_copper".to_string(),
        RuleType::TrackWidth => "track_width".to_string(),
        RuleType::ViaHole => "via_hole".to_string(),
        RuleType::ViaAnnularRing => "via_annular_ring".to_string(),
        RuleType::HoleSize => "hole_size".to_string(),
        RuleType::SilkClearance => "silk_clearance".to_string(),
        RuleType::Connectivity => "connectivity".to_string(),
    }
}

fn deterministic_net_class_uuid(net_uuid: Uuid, class_name: &str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_OID, format!("{net_uuid}:{class_name}").as_bytes())
}

fn persist_rule_sidecar(
    board_path: &Path,
    board_contents: &str,
    rules: Vec<Rule>,
    loaded_rule_sidecar: bool,
) -> Result<(), EngineError> {
    let sidecar_path = rules_sidecar::sidecar_path_for_source(board_path);
    if rules.is_empty() {
        if loaded_rule_sidecar && sidecar_path.exists() {
            std::fs::remove_file(&sidecar_path)?;
        }
        return Ok(());
    }

    let source_hash = ids_sidecar::compute_source_hash_bytes(board_contents.as_bytes());
    let source_file = board_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| board_path.display().to_string());
    let sidecar = rules_sidecar::RuleSidecar::new(source_file, source_hash, rules);
    rules_sidecar::write_sidecar(&sidecar_path, &sidecar)
}

fn persist_part_assignment_sidecar(
    board_path: &Path,
    board_contents: &str,
    assignments: BTreeMap<uuid::Uuid, uuid::Uuid>,
    loaded_part_assignment_sidecar: bool,
) -> Result<(), EngineError> {
    let sidecar_path = part_assignments_sidecar::sidecar_path_for_source(board_path);
    if assignments.is_empty() {
        if loaded_part_assignment_sidecar && sidecar_path.exists() {
            std::fs::remove_file(&sidecar_path)?;
        }
        return Ok(());
    }

    let source_hash = ids_sidecar::compute_source_hash_bytes(board_contents.as_bytes());
    let source_file = board_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| board_path.display().to_string());
    let sidecar =
        part_assignments_sidecar::PartAssignmentsSidecar::new(source_file, source_hash, assignments);
    part_assignments_sidecar::write_sidecar(&sidecar_path, &sidecar)
}

fn persist_package_assignment_sidecar(
    board_path: &Path,
    board_contents: &str,
    assignments: BTreeMap<uuid::Uuid, uuid::Uuid>,
    loaded_package_assignment_sidecar: bool,
) -> Result<(), EngineError> {
    let sidecar_path = package_assignments_sidecar::sidecar_path_for_source(board_path);
    if assignments.is_empty() {
        if loaded_package_assignment_sidecar && sidecar_path.exists() {
            std::fs::remove_file(&sidecar_path)?;
        }
        return Ok(());
    }

    let source_hash = ids_sidecar::compute_source_hash_bytes(board_contents.as_bytes());
    let source_file = board_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| board_path.display().to_string());
    let sidecar = package_assignments_sidecar::PackageAssignmentsSidecar::new(
        source_file,
        source_hash,
        assignments,
    );
    package_assignments_sidecar::write_sidecar(&sidecar_path, &sidecar)
}

fn net_class_sidecar_payload(board: &Board) -> (Vec<NetClass>, BTreeMap<Uuid, Uuid>) {
    let assignments: BTreeMap<Uuid, Uuid> = board
        .nets
        .values()
        .filter(|net| net.class != Uuid::nil())
        .map(|net| (net.uuid, net.class))
        .collect();
    let classes = assignments
        .values()
        .filter_map(|uuid| board.net_classes.get(uuid).cloned())
        .collect();
    (classes, assignments)
}

fn persist_net_class_sidecar(
    board_path: &Path,
    board_contents: &str,
    payload: (Vec<NetClass>, BTreeMap<Uuid, Uuid>),
    loaded_net_class_sidecar: bool,
) -> Result<(), EngineError> {
    let sidecar_path = net_classes_sidecar::sidecar_path_for_source(board_path);
    let (classes, assignments) = payload;
    if classes.is_empty() || assignments.is_empty() {
        if loaded_net_class_sidecar && sidecar_path.exists() {
            std::fs::remove_file(&sidecar_path)?;
        }
        return Ok(());
    }

    let source_hash = ids_sidecar::compute_source_hash_bytes(board_contents.as_bytes());
    let source_file = board_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| board_path.display().to_string());
    let sidecar =
        net_classes_sidecar::NetClassesSidecar::new(source_file, source_hash, classes, assignments);
    net_classes_sidecar::write_sidecar(&sidecar_path, &sidecar)
}

fn summarize_schematic_checks(
    diagnostics: &[ConnectivityDiagnosticInfo],
    erc_findings: &[ErcFinding],
) -> CheckSummary {
    let mut summary = summarize_diagnostics(diagnostics);

    for finding in erc_findings {
        if finding.waived {
            summary.waived += 1;
            continue;
        }
        match finding.severity {
            erc::ErcSeverity::Error => summary.errors += 1,
            erc::ErcSeverity::Warning => summary.warnings += 1,
            erc::ErcSeverity::Info => summary.infos += 1,
        }
    }

    for (code, count) in summarize_erc_codes(erc_findings) {
        if let Some(existing) = summary.by_code.iter_mut().find(|entry| entry.code == code) {
            existing.count += count;
        } else {
            summary.by_code.push(CheckCodeCount { code, count });
        }
    }
    summary.by_code.sort_by(|a, b| a.code.cmp(&b.code));

    summary.status = derive_status(summary.errors, summary.warnings, summary.infos);
    summary
}

fn derive_status(errors: usize, warnings: usize, infos: usize) -> CheckStatus {
    if errors > 0 {
        CheckStatus::Error
    } else if warnings > 0 {
        CheckStatus::Warning
    } else if infos > 0 {
        CheckStatus::Info
    } else {
        CheckStatus::Ok
    }
}

fn summarize_diagnostic_codes(diagnostics: &[ConnectivityDiagnosticInfo]) -> Vec<CheckCodeCount> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for diagnostic in diagnostics {
        *counts.entry(diagnostic.kind.clone()).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(code, count)| CheckCodeCount { code, count })
        .collect()
}

fn summarize_erc_codes(findings: &[ErcFinding]) -> Vec<(String, usize)> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for finding in findings {
        *counts.entry(finding.code.to_string()).or_default() += 1;
    }
    counts.into_iter().collect()
}

fn erc_suggestion(code: &str) -> &'static str {
    match code {
        "output_to_output_conflict" => {
            "Ensure only one active output drives the net or add isolation between outputs."
        }
        "undriven_input_pin" => {
            "Connect the input to a valid driver or mark it intentionally unused."
        }
        "input_without_explicit_driver" => {
            "If intentional analog biasing is present, keep the passive network documented or add an explicit driver."
        }
        "power_in_without_source" => {
            "Add a valid power source pin on this net or connect to a driven power rail."
        }
        "noconnect_connected" => "Remove the no-connect marker or disconnect the pin from the net.",
        "unconnected_component_pin" => {
            "Wire the pin to a valid net or add a no-connect marker if intentionally left open."
        }
        "unconnected_interface_port" => {
            "Connect the hierarchical port to a net or remove the unused interface port."
        }
        "undriven_power_net" | "undriven_named_net" => {
            "Add a driving source or connect this net to its intended source rail."
        }
        _ => "Review net intent and either fix connectivity or apply a justified waiver.",
    }
}

fn drc_suggestion(code: &str) -> &'static str {
    match code {
        "connectivity_no_copper" | "connectivity_unrouted_net" => {
            "Route the remaining airwires so all required pins on the net are electrically connected."
        }
        "connectivity_unconnected_pin" => {
            "Route copper from this pin to the intended net or remove the unintended net assignment."
        }
        "clearance_copper" => {
            "Increase spacing between copper features or relax the applicable clearance rule."
        }
        "track_width_below_min" => {
            "Increase track width to meet the assigned net class or adjust rule constraints."
        }
        "via_hole_out_of_range" | "via_annular_below_min" => {
            "Use a via size compliant with the current drill and annular-ring rule limits."
        }
        "silk_clearance_copper" => {
            "Move or resize silkscreen text to satisfy copper clearance requirements."
        }
        _ => "Adjust geometry or rules so the reported objects satisfy constraint checks.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Net, PlacedPackage, Stackup, StackupLayer, StackupLayerType};
    use crate::ir::geometry::{Point, Polygon};
    use crate::ir::serialization::to_json_deterministic;
    use crate::schematic::{CheckDomain, LabelKind, NetLabel, Sheet, WaiverTarget};
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn summaries_require_open_project() {
        let engine = Engine::new().expect("engine should initialize");
        assert!(matches!(
            engine.get_board_summary(),
            Err(EngineError::NoProjectOpen)
        ));
        assert!(matches!(
            engine.get_schematic_summary(),
            Err(EngineError::NoProjectOpen)
        ));
    }

    #[test]
    fn schematic_check_summary_includes_info_level_erc_codes() {
        let summary = summarize_schematic_checks(
            &[],
            &[crate::erc::ErcFinding {
                id: uuid::Uuid::new_v4(),
                code: "input_without_explicit_driver",
                severity: crate::erc::ErcSeverity::Info,
                message: "analog input has passive biasing".into(),
                net_name: Some("IN_P".into()),
                component: None,
                pin: None,
                objects: vec![crate::erc::ErcObjectRef {
                    kind: "pin",
                    key: "Q1.1".into(),
                }],
                object_uuids: vec![uuid::Uuid::new_v4()],
                waived: false,
            }],
        );

        assert_eq!(summary.status, CheckStatus::Info);
        assert_eq!(summary.errors, 0);
        assert_eq!(summary.warnings, 0);
        assert_eq!(summary.infos, 1);
        assert_eq!(summary.waived, 0);
        assert!(
            summary
                .by_code
                .iter()
                .any(|entry| entry.code == "input_without_explicit_driver" && entry.count == 1)
        );
    }

    #[test]
    fn summaries_read_from_in_memory_design() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine.design = Some(Design {
            board: Some(Board {
                uuid: uuid::Uuid::new_v4(),
                name: "demo-board".into(),
                stackup: Stackup {
                    layers: vec![StackupLayer {
                        id: 1,
                        name: "Top".into(),
                        layer_type: StackupLayerType::Copper,
                        thickness_nm: 35_000,
                    }],
                },
                outline: Polygon::new(vec![
                    Point::new(0, 0),
                    Point::new(10, 0),
                    Point::new(10, 10),
                    Point::new(0, 10),
                ]),
                packages: HashMap::from([(
                    uuid::Uuid::new_v4(),
                    PlacedPackage {
                        uuid: uuid::Uuid::new_v4(),
                        part: uuid::Uuid::new_v4(),
                        package: uuid::Uuid::nil(),
                        reference: "R1".into(),
                        value: "10k".into(),
                        position: Point::new(0, 0),
                        rotation: 0,
                        layer: 1,
                        locked: false,
                    },
                )]),
                pads: HashMap::new(),
                tracks: HashMap::new(),
                vias: HashMap::new(),
                zones: HashMap::new(),
                nets: HashMap::from([(
                    uuid::Uuid::new_v4(),
                    Net {
                        uuid: uuid::Uuid::new_v4(),
                        name: "VCC".into(),
                        class: uuid::Uuid::new_v4(),
                    },
                )]),
                net_classes: HashMap::new(),
                rules: Vec::new(),
                keepouts: Vec::new(),
                dimensions: Vec::new(),
                texts: Vec::new(),
            }),
            schematic: Some(Schematic {
                uuid: uuid::Uuid::new_v4(),
                sheets: HashMap::from([(
                    uuid::Uuid::new_v4(),
                    Sheet {
                        uuid: uuid::Uuid::new_v4(),
                        name: "Sheet1".into(),
                        frame: None,
                        symbols: HashMap::new(),
                        wires: HashMap::new(),
                        junctions: HashMap::new(),
                        labels: HashMap::from([(
                            uuid::Uuid::new_v4(),
                            NetLabel {
                                uuid: uuid::Uuid::new_v4(),
                                kind: LabelKind::Local,
                                name: "VCC".into(),
                                position: Point::new(0, 0),
                            },
                        )]),
                        buses: HashMap::new(),
                        bus_entries: HashMap::new(),
                        ports: HashMap::new(),
                        noconnects: HashMap::new(),
                        texts: HashMap::new(),
                        drawings: HashMap::new(),
                    },
                )]),
                sheet_definitions: HashMap::new(),
                sheet_instances: HashMap::new(),
                variants: HashMap::new(),
                waivers: Vec::new(),
            }),
        });

        let board_summary = engine
            .get_board_summary()
            .expect("board summary should exist");
        let schematic_summary = engine
            .get_schematic_summary()
            .expect("schematic summary should exist");

        assert_eq!(board_summary.name, "demo-board");
        assert_eq!(board_summary.component_count, 1);
        assert_eq!(schematic_summary.sheet_count, 1);
        assert_eq!(schematic_summary.net_label_count, 1);
        assert_eq!(engine.get_sheets().unwrap().len(), 1);
    }

    #[test]
    fn get_symbol_fields_returns_fields_for_matching_symbol() {
        let symbol_uuid = uuid::Uuid::new_v4();
        let field_uuid = uuid::Uuid::new_v4();
        let mut engine = Engine::new().expect("engine should initialize");
        engine.design = Some(Design {
            board: None,
            schematic: Some(Schematic {
                uuid: uuid::Uuid::new_v4(),
                sheets: HashMap::from([(
                    uuid::Uuid::new_v4(),
                    Sheet {
                        uuid: uuid::Uuid::new_v4(),
                        name: "Root".into(),
                        frame: None,
                        symbols: HashMap::from([(
                            symbol_uuid,
                            crate::schematic::PlacedSymbol {
                                uuid: symbol_uuid,
                                part: None,
                                entity: None,
                                gate: None,
                                lib_id: None,
                                reference: "R1".into(),
                                value: "10k".into(),
                                fields: vec![crate::schematic::SymbolField {
                                    uuid: field_uuid,
                                    key: "Value".into(),
                                    value: "10k".into(),
                                    position: Some(Point::new(1, 2)),
                                    visible: true,
                                }],
                                pins: Vec::new(),
                                position: Point::new(0, 0),
                                rotation: 0,
                                mirrored: false,
                                unit_selection: None,
                                display_mode: crate::schematic::SymbolDisplayMode::LibraryDefault,
                                pin_overrides: Vec::new(),
                                hidden_power_behavior:
                                    crate::schematic::HiddenPowerBehavior::PreservedAsImportedMetadata,
                            },
                        )]),
                        wires: HashMap::new(),
                        junctions: HashMap::new(),
                        labels: HashMap::new(),
                        buses: HashMap::new(),
                        bus_entries: HashMap::new(),
                        ports: HashMap::new(),
                        noconnects: HashMap::new(),
                        texts: HashMap::new(),
                        drawings: HashMap::new(),
                    },
                )]),
                sheet_definitions: HashMap::new(),
                sheet_instances: HashMap::new(),
                variants: HashMap::new(),
                waivers: Vec::new(),
            }),
        });

        let fields = engine
            .get_symbol_fields(&symbol_uuid)
            .expect("symbol fields query should succeed");
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].uuid, field_uuid);
        assert_eq!(fields[0].symbol, symbol_uuid);
        assert_eq!(fields[0].key, "Value");
        assert_eq!(fields[0].value, "10k");
        assert!(fields[0].visible);
    }

    #[test]
    fn get_symbol_fields_returns_not_found_for_unknown_symbol() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("simple-demo.kicad_sch"))
            .expect("fixture should import");
        let missing = uuid::Uuid::new_v4();
        assert!(matches!(
            engine.get_symbol_fields(&missing),
            Err(EngineError::NotFound {
                object_type: "symbol",
                uuid
            }) if uuid == missing
        ));
    }

    #[test]
    fn get_bus_entries_returns_entries_for_matching_sheet_selection() {
        let sheet_uuid = uuid::Uuid::new_v4();
        let bus_uuid = uuid::Uuid::new_v4();
        let entry_uuid = uuid::Uuid::new_v4();
        let mut engine = Engine::new().expect("engine should initialize");
        engine.design = Some(Design {
            board: None,
            schematic: Some(Schematic {
                uuid: uuid::Uuid::new_v4(),
                sheets: HashMap::from([(
                    sheet_uuid,
                    Sheet {
                        uuid: sheet_uuid,
                        name: "Root".into(),
                        frame: None,
                        symbols: HashMap::new(),
                        wires: HashMap::new(),
                        junctions: HashMap::new(),
                        labels: HashMap::new(),
                        buses: HashMap::from([(
                            bus_uuid,
                            crate::schematic::Bus {
                                uuid: bus_uuid,
                                name: "DATA".into(),
                                members: vec!["DATA0".into()],
                            },
                        )]),
                        bus_entries: HashMap::from([(
                            entry_uuid,
                            crate::schematic::BusEntry {
                                uuid: entry_uuid,
                                bus: bus_uuid,
                                wire: None,
                                position: Point::new(10, 20),
                            },
                        )]),
                        ports: HashMap::new(),
                        noconnects: HashMap::new(),
                        texts: HashMap::new(),
                        drawings: HashMap::new(),
                    },
                )]),
                sheet_definitions: HashMap::new(),
                sheet_instances: HashMap::new(),
                variants: HashMap::new(),
                waivers: Vec::new(),
            }),
        });

        let all = engine
            .get_bus_entries(None)
            .expect("all-sheet bus entries should succeed");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].uuid, entry_uuid);
        assert_eq!(all[0].sheet, sheet_uuid);

        let selected = engine
            .get_bus_entries(Some(&sheet_uuid))
            .expect("sheet-select bus entries should succeed");
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].bus, bus_uuid);
    }

    #[test]
    fn import_dispatch_recognizes_kicad_and_eagle_paths() {
        let mut engine = Engine::new().expect("engine should initialize");

        let err = engine
            .import(Path::new("demo.kicad_pcb"))
            .expect_err("bare KiCad board path should fail because fixture is absent");
        assert!(matches!(err, EngineError::Io(_)), "{err}");

        let report = engine
            .import(&fixture_path("simple-demo.kicad_pcb"))
            .expect("KiCad board skeleton import should succeed");
        assert!(matches!(report.kind, ImportKind::KiCadBoard));
        assert_eq!(
            report.metadata.get("footprint_count").map(String::as_str),
            Some("1")
        );
        let board_summary = engine
            .get_board_summary()
            .expect("imported board should populate in-memory design");
        assert_eq!(board_summary.name, "simple-demo");
        assert_eq!(board_summary.component_count, 1);
        assert_eq!(board_summary.net_count, 2);
        let components = engine
            .get_components()
            .expect("component query should succeed");
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].reference, "R1");
        let nets = engine.get_net_info().expect("net query should succeed");
        assert_eq!(nets.len(), 2);
        let gnd = nets
            .iter()
            .find(|net| net.name == "GND")
            .expect("GND net should exist");
        assert_eq!(gnd.tracks, 1);
        assert_eq!(gnd.vias, 1);
        let stackup = engine.get_stackup().expect("stackup query should succeed");
        assert_eq!(stackup.layers.len(), 3);
        assert_eq!(stackup.layers[0].name, "F.Cu");
        let board_diagnostics = engine
            .get_connectivity_diagnostics()
            .expect("board diagnostics query should succeed");
        assert_eq!(board_diagnostics.len(), 1);
        assert_eq!(board_diagnostics[0].kind, "net_without_copper");
        assert_eq!(board_diagnostics[0].objects.len(), 1);
        let board_report = engine
            .get_check_report()
            .expect("board check report should succeed");
        match board_report {
            CheckReport::Board {
                summary,
                diagnostics,
            } => {
                assert_eq!(summary.status, CheckStatus::Info);
                assert_eq!(summary.errors, 0);
                assert_eq!(summary.warnings, 0);
                assert_eq!(summary.infos, 1);
                assert_eq!(summary.waived, 0);
                assert_eq!(summary.by_code.len(), 1);
                assert_eq!(summary.by_code[0].code, "net_without_copper");
                assert_eq!(summary.by_code[0].count, 1);
                assert_eq!(diagnostics.len(), 1);
                assert_eq!(diagnostics[0].kind, "net_without_copper");
            }
            other => panic!("expected board check report, got {other:?}"),
        }

        let airwire_report = engine
            .import(&fixture_path("airwire-demo.kicad_pcb"))
            .expect("airwire fixture import should succeed");
        assert!(matches!(airwire_report.kind, ImportKind::KiCadBoard));
        let unrouted = engine
            .get_unrouted()
            .expect("unrouted query should succeed");
        assert_eq!(unrouted.len(), 1);
        assert_eq!(unrouted[0].net_name, "SIG");
        assert_eq!(unrouted[0].from.component, "R1");
        assert_eq!(unrouted[0].to.component, "R2");

        let partial_route_report = engine
            .import(&fixture_path("partial-route-demo.kicad_pcb"))
            .expect("partial-route fixture import should succeed");
        assert!(matches!(partial_route_report.kind, ImportKind::KiCadBoard));
        let partial_unrouted = engine
            .get_unrouted()
            .expect("partial-route unrouted query should succeed");
        assert_eq!(partial_unrouted.len(), 1);
        assert_eq!(partial_unrouted[0].net_name, "SIG");
        assert_eq!(partial_unrouted[0].from.component, "R1");
        assert_eq!(partial_unrouted[0].to.component, "R2");
        let partial_diagnostics = engine
            .get_connectivity_diagnostics()
            .expect("partial-route diagnostics query should succeed");
        assert_eq!(partial_diagnostics.len(), 2);
        assert!(
            partial_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.kind == "partially_routed_net"
                    && diagnostic.severity == "warning")
        );
        assert!(
            partial_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.kind == "net_without_copper"
                    && diagnostic.severity == "info")
        );
        let partial_report = engine
            .get_check_report()
            .expect("partial-route check report should succeed");
        match partial_report {
            CheckReport::Board {
                summary,
                diagnostics,
            } => {
                assert_eq!(summary.status, CheckStatus::Warning);
                assert_eq!(summary.errors, 0);
                assert_eq!(summary.warnings, 1);
                assert_eq!(summary.infos, 1);
                assert_eq!(summary.by_code.len(), 2);
                assert!(
                    summary
                        .by_code
                        .iter()
                        .any(|entry| entry.code == "partially_routed_net" && entry.count == 1)
                );
                assert!(
                    summary
                        .by_code
                        .iter()
                        .any(|entry| entry.code == "net_without_copper" && entry.count == 1)
                );
                assert_eq!(diagnostics.len(), 2);
                assert!(
                    diagnostics
                        .iter()
                        .any(|diagnostic| diagnostic.kind == "partially_routed_net")
                );
            }
            other => panic!("expected board check report, got {other:?}"),
        }

        let err = engine
            .import(Path::new("demo.kicad_sch"))
            .expect_err("bare KiCad schematic path should fail because fixture is absent");
        assert!(matches!(err, EngineError::Io(_)), "{err}");

        let report = engine
            .import(&fixture_path("simple-demo.kicad_sch"))
            .expect("KiCad schematic skeleton import should succeed");
        assert!(matches!(report.kind, ImportKind::KiCadSchematic));
        assert_eq!(
            report.metadata.get("symbol_count").map(String::as_str),
            Some("1")
        );
        let schematic_summary = engine
            .get_schematic_summary()
            .expect("imported schematic should populate in-memory design");
        assert_eq!(schematic_summary.sheet_count, 1);
        assert_eq!(schematic_summary.net_label_count, 3);
        let sheets = engine.get_sheets().expect("sheet query should succeed");
        assert_eq!(sheets.len(), 1);
        assert_eq!(sheets[0].labels, 3);
        assert_eq!(sheets[0].symbols, 1);
        assert_eq!(sheets[0].ports, 1);
        let symbols = engine
            .get_symbols(None)
            .expect("symbol query should succeed");
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].reference, "R1");
        let ports = engine.get_ports(None).expect("port query should succeed");
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].name, "SUB_IN");
        let labels = engine.get_labels(None).expect("label query should succeed");
        assert_eq!(labels.len(), 3);
        let buses = engine.get_buses(None).expect("bus query should succeed");
        assert_eq!(buses.len(), 1);
        let noconnects = engine
            .get_noconnects(None)
            .expect("no-connect query should succeed");
        assert_eq!(noconnects.len(), 1);
        let hierarchy = engine
            .get_hierarchy()
            .expect("hierarchy query should succeed");
        assert_eq!(hierarchy.instances.len(), 1);
        assert!(hierarchy.links.is_empty());
        assert_eq!(hierarchy.instances[0].name, "Sub");
        let nets = engine
            .get_schematic_net_info()
            .expect("schematic net query should succeed");
        assert_eq!(nets.len(), 4);
        let scl = nets
            .iter()
            .find(|net| net.name == "SCL")
            .expect("SCL net should exist");
        assert_eq!(scl.labels, 1);
        assert_eq!(scl.ports, 0);
        assert_eq!(scl.pins.len(), 1);
        assert_eq!(scl.pins[0].component, "R1");
        assert_eq!(scl.pins[0].pin, "1");
        assert_eq!(scl.sheets, vec!["Root".to_string()]);
        let vcc = nets
            .iter()
            .find(|net| net.name == "VCC")
            .expect("VCC net should exist");
        assert_eq!(vcc.semantic_class.as_deref(), Some("power"));
        let sub_in = nets
            .iter()
            .find(|net| net.name == "SUB_IN")
            .expect("SUB_IN net should exist");
        assert_eq!(sub_in.ports, 1);
        assert_eq!(sub_in.labels, 1);
        let diagnostics = engine
            .get_connectivity_diagnostics()
            .expect("diagnostics query should succeed");
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.kind == "dangling_component_pin")
        );
        let report = engine
            .get_check_report()
            .expect("schematic check report should succeed");
        match report {
            CheckReport::Schematic {
                summary,
                diagnostics,
                erc,
            } => {
                assert_eq!(summary.status, CheckStatus::Warning);
                assert_eq!(summary.errors, 0);
                assert_eq!(summary.warnings, 3);
                assert_eq!(summary.infos, 0);
                assert_eq!(summary.waived, 0);
                assert_eq!(summary.by_code.len(), 3);
                assert!(
                    summary
                        .by_code
                        .iter()
                        .any(|entry| entry.code == "dangling_component_pin" && entry.count == 1)
                );
                assert!(
                    summary
                        .by_code
                        .iter()
                        .any(|entry| entry.code == "unconnected_component_pin" && entry.count == 1)
                );
                assert!(
                    summary
                        .by_code
                        .iter()
                        .any(|entry| entry.code == "undriven_power_net" && entry.count == 1)
                );
                assert_eq!(diagnostics.len(), 1);
                assert_eq!(erc.len(), 2);
            }
            other => panic!("expected schematic check report, got {other:?}"),
        }
        let dangling = nets
            .iter()
            .find(|net| net.name.starts_with("N$"))
            .expect("dangling symbol pin net should exist");
        assert_eq!(dangling.pins.len(), 1);
        assert_eq!(dangling.pins[0].component, "R1");
        assert_eq!(dangling.pins[0].pin, "2");
        let erc = engine
            .run_erc_prechecks()
            .expect("ERC precheck should succeed");
        assert_eq!(erc.len(), 2);
        let dangling_pin = erc
            .iter()
            .find(|finding| finding.code == "unconnected_component_pin")
            .expect("dangling pin finding should exist");
        assert_eq!(dangling_pin.component.as_deref(), Some("R1"));
        assert_eq!(dangling_pin.pin.as_deref(), Some("2"));
        let undriven_vcc = erc
            .iter()
            .find(|finding| finding.code == "undriven_power_net")
            .expect("undriven VCC finding should exist");
        assert_eq!(undriven_vcc.net_name.as_deref(), Some("VCC"));
        let mut config = ErcConfig::default();
        config
            .severity_overrides
            .insert("undriven_power_net".into(), erc::ErcSeverity::Error);
        let overridden = engine
            .run_erc_prechecks_with_config(&config)
            .expect("configured ERC precheck should succeed");
        let overridden_vcc = overridden
            .iter()
            .find(|finding| finding.code == "undriven_power_net")
            .expect("configured VCC finding should exist");
        assert_eq!(overridden_vcc.severity, erc::ErcSeverity::Error);
        assert_eq!(overridden_vcc.id, undriven_vcc.id);

        let waiver = CheckWaiver {
            uuid: uuid::Uuid::new_v4(),
            domain: CheckDomain::ERC,
            target: WaiverTarget::Object(dangling_pin.object_uuids[0]),
            rationale: "Intentional dangling input".into(),
            created_by: Some("api-test".into()),
        };
        let waived = engine
            .run_erc_prechecks_with_config_and_waivers(&ErcConfig::default(), &[waiver])
            .expect("configured ERC precheck with waiver should succeed");
        let waived_dangling = waived
            .iter()
            .find(|finding| finding.code == "unconnected_component_pin")
            .expect("waived dangling pin finding should exist");
        assert!(waived_dangling.waived);
        let still_unwaived_vcc = waived
            .iter()
            .find(|finding| finding.code == "undriven_power_net")
            .expect("unwaived VCC finding should exist");
        assert!(!still_unwaived_vcc.waived);

        let report = engine
            .import(&fixture_path("simple-demo.kicad_pro"))
            .expect("KiCad project metadata import should succeed");
        assert!(matches!(report.kind, ImportKind::KiCadProject));
        assert_eq!(
            report.metadata.get("project_name").map(String::as_str),
            Some("simple-demo")
        );

        let err = engine
            .import(Path::new("legacy.brd"))
            .expect_err("Eagle board import should be recognized but unimplemented");
        assert!(
            err.to_string()
                .contains("Eagle board import is not implemented yet"),
            "{}",
            err
        );
    }

    #[test]
    fn import_dispatch_rejects_unknown_extensions() {
        let mut engine = Engine::new().expect("engine should initialize");
        let err = engine
            .import(Path::new("unknown.txt"))
            .expect_err("unknown extension must fail");
        assert!(err.to_string().contains("unsupported import path"), "{err}");
    }

    #[test]
    fn close_project_clears_open_design() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("simple-demo.kicad_pcb"))
            .expect("fixture should import");
        assert!(engine.has_open_project());
        engine.close_project();
        assert!(!engine.has_open_project());
        assert!(matches!(
            engine.get_board_summary(),
            Err(EngineError::NoProjectOpen)
        ));
    }

    #[test]
    fn get_part_returns_pool_part_details() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&eagle_fixture_path("bjt-sot23.lbr"))
            .expect("fixture should import");
        let part_uuid = engine
            .search_pool("sot23")
            .expect("search should succeed")
            .first()
            .expect("part should exist")
            .uuid;
        let part = engine
            .get_part(&part_uuid)
            .expect("part query should succeed");
        assert_eq!(part.uuid, part_uuid);
        assert!(!part.package.name.is_empty());
        assert!(part.package.pads > 0);
        assert!(!part.entity.gates.is_empty());
    }

    #[test]
    fn get_package_returns_pool_package_details() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&eagle_fixture_path("bjt-sot23.lbr"))
            .expect("fixture should import");
        let part_uuid = engine
            .search_pool("sot23")
            .expect("search should succeed")
            .first()
            .expect("part should exist")
            .uuid;
        let part = engine
            .get_part(&part_uuid)
            .expect("part query should succeed");
        let package_uuid = engine
            .pool
            .parts
            .get(&part_uuid)
            .expect("part should exist in pool")
            .package;
        let package = engine
            .get_package(&package_uuid)
            .expect("package query should succeed");
        assert_eq!(package.uuid, package_uuid);
        assert_eq!(package.name, part.package.name);
        assert!(!package.pads.is_empty());
    }

    #[test]
    fn get_netlist_returns_board_nets_for_board_project() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("simple-demo.kicad_pcb"))
            .expect("fixture should import");
        let nets = engine.get_netlist().expect("netlist query should succeed");
        assert_eq!(nets.len(), 2);
        let gnd = nets
            .iter()
            .find(|net| net.name == "GND")
            .expect("GND net should exist");
        assert_eq!(gnd.class.as_deref(), Some("Default"));
        assert!(gnd.routed_pct.is_some());
        assert!(gnd.labels.is_none());
        assert!(gnd.ports.is_none());
    }

    #[test]
    fn explain_violation_returns_erc_explanation_for_valid_index() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("simple-demo.kicad_sch"))
            .expect("fixture should import");
        let explanation = engine
            .explain_violation(ViolationDomain::Erc, 0)
            .expect("explanation should succeed");
        assert!(!explanation.explanation.is_empty());
        assert!(explanation.rule_detail.starts_with("erc "));
        assert!(!explanation.suggestion.is_empty());
    }

    #[test]
    fn explain_violation_returns_drc_explanation_for_valid_index() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("partial-route-demo.kicad_pcb"))
            .expect("fixture should import");
        let explanation = engine
            .explain_violation(ViolationDomain::Drc, 0)
            .expect("explanation should succeed");
        assert!(!explanation.explanation.is_empty());
        assert!(explanation.rule_detail.starts_with("drc "));
        assert!(!explanation.suggestion.is_empty());
    }

    #[test]
    fn get_netlist_returns_schematic_nets_for_schematic_project() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("simple-demo.kicad_sch"))
            .expect("fixture should import");
        let nets = engine.get_netlist().expect("netlist query should succeed");
        assert_eq!(nets.len(), 4);
        let vcc = nets
            .iter()
            .find(|net| net.name == "VCC")
            .expect("VCC net should exist");
        assert_eq!(vcc.semantic_class.as_deref(), Some("power"));
        assert!(vcc.routed_pct.is_none());
        assert_eq!(vcc.labels, Some(1));
    }

    #[test]
    fn erc_golden_simple_demo_matches_checked_in_fixture() {
        assert_erc_matches_golden("simple-demo.kicad_sch");
    }

    #[test]
    fn erc_golden_analog_input_demo_matches_checked_in_fixture() {
        assert_erc_matches_golden("analog-input-demo.kicad_sch");
    }

    #[test]
    fn erc_golden_analog_input_bias_demo_matches_checked_in_fixture() {
        assert_erc_matches_golden("analog-input-bias-demo.kicad_sch");
    }

    #[test]
    fn erc_golden_coverage_demo_matches_checked_in_fixture() {
        assert_erc_matches_golden("erc-coverage-demo.kicad_sch");
    }

    #[test]
    fn erc_golden_hierarchy_mismatch_demo_matches_checked_in_fixture() {
        assert_erc_matches_golden("hierarchy-mismatch-demo.kicad_sch");
    }

    #[test]
    fn drc_golden_simple_demo_matches_checked_in_fixture() {
        assert_drc_matches_golden("simple-demo.kicad_pcb");
    }

    #[test]
    fn drc_golden_partial_route_demo_matches_checked_in_fixture() {
        assert_drc_matches_golden("partial-route-demo.kicad_pcb");
    }

    #[test]
    fn drc_golden_clearance_violation_demo_matches_checked_in_fixture() {
        assert_drc_matches_golden("clearance-violation-demo.kicad_pcb");
    }

    #[test]
    fn drc_golden_coverage_demo_matches_checked_in_fixture() {
        assert_drc_matches_golden("drc-coverage-demo.kicad_pcb");
    }

    #[test]
    fn drc_golden_silk_clearance_demo_matches_checked_in_fixture() {
        assert_drc_matches_golden("silk-clearance-demo.kicad_pcb");
    }

    #[test]
    fn drc_golden_airwire_demo_matches_checked_in_fixture() {
        assert_drc_matches_golden("airwire-demo.kicad_pcb");
    }

    #[test]
    fn erc_golden_corpus_covers_required_m2_codes_for_current_implementation_slice() {
        let fixtures = [
            "simple-demo.kicad_sch",
            "analog-input-demo.kicad_sch",
            "erc-coverage-demo.kicad_sch",
            "hierarchy-mismatch-demo.kicad_sch",
        ];
        let mut seen = std::collections::BTreeSet::<String>::new();
        for fixture in fixtures {
            let mut engine = Engine::new().expect("engine should initialize");
            engine
                .import(&fixture_path(fixture))
                .unwrap_or_else(|err| panic!("fixture should import: {err}"));
            for finding in engine
                .run_erc_prechecks()
                .unwrap_or_else(|err| panic!("ERC should run: {err}"))
            {
                seen.insert(finding.code.to_string());
            }
        }
        for required in [
            "output_to_output_conflict",
            "undriven_input_pin",
            "input_without_explicit_driver",
            "power_in_without_source",
            "unconnected_component_pin",
            "undriven_power_net",
            "noconnect_connected",
            "hierarchical_connectivity_mismatch",
        ] {
            assert!(
                seen.contains(required),
                "ERC golden corpus missing required code: {required}"
            );
        }
    }

    fn assert_erc_matches_golden(fixture: &str) {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path(fixture))
            .unwrap_or_else(|err| panic!("fixture should import: {err}"));
        let findings = engine
            .run_erc_prechecks()
            .unwrap_or_else(|err| panic!("ERC should run: {err}"));
        let normalized = normalize_erc_findings(&findings);
        let actual = to_json_deterministic(&normalized)
            .unwrap_or_else(|err| panic!("failed to serialize ERC findings: {err}"));

        let golden = golden_path_for_erc_fixture(fixture);
        if std::env::var_os("UPDATE_GOLDENS").is_some() {
            if let Some(parent) = golden.parent() {
                fs::create_dir_all(parent).unwrap_or_else(|err| {
                    panic!(
                        "failed to create golden directory {}: {err}",
                        parent.display()
                    )
                });
            }
            fs::write(&golden, &actual)
                .unwrap_or_else(|err| panic!("failed to write golden {}: {err}", golden.display()));
            return;
        }

        let expected = fs::read_to_string(&golden).unwrap_or_else(|err| {
            panic!(
                "failed to read golden {} (set UPDATE_GOLDENS=1 to regenerate): {err}",
                golden.display()
            )
        });
        assert_eq!(
            actual, expected,
            "ERC golden mismatch for fixture {}",
            fixture
        );
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
    struct ErcGoldenFinding {
        code: String,
        severity: String,
        message: String,
        net_name: Option<String>,
        component: Option<String>,
        pin: Option<String>,
        objects: Vec<(String, String)>,
        waived: bool,
    }

    fn normalize_erc_findings(findings: &[ErcFinding]) -> Vec<ErcGoldenFinding> {
        let mut normalized: Vec<_> = findings
            .iter()
            .map(|finding| ErcGoldenFinding {
                code: finding.code.to_string(),
                severity: format!("{:?}", finding.severity),
                message: finding.message.clone(),
                net_name: finding.net_name.as_ref().map(|net| {
                    if net.starts_with("N$") {
                        "N$<anon>".to_string()
                    } else {
                        net.clone()
                    }
                }),
                component: finding.component.clone(),
                pin: finding.pin.clone(),
                objects: finding
                    .objects
                    .iter()
                    .map(|obj| (obj.kind.to_string(), obj.key.clone()))
                    .collect(),
                waived: finding.waived,
            })
            .collect();
        normalized.sort_by(|a, b| {
            a.code
                .cmp(&b.code)
                .then_with(|| a.net_name.cmp(&b.net_name))
                .then_with(|| a.component.cmp(&b.component))
                .then_with(|| a.pin.cmp(&b.pin))
                .then_with(|| a.message.cmp(&b.message))
        });
        normalized
    }

    fn assert_drc_matches_golden(fixture: &str) {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path(fixture))
            .unwrap_or_else(|err| panic!("fixture should import: {err}"));
        let report = engine
            .run_drc(&[
                RuleType::Connectivity,
                RuleType::ClearanceCopper,
                RuleType::TrackWidth,
                RuleType::ViaHole,
                RuleType::ViaAnnularRing,
                RuleType::SilkClearance,
            ])
            .unwrap_or_else(|err| panic!("DRC should run: {err}"));
        let normalized = normalize_drc_report(&report);
        let actual = to_json_deterministic(&normalized)
            .unwrap_or_else(|err| panic!("failed to serialize DRC report: {err}"));

        let golden = golden_path_for_drc_fixture(fixture);
        if std::env::var_os("UPDATE_GOLDENS").is_some() {
            if let Some(parent) = golden.parent() {
                fs::create_dir_all(parent).unwrap_or_else(|err| {
                    panic!(
                        "failed to create golden directory {}: {err}",
                        parent.display()
                    )
                });
            }
            fs::write(&golden, &actual)
                .unwrap_or_else(|err| panic!("failed to write golden {}: {err}", golden.display()));
            return;
        }

        let expected = fs::read_to_string(&golden).unwrap_or_else(|err| {
            panic!(
                "failed to read golden {} (set UPDATE_GOLDENS=1 to regenerate): {err}",
                golden.display()
            )
        });
        assert_eq!(
            actual, expected,
            "DRC golden mismatch for fixture {}",
            fixture
        );
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
    struct DrcGoldenReport {
        passed: bool,
        summary: (usize, usize),
        violations: Vec<DrcGoldenViolation>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
    struct DrcGoldenViolation {
        code: String,
        rule_type: String,
        severity: String,
        message: String,
        location: Option<(i64, i64, Option<i32>)>,
        objects: Vec<String>,
    }

    fn normalize_drc_report(report: &DrcReport) -> DrcGoldenReport {
        let mut violations: Vec<_> = report
            .violations
            .iter()
            .map(|violation| DrcGoldenViolation {
                code: violation.code.clone(),
                rule_type: format!("{:?}", violation.rule_type),
                severity: match violation.severity {
                    crate::drc::DrcSeverity::Error => "error".to_string(),
                    crate::drc::DrcSeverity::Warning => "warning".to_string(),
                },
                message: violation.message.clone(),
                location: violation
                    .location
                    .as_ref()
                    .map(|loc| (loc.x_nm, loc.y_nm, loc.layer)),
                objects: violation
                    .objects
                    .iter()
                    .map(|uuid| uuid.to_string())
                    .collect(),
            })
            .collect();
        violations.sort_by(|a, b| {
            a.code
                .cmp(&b.code)
                .then_with(|| a.message.cmp(&b.message))
                .then_with(|| a.objects.cmp(&b.objects))
        });
        DrcGoldenReport {
            passed: report.passed,
            summary: (report.summary.errors, report.summary.warnings),
            violations,
        }
    }

    fn golden_path_for_erc_fixture(fixture: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("testdata/golden/erc")
            .join(format!("{fixture}.json"))
    }

    fn golden_path_for_drc_fixture(fixture: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("testdata/golden/drc")
            .join(format!("{fixture}.json"))
    }

    #[test]
    fn run_drc_returns_connectivity_violation_for_partial_route_fixture() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("partial-route-demo.kicad_pcb"))
            .expect("partial-route fixture import should succeed");

        let report = engine
            .run_drc(&[RuleType::Connectivity])
            .expect("drc should run on imported board");

        assert!(!report.passed);
        assert!(report.summary.errors >= 1);
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.code == "connectivity_unrouted_net")
        );
    }

    #[test]
    fn get_design_rules_returns_empty_rules_for_imported_fixture() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("simple-demo.kicad_pcb"))
            .expect("fixture import should succeed");
        let rules = engine
            .get_design_rules()
            .expect("design rules should query");
        assert!(rules.is_empty());
    }

    #[test]
    fn save_writes_byte_identical_kicad_board_for_current_m3_slice() {
        let source = fixture_path("simple-demo.kicad_pcb");
        let expected = fs::read_to_string(&source).expect("fixture should read");
        let target = unique_temp_path("datum-eda-save-simple-demo.kicad_pcb");

        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&source)
            .expect("fixture import should succeed");
        engine.save(&target).expect("save should succeed");

        let actual = fs::read_to_string(&target).expect("saved file should read");
        assert_eq!(actual, expected);

        let _ = fs::remove_file(&target);
    }

    #[test]
    fn save_rejects_non_board_projects_in_current_m3_slice() {
        let source = fixture_path("simple-demo.kicad_sch");
        let target = unique_temp_path("datum-eda-save-simple-demo.kicad_sch");

        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&source)
            .expect("fixture import should succeed");
        let err = engine
            .save(&target)
            .expect_err("schematic save should be rejected");
        assert!(
            err.to_string()
                .contains("save is currently implemented only for imported KiCad boards"),
            "{err}"
        );
    }

    #[test]
    fn delete_track_updates_board_and_undo_redo_restore_it() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("partial-route-demo.kicad_pcb"))
            .expect("fixture import should succeed");

        let before = engine.get_net_info().expect("net info should query");
        let deleted_uuid = engine
            .design
            .as_ref()
            .and_then(|design| design.board.as_ref())
            .and_then(|board| board.tracks.keys().next().copied())
            .expect("fixture should have at least one track");

        let delete = engine
            .delete_track(&deleted_uuid)
            .expect("delete_track should succeed");
        assert_eq!(delete.diff.deleted.len(), 1);
        assert!(engine.can_undo());

        let after_delete = engine.get_net_info().expect("net info should query");
        assert_ne!(before, after_delete);

        let undo = engine.undo().expect("undo should succeed");
        assert_eq!(undo.diff.created.len(), 1);
        let after_undo = engine.get_net_info().expect("net info should query");
        assert_eq!(before, after_undo);
        assert!(engine.can_redo());

        let redo = engine.redo().expect("redo should succeed");
        assert_eq!(redo.diff.deleted.len(), 1);
        let after_redo = engine.get_net_info().expect("net info should query");
        assert_eq!(after_delete, after_redo);
    }

    #[test]
    fn delete_via_updates_board_and_undo_redo_restore_it() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("simple-demo.kicad_pcb"))
            .expect("fixture import should succeed");

        let before = engine.get_net_info().expect("net info should query");
        let deleted_uuid = engine
            .design
            .as_ref()
            .and_then(|design| design.board.as_ref())
            .and_then(|board| board.vias.keys().next().copied())
            .expect("fixture should have at least one via");

        let delete = engine
            .delete_via(&deleted_uuid)
            .expect("delete_via should succeed");
        assert_eq!(delete.diff.deleted.len(), 1);
        assert_eq!(delete.diff.deleted[0].object_type, "via");
        assert!(engine.can_undo());

        let after_delete = engine.get_net_info().expect("net info should query");
        assert_ne!(before, after_delete);

        let undo = engine.undo().expect("undo should succeed");
        assert_eq!(undo.diff.created.len(), 1);
        assert_eq!(undo.diff.created[0].object_type, "via");
        let after_undo = engine.get_net_info().expect("net info should query");
        assert_eq!(before, after_undo);
        assert!(engine.can_redo());

        let redo = engine.redo().expect("redo should succeed");
        assert_eq!(redo.diff.deleted.len(), 1);
        assert_eq!(redo.diff.deleted[0].object_type, "via");
        let after_redo = engine.get_net_info().expect("net info should query");
        assert_eq!(after_delete, after_redo);
    }

    #[test]
    fn delete_component_updates_board_and_undo_redo_restore_it() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("partial-route-demo.kicad_pcb"))
            .expect("fixture import should succeed");

        let before = engine.get_components().expect("components should query");
        let deleted_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();

        let delete = engine
            .delete_component(&deleted_uuid)
            .expect("delete_component should succeed");
        assert_eq!(delete.diff.deleted.len(), 1);
        assert_eq!(delete.diff.deleted[0].object_type, "component");
        assert!(engine.can_undo());

        let after_delete = engine.get_components().expect("components should query");
        assert!(after_delete.iter().all(|component| component.uuid != deleted_uuid));

        let undo = engine.undo().expect("undo should succeed");
        assert_eq!(undo.diff.created.len(), 1);
        assert_eq!(undo.diff.created[0].object_type, "component");
        let after_undo = engine.get_components().expect("components should query");
        assert_eq!(before, after_undo);
        assert!(engine.can_redo());

        let redo = engine.redo().expect("redo should succeed");
        assert_eq!(redo.diff.deleted.len(), 1);
        assert_eq!(redo.diff.deleted[0].object_type, "component");
        let after_redo = engine.get_components().expect("components should query");
        assert_eq!(after_delete, after_redo);
    }

    #[test]
    fn delete_track_updates_derived_board_views_immediately() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("partial-route-demo.kicad_pcb"))
            .expect("fixture import should succeed");

        let baseline_sig = engine
            .get_net_info()
            .expect("net info should query")
            .into_iter()
            .find(|net| net.name == "SIG")
            .expect("SIG net should exist");
        let baseline_airwires = engine.get_unrouted().expect("unrouted should query");
        let baseline_diagnostics = engine
            .get_connectivity_diagnostics()
            .expect("diagnostics should query");
        let baseline_drc = engine
            .run_drc(&[RuleType::Connectivity])
            .expect("drc should run");

        assert_eq!(baseline_sig.tracks, 1);
        assert_eq!(baseline_airwires.len(), 1);
        assert!(
            baseline_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.kind == "partially_routed_net")
        );
        assert!(
            baseline_drc
                .violations
                .iter()
                .any(|violation| violation.code == "connectivity_unrouted_net")
        );
        assert!(
            !baseline_drc
                .violations
                .iter()
                .any(|violation| violation.code == "connectivity_no_copper")
        );

        engine
            .delete_track(&uuid::Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").unwrap())
            .expect("delete_track should succeed");

        let after_sig = engine
            .get_net_info()
            .expect("net info should query")
            .into_iter()
            .find(|net| net.name == "SIG")
            .expect("SIG net should exist");
        let after_airwires = engine.get_unrouted().expect("unrouted should query");
        let after_diagnostics = engine
            .get_connectivity_diagnostics()
            .expect("diagnostics should query");
        let after_drc = engine
            .run_drc(&[RuleType::Connectivity])
            .expect("drc should run");

        assert_eq!(after_sig.tracks, 0);
        assert_eq!(after_sig.routed_length_nm, 0);
        assert_eq!(after_airwires.len(), baseline_airwires.len());
        assert!(
            after_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.kind == "net_without_copper")
        );
        assert!(
            !after_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.kind == "partially_routed_net")
        );
        assert!(
            after_drc
                .violations
                .iter()
                .any(|violation| violation.code == "connectivity_no_copper")
        );
        assert!(
            after_drc
                .violations
                .iter()
                .any(|violation| violation.code == "connectivity_unrouted_net")
        );
    }

    #[test]
    fn delete_via_updates_derived_net_state_immediately() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("simple-demo.kicad_pcb"))
            .expect("fixture import should succeed");

        let baseline_gnd = engine
            .get_net_info()
            .expect("net info should query")
            .into_iter()
            .find(|net| net.name == "GND")
            .expect("GND net should exist");
        assert_eq!(baseline_gnd.vias, 1);

        engine
            .delete_via(&uuid::Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").unwrap())
            .expect("delete_via should succeed");

        let after_gnd = engine
            .get_net_info()
            .expect("net info should query")
            .into_iter()
            .find(|net| net.name == "GND")
            .expect("GND net should exist");
        assert_eq!(after_gnd.vias, 0);
        assert_eq!(after_gnd.tracks, baseline_gnd.tracks);
        assert_eq!(after_gnd.routed_length_nm, baseline_gnd.routed_length_nm);
    }

    #[test]
    fn save_persists_deleted_track_for_current_m3_slice() {
        let source = fixture_path("partial-route-demo.kicad_pcb");
        let target = unique_temp_path("datum-eda-save-modified-board.kicad_pcb");

        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&source)
            .expect("fixture import should succeed");
        let deleted_uuid = engine
            .design
            .as_ref()
            .and_then(|design| design.board.as_ref())
            .and_then(|board| board.tracks.keys().next().copied())
            .expect("fixture should have at least one track");
        engine
            .delete_track(&deleted_uuid)
            .expect("delete_track should succeed");

        let expected_after_delete = engine.get_net_info().expect("net info should query");

        engine
            .save(&target)
            .expect("save should persist current delete_track slice");

        let saved = fs::read_to_string(&target).expect("saved file should read");
        assert!(!saved.contains(&deleted_uuid.to_string()));

        let mut reloaded = Engine::new().expect("engine should initialize");
        reloaded
            .import(&target)
            .expect("saved board should reimport successfully");
        let actual_after_reload = reloaded.get_net_info().expect("net info should query");
        assert_eq!(actual_after_reload, expected_after_delete);

        let _ = fs::remove_file(&target);
    }

    #[test]
    fn save_persists_deleted_via_for_current_m3_slice() {
        let source = fixture_path("simple-demo.kicad_pcb");
        let target = unique_temp_path("datum-eda-save-modified-via-board.kicad_pcb");

        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&source)
            .expect("fixture import should succeed");
        let deleted_uuid = engine
            .design
            .as_ref()
            .and_then(|design| design.board.as_ref())
            .and_then(|board| board.vias.keys().next().copied())
            .expect("fixture should have at least one via");
        engine
            .delete_via(&deleted_uuid)
            .expect("delete_via should succeed");

        let expected_after_delete = engine.get_net_info().expect("net info should query");

        engine
            .save(&target)
            .expect("save should persist current delete_via slice");

        let saved = fs::read_to_string(&target).expect("saved file should read");
        assert!(!saved.contains(&deleted_uuid.to_string()));

        let mut reloaded = Engine::new().expect("engine should initialize");
        reloaded
            .import(&target)
            .expect("saved board should reimport successfully");
        let actual_after_reload = reloaded.get_net_info().expect("net info should query");
        assert_eq!(actual_after_reload, expected_after_delete);

        let _ = fs::remove_file(&target);
    }

    #[test]
    fn save_persists_deleted_component_for_current_m3_slice() {
        let source = fixture_path("partial-route-demo.kicad_pcb");
        let target = unique_temp_path("datum-eda-save-deleted-component-board.kicad_pcb");

        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&source)
            .expect("fixture import should succeed");
        let deleted_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        engine
            .delete_component(&deleted_uuid)
            .expect("delete_component should succeed");

        let expected_after_delete = engine.get_components().expect("components should query");

        engine
            .save(&target)
            .expect("save should persist current delete_component slice");

        let saved = fs::read_to_string(&target).expect("saved file should read");
        assert!(!saved.contains(&deleted_uuid.to_string()));

        let mut reloaded = Engine::new().expect("engine should initialize");
        reloaded
            .import(&target)
            .expect("saved board should reimport successfully");
        let actual_after_reload = reloaded.get_components().expect("components should query");
        assert_eq!(actual_after_reload, expected_after_delete);

        let _ = fs::remove_file(&target);
    }

    #[test]
    fn set_design_rule_updates_board_and_undo_redo_restore_it() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("simple-demo.kicad_pcb"))
            .expect("fixture import should succeed");

        let before = engine.get_design_rules().expect("rules should query");
        assert!(before.is_empty());

        let set = engine
            .set_design_rule(SetDesignRuleInput {
                rule_type: RuleType::ClearanceCopper,
                scope: crate::rules::ast::RuleScope::All,
                parameters: crate::rules::ast::RuleParams::Clearance { min: 125_000 },
                priority: 10,
                name: Some("default clearance".to_string()),
            })
            .expect("set_design_rule should succeed");
        assert_eq!(set.diff.created.len(), 1);

        let after_set = engine.get_design_rules().expect("rules should query");
        assert_eq!(after_set.len(), 1);
        assert_eq!(after_set[0].name, "default clearance");

        let undo = engine.undo().expect("undo should succeed");
        assert_eq!(undo.diff.deleted.len(), 1);
        let after_undo = engine.get_design_rules().expect("rules should query");
        assert!(after_undo.is_empty());

        let redo = engine.redo().expect("redo should succeed");
        assert_eq!(redo.diff.created.len(), 1);
        let after_redo = engine.get_design_rules().expect("rules should query");
        assert_eq!(after_redo.len(), 1);
        assert_eq!(after_redo[0].name, "default clearance");
    }

    #[test]
    fn set_value_updates_board_and_undo_redo_restore_it() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("partial-route-demo.kicad_pcb"))
            .expect("fixture import should succeed");

        let before = engine.get_components().expect("components should query");
        let package_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let set_result = engine
            .set_value(SetValueInput {
                uuid: package_uuid,
                value: "22k".to_string(),
            })
            .expect("set_value should succeed");
        assert_eq!(set_result.diff.modified.len(), 1);

        let after_set = engine.get_components().expect("components should query");
        let r1_after_set = after_set
            .iter()
            .find(|component| component.reference == "R1")
            .unwrap();
        assert_eq!(r1_after_set.value, "22k");

        let undo = engine.undo().expect("undo should succeed");
        assert_eq!(undo.diff.modified.len(), 1);
        let after_undo = engine.get_components().expect("components should query");
        assert_eq!(before, after_undo);

        let redo = engine.redo().expect("redo should succeed");
        assert_eq!(redo.diff.modified.len(), 1);
        let after_redo = engine.get_components().expect("components should query");
        assert_eq!(after_set, after_redo);
    }

    #[test]
    fn save_persists_set_value_for_current_m3_slice() {
        let source = fixture_path("partial-route-demo.kicad_pcb");
        let target = unique_temp_path("datum-eda-save-set-value-board.kicad_pcb");

        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&source)
            .expect("fixture import should succeed");
        engine
            .set_value(SetValueInput {
                uuid: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                value: "22k".to_string(),
            })
            .expect("set_value should succeed");

        engine.save(&target).expect("save should succeed");

        let saved = fs::read_to_string(&target).expect("saved file should read");
        assert!(saved.contains("(property \"Value\" \"22k\""));

        let mut reloaded = Engine::new().expect("engine should initialize");
        reloaded
            .import(&target)
            .expect("saved board should reimport successfully");
        let components = reloaded.get_components().expect("components should query");
        let r1 = components
            .iter()
            .find(|component| component.reference == "R1")
            .unwrap();
        assert_eq!(r1.value, "22k");

        let _ = fs::remove_file(&target);
    }

    #[test]
    fn set_reference_updates_board_and_undo_redo_restore_it() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("partial-route-demo.kicad_pcb"))
            .expect("fixture import should succeed");

        let before = engine.get_components().expect("components should query");
        let package_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let set_result = engine
            .set_reference(SetReferenceInput {
                uuid: package_uuid,
                reference: "R10".to_string(),
            })
            .expect("set_reference should succeed");
        assert_eq!(set_result.diff.modified.len(), 1);

        let after_set = engine.get_components().expect("components should query");
        let r1_after_set = after_set
            .iter()
            .find(|component| component.uuid == package_uuid)
            .unwrap();
        assert_eq!(r1_after_set.reference, "R10");

        let undo = engine.undo().expect("undo should succeed");
        assert_eq!(undo.diff.modified.len(), 1);
        let after_undo = engine.get_components().expect("components should query");
        assert_eq!(before, after_undo);

        let redo = engine.redo().expect("redo should succeed");
        assert_eq!(redo.diff.modified.len(), 1);
        let after_redo = engine.get_components().expect("components should query");
        assert_eq!(after_set, after_redo);
    }

    #[test]
    fn save_persists_set_reference_for_current_m3_slice() {
        let source = fixture_path("partial-route-demo.kicad_pcb");
        let target = unique_temp_path("datum-eda-save-set-reference-board.kicad_pcb");

        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&source)
            .expect("fixture import should succeed");
        engine
            .set_reference(SetReferenceInput {
                uuid: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                reference: "R10".to_string(),
            })
            .expect("set_reference should succeed");

        engine.save(&target).expect("save should succeed");

        let saved = fs::read_to_string(&target).expect("saved file should read");
        assert!(saved.contains("(property \"Reference\" \"R10\""));

        let mut reloaded = Engine::new().expect("engine should initialize");
        reloaded
            .import(&target)
            .expect("saved board should reimport successfully");
        let components = reloaded.get_components().expect("components should query");
        let r1 = components
            .iter()
            .find(|component| component.uuid == uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
            .unwrap();
        assert_eq!(r1.reference, "R10");

        let _ = fs::remove_file(&target);
    }

    #[test]
    fn rotate_component_updates_board_and_undo_redo_restore_it() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("partial-route-demo.kicad_pcb"))
            .expect("fixture import should succeed");

        let before = engine.get_components().expect("components should query");
        let package_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let rotate_result = engine
            .rotate_component(RotateComponentInput {
                uuid: package_uuid,
                rotation: 180,
            })
            .expect("rotate_component should succeed");
        assert_eq!(rotate_result.diff.modified.len(), 1);

        let after_rotate = engine.get_components().expect("components should query");
        let rotated = after_rotate
            .iter()
            .find(|component| component.uuid == package_uuid)
            .unwrap();
        assert_eq!(rotated.rotation, 180);

        let undo = engine.undo().expect("undo should succeed");
        assert_eq!(undo.diff.modified.len(), 1);
        let after_undo = engine.get_components().expect("components should query");
        assert_eq!(before, after_undo);

        let redo = engine.redo().expect("redo should succeed");
        assert_eq!(redo.diff.modified.len(), 1);
        let after_redo = engine.get_components().expect("components should query");
        assert_eq!(after_rotate, after_redo);
    }

    #[test]
    fn save_persists_rotate_component_for_current_m3_slice() {
        let source = fixture_path("partial-route-demo.kicad_pcb");
        let target = unique_temp_path("datum-eda-save-rotate-component-board.kicad_pcb");

        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&source)
            .expect("fixture import should succeed");
        engine
            .rotate_component(RotateComponentInput {
                uuid: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                rotation: 180,
            })
            .expect("rotate_component should succeed");

        engine.save(&target).expect("save should succeed");

        let saved = fs::read_to_string(&target).expect("saved file should read");
        assert!(saved.contains("(at 10 10 180)"));

        let mut reloaded = Engine::new().expect("engine should initialize");
        reloaded
            .import(&target)
            .expect("saved board should reimport successfully");
        let components = reloaded.get_components().expect("components should query");
        let rotated = components
            .iter()
            .find(|component| component.uuid == uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
            .unwrap();
        assert_eq!(rotated.rotation, 180);

        let _ = fs::remove_file(&target);
    }

    #[test]
    fn assign_part_updates_board_and_undo_redo_restore_it() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        engine
            .import(&fixture_path("partial-route-demo.kicad_pcb"))
            .expect("fixture import should succeed");

        let part_uuid = engine
            .search_pool("ALTAMP")
            .expect("search should succeed")
            .first()
            .map(|part| part.uuid)
            .expect("ALTAMP part should exist");
        let before = engine.get_components().expect("components should query");
        let package_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let assign_result = engine
            .assign_part(AssignPartInput {
                uuid: package_uuid,
                part_uuid,
            })
            .expect("assign_part should succeed");
        assert_eq!(assign_result.diff.modified.len(), 1);

        let after_assign = engine.get_components().expect("components should query");
        let updated = after_assign
            .iter()
            .find(|component| component.uuid == package_uuid)
            .unwrap();
        assert_eq!(updated.value, "ALTAMP");
        assert_eq!(
            updated.package_uuid,
            uuid::Uuid::parse_str("3bbffc1f-f562-563a-b9da-4e0d73ab019e").unwrap()
        );
        let sig = engine
            .get_net_info()
            .expect("net info should query")
            .into_iter()
            .find(|net| net.name == "SIG")
            .expect("SIG net should exist");
        assert_eq!(sig.pins.len(), 1);

        let undo = engine.undo().expect("undo should succeed");
        assert_eq!(undo.diff.modified.len(), 1);
        let after_undo = engine.get_components().expect("components should query");
        assert_eq!(before, after_undo);

        let redo = engine.redo().expect("redo should succeed");
        assert_eq!(redo.diff.modified.len(), 1);
        let after_redo = engine.get_components().expect("components should query");
        assert_eq!(after_assign, after_redo);
    }

    #[test]
    fn save_persists_assign_part_for_current_m3_slice() {
        let source = fixture_path("partial-route-demo.kicad_pcb");
        let target = unique_temp_path("datum-eda-save-assign-part-board.kicad_pcb");

        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        engine
            .import(&source)
            .expect("fixture import should succeed");
        let part_uuid = engine
            .search_pool("ALTAMP")
            .expect("search should succeed")
            .first()
            .map(|part| part.uuid)
            .expect("ALTAMP part should exist");
        engine
            .assign_part(AssignPartInput {
                uuid: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                part_uuid,
            })
            .expect("assign_part should succeed");

        engine.save(&target).expect("save should succeed");

        let saved = fs::read_to_string(&target).expect("saved file should read");
        assert!(saved.contains("(property \"Value\" \"ALTAMP\""));
        assert!(saved.contains("(footprint \"ALT-3\""));

        let mut reloaded = Engine::new().expect("engine should initialize");
        reloaded
            .import(&target)
            .expect("saved board should reimport successfully");
        let components = reloaded.get_components().expect("components should query");
        let updated = components
            .iter()
            .find(|component| component.uuid == uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
            .unwrap();
        assert_eq!(updated.value, "ALTAMP");
        let restored_part = reloaded
            .design
            .as_ref()
            .and_then(|design| design.board.as_ref())
            .and_then(|board| board.packages.get(&uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap()))
            .map(|package| package.part)
            .expect("reloaded package should exist");
        assert_eq!(restored_part, part_uuid);
        let restored_sig = reloaded
            .get_net_info()
            .expect("reloaded net info should query")
            .into_iter()
            .find(|net| net.name == "SIG")
            .expect("reloaded SIG net should exist");
        assert_eq!(restored_sig.pins.len(), 1);

        let _ = fs::remove_file(&target);
        let _ = fs::remove_file(part_assignments_sidecar::sidecar_path_for_source(&target));
    }

    #[test]
    fn set_package_updates_board_and_undo_redo_restore_it() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        engine
            .import(&fixture_path("partial-route-demo.kicad_pcb"))
            .expect("fixture import should succeed");

        let package_uuid = engine
            .search_pool("ALTAMP")
            .expect("search should succeed")
            .first()
            .map(|part| part.package_uuid)
            .expect("ALTAMP package should exist");
        let before = engine.get_components().expect("components should query");
        let component_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let set_result = engine
            .set_package(SetPackageInput {
                uuid: component_uuid,
                package_uuid,
            })
            .expect("set_package should succeed");
        assert_eq!(set_result.diff.modified.len(), 1);

        let after_set = engine.get_components().expect("components should query");
        let updated = after_set
            .iter()
            .find(|component| component.uuid == component_uuid)
            .unwrap();
        assert_eq!(updated.package_uuid, package_uuid);
        let pad_count_after_set = engine
            .design
            .as_ref()
            .and_then(|design| design.board.as_ref())
            .map(|board| board.pads.values().filter(|pad| pad.package == component_uuid).count())
            .expect("board should exist");
        assert_eq!(pad_count_after_set, 3);
        let net_info_after_set = engine.get_net_info().expect("net info should query");
        let sig_after_set = net_info_after_set
            .iter()
            .find(|net| net.name == "SIG")
            .expect("SIG net should exist");
        assert_eq!(sig_after_set.pins.len(), 1);

        let undo = engine.undo().expect("undo should succeed");
        assert_eq!(undo.diff.modified.len(), 1);
        let after_undo = engine.get_components().expect("components should query");
        assert_eq!(before, after_undo);

        let redo = engine.redo().expect("redo should succeed");
        assert_eq!(redo.diff.modified.len(), 1);
        let after_redo = engine.get_components().expect("components should query");
        assert_eq!(after_set, after_redo);
    }

    #[test]
    fn save_persists_set_package_for_current_m3_slice() {
        let source = fixture_path("partial-route-demo.kicad_pcb");
        let target = unique_temp_path("datum-eda-save-set-package-board.kicad_pcb");

        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        engine
            .import(&source)
            .expect("fixture import should succeed");
        let package_uuid = engine
            .search_pool("ALTAMP")
            .expect("search should succeed")
            .first()
            .map(|part| part.package_uuid)
            .expect("ALTAMP package should exist");
        engine
            .set_package(SetPackageInput {
                uuid: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                package_uuid,
            })
            .expect("set_package should succeed");

        engine.save(&target).expect("save should succeed");

        let saved = fs::read_to_string(&target).expect("saved file should read");
        assert!(saved.contains("(footprint \"ALT-3\""));
        assert_eq!(saved.matches("(pad \"").count(), 4);

        let mut reloaded = Engine::new().expect("engine should initialize");
        reloaded
            .import(&target)
            .expect("saved board should reimport successfully");
        let components = reloaded.get_components().expect("components should query");
        let updated = components
            .iter()
            .find(|component| component.uuid == uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
            .unwrap();
        assert_eq!(updated.package_uuid, package_uuid);
        let restored_package = reloaded
            .design
            .as_ref()
            .and_then(|design| design.board.as_ref())
            .and_then(|board| board.packages.get(&uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap()))
            .map(|package| package.package)
            .expect("reloaded package should exist");
        assert_eq!(restored_package, package_uuid);

        let _ = fs::remove_file(&target);
        let _ = fs::remove_file(package_assignments_sidecar::sidecar_path_for_source(&target));
    }

    #[test]
    fn set_net_class_updates_board_and_undo_redo_restore_it() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("simple-demo.kicad_pcb"))
            .expect("fixture import should succeed");

        let before = engine.get_net_info().expect("net info should query");
        let gnd = before
            .iter()
            .find(|net| net.name == "GND")
            .expect("GND net should exist");
        assert_eq!(gnd.class, "Default");

        let set_result = engine
            .set_net_class(SetNetClassInput {
                net_uuid: gnd.uuid,
                class_name: "power".to_string(),
                clearance: 125_000,
                track_width: 250_000,
                via_drill: 300_000,
                via_diameter: 600_000,
                diffpair_width: 0,
                diffpair_gap: 0,
            })
            .expect("set_net_class should succeed");
        assert_eq!(set_result.diff.modified.len(), 1);

        let after_set = engine.get_net_info().expect("net info should query");
        let gnd_after_set = after_set
            .iter()
            .find(|net| net.name == "GND")
            .expect("GND net should exist after set");
        assert_eq!(gnd_after_set.class, "power");

        let undo = engine.undo().expect("undo should succeed");
        assert_eq!(undo.diff.modified.len(), 1);
        let after_undo = engine.get_net_info().expect("net info should query");
        let gnd_after_undo = after_undo
            .iter()
            .find(|net| net.name == "GND")
            .expect("GND net should exist after undo");
        assert_eq!(gnd_after_undo.class, "Default");

        let redo = engine.redo().expect("redo should succeed");
        assert_eq!(redo.diff.modified.len(), 1);
        let after_redo = engine.get_net_info().expect("net info should query");
        let gnd_after_redo = after_redo
            .iter()
            .find(|net| net.name == "GND")
            .expect("GND net should exist after redo");
        assert_eq!(gnd_after_redo.class, "power");
    }

    #[test]
    fn save_persists_set_net_class_sidecar_for_current_m3_slice() {
        let source = fixture_path("simple-demo.kicad_pcb");
        let target = unique_temp_path("datum-eda-save-net-class-board.kicad_pcb");

        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&source)
            .expect("fixture import should succeed");
        let gnd_uuid = engine
            .get_net_info()
            .expect("net info should query")
            .into_iter()
            .find(|net| net.name == "GND")
            .expect("GND net should exist")
            .uuid;
        engine
            .set_net_class(SetNetClassInput {
                net_uuid: gnd_uuid,
                class_name: "power".to_string(),
                clearance: 125_000,
                track_width: 250_000,
                via_drill: 300_000,
                via_diameter: 600_000,
                diffpair_width: 0,
                diffpair_gap: 0,
            })
            .expect("set_net_class should succeed");

        engine.save(&target).expect("save should succeed");

        let sidecar = net_classes_sidecar::sidecar_path_for_source(&target);
        assert!(sidecar.exists());

        let mut reloaded = Engine::new().expect("engine should initialize");
        reloaded
            .import(&target)
            .expect("saved board should reimport successfully");
        let gnd = reloaded
            .get_net_info()
            .expect("net info should query")
            .into_iter()
            .find(|net| net.name == "GND")
            .expect("GND net should exist after reload");
        assert_eq!(gnd.class, "power");

        let _ = fs::remove_file(&target);
        let _ = fs::remove_file(sidecar);
    }

    #[test]
    fn save_persists_set_design_rule_sidecar_for_current_m3_slice() {
        let source = fixture_path("simple-demo.kicad_pcb");
        let target = unique_temp_path("datum-eda-save-rule-board.kicad_pcb");

        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&source)
            .expect("fixture import should succeed");
        engine
            .set_design_rule(SetDesignRuleInput {
                rule_type: RuleType::ClearanceCopper,
                scope: crate::rules::ast::RuleScope::All,
                parameters: crate::rules::ast::RuleParams::Clearance { min: 125_000 },
                priority: 10,
                name: Some("default clearance".to_string()),
            })
            .expect("set_design_rule should succeed");

        engine.save(&target).expect("save should succeed");

        let sidecar = crate::import::rules_sidecar::sidecar_path_for_source(&target);
        assert!(sidecar.exists());

        let mut reloaded = Engine::new().expect("engine should initialize");
        reloaded
            .import(&target)
            .expect("saved board should reimport successfully");
        let rules = reloaded.get_design_rules().expect("rules should query");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "default clearance");

        let _ = fs::remove_file(&target);
        let _ = fs::remove_file(&sidecar);
    }

    #[test]
    fn move_component_updates_board_and_undo_redo_restore_it() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("partial-route-demo.kicad_pcb"))
            .expect("fixture import should succeed");

        let before = engine.get_components().expect("components should query");
        let package_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let move_result = engine
            .move_component(MoveComponentInput {
                uuid: package_uuid,
                position: crate::ir::geometry::Point::new(15_000_000, 12_000_000),
                rotation: Some(90),
            })
            .expect("move_component should succeed");
        assert_eq!(move_result.diff.modified.len(), 1);

        let after_move = engine.get_components().expect("components should query");
        assert_ne!(before, after_move);

        let undo = engine.undo().expect("undo should succeed");
        assert_eq!(undo.diff.modified.len(), 1);
        let after_undo = engine.get_components().expect("components should query");
        assert_eq!(before, after_undo);

        let redo = engine.redo().expect("redo should succeed");
        assert_eq!(redo.diff.modified.len(), 1);
        let after_redo = engine.get_components().expect("components should query");
        assert_eq!(after_move, after_redo);
    }

    #[test]
    fn move_component_updates_derived_board_views_immediately() {
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&fixture_path("partial-route-demo.kicad_pcb"))
            .expect("fixture import should succeed");

        let baseline_airwires = engine.get_unrouted().expect("unrouted should query");
        let baseline_diagnostics = engine
            .get_connectivity_diagnostics()
            .expect("diagnostics should query");
        let baseline_drc = engine
            .run_drc(&[RuleType::Connectivity])
            .expect("drc should run");

        assert_eq!(baseline_airwires.len(), 1);
        assert!(
            baseline_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.kind == "partially_routed_net")
        );
        assert!(
            baseline_drc
                .violations
                .iter()
                .any(|violation| violation.code == "connectivity_unrouted_net")
        );

        engine
            .move_component(MoveComponentInput {
                uuid: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                position: crate::ir::geometry::Point::new(15_000_000, 12_000_000),
                rotation: Some(90),
            })
            .expect("move_component should succeed");

        let after_airwires = engine.get_unrouted().expect("unrouted should query");
        let after_diagnostics = engine
            .get_connectivity_diagnostics()
            .expect("diagnostics should query");
        let after_drc = engine
            .run_drc(&[RuleType::Connectivity])
            .expect("drc should run");

        assert_eq!(after_airwires.len(), 1);
        assert_ne!(
            after_airwires[0].distance_nm,
            baseline_airwires[0].distance_nm
        );
        assert!(
            after_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.kind == "partially_routed_net")
        );
        assert!(
            after_drc
                .violations
                .iter()
                .any(|violation| violation.code == "connectivity_unrouted_net")
        );
    }

    #[test]
    fn save_persists_moved_component_for_current_m3_slice() {
        let source = fixture_path("partial-route-demo.kicad_pcb");
        let target = unique_temp_path("datum-eda-save-moved-component-board.kicad_pcb");

        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import(&source)
            .expect("fixture import should succeed");
        engine
            .move_component(MoveComponentInput {
                uuid: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                position: crate::ir::geometry::Point::new(15_000_000, 12_000_000),
                rotation: Some(90),
            })
            .expect("move_component should succeed");

        engine.save(&target).expect("save should succeed");

        let saved = fs::read_to_string(&target).expect("saved file should read");
        assert!(saved.contains("(at 15 12 90)"));

        let mut reloaded = Engine::new().expect("engine should initialize");
        reloaded
            .import(&target)
            .expect("saved board should reimport successfully");
        let moved = reloaded.get_components().expect("components should query");
        let r1 = moved
            .iter()
            .find(|component| component.reference == "R1")
            .unwrap();
        assert_eq!(r1.position.x, 15_000_000);
        assert_eq!(r1.position.y, 12_000_000);
        assert_eq!(r1.rotation, 90);

        let _ = fs::remove_file(&target);
    }

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
