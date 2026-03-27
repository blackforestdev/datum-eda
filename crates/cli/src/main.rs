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
    PlannedComponentReplacementInput, PolicyDrivenComponentReplacementInput,
    ReplaceComponentInput, RotateComponentInput, ScopedComponentReplacementPlan,
    ScopedComponentReplacementPlanEdit, ScopedComponentReplacementOverride,
    ScopedComponentReplacementPolicyInput, SetDesignRuleInput, SetNetClassInput, SetPackageInput,
    SetPackageWithPartInput, SetReferenceInput, SetValueInput,
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
use serde::Serialize;
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
    nets: usize,
    tracks: usize,
    vias: usize,
    zones: usize,
    keepouts: usize,
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
        format!("board_nets: {}", report.board.nets),
        format!("board_tracks: {}", report.board.tracks),
        format!("board_vias: {}", report.board.vias),
        format!("board_zones: {}", report.board.zones),
        format!("board_keepouts: {}", report.board.keepouts),
        format!("board_texts: {}", report.board.texts),
        format!("rule_count: {}", report.rules.count),
    ]
    .join("\n")
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

fn render_native_project_label_mutation_text(report: &NativeProjectLabelMutationReportView) -> String {
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

fn render_native_project_wire_mutation_text(report: &NativeProjectWireMutationReportView) -> String {
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

fn render_native_project_port_mutation_text(report: &NativeProjectPortMutationReportView) -> String {
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

fn render_native_project_symbol_mutation_text(report: &NativeProjectSymbolMutationReportView) -> String {
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

fn render_native_project_text_mutation_text(report: &NativeProjectTextMutationReportView) -> String {
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
        serde_json::to_string_pretty(report).expect("CLI text formatting serialization must succeed")
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
