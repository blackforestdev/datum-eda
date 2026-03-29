// eda CLI — batch operations for PCB design analysis.
// Links directly to eda-engine (no daemon required for CLI).
// See specs/PROGRAM_SPEC.md for command requirements per milestone.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use eda_engine::api::{
    AssignPartInput, CheckReport, CheckStatus, ComponentReplacementPlan,
    ComponentReplacementPolicy, ComponentReplacementScope, Engine, MoveComponentInput,
    PackageChangeCompatibilityReport, PartChangeCompatibilityReport,
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
mod main_board_component;
mod main_drill;
mod main_forward_annotation;
mod main_forward_annotation_audit_views;
mod main_forward_annotation_reports;
mod main_forward_annotation_views;
mod main_gerber_views;
mod main_gerber_inspect;
mod main_gerber_mechanical;
mod main_gerber_set;
mod main_gerber_silkscreen;
mod main_inspect;
mod main_inventory;
mod main_import_report;
mod main_manufacturing;
mod main_modify;
mod main_project;
mod main_summary;

use cli_args::*;
use command_plan::*;
use command_project::*;
use command_query::*;
pub(crate) use main_board_component::*;
pub(crate) use main_drill::*;
pub(crate) use main_forward_annotation::*;
pub(crate) use main_forward_annotation_audit_views::*;
pub(crate) use main_forward_annotation_reports::*;
pub(crate) use main_forward_annotation_views::*;
pub(crate) use main_gerber_views::*;
pub(crate) use main_gerber_inspect::*;
pub(crate) use main_gerber_mechanical::*;
pub(crate) use main_gerber_set::*;
pub(crate) use main_gerber_silkscreen::*;
pub(crate) use main_inspect::*;
pub(crate) use main_inventory::*;
pub(crate) use main_import_report::*;
pub(crate) use main_manufacturing::*;
pub(crate) use main_modify::*;
pub(crate) use main_project::*;
pub(crate) use main_summary::*;

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
struct NativeProjectRulesView {
    domain: &'static str,
    count: usize,
    rules: Vec<serde_json::Value>,
}

pub(crate) use main_forward_annotation_reports::*;

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
    height_nm: i64,
    stroke_width_nm: i64,
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
    shape: String,
    diameter_nm: i64,
    width_nm: i64,
    height_nm: i64,
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
    layer: i32,
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

fn render_native_project_gerber_soldermask_export_text(
    report: &NativeProjectGerberSoldermaskExportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("source_copper_layer: {}", report.source_copper_layer),
        format!("pad_count: {}", report.pad_count),
    ]
    .join("\n")
}

fn render_native_project_gerber_paste_export_text(
    report: &NativeProjectGerberPasteExportView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("source_copper_layer: {}", report.source_copper_layer),
        format!("pad_count: {}", report.pad_count),
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

fn render_native_project_gerber_soldermask_validation_text(
    report: &NativeProjectGerberSoldermaskValidationView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("source_copper_layer: {}", report.source_copper_layer),
        format!("matches_expected: {}", report.matches_expected),
        format!("expected_bytes: {}", report.expected_bytes),
        format!("actual_bytes: {}", report.actual_bytes),
        format!("pad_count: {}", report.pad_count),
    ]
    .join("\n")
}

fn render_native_project_gerber_paste_validation_text(
    report: &NativeProjectGerberPasteValidationView,
) -> String {
    [
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("source_copper_layer: {}", report.source_copper_layer),
        format!("matches_expected: {}", report.matches_expected),
        format!("expected_bytes: {}", report.expected_bytes),
        format!("actual_bytes: {}", report.actual_bytes),
        format!("pad_count: {}", report.pad_count),
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

fn render_native_project_gerber_soldermask_comparison_text(
    report: &NativeProjectGerberSoldermaskComparisonView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("source_copper_layer: {}", report.source_copper_layer),
        format!("expected_pad_count: {}", report.expected_pad_count),
        format!("actual_pad_count: {}", report.actual_pad_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("extra_count: {}", report.extra_count),
    ];
    append_gerber_geometry_entries(&mut lines, "matched", &report.matched);
    append_gerber_geometry_entries(&mut lines, "missing", &report.missing);
    append_gerber_geometry_entries(&mut lines, "extra", &report.extra);
    lines.join("\n")
}

fn render_native_project_gerber_paste_comparison_text(
    report: &NativeProjectGerberPasteComparisonView,
) -> String {
    let mut lines = vec![
        format!("action: {}", report.action),
        format!("project_root: {}", report.project_root),
        format!("board_path: {}", report.board_path),
        format!("gerber_path: {}", report.gerber_path),
        format!("layer: {}", report.layer),
        format!("source_copper_layer: {}", report.source_copper_layer),
        format!("expected_pad_count: {}", report.expected_pad_count),
        format!("actual_pad_count: {}", report.actual_pad_count),
        format!("matched_count: {}", report.matched_count),
        format!("missing_count: {}", report.missing_count),
        format!("extra_count: {}", report.extra_count),
    ];
    append_gerber_geometry_entries(&mut lines, "matched", &report.matched);
    append_gerber_geometry_entries(&mut lines, "missing", &report.missing);
    append_gerber_geometry_entries(&mut lines, "extra", &report.extra);
    lines.join("\n")
}

pub(crate) fn append_gerber_geometry_entries(
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
        format!("height_nm: {}", report.height_nm),
        format!("stroke_width_nm: {}", report.stroke_width_nm),
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
        format!("shape: {}", report.shape),
        format!("diameter_nm: {}", report.diameter_nm),
        format!("width_nm: {}", report.width_nm),
        format!("height_nm: {}", report.height_nm),
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
        format!("layer: {}", report.layer),
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

#[cfg(test)]
mod main_tests;
