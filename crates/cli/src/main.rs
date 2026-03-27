// eda CLI — batch operations for PCB design analysis.
// Links directly to eda-engine (no daemon required for CLI).
// See specs/PROGRAM_SPEC.md for command requirements per milestone.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use eda_engine::api::{
    AssignPartInput, CheckReport, CheckStatus, ComponentReplacementPlan,
    ComponentReplacementPolicy, ComponentReplacementScope, Engine, MoveComponentInput,
    OperationResult, PackageChangeCompatibilityReport, PartChangeCompatibilityReport,
    PlannedComponentReplacementInput, PolicyDrivenComponentReplacementInput, ReplaceComponentInput,
    RotateComponentInput, ScopedComponentReplacementOverride, ScopedComponentReplacementPlan,
    ScopedComponentReplacementPlanEdit, ScopedComponentReplacementPolicyInput, SetDesignRuleInput,
    SetNetClassInput, SetPackageInput, SetPackageWithPartInput, SetReferenceInput, SetValueInput,
};
use eda_engine::drc::DrcReport;
use eda_engine::erc::ErcFinding;
use eda_engine::error::EngineError;
use eda_engine::import::ImportReport;
use eda_engine::pool::PartSummary;
use eda_engine::rules::ast::{Rule, RuleParams, RuleScope, RuleType};
use eda_engine::schematic::{
    ConnectivityDiagnosticInfo, HierarchyInfo, LabelInfo, PortInfo, SchematicNetInfo,
};
use eda_engine::{board::Airwire, board::BoardNetInfo, board::ComponentInfo};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod cli_args;
mod command_exec;
mod command_modify;
mod command_plan;
mod command_project;
mod command_query;

use cli_args::*;
use command_plan::*;
use command_project::*;
use command_query::*;

fn main() {
    match run() {
        Ok(code) => {
            if code != 0 {
                std::process::exit(code);
            }
        }
        Err(err) => {
            eprintln!("{err:#}");
            std::process::exit(2);
        }
    }
}

fn run() -> Result<i32> {
    let cli = Cli::parse();
    let (output, exit_code) = execute_with_exit_code(cli)?;
    if !output.is_empty() {
        println!("{output}");
    }
    Ok(exit_code)
}

fn execute_with_exit_code(cli: Cli) -> Result<(String, i32)> {
    command_exec::execute_with_exit_code(cli)
}

#[derive(Debug, Clone, Serialize)]
struct ModifyReportView {
    actions: Vec<String>,
    last_result: Option<OperationResult>,
    saved_path: Option<String>,
    applied_scoped_replacement_manifests: Vec<AppliedScopedReplacementManifestView>,
}

#[derive(Debug, Clone, Serialize)]
struct AppliedScopedReplacementManifestView {
    path: String,
    source_version: u32,
    version: u32,
    migration_applied: bool,
    replacements: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectCreateReportView {
    project_root: String,
    project_name: String,
    project_uuid: String,
    schematic_uuid: String,
    board_uuid: String,
    files_written: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectInspectReportView {
    project_root: String,
    project_name: String,
    schema_version: u32,
    project_uuid: String,
    schematic_uuid: String,
    board_uuid: String,
    pools: usize,
    schematic_path: String,
    board_path: String,
    rules_path: String,
    sheet_count: usize,
    sheet_definition_count: usize,
    sheet_instance_count: usize,
    variant_count: usize,
    board_package_count: usize,
    board_pad_count: usize,
    board_net_count: usize,
    board_track_count: usize,
    board_via_count: usize,
    board_zone_count: usize,
    rule_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectSummaryView {
    domain: &'static str,
    project_name: String,
    schema_version: u32,
    pools: usize,
    schematic: NativeProjectSchematicSummaryView,
    board: NativeProjectBoardSummaryView,
    rules: NativeProjectRulesSummaryView,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectBomExportView {
    action: String,
    project_root: String,
    bom_path: String,
    rows: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectBomDriftView {
    reference: String,
    fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectBomComparisonView {
    action: String,
    project_root: String,
    board_path: String,
    bom_path: String,
    expected_count: usize,
    actual_count: usize,
    matched_count: usize,
    missing_count: usize,
    extra_count: usize,
    drift_count: usize,
    matched: Vec<String>,
    missing: Vec<String>,
    extra: Vec<String>,
    drift: Vec<NativeProjectBomDriftView>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectPnpExportView {
    action: String,
    project_root: String,
    pnp_path: String,
    rows: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectPnpDriftView {
    reference: String,
    fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectPnpComparisonView {
    action: String,
    project_root: String,
    board_path: String,
    pnp_path: String,
    expected_count: usize,
    actual_count: usize,
    matched_count: usize,
    missing_count: usize,
    extra_count: usize,
    drift_count: usize,
    matched: Vec<String>,
    missing: Vec<String>,
    extra: Vec<String>,
    drift: Vec<NativeProjectPnpDriftView>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectDrillExportView {
    action: String,
    project_root: String,
    drill_path: String,
    rows: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectExcellonDrillExportView {
    action: String,
    project_root: String,
    board_path: String,
    drill_path: String,
    via_count: usize,
    tool_count: usize,
    tools: Vec<NativeProjectExcellonDrillToolView>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectExcellonDrillValidationView {
    action: String,
    project_root: String,
    board_path: String,
    drill_path: String,
    matches_expected: bool,
    expected_bytes: usize,
    actual_bytes: usize,
    via_count: usize,
    tool_count: usize,
    tools: Vec<NativeProjectExcellonDrillToolView>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectExcellonDrillToolView {
    tool: String,
    diameter_mm: String,
    hits: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectExcellonDrillInspectionView {
    action: String,
    drill_path: String,
    metric: bool,
    tool_count: usize,
    hit_count: usize,
    tools: Vec<NativeProjectExcellonDrillToolView>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectExcellonDrillHitDriftView {
    diameter_mm: String,
    expected_hits: usize,
    actual_hits: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectExcellonDrillComparisonView {
    action: String,
    project_root: String,
    board_path: String,
    drill_path: String,
    expected_tool_count: usize,
    actual_tool_count: usize,
    expected_hit_count: usize,
    actual_hit_count: usize,
    matched_count: usize,
    missing_count: usize,
    extra_count: usize,
    hit_drift_count: usize,
    matched: Vec<String>,
    missing: Vec<String>,
    extra: Vec<String>,
    hit_drift: Vec<NativeProjectExcellonDrillHitDriftView>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectDrillHoleClassBucketView {
    class: String,
    from_layer: i32,
    to_layer: i32,
    via_count: usize,
    tool_count: usize,
    tools: Vec<NativeProjectExcellonDrillToolView>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectDrillHoleClassReportView {
    action: String,
    project_root: String,
    board_path: String,
    copper_layer_count: usize,
    via_count: usize,
    class_count: usize,
    classes: Vec<NativeProjectDrillHoleClassBucketView>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectGerberOutlineExportView {
    action: String,
    project_root: String,
    board_path: String,
    gerber_path: String,
    outline_vertex_count: usize,
    outline_closed: bool,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectGerberCopperExportView {
    action: String,
    project_root: String,
    board_path: String,
    gerber_path: String,
    layer: i32,
    pad_count: usize,
    track_count: usize,
    zone_count: usize,
    via_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectGerberOutlineValidationView {
    action: String,
    project_root: String,
    board_path: String,
    gerber_path: String,
    matches_expected: bool,
    expected_bytes: usize,
    actual_bytes: usize,
    outline_vertex_count: usize,
    outline_closed: bool,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectGerberCopperValidationView {
    action: String,
    project_root: String,
    board_path: String,
    gerber_path: String,
    layer: i32,
    matches_expected: bool,
    expected_bytes: usize,
    actual_bytes: usize,
    pad_count: usize,
    track_count: usize,
    zone_count: usize,
    via_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectGerberGeometryEntryView {
    kind: String,
    geometry: String,
    count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectGerberOutlineComparisonView {
    action: String,
    project_root: String,
    board_path: String,
    gerber_path: String,
    expected_outline_count: usize,
    actual_geometry_count: usize,
    matched_count: usize,
    missing_count: usize,
    extra_count: usize,
    matched: Vec<NativeProjectGerberGeometryEntryView>,
    missing: Vec<NativeProjectGerberGeometryEntryView>,
    extra: Vec<NativeProjectGerberGeometryEntryView>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectGerberCopperComparisonView {
    action: String,
    project_root: String,
    board_path: String,
    gerber_path: String,
    layer: i32,
    expected_pad_count: usize,
    actual_pad_count: usize,
    expected_track_count: usize,
    actual_track_count: usize,
    expected_zone_count: usize,
    actual_zone_count: usize,
    expected_via_count: usize,
    actual_via_count: usize,
    matched_count: usize,
    missing_count: usize,
    extra_count: usize,
    matched: Vec<NativeProjectGerberGeometryEntryView>,
    missing: Vec<NativeProjectGerberGeometryEntryView>,
    extra: Vec<NativeProjectGerberGeometryEntryView>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectGerberPlanArtifactView {
    kind: String,
    layer_id: Option<i32>,
    layer_name: Option<String>,
    filename: String,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectGerberPlanView {
    action: String,
    project_root: String,
    board_path: String,
    prefix: String,
    outline_vertex_count: usize,
    outline_closed: bool,
    copper_layers: usize,
    soldermask_layers: usize,
    silkscreen_layers: usize,
    paste_layers: usize,
    mechanical_layers: usize,
    artifacts: Vec<NativeProjectGerberPlanArtifactView>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectGerberPlanComparisonView {
    action: String,
    project_root: String,
    output_dir: String,
    prefix: String,
    expected_count: usize,
    present_count: usize,
    missing_count: usize,
    extra_count: usize,
    matched: Vec<String>,
    missing: Vec<String>,
    extra: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectSchematicSummaryView {
    sheets: usize,
    sheet_definitions: usize,
    sheet_instances: usize,
    variants: usize,
    symbols: usize,
    wires: usize,
    junctions: usize,
    labels: usize,
    ports: usize,
    buses: usize,
    bus_entries: usize,
    noconnects: usize,
    texts: usize,
    drawings: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectBoardSummaryView {
    name: String,
    layers: usize,
    components: usize,
    pads: usize,
    nets: usize,
    net_classes: usize,
    tracks: usize,
    vias: usize,
    zones: usize,
    keepouts: usize,
    dimensions: usize,
    texts: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectRulesSummaryView {
    count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectRulesView {
    domain: &'static str,
    count: usize,
    rules: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationAuditView {
    domain: &'static str,
    schematic_symbol_count: usize,
    board_component_count: usize,
    matched_count: usize,
    unresolved_symbol_count: usize,
    missing_on_board: Vec<NativeProjectForwardAnnotationMissingView>,
    orphaned_on_board: Vec<NativeProjectForwardAnnotationOrphanView>,
    value_mismatches: Vec<NativeProjectForwardAnnotationValueMismatchView>,
    part_mismatches: Vec<NativeProjectForwardAnnotationPartMismatchView>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationMissingView {
    symbol_uuid: String,
    sheet_uuid: String,
    reference: String,
    value: String,
    part_uuid: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationOrphanView {
    component_uuid: String,
    reference: String,
    value: String,
    part_uuid: String,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationValueMismatchView {
    reference: String,
    symbol_uuid: String,
    component_uuid: String,
    schematic_value: String,
    board_value: String,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationPartMismatchView {
    reference: String,
    symbol_uuid: String,
    component_uuid: String,
    schematic_part_uuid: String,
    board_part_uuid: String,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationProposalView {
    domain: &'static str,
    total_actions: usize,
    add_component_actions: usize,
    remove_component_actions: usize,
    update_component_actions: usize,
    add_component_group: Vec<NativeProjectForwardAnnotationProposalActionView>,
    remove_component_group: Vec<NativeProjectForwardAnnotationProposalActionView>,
    update_component_group: Vec<NativeProjectForwardAnnotationProposalActionView>,
    actions: Vec<NativeProjectForwardAnnotationProposalActionView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeProjectForwardAnnotationProposalActionView {
    action_id: String,
    action: String,
    reference: String,
    symbol_uuid: Option<String>,
    component_uuid: Option<String>,
    reason: String,
    schematic_value: Option<String>,
    board_value: Option<String>,
    schematic_part_uuid: Option<String>,
    board_part_uuid: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationReviewView {
    domain: &'static str,
    total_reviews: usize,
    deferred_actions: usize,
    rejected_actions: usize,
    actions: Vec<NativeProjectForwardAnnotationReviewActionView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NativeProjectForwardAnnotationReviewActionView {
    action_id: String,
    decision: String,
    proposal_action: String,
    reference: String,
    reason: String,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationReviewReportView {
    action: String,
    action_id: String,
    decision: String,
    proposal_action: String,
    reference: String,
    reason: String,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationApplyReportView {
    action: String,
    action_id: String,
    proposal_action: String,
    reason: String,
    component_report: NativeProjectBoardComponentMutationReportView,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationBatchApplySkippedActionView {
    action_id: String,
    proposal_action: String,
    reference: String,
    reason: String,
    skip_reason: String,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationBatchApplyReportView {
    action: String,
    domain: &'static str,
    proposal_actions: usize,
    applied_actions: usize,
    skipped_deferred_actions: usize,
    skipped_rejected_actions: usize,
    skipped_requires_input_actions: usize,
    applied: Vec<NativeProjectForwardAnnotationApplyReportView>,
    skipped: Vec<NativeProjectForwardAnnotationBatchApplySkippedActionView>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationExportReportView {
    action: String,
    artifact_path: String,
    kind: String,
    version: u32,
    project_uuid: String,
    actions: usize,
    reviews: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationArtifactInspectionView {
    artifact_path: String,
    kind: String,
    source_version: u32,
    version: u32,
    migration_applied: bool,
    project_uuid: String,
    project_name: String,
    actions: usize,
    reviews: usize,
    add_component_actions: usize,
    remove_component_actions: usize,
    update_component_actions: usize,
    deferred_reviews: usize,
    rejected_reviews: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationArtifactComparisonActionView {
    action_id: String,
    proposal_action: String,
    reference: String,
    reason: String,
    status: String,
    review_decision: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationArtifactComparisonView {
    artifact_path: String,
    project_root: String,
    kind: String,
    artifact_version: u32,
    current_project_uuid: String,
    artifact_project_uuid: String,
    artifact_actions: usize,
    applicable_actions: usize,
    drifted_actions: usize,
    stale_actions: usize,
    actions: Vec<NativeProjectForwardAnnotationArtifactComparisonActionView>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationArtifactFilterView {
    action: String,
    input_artifact_path: String,
    output_artifact_path: String,
    project_root: String,
    kind: String,
    version: u32,
    artifact_actions: usize,
    applicable_actions: usize,
    filtered_reviews: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationArtifactApplyPlanActionView {
    action_id: String,
    proposal_action: String,
    reference: String,
    reason: String,
    applicability: String,
    execution: String,
    review_decision: Option<String>,
    required_inputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationArtifactApplyPlanView {
    action: String,
    artifact_path: String,
    project_root: String,
    kind: String,
    artifact_version: u32,
    artifact_actions: usize,
    self_sufficient_actions: usize,
    requires_input_actions: usize,
    not_applicable_actions: usize,
    actions: Vec<NativeProjectForwardAnnotationArtifactApplyPlanActionView>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationArtifactApplyView {
    action: String,
    artifact_path: String,
    project_root: String,
    artifact_actions: usize,
    applied_actions: usize,
    skipped_deferred_actions: usize,
    skipped_rejected_actions: usize,
    applied: Vec<NativeProjectForwardAnnotationApplyReportView>,
    skipped: Vec<NativeProjectForwardAnnotationBatchApplySkippedActionView>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationArtifactReviewImportView {
    action: String,
    artifact_path: String,
    project_root: String,
    imported_reviews: usize,
    skipped_missing_live_actions: usize,
    total_artifact_reviews: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectForwardAnnotationArtifactReviewReplaceView {
    action: String,
    artifact_path: String,
    project_root: String,
    replaced_reviews: usize,
    removed_existing_reviews: usize,
    skipped_missing_live_actions: usize,
    total_artifact_reviews: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectLabelMutationReportView {
    action: String,
    project_root: String,
    sheet_uuid: String,
    sheet_path: String,
    label_uuid: String,
    name: String,
    kind: String,
    x_nm: i64,
    y_nm: i64,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectWireMutationReportView {
    action: String,
    project_root: String,
    sheet_uuid: String,
    sheet_path: String,
    wire_uuid: String,
    from_x_nm: i64,
    from_y_nm: i64,
    to_x_nm: i64,
    to_y_nm: i64,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectJunctionMutationReportView {
    action: String,
    project_root: String,
    sheet_uuid: String,
    sheet_path: String,
    junction_uuid: String,
    x_nm: i64,
    y_nm: i64,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectPortMutationReportView {
    action: String,
    project_root: String,
    sheet_uuid: String,
    sheet_path: String,
    port_uuid: String,
    name: String,
    direction: String,
    x_nm: i64,
    y_nm: i64,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectBusMutationReportView {
    action: String,
    project_root: String,
    sheet_uuid: String,
    sheet_path: String,
    bus_uuid: String,
    name: String,
    members: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectBusEntryMutationReportView {
    action: String,
    project_root: String,
    sheet_uuid: String,
    sheet_path: String,
    bus_entry_uuid: String,
    bus_uuid: String,
    wire_uuid: Option<String>,
    x_nm: i64,
    y_nm: i64,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectNoConnectMutationReportView {
    action: String,
    project_root: String,
    sheet_uuid: String,
    sheet_path: String,
    noconnect_uuid: String,
    symbol_uuid: String,
    pin_uuid: String,
    x_nm: i64,
    y_nm: i64,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectSymbolMutationReportView {
    action: String,
    project_root: String,
    sheet_uuid: String,
    sheet_path: String,
    symbol_uuid: String,
    reference: String,
    value: String,
    lib_id: Option<String>,
    x_nm: i64,
    y_nm: i64,
    rotation_deg: i32,
    mirrored: bool,
    gate_uuid: Option<String>,
    unit_selection: Option<String>,
    display_mode: String,
    hidden_power_behavior: String,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectSymbolFieldMutationReportView {
    action: String,
    project_root: String,
    sheet_uuid: String,
    sheet_path: String,
    symbol_uuid: String,
    field_uuid: String,
    key: String,
    value: String,
    visible: bool,
    x_nm: Option<i64>,
    y_nm: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectSymbolSemanticsView {
    symbol_uuid: String,
    gate_uuid: Option<String>,
    unit_selection: Option<String>,
    hidden_power_behavior: String,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectSymbolPinInfoView {
    symbol_uuid: String,
    pin_uuid: String,
    number: String,
    name: String,
    electrical_type: String,
    x_nm: i64,
    y_nm: i64,
    visible_override: Option<bool>,
    override_x_nm: Option<i64>,
    override_y_nm: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectPinOverrideMutationReportView {
    action: String,
    project_root: String,
    sheet_uuid: String,
    sheet_path: String,
    symbol_uuid: String,
    pin_uuid: String,
    visible: Option<bool>,
    x_nm: Option<i64>,
    y_nm: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectTextMutationReportView {
    action: String,
    project_root: String,
    sheet_uuid: String,
    sheet_path: String,
    text_uuid: String,
    text: String,
    x_nm: i64,
    y_nm: i64,
    rotation_deg: i32,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectDrawingMutationReportView {
    action: String,
    project_root: String,
    sheet_uuid: String,
    sheet_path: String,
    drawing_uuid: String,
    kind: String,
    from_x_nm: i64,
    from_y_nm: i64,
    to_x_nm: i64,
    to_y_nm: i64,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectBoardTextMutationReportView {
    action: String,
    project_root: String,
    board_path: String,
    text_uuid: String,
    text: String,
    x_nm: i64,
    y_nm: i64,
    rotation_deg: i32,
    layer: i32,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectBoardKeepoutMutationReportView {
    action: String,
    project_root: String,
    board_path: String,
    keepout_uuid: String,
    kind: String,
    layer_count: usize,
    vertex_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectBoardOutlineMutationReportView {
    action: String,
    project_root: String,
    board_path: String,
    vertex_count: usize,
    closed: bool,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectBoardStackupMutationReportView {
    action: String,
    project_root: String,
    board_path: String,
    layer_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectBoardNetMutationReportView {
    action: String,
    project_root: String,
    board_path: String,
    net_uuid: String,
    name: String,
    class_uuid: String,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectBoardComponentMutationReportView {
    action: String,
    project_root: String,
    board_path: String,
    component_uuid: String,
    part_uuid: String,
    package_uuid: String,
    reference: String,
    value: String,
    x_nm: i64,
    y_nm: i64,
    rotation_deg: i32,
    layer: i32,
    locked: bool,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectBoardTrackMutationReportView {
    action: String,
    project_root: String,
    board_path: String,
    track_uuid: String,
    net_uuid: String,
    from_x_nm: i64,
    from_y_nm: i64,
    to_x_nm: i64,
    to_y_nm: i64,
    width_nm: i64,
    layer: i32,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectBoardViaMutationReportView {
    action: String,
    project_root: String,
    board_path: String,
    via_uuid: String,
    net_uuid: String,
    x_nm: i64,
    y_nm: i64,
    drill_nm: i64,
    diameter_nm: i64,
    from_layer: i32,
    to_layer: i32,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectBoardZoneMutationReportView {
    action: String,
    project_root: String,
    board_path: String,
    zone_uuid: String,
    net_uuid: String,
    layer: i32,
    priority: u32,
    thermal_relief: bool,
    thermal_gap_nm: i64,
    thermal_spoke_width_nm: i64,
    vertex_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectBoardPadMutationReportView {
    action: String,
    project_root: String,
    board_path: String,
    pad_uuid: String,
    package_uuid: String,
    name: String,
    net_uuid: Option<String>,
    x_nm: i64,
    y_nm: i64,
    layer: i32,
    diameter_nm: i64,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectBoardNetClassMutationReportView {
    action: String,
    project_root: String,
    board_path: String,
    net_class_uuid: String,
    name: String,
    clearance_nm: i64,
    track_width_nm: i64,
    via_drill_nm: i64,
    via_diameter_nm: i64,
    diffpair_width_nm: i64,
    diffpair_gap_nm: i64,
}

#[derive(Debug, Clone, Serialize)]
struct NativeProjectBoardDimensionMutationReportView {
    action: String,
    project_root: String,
    board_path: String,
    dimension_uuid: String,
    from_x_nm: i64,
    from_y_nm: i64,
    to_x_nm: i64,
    to_y_nm: i64,
    text: Option<String>,
}

fn render_native_project_create_report_text(report: &NativeProjectCreateReportView) -> String {
    let mut lines = vec![
        format!("project_root: {}", report.project_root),
        format!("project_name: {}", report.project_name),
        format!("project_uuid: {}", report.project_uuid),
        format!("schematic_uuid: {}", report.schematic_uuid),
        format!("board_uuid: {}", report.board_uuid),
    ];
    if !report.files_written.is_empty() {
        lines.push("files_written:".to_string());
        for path in &report.files_written {
            lines.push(format!("  {path}"));
        }
    }
    lines.join("\n")
}

fn render_native_project_inspect_report_text(report: &NativeProjectInspectReportView) -> String {
    [
        format!("project_root: {}", report.project_root),
        format!("project_name: {}", report.project_name),
        format!("schema_version: {}", report.schema_version),
        format!("project_uuid: {}", report.project_uuid),
        format!("schematic_uuid: {}", report.schematic_uuid),
        format!("board_uuid: {}", report.board_uuid),
        format!("pools: {}", report.pools),
        format!("schematic_path: {}", report.schematic_path),
        format!("board_path: {}", report.board_path),
        format!("rules_path: {}", report.rules_path),
        format!("sheet_count: {}", report.sheet_count),
        format!("sheet_definition_count: {}", report.sheet_definition_count),
        format!("sheet_instance_count: {}", report.sheet_instance_count),
        format!("variant_count: {}", report.variant_count),
        format!("board_package_count: {}", report.board_package_count),
        format!("board_pad_count: {}", report.board_pad_count),
        format!("board_net_count: {}", report.board_net_count),
        format!("board_track_count: {}", report.board_track_count),
        format!("board_via_count: {}", report.board_via_count),
        format!("board_zone_count: {}", report.board_zone_count),
        format!("rule_count: {}", report.rule_count),
    ]
    .join("\n")
}

fn render_native_project_summary_text(report: &NativeProjectSummaryView) -> String {
    [
        format!("project_name: {}", report.project_name),
        format!("schema_version: {}", report.schema_version),
        format!("pools: {}", report.pools),
        format!("schematic_sheets: {}", report.schematic.sheets),
        format!(
            "schematic_sheet_definitions: {}",
            report.schematic.sheet_definitions
        ),
        format!(
            "schematic_sheet_instances: {}",
            report.schematic.sheet_instances
        ),
        format!("schematic_variants: {}", report.schematic.variants),
        format!("schematic_symbols: {}", report.schematic.symbols),
        format!("schematic_wires: {}", report.schematic.wires),
        format!("schematic_junctions: {}", report.schematic.junctions),
        format!("schematic_labels: {}", report.schematic.labels),
        format!("schematic_ports: {}", report.schematic.ports),
        format!("schematic_buses: {}", report.schematic.buses),
        format!("schematic_bus_entries: {}", report.schematic.bus_entries),
        format!("schematic_noconnects: {}", report.schematic.noconnects),
        format!("schematic_texts: {}", report.schematic.texts),
        format!("schematic_drawings: {}", report.schematic.drawings),
        format!("board_name: {}", report.board.name),
        format!("board_layers: {}", report.board.layers),
        format!("board_components: {}", report.board.components),
        format!("board_pads: {}", report.board.pads),
        format!("board_nets: {}", report.board.nets),
        format!("board_net_classes: {}", report.board.net_classes),
        format!("board_tracks: {}", report.board.tracks),
        format!("board_vias: {}", report.board.vias),
        format!("board_zones: {}", report.board.zones),
        format!("board_keepouts: {}", report.board.keepouts),
        format!("board_dimensions: {}", report.board.dimensions),
        format!("board_texts: {}", report.board.texts),
        format!("rule_count: {}", report.rules.count),
    ]
    .join("\n")
}

fn render_native_project_bom_export_text(report: &NativeProjectBomExportView) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("bom_path: {}", report.bom_path),
        format!("rows: {}", report.rows),
    ]
    .join("\n")
}

fn render_native_project_bom_comparison_text(report: &NativeProjectBomComparisonView) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("bom_path: {}", report.bom_path),
        format!("expected_count: {}", report.expected_count),
        format!("actual_count: {}", report.actual_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("extra_count: {}", report.extra_count),
        format!("drift_count: {}", report.drift_count),
    ];
    if !report.matched.is_empty() {
        lines.push("matched:".to_string());
        for entry in &report.matched {
            lines.push(format!("- {}", entry));
        }
    }
    if !report.missing.is_empty() {
        lines.push("missing:".to_string());
        for entry in &report.missing {
            lines.push(format!("- {}", entry));
        }
    }
    if !report.extra.is_empty() {
        lines.push("extra:".to_string());
        for entry in &report.extra {
            lines.push(format!("- {}", entry));
        }
    }
    if !report.drift.is_empty() {
        lines.push("drift:".to_string());
        for entry in &report.drift {
            lines.push(format!(
                "- {} [{}]",
                entry.reference,
                entry.fields.join(", ")
            ));
        }
    }
    lines.join("\n")
}

fn render_native_project_pnp_export_text(report: &NativeProjectPnpExportView) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("pnp_path: {}", report.pnp_path),
        format!("rows: {}", report.rows),
    ]
    .join("\n")
}

fn render_native_project_pnp_comparison_text(report: &NativeProjectPnpComparisonView) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("pnp_path: {}", report.pnp_path),
        format!("expected_count: {}", report.expected_count),
        format!("actual_count: {}", report.actual_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("extra_count: {}", report.extra_count),
        format!("drift_count: {}", report.drift_count),
    ];
    if !report.matched.is_empty() {
        lines.push("matched:".to_string());
        for entry in &report.matched {
            lines.push(format!("- {}", entry));
        }
    }
    if !report.missing.is_empty() {
        lines.push("missing:".to_string());
        for entry in &report.missing {
            lines.push(format!("- {}", entry));
        }
    }
    if !report.extra.is_empty() {
        lines.push("extra:".to_string());
        for entry in &report.extra {
            lines.push(format!("- {}", entry));
        }
    }
    if !report.drift.is_empty() {
        lines.push("drift:".to_string());
        for entry in &report.drift {
            lines.push(format!(
                "- {} fields={}",
                entry.reference,
                entry.fields.join(",")
            ));
        }
    }
    lines.join("\n")
}

fn render_native_project_drill_export_text(report: &NativeProjectDrillExportView) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("drill_path: {}", report.drill_path),
        format!("rows: {}", report.rows),
    ]
    .join("\n")
}

fn render_native_project_excellon_drill_export_text(
    report: &NativeProjectExcellonDrillExportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("drill_path: {}", report.drill_path),
        format!("via_count: {}", report.via_count),
        format!("tool_count: {}", report.tool_count),
    ];
    if !report.tools.is_empty() {
        lines.push("tools:".to_string());
        for tool in &report.tools {
            lines.push(format!(
                "- {} diameter_mm={} hits={}",
                tool.tool, tool.diameter_mm, tool.hits
            ));
        }
    }
    lines.join("\n")
}

fn render_native_project_excellon_drill_validation_text(
    report: &NativeProjectExcellonDrillValidationView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("drill_path: {}", report.drill_path),
        format!("matches_expected: {}", report.matches_expected),
        format!("expected_bytes: {}", report.expected_bytes),
        format!("actual_bytes: {}", report.actual_bytes),
        format!("via_count: {}", report.via_count),
        format!("tool_count: {}", report.tool_count),
    ];
    if !report.tools.is_empty() {
        lines.push("tools:".to_string());
        for tool in &report.tools {
            lines.push(format!(
                "- {} diameter_mm={} hits={}",
                tool.tool, tool.diameter_mm, tool.hits
            ));
        }
    }
    lines.join("\n")
}

fn render_native_project_excellon_drill_inspection_text(
    report: &NativeProjectExcellonDrillInspectionView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("drill_path: {}", report.drill_path),
        format!("metric: {}", report.metric),
        format!("tool_count: {}", report.tool_count),
        format!("hit_count: {}", report.hit_count),
    ];
    if !report.tools.is_empty() {
        lines.push("tools:".to_string());
        for tool in &report.tools {
            lines.push(format!(
                "- {} diameter_mm={} hits={}",
                tool.tool, tool.diameter_mm, tool.hits
            ));
        }
    }
    lines.join("\n")
}

fn render_native_project_excellon_drill_comparison_text(
    report: &NativeProjectExcellonDrillComparisonView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("drill_path: {}", report.drill_path),
        format!("expected_tool_count: {}", report.expected_tool_count),
        format!("actual_tool_count: {}", report.actual_tool_count),
        format!("expected_hit_count: {}", report.expected_hit_count),
        format!("actual_hit_count: {}", report.actual_hit_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("extra_count: {}", report.extra_count),
        format!("hit_drift_count: {}", report.hit_drift_count),
    ];
    if !report.matched.is_empty() {
        lines.push("matched:".to_string());
        for entry in &report.matched {
            lines.push(format!("- {}", entry));
        }
    }
    if !report.missing.is_empty() {
        lines.push("missing:".to_string());
        for entry in &report.missing {
            lines.push(format!("- {}", entry));
        }
    }
    if !report.extra.is_empty() {
        lines.push("extra:".to_string());
        for entry in &report.extra {
            lines.push(format!("- {}", entry));
        }
    }
    if !report.hit_drift.is_empty() {
        lines.push("hit_drift:".to_string());
        for entry in &report.hit_drift {
            lines.push(format!(
                "- diameter_mm={} expected_hits={} actual_hits={}",
                entry.diameter_mm, entry.expected_hits, entry.actual_hits
            ));
        }
    }
    lines.join("\n")
}

fn render_native_project_drill_hole_class_report_text(
    report: &NativeProjectDrillHoleClassReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("copper_layer_count: {}", report.copper_layer_count),
        format!("via_count: {}", report.via_count),
        format!("class_count: {}", report.class_count),
    ];
    if !report.classes.is_empty() {
        lines.push("classes:".to_string());
        for class in &report.classes {
            lines.push(format!(
                "- class={} span=L{}-L{} via_count={} tool_count={}",
                class.class, class.from_layer, class.to_layer, class.via_count, class.tool_count
            ));
            for tool in &class.tools {
                lines.push(format!(
                    "  tool={} diameter_mm={} hits={}",
                    tool.tool, tool.diameter_mm, tool.hits
                ));
            }
        }
    }
    lines.join("\n")
}

fn render_native_project_gerber_outline_export_text(
    report: &NativeProjectGerberOutlineExportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("outline_vertex_count: {}", report.outline_vertex_count),
        format!("outline_closed: {}", report.outline_closed),
    ]
    .join("\n")
}

fn render_native_project_gerber_copper_export_text(
    report: &NativeProjectGerberCopperExportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("pad_count: {}", report.pad_count),
        format!("track_count: {}", report.track_count),
        format!("zone_count: {}", report.zone_count),
        format!("via_count: {}", report.via_count),
    ]
    .join("\n")
}

fn render_native_project_gerber_outline_validation_text(
    report: &NativeProjectGerberOutlineValidationView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("matches_expected: {}", report.matches_expected),
        format!("expected_bytes: {}", report.expected_bytes),
        format!("actual_bytes: {}", report.actual_bytes),
        format!("outline_vertex_count: {}", report.outline_vertex_count),
        format!("outline_closed: {}", report.outline_closed),
    ]
    .join("\n")
}

fn render_native_project_gerber_copper_validation_text(
    report: &NativeProjectGerberCopperValidationView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("matches_expected: {}", report.matches_expected),
        format!("expected_bytes: {}", report.expected_bytes),
        format!("actual_bytes: {}", report.actual_bytes),
        format!("pad_count: {}", report.pad_count),
        format!("track_count: {}", report.track_count),
        format!("zone_count: {}", report.zone_count),
        format!("via_count: {}", report.via_count),
    ]
    .join("\n")
}

fn render_native_project_gerber_outline_comparison_text(
    report: &NativeProjectGerberOutlineComparisonView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("expected_outline_count: {}", report.expected_outline_count),
        format!("actual_geometry_count: {}", report.actual_geometry_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("extra_count: {}", report.extra_count),
    ];
    append_gerber_geometry_entries(&mut lines, "matched", &report.matched);
    append_gerber_geometry_entries(&mut lines, "missing", &report.missing);
    append_gerber_geometry_entries(&mut lines, "extra", &report.extra);
    lines.join("\n")
}

fn render_native_project_gerber_copper_comparison_text(
    report: &NativeProjectGerberCopperComparisonView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("expected_pad_count: {}", report.expected_pad_count),
        format!("actual_pad_count: {}", report.actual_pad_count),
        format!("expected_track_count: {}", report.expected_track_count),
        format!("actual_track_count: {}", report.actual_track_count),
        format!("expected_zone_count: {}", report.expected_zone_count),
        format!("actual_zone_count: {}", report.actual_zone_count),
        format!("expected_via_count: {}", report.expected_via_count),
        format!("actual_via_count: {}", report.actual_via_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("extra_count: {}", report.extra_count),
    ];
    append_gerber_geometry_entries(&mut lines, "matched", &report.matched);
    append_gerber_geometry_entries(&mut lines, "missing", &report.missing);
    append_gerber_geometry_entries(&mut lines, "extra", &report.extra);
    lines.join("\n")
}

fn append_gerber_geometry_entries(
    lines: &mut Vec<String>,
    label: &str,
    entries: &[NativeProjectGerberGeometryEntryView],
) {
    if entries.is_empty() {
        return;
    }
    lines.push(format!("{label}:"));
    for entry in entries {
        lines.push(format!(
            "- kind={} count={} geometry={}",
            entry.kind, entry.count, entry.geometry
        ));
    }
}

fn render_native_project_gerber_plan_text(report: &NativeProjectGerberPlanView) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("prefix: {}", report.prefix),
        format!("outline_vertex_count: {}", report.outline_vertex_count),
        format!("outline_closed: {}", report.outline_closed),
        format!("copper_layers: {}", report.copper_layers),
        format!("soldermask_layers: {}", report.soldermask_layers),
        format!("silkscreen_layers: {}", report.silkscreen_layers),
        format!("paste_layers: {}", report.paste_layers),
        format!("mechanical_layers: {}", report.mechanical_layers),
    ];
    if !report.artifacts.is_empty() {
        lines.push("artifacts:".to_string());
        for artifact in &report.artifacts {
            let layer_suffix = match (artifact.layer_id, artifact.layer_name.as_ref()) {
                (Some(layer_id), Some(layer_name)) => format!(" layer={layer_id}:{layer_name}"),
                (Some(layer_id), None) => format!(" layer={layer_id}"),
                _ => String::new(),
            };
            lines.push(format!(
                "  {}:{}{}",
                artifact.kind, artifact.filename, layer_suffix
            ));
        }
    }
    lines.join("\n")
}

fn render_native_project_gerber_plan_comparison_text(
    report: &NativeProjectGerberPlanComparisonView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("output_dir: {}", report.output_dir),
        format!("prefix: {}", report.prefix),
        format!("expected_count: {}", report.expected_count),
        format!("present_count: {}", report.present_count),
        format!("missing_count: {}", report.missing_count),
        format!("extra_count: {}", report.extra_count),
    ];
    if !report.matched.is_empty() {
        lines.push("matched:".to_string());
        for file in &report.matched {
            lines.push(format!("  {file}"));
        }
    }
    if !report.missing.is_empty() {
        lines.push("missing:".to_string());
        for file in &report.missing {
            lines.push(format!("  {file}"));
        }
    }
    if !report.extra.is_empty() {
        lines.push("extra:".to_string());
        for file in &report.extra {
            lines.push(format!("  {file}"));
        }
    }
    lines.join("\n")
}

fn render_native_project_rules_text(report: &NativeProjectRulesView) -> String {
    let mut lines = vec![format!("rule_count: {}", report.count)];
    if !report.rules.is_empty() {
        lines.push("rules:".to_string());
        for rule in &report.rules {
            lines.push(format!(
                "  {}",
                serde_json::to_string(rule)
                    .expect("CLI text formatting rule serialization must succeed")
            ));
        }
    }
    lines.join("\n")
}

fn render_native_forward_annotation_audit_text(
    report: &NativeProjectForwardAnnotationAuditView,
) -> String {
    let mut lines = vec![
        format!("schematic_symbol_count: {}", report.schematic_symbol_count),
        format!("board_component_count: {}", report.board_component_count),
        format!("matched_count: {}", report.matched_count),
        format!(
            "unresolved_symbol_count: {}",
            report.unresolved_symbol_count
        ),
        format!("missing_on_board_count: {}", report.missing_on_board.len()),
        format!(
            "orphaned_on_board_count: {}",
            report.orphaned_on_board.len()
        ),
        format!("value_mismatch_count: {}", report.value_mismatches.len()),
        format!("part_mismatch_count: {}", report.part_mismatches.len()),
    ];
    if !report.missing_on_board.is_empty() {
        lines.push("missing_on_board:".to_string());
        for entry in &report.missing_on_board {
            lines.push(format!(
                "  {} value={} part_uuid={}",
                entry.reference,
                entry.value,
                entry.part_uuid.as_deref().unwrap_or("none")
            ));
        }
    }
    if !report.orphaned_on_board.is_empty() {
        lines.push("orphaned_on_board:".to_string());
        for entry in &report.orphaned_on_board {
            lines.push(format!(
                "  {} value={} part_uuid={}",
                entry.reference, entry.value, entry.part_uuid
            ));
        }
    }
    if !report.value_mismatches.is_empty() {
        lines.push("value_mismatches:".to_string());
        for entry in &report.value_mismatches {
            lines.push(format!(
                "  {} schematic={} board={}",
                entry.reference, entry.schematic_value, entry.board_value
            ));
        }
    }
    if !report.part_mismatches.is_empty() {
        lines.push("part_mismatches:".to_string());
        for entry in &report.part_mismatches {
            lines.push(format!(
                "  {} schematic_part_uuid={} board_part_uuid={}",
                entry.reference, entry.schematic_part_uuid, entry.board_part_uuid
            ));
        }
    }
    lines.join("\n")
}

fn render_native_forward_annotation_proposal_text(
    report: &NativeProjectForwardAnnotationProposalView,
) -> String {
    let mut lines = vec![
        format!("total_actions: {}", report.total_actions),
        format!("add_component_actions: {}", report.add_component_actions),
        format!(
            "remove_component_actions: {}",
            report.remove_component_actions
        ),
        format!(
            "update_component_actions: {}",
            report.update_component_actions
        ),
    ];
    if !report.actions.is_empty() {
        lines.push("actions:".to_string());
        for action in &report.actions {
            lines.push(format!(
                "  {} {} id={} reason={}",
                action.action, action.reference, action.action_id, action.reason
            ));
        }
    }
    lines.join("\n")
}

fn render_native_forward_annotation_review_text(
    report: &NativeProjectForwardAnnotationReviewView,
) -> String {
    let mut lines = vec![
        format!("domain: {}", report.domain),
        format!("total_reviews: {}", report.total_reviews),
        format!("deferred_actions: {}", report.deferred_actions),
        format!("rejected_actions: {}", report.rejected_actions),
    ];
    for action in &report.actions {
        lines.push(String::new());
        lines.push(format!("action_id: {}", action.action_id));
        lines.push(format!("decision: {}", action.decision));
        lines.push(format!("proposal_action: {}", action.proposal_action));
        lines.push(format!("reference: {}", action.reference));
        lines.push(format!("reason: {}", action.reason));
    }
    lines.join("\n")
}

fn render_native_forward_annotation_review_report_text(
    report: &NativeProjectForwardAnnotationReviewReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("action_id: {}", report.action_id),
        format!("decision: {}", report.decision),
        format!("proposal_action: {}", report.proposal_action),
        format!("reference: {}", report.reference),
        format!("reason: {}", report.reason),
    ]
    .join("\n")
}

fn render_native_forward_annotation_apply_text(
    report: &NativeProjectForwardAnnotationApplyReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("action_id: {}", report.action_id),
        format!("proposal_action: {}", report.proposal_action),
        format!("reason: {}", report.reason),
        format!("component_uuid: {}", report.component_report.component_uuid),
        format!("part_uuid: {}", report.component_report.part_uuid),
        format!("package_uuid: {}", report.component_report.package_uuid),
        format!("reference: {}", report.component_report.reference),
        format!("value: {}", report.component_report.value),
        format!("x_nm: {}", report.component_report.x_nm),
        format!("y_nm: {}", report.component_report.y_nm),
        format!("rotation_deg: {}", report.component_report.rotation_deg),
        format!("layer: {}", report.component_report.layer),
        format!("locked: {}", report.component_report.locked),
    ]
    .join("\n")
}

fn render_native_forward_annotation_batch_apply_text(
    report: &NativeProjectForwardAnnotationBatchApplyReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("domain: {}", report.domain),
        format!("proposal_actions: {}", report.proposal_actions),
        format!("applied_actions: {}", report.applied_actions),
        format!(
            "skipped_deferred_actions: {}",
            report.skipped_deferred_actions
        ),
        format!(
            "skipped_rejected_actions: {}",
            report.skipped_rejected_actions
        ),
        format!(
            "skipped_requires_input_actions: {}",
            report.skipped_requires_input_actions
        ),
    ];
    for applied in &report.applied {
        lines.push(String::new());
        lines.push(format!("applied_action_id: {}", applied.action_id));
        lines.push(format!("proposal_action: {}", applied.proposal_action));
        lines.push(format!("reference: {}", applied.component_report.reference));
        lines.push(format!("reason: {}", applied.reason));
    }
    for skipped in &report.skipped {
        lines.push(String::new());
        lines.push(format!("skipped_action_id: {}", skipped.action_id));
        lines.push(format!("proposal_action: {}", skipped.proposal_action));
        lines.push(format!("reference: {}", skipped.reference));
        lines.push(format!("reason: {}", skipped.reason));
        lines.push(format!("skip_reason: {}", skipped.skip_reason));
    }
    lines.join("\n")
}

fn render_native_forward_annotation_export_text(
    report: &NativeProjectForwardAnnotationExportReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("artifact_path: {}", report.artifact_path),
        format!("kind: {}", report.kind),
        format!("version: {}", report.version),
        format!("project_uuid: {}", report.project_uuid),
        format!("actions: {}", report.actions),
        format!("reviews: {}", report.reviews),
    ]
    .join("\n")
}

fn render_native_forward_annotation_artifact_inspection_text(
    report: &NativeProjectForwardAnnotationArtifactInspectionView,
) -> String {
    [
        format!("artifact_path: {}", report.artifact_path),
        format!("kind: {}", report.kind),
        format!("source_version: {}", report.source_version),
        format!("version: {}", report.version),
        format!("migration_applied: {}", report.migration_applied),
        format!("project_uuid: {}", report.project_uuid),
        format!("project_name: {}", report.project_name),
        format!("actions: {}", report.actions),
        format!("reviews: {}", report.reviews),
        format!("add_component_actions: {}", report.add_component_actions),
        format!(
            "remove_component_actions: {}",
            report.remove_component_actions
        ),
        format!(
            "update_component_actions: {}",
            report.update_component_actions
        ),
        format!("deferred_reviews: {}", report.deferred_reviews),
        format!("rejected_reviews: {}", report.rejected_reviews),
    ]
    .join("\n")
}

fn render_native_forward_annotation_artifact_comparison_text(
    report: &NativeProjectForwardAnnotationArtifactComparisonView,
) -> String {
    let mut lines = vec![
        format!("artifact_path: {}", report.artifact_path),
        format!("project_root: {}", report.project_root),
        format!("kind: {}", report.kind),
        format!("artifact_version: {}", report.artifact_version),
        format!("current_project_uuid: {}", report.current_project_uuid),
        format!("artifact_project_uuid: {}", report.artifact_project_uuid),
        format!("artifact_actions: {}", report.artifact_actions),
        format!("applicable_actions: {}", report.applicable_actions),
        format!("drifted_actions: {}", report.drifted_actions),
        format!("stale_actions: {}", report.stale_actions),
    ];
    for action in &report.actions {
        lines.push(String::new());
        lines.push(format!("action_id: {}", action.action_id));
        lines.push(format!("proposal_action: {}", action.proposal_action));
        lines.push(format!("reference: {}", action.reference));
        lines.push(format!("reason: {}", action.reason));
        lines.push(format!("status: {}", action.status));
        if let Some(review_decision) = &action.review_decision {
            lines.push(format!("review_decision: {}", review_decision));
        }
    }
    lines.join("\n")
}

fn render_native_forward_annotation_artifact_filter_text(
    report: &NativeProjectForwardAnnotationArtifactFilterView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("input_artifact_path: {}", report.input_artifact_path),
        format!("output_artifact_path: {}", report.output_artifact_path),
        format!("project_root: {}", report.project_root),
        format!("kind: {}", report.kind),
        format!("version: {}", report.version),
        format!("artifact_actions: {}", report.artifact_actions),
        format!("applicable_actions: {}", report.applicable_actions),
        format!("filtered_reviews: {}", report.filtered_reviews),
    ]
    .join("\n")
}

fn render_native_forward_annotation_artifact_apply_plan_text(
    report: &NativeProjectForwardAnnotationArtifactApplyPlanView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("artifact_path: {}", report.artifact_path),
        format!("project_root: {}", report.project_root),
        format!("kind: {}", report.kind),
        format!("artifact_version: {}", report.artifact_version),
        format!("artifact_actions: {}", report.artifact_actions),
        format!(
            "self_sufficient_actions: {}",
            report.self_sufficient_actions
        ),
        format!("requires_input_actions: {}", report.requires_input_actions),
        format!("not_applicable_actions: {}", report.not_applicable_actions),
    ];
    for action in &report.actions {
        lines.push(String::new());
        lines.push(format!("action_id: {}", action.action_id));
        lines.push(format!("proposal_action: {}", action.proposal_action));
        lines.push(format!("reference: {}", action.reference));
        lines.push(format!("reason: {}", action.reason));
        lines.push(format!("applicability: {}", action.applicability));
        lines.push(format!("execution: {}", action.execution));
        if let Some(review_decision) = &action.review_decision {
            lines.push(format!("review_decision: {}", review_decision));
        }
        for required_input in &action.required_inputs {
            lines.push(format!("required_input: {}", required_input));
        }
    }
    lines.join("\n")
}

fn render_native_forward_annotation_artifact_apply_text(
    report: &NativeProjectForwardAnnotationArtifactApplyView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("artifact_path: {}", report.artifact_path),
        format!("project_root: {}", report.project_root),
        format!("artifact_actions: {}", report.artifact_actions),
        format!("applied_actions: {}", report.applied_actions),
        format!(
            "skipped_deferred_actions: {}",
            report.skipped_deferred_actions
        ),
        format!(
            "skipped_rejected_actions: {}",
            report.skipped_rejected_actions
        ),
    ];
    for applied in &report.applied {
        lines.push(String::new());
        lines.push(format!("applied_action_id: {}", applied.action_id));
        lines.push(format!("proposal_action: {}", applied.proposal_action));
        lines.push(format!("reason: {}", applied.reason));
        lines.push(format!(
            "component_reference: {}",
            applied.component_report.reference
        ));
    }
    for skipped in &report.skipped {
        lines.push(String::new());
        lines.push(format!("skipped_action_id: {}", skipped.action_id));
        lines.push(format!("proposal_action: {}", skipped.proposal_action));
        lines.push(format!("reference: {}", skipped.reference));
        lines.push(format!("reason: {}", skipped.reason));
        lines.push(format!("skip_reason: {}", skipped.skip_reason));
    }
    lines.join("\n")
}

fn render_native_forward_annotation_artifact_review_import_text(
    report: &NativeProjectForwardAnnotationArtifactReviewImportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("artifact_path: {}", report.artifact_path),
        format!("project_root: {}", report.project_root),
        format!("total_artifact_reviews: {}", report.total_artifact_reviews),
        format!("imported_reviews: {}", report.imported_reviews),
        format!(
            "skipped_missing_live_actions: {}",
            report.skipped_missing_live_actions
        ),
    ]
    .join("\n")
}

fn render_native_forward_annotation_artifact_review_replace_text(
    report: &NativeProjectForwardAnnotationArtifactReviewReplaceView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("artifact_path: {}", report.artifact_path),
        format!("project_root: {}", report.project_root),
        format!("total_artifact_reviews: {}", report.total_artifact_reviews),
        format!("replaced_reviews: {}", report.replaced_reviews),
        format!(
            "removed_existing_reviews: {}",
            report.removed_existing_reviews
        ),
        format!(
            "skipped_missing_live_actions: {}",
            report.skipped_missing_live_actions
        ),
    ]
    .join("\n")
}

fn render_native_project_label_mutation_text(
    report: &NativeProjectLabelMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("label_uuid: {}", report.label_uuid),
        format!("name: {}", report.name),
        format!("kind: {}", report.kind),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
    ]
    .join("\n")
}

fn render_native_project_wire_mutation_text(
    report: &NativeProjectWireMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("wire_uuid: {}", report.wire_uuid),
        format!("from_x_nm: {}", report.from_x_nm),
        format!("from_y_nm: {}", report.from_y_nm),
        format!("to_x_nm: {}", report.to_x_nm),
        format!("to_y_nm: {}", report.to_y_nm),
    ]
    .join("\n")
}

fn render_native_project_junction_mutation_text(
    report: &NativeProjectJunctionMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("junction_uuid: {}", report.junction_uuid),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
    ]
    .join("\n")
}

fn render_native_project_port_mutation_text(
    report: &NativeProjectPortMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("port_uuid: {}", report.port_uuid),
        format!("name: {}", report.name),
        format!("direction: {}", report.direction),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
    ]
    .join("\n")
}

fn render_native_project_bus_mutation_text(report: &NativeProjectBusMutationReportView) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("bus_uuid: {}", report.bus_uuid),
        format!("name: {}", report.name),
    ];
    if !report.members.is_empty() {
        lines.push("members:".to_string());
        for member in &report.members {
            lines.push(format!("  {member}"));
        }
    }
    lines.join("\n")
}

fn render_native_project_bus_entry_mutation_text(
    report: &NativeProjectBusEntryMutationReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("bus_entry_uuid: {}", report.bus_entry_uuid),
        format!("bus_uuid: {}", report.bus_uuid),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
    ];
    if let Some(wire_uuid) = &report.wire_uuid {
        lines.push(format!("wire_uuid: {}", wire_uuid));
    }
    lines.join("\n")
}

fn render_native_project_noconnect_mutation_text(
    report: &NativeProjectNoConnectMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("noconnect_uuid: {}", report.noconnect_uuid),
        format!("symbol_uuid: {}", report.symbol_uuid),
        format!("pin_uuid: {}", report.pin_uuid),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
    ]
    .join("\n")
}

fn render_native_project_symbol_mutation_text(
    report: &NativeProjectSymbolMutationReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("symbol_uuid: {}", report.symbol_uuid),
        format!("reference: {}", report.reference),
        format!("value: {}", report.value),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
        format!("rotation_deg: {}", report.rotation_deg),
        format!("mirrored: {}", report.mirrored),
    ];
    if let Some(lib_id) = &report.lib_id {
        lines.push(format!("lib_id: {}", lib_id));
    }
    if let Some(gate_uuid) = &report.gate_uuid {
        lines.push(format!("gate_uuid: {}", gate_uuid));
    }
    if let Some(unit_selection) = &report.unit_selection {
        lines.push(format!("unit_selection: {}", unit_selection));
    }
    lines.push(format!("display_mode: {}", report.display_mode));
    lines.push(format!(
        "hidden_power_behavior: {}",
        report.hidden_power_behavior
    ));
    lines.join("\n")
}

fn render_native_project_pin_override_mutation_text(
    report: &NativeProjectPinOverrideMutationReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("symbol_uuid: {}", report.symbol_uuid),
        format!("pin_uuid: {}", report.pin_uuid),
    ];
    if let Some(visible) = report.visible {
        lines.push(format!("visible: {}", visible));
    }
    if let Some(x_nm) = report.x_nm {
        lines.push(format!("x_nm: {}", x_nm));
    }
    if let Some(y_nm) = report.y_nm {
        lines.push(format!("y_nm: {}", y_nm));
    }
    lines.join("\n")
}

fn render_native_project_symbol_field_mutation_text(
    report: &NativeProjectSymbolFieldMutationReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("symbol_uuid: {}", report.symbol_uuid),
        format!("field_uuid: {}", report.field_uuid),
        format!("key: {}", report.key),
        format!("value: {}", report.value),
        format!("visible: {}", report.visible),
    ];
    if let Some(x_nm) = report.x_nm {
        lines.push(format!("x_nm: {}", x_nm));
    }
    if let Some(y_nm) = report.y_nm {
        lines.push(format!("y_nm: {}", y_nm));
    }
    lines.join("\n")
}

fn render_native_project_text_mutation_text(
    report: &NativeProjectTextMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("text_uuid: {}", report.text_uuid),
        format!("text: {}", report.text),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
        format!("rotation_deg: {}", report.rotation_deg),
    ]
    .join("\n")
}

fn render_native_project_drawing_mutation_text(
    report: &NativeProjectDrawingMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("sheet_uuid: {}", report.sheet_uuid),
        format!("sheet_path: {}", report.sheet_path),
        format!("drawing_uuid: {}", report.drawing_uuid),
        format!("kind: {}", report.kind),
        format!("from_x_nm: {}", report.from_x_nm),
        format!("from_y_nm: {}", report.from_y_nm),
        format!("to_x_nm: {}", report.to_x_nm),
        format!("to_y_nm: {}", report.to_y_nm),
    ]
    .join("\n")
}

fn render_native_project_board_text_mutation_text(
    report: &NativeProjectBoardTextMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("text_uuid: {}", report.text_uuid),
        format!("text: {}", report.text),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
        format!("rotation_deg: {}", report.rotation_deg),
        format!("layer: {}", report.layer),
    ]
    .join("\n")
}

fn render_native_project_board_keepout_mutation_text(
    report: &NativeProjectBoardKeepoutMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("keepout_uuid: {}", report.keepout_uuid),
        format!("kind: {}", report.kind),
        format!("layer_count: {}", report.layer_count),
        format!("vertex_count: {}", report.vertex_count),
    ]
    .join("\n")
}

fn render_native_project_board_outline_mutation_text(
    report: &NativeProjectBoardOutlineMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("vertex_count: {}", report.vertex_count),
        format!("closed: {}", report.closed),
    ]
    .join("\n")
}

fn render_native_project_board_stackup_mutation_text(
    report: &NativeProjectBoardStackupMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("layer_count: {}", report.layer_count),
    ]
    .join("\n")
}

fn render_native_project_board_net_mutation_text(
    report: &NativeProjectBoardNetMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("net_uuid: {}", report.net_uuid),
        format!("name: {}", report.name),
        format!("class_uuid: {}", report.class_uuid),
    ]
    .join("\n")
}

fn render_native_project_board_component_mutation_text(
    report: &NativeProjectBoardComponentMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("component_uuid: {}", report.component_uuid),
        format!("part_uuid: {}", report.part_uuid),
        format!("package_uuid: {}", report.package_uuid),
        format!("reference: {}", report.reference),
        format!("value: {}", report.value),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
        format!("rotation_deg: {}", report.rotation_deg),
        format!("layer: {}", report.layer),
        format!("locked: {}", report.locked),
    ]
    .join("\n")
}

fn render_native_project_board_track_mutation_text(
    report: &NativeProjectBoardTrackMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("track_uuid: {}", report.track_uuid),
        format!("net_uuid: {}", report.net_uuid),
        format!("from_x_nm: {}", report.from_x_nm),
        format!("from_y_nm: {}", report.from_y_nm),
        format!("to_x_nm: {}", report.to_x_nm),
        format!("to_y_nm: {}", report.to_y_nm),
        format!("width_nm: {}", report.width_nm),
        format!("layer: {}", report.layer),
    ]
    .join("\n")
}

fn render_native_project_board_via_mutation_text(
    report: &NativeProjectBoardViaMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("via_uuid: {}", report.via_uuid),
        format!("net_uuid: {}", report.net_uuid),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
        format!("drill_nm: {}", report.drill_nm),
        format!("diameter_nm: {}", report.diameter_nm),
        format!("from_layer: {}", report.from_layer),
        format!("to_layer: {}", report.to_layer),
    ]
    .join("\n")
}

fn render_native_project_board_zone_mutation_text(
    report: &NativeProjectBoardZoneMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("zone_uuid: {}", report.zone_uuid),
        format!("net_uuid: {}", report.net_uuid),
        format!("layer: {}", report.layer),
        format!("priority: {}", report.priority),
        format!("thermal_relief: {}", report.thermal_relief),
        format!("thermal_gap_nm: {}", report.thermal_gap_nm),
        format!("thermal_spoke_width_nm: {}", report.thermal_spoke_width_nm),
        format!("vertex_count: {}", report.vertex_count),
    ]
    .join("\n")
}

fn render_native_project_board_pad_mutation_text(
    report: &NativeProjectBoardPadMutationReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("pad_uuid: {}", report.pad_uuid),
        format!("package_uuid: {}", report.package_uuid),
        format!("name: {}", report.name),
        format!("x_nm: {}", report.x_nm),
        format!("y_nm: {}", report.y_nm),
        format!("layer: {}", report.layer),
        format!("diameter_nm: {}", report.diameter_nm),
    ];
    if let Some(net_uuid) = &report.net_uuid {
        lines.push(format!("net_uuid: {}", net_uuid));
    }
    lines.join("\n")
}

fn render_native_project_board_net_class_mutation_text(
    report: &NativeProjectBoardNetClassMutationReportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("net_class_uuid: {}", report.net_class_uuid),
        format!("name: {}", report.name),
        format!("clearance_nm: {}", report.clearance_nm),
        format!("track_width_nm: {}", report.track_width_nm),
        format!("via_drill_nm: {}", report.via_drill_nm),
        format!("via_diameter_nm: {}", report.via_diameter_nm),
        format!("diffpair_width_nm: {}", report.diffpair_width_nm),
        format!("diffpair_gap_nm: {}", report.diffpair_gap_nm),
    ]
    .join("\n")
}

fn render_native_project_board_dimension_mutation_text(
    report: &NativeProjectBoardDimensionMutationReportView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("dimension_uuid: {}", report.dimension_uuid),
        format!("from_x_nm: {}", report.from_x_nm),
        format!("from_y_nm: {}", report.from_y_nm),
        format!("to_x_nm: {}", report.to_x_nm),
        format!("to_y_nm: {}", report.to_y_nm),
    ];
    if let Some(text) = &report.text {
        lines.push(format!("text: {}", text));
    }
    lines.join("\n")
}

fn render_modify_report_text(report: &ModifyReportView) -> String {
    let mut lines = Vec::new();
    if !report.actions.is_empty() {
        lines.push("actions:".to_string());
        for action in &report.actions {
            lines.push(format!("  {action}"));
        }
    }
    if let Some(saved_path) = &report.saved_path {
        lines.push(format!("saved_path: {saved_path}"));
    }
    if !report.applied_scoped_replacement_manifests.is_empty() {
        lines.push("applied_scoped_replacement_manifests:".to_string());
        for manifest in &report.applied_scoped_replacement_manifests {
            lines.push(format!(
                "  {} source_version={} version={} migration_applied={} replacements={}",
                manifest.path,
                manifest.source_version,
                manifest.version,
                manifest.migration_applied,
                manifest.replacements
            ));
        }
    }
    if lines.is_empty() {
        serde_json::to_string_pretty(report)
            .expect("CLI text formatting serialization must succeed")
    } else {
        lines.join("\n")
    }
}

fn import_path(path: &Path) -> Result<ImportReport> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("lbr") => {
            let mut engine = Engine::new().context("failed to initialize engine")?;
            engine
                .import_eagle_library(path)
                .with_context(|| format!("failed to import Eagle library {}", path.display()))
        }
        _ => bail!(
            "import is only implemented for Eagle .lbr in M0; unsupported path {}",
            path.display()
        ),
    }
}

fn search_pool(query: &str, libraries: &[PathBuf]) -> Result<Vec<PartSummary>> {
    let mut engine = Engine::new().context("failed to initialize engine")?;
    for path in libraries {
        if path.extension().and_then(|ext| ext.to_str()) != Some("lbr") {
            bail!(
                "pool search currently only accepts Eagle .lbr inputs in M0: {}",
                path.display()
            );
        }
        engine
            .import_eagle_library(path)
            .with_context(|| format!("failed to import Eagle library {}", path.display()))?;
    }

    engine
        .search_pool(query)
        .with_context(|| format!("failed to search pool for {query}"))
}

fn modify_board_with_plan(
    path: &Path,
    delete_track: &[Uuid],
    delete_via: &[Uuid],
    delete_component: &[Uuid],
    libraries: &[PathBuf],
    move_component: &[MoveComponentInput],
    rotate_component: &[RotateComponentInput],
    set_value: &[SetValueInput],
    assign_part: &[AssignPartInput],
    set_package: &[SetPackageInput],
    set_package_with_part: &[SetPackageWithPartInput],
    replace_component: &[ReplaceComponentInput],
    set_net_class: &[SetNetClassInput],
    set_reference: &[SetReferenceInput],
    set_clearance_min_nm: Option<i64>,
    undo: usize,
    redo: usize,
    save: Option<&Path>,
    save_original: bool,
    apply_replacement_plan: &[PlannedComponentReplacementInput],
    apply_replacement_policy: &[PolicyDrivenComponentReplacementInput],
    apply_scoped_replacement_policy: &[ScopedComponentReplacementPolicyInput],
    apply_scoped_replacement_plan: &[ScopedComponentReplacementPlan],
) -> Result<ModifyReportView> {
    command_modify::modify_board(
        path,
        delete_track,
        delete_via,
        delete_component,
        libraries,
        move_component,
        rotate_component,
        set_value,
        assign_part,
        set_package,
        set_package_with_part,
        replace_component,
        set_net_class,
        set_reference,
        set_clearance_min_nm,
        undo,
        redo,
        save,
        save_original,
        apply_replacement_plan,
        apply_replacement_policy,
        apply_scoped_replacement_policy,
        apply_scoped_replacement_plan,
    )
}

#[derive(Debug, Serialize)]
struct ImportReportView {
    kind: &'static str,
    source: String,
    counts: ImportCountsView,
    warnings: Vec<String>,
    metadata: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
struct ImportCountsView {
    units: usize,
    symbols: usize,
    entities: usize,
    padstacks: usize,
    packages: usize,
    parts: usize,
}

impl From<ImportReport> for ImportReportView {
    fn from(report: ImportReport) -> Self {
        Self {
            kind: report.kind.as_str(),
            source: report.source.display().to_string(),
            counts: ImportCountsView {
                units: report.counts.units,
                symbols: report.counts.symbols,
                entities: report.counts.entities,
                padstacks: report.counts.padstacks,
                packages: report.counts.packages,
                parts: report.counts.parts,
            },
            warnings: report.warnings,
            metadata: report.metadata,
        }
    }
}

#[cfg(test)]
mod main_tests;
