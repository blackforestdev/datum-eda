use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod check_summary;
use check_summary::{drc_suggestion, erc_suggestion, summarize_diagnostics, summarize_schematic_checks};
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
pub struct SetPackageWithPartInput {
    pub uuid: uuid::Uuid,
    pub package_uuid: uuid::Uuid,
    pub part_uuid: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplaceComponentInput {
    pub uuid: uuid::Uuid,
    pub package_uuid: uuid::Uuid,
    pub part_uuid: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedComponentReplacementInput {
    pub uuid: uuid::Uuid,
    pub package_uuid: Option<uuid::Uuid>,
    pub part_uuid: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentReplacementPolicy {
    BestCompatiblePackage,
    BestCompatiblePart,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyDrivenComponentReplacementInput {
    pub uuid: uuid::Uuid,
    pub policy: ComponentReplacementPolicy,
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
    Batch {
        description: String,
        records: Vec<TransactionRecord>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageChangeCompatibilityStatus {
    NoKnownPart,
    NoCompatiblePackages,
    CandidatesAvailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageChangeCandidate {
    pub package_uuid: uuid::Uuid,
    pub package_name: String,
    pub compatible_part_uuid: uuid::Uuid,
    pub compatible_part_value: String,
    pub pin_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageChangeCompatibilityReport {
    pub component_uuid: uuid::Uuid,
    pub current_part_uuid: Option<uuid::Uuid>,
    pub current_package_uuid: uuid::Uuid,
    pub current_package_name: String,
    pub current_value: String,
    pub status: PackageChangeCompatibilityStatus,
    pub ambiguous_package_count: usize,
    pub candidates: Vec<PackageChangeCandidate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartChangeCompatibilityStatus {
    NoKnownPart,
    NoCompatibleParts,
    CandidatesAvailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartChangeCandidate {
    pub part_uuid: uuid::Uuid,
    pub package_uuid: uuid::Uuid,
    pub package_name: String,
    pub value: String,
    pub mpn: String,
    pub manufacturer: String,
    pub pin_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartChangeCompatibilityReport {
    pub component_uuid: uuid::Uuid,
    pub current_part_uuid: Option<uuid::Uuid>,
    pub current_package_uuid: uuid::Uuid,
    pub current_package_name: String,
    pub current_value: String,
    pub status: PartChangeCompatibilityStatus,
    pub candidates: Vec<PartChangeCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentReplacementPlan {
    pub component_uuid: uuid::Uuid,
    pub current_reference: String,
    pub current_value: String,
    pub current_part_uuid: Option<uuid::Uuid>,
    pub current_package_uuid: uuid::Uuid,
    pub current_package_name: String,
    pub package_change: PackageChangeCompatibilityReport,
    pub part_change: PartChangeCompatibilityReport,
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

fn replace_component_pads_for_assign_part(
    board: &mut Board,
    previous_component: &crate::board::PlacedPackage,
    next_component: &crate::board::PlacedPackage,
    target_part: &crate::pool::Part,
    target_package: &crate::pool::Package,
    pool: &Pool,
) -> Result<(), EngineError> {
    let net_by_name: BTreeMap<String, Option<uuid::Uuid>> = component_pads(board, previous_component.uuid)
        .into_iter()
        .map(|pad| (pad.name, pad.net))
        .collect();
    let mut net_by_pin = BTreeMap::new();

    if previous_component.part != uuid::Uuid::nil()
        && let Some(current_part) = pool.parts.get(&previous_component.part)
        && current_part.package == previous_component.package
    {
                let current_package = pool.packages.get(&previous_component.package).ok_or(
                    EngineError::DanglingReference {
                        source_type: "component",
                        source_uuid: previous_component.uuid,
                        target_type: "package",
                        target_uuid: previous_component.package,
                    },
                )?;
                let current_pad_name_by_uuid: BTreeMap<uuid::Uuid, &str> = current_package
                    .pads
                    .values()
                    .map(|pad| (pad.uuid, pad.name.as_str()))
                    .collect();

                for (pad_uuid, entry) in &current_part.pad_map {
                    if let Some(pad_name) = current_pad_name_by_uuid.get(pad_uuid)
                        && let Some(net) = net_by_name.get(*pad_name).copied().flatten()
                    {
                        net_by_pin.insert(entry.pin, net);
                    }
                }
    }

    let mut regenerated = Vec::new();
    for package_pad in target_package.pads.values() {
        let net = target_part
            .pad_map
            .get(&package_pad.uuid)
            .and_then(|entry| net_by_pin.get(&entry.pin).copied())
            .or_else(|| net_by_name.get(&package_pad.name).copied().flatten());
        regenerated.push(crate::board::PlacedPad {
            uuid: deterministic_component_pad_uuid(next_component.uuid, &package_pad.name),
            package: next_component.uuid,
            name: package_pad.name.clone(),
            net,
            position: transform_board_local_point(
                next_component.position,
                next_component.rotation,
                package_pad.position,
            ),
            layer: package_pad.layer,
        });
    }
    regenerated.sort_by_key(|pad| pad.uuid);
    restore_component_pads(board, next_component.uuid, &regenerated);
    Ok(())
}

fn resolve_compatible_part_for_package_change(
    current_part_uuid: uuid::Uuid,
    target_package_uuid: uuid::Uuid,
    pool: &Pool,
) -> Option<uuid::Uuid> {
    let current_part = pool.parts.get(&current_part_uuid)?;
    let current_signature = part_pin_signature(current_part, pool)?;
    let mut candidates = pool
        .parts
        .values()
        .filter(|part| {
            part.package == target_package_uuid
                && part_pin_signature(part, pool).as_ref() == Some(&current_signature)
        })
        .map(|part| part.uuid);
    let first = candidates.next()?;
    if candidates.next().is_some() {
        None
    } else {
        Some(first)
    }
}

fn part_pin_signature(part: &crate::pool::Part, pool: &Pool) -> Option<BTreeSet<String>> {
    let entity = pool.entities.get(&part.entity)?;
    let mut pins = BTreeSet::new();
    for entry in part.pad_map.values() {
        let gate = entity.gates.get(&entry.gate)?;
        let unit = pool.units.get(&gate.unit)?;
        let pin = unit.pins.get(&entry.pin)?;
        pins.insert(pin.name.clone());
    }
    Some(pins)
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
