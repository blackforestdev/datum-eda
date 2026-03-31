use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::drc::DrcViolation;
use crate::erc::ErcFinding;
use crate::rules::ast::RuleType;
use crate::schematic::ConnectivityDiagnosticInfo;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ComponentReplacementScope {
    pub reference_prefix: Option<String>,
    pub value_equals: Option<String>,
    pub current_package_uuid: Option<uuid::Uuid>,
    pub current_part_uuid: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopedComponentReplacementPolicyInput {
    pub scope: ComponentReplacementScope,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
pub enum CheckReport {
    Board {
        summary: CheckSummary,
        diagnostics: Vec<ConnectivityDiagnosticInfo>,
    },
    Combined {
        summary: CheckSummary,
        diagnostics: Vec<ConnectivityDiagnosticInfo>,
        erc: Vec<ErcFinding>,
        drc: Vec<DrcViolation>,
    },
    Schematic {
        summary: CheckSummary,
        diagnostics: Vec<ConnectivityDiagnosticInfo>,
        erc: Vec<ErcFinding>,
        drc: Vec<DrcViolation>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopedComponentReplacementPlanItem {
    pub component_uuid: uuid::Uuid,
    pub current_reference: String,
    pub current_value: String,
    pub current_part_uuid: Option<uuid::Uuid>,
    pub current_package_uuid: uuid::Uuid,
    pub target_part_uuid: uuid::Uuid,
    pub target_package_uuid: uuid::Uuid,
    pub target_value: String,
    pub target_package_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopedComponentReplacementPlan {
    pub scope: ComponentReplacementScope,
    pub policy: ComponentReplacementPolicy,
    pub replacements: Vec<ScopedComponentReplacementPlanItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopedComponentReplacementOverride {
    pub component_uuid: uuid::Uuid,
    pub target_package_uuid: uuid::Uuid,
    pub target_part_uuid: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ScopedComponentReplacementPlanEdit {
    pub exclude_component_uuids: Vec<uuid::Uuid>,
    pub overrides: Vec<ScopedComponentReplacementOverride>,
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
