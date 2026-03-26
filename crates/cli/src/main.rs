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
mod command_query;

use cli_args::*;
use command_plan::*;
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
