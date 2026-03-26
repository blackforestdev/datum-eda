// eda CLI — batch operations for PCB design analysis.
// Links directly to eda-engine (no daemon required for CLI).
// See specs/PROGRAM_SPEC.md for command requirements per milestone.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use eda_engine::api::{
    AssignPartInput, CheckReport, CheckStatus, ComponentReplacementPlan,
    ComponentReplacementPolicy, ComponentReplacementScope, Engine, MoveComponentInput,
    OperationResult, PackageChangeCompatibilityReport, PartChangeCompatibilityReport,
    PlannedComponentReplacementInput, PolicyDrivenComponentReplacementInput,
    ReplaceComponentInput, RotateComponentInput, ScopedComponentReplacementPlan,
    ScopedComponentReplacementPolicyInput, SetDesignRuleInput, SetNetClassInput,
    SetPackageInput, SetPackageWithPartInput, SetReferenceInput, SetValueInput,
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

mod command_exec;
mod command_modify;

#[derive(Parser)]
#[command(name = "eda", about = "PCB design analysis and automation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format
    #[arg(long, default_value = "text")]
    format: OutputFormat,
}

#[derive(Clone, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum FailOn {
    Info,
    Warning,
    Error,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
enum Commands {
    /// Import a KiCad or Eagle design
    Import {
        /// Path to design file (.kicad_pcb, .brd, .lbr)
        path: PathBuf,
    },
    /// Query design data
    Query {
        /// Path to design file
        path: PathBuf,
        /// What to query
        #[command(subcommand)]
        what: QueryCommands,
    },
    /// Run design rule checks
    Drc {
        /// Path to design file
        path: String,
    },
    /// Run electrical rule checks on a schematic
    Erc {
        /// Path to schematic file (.kicad_sch in current M1 slice)
        path: PathBuf,
    },
    /// Run the current unified check surface for an imported design
    Check {
        /// Path to design file
        path: PathBuf,

        /// Exit nonzero if the check report status meets or exceeds this level
        #[arg(long, value_enum)]
        fail_on: Option<FailOn>,
    },
    /// Search the component pool
    Pool {
        #[command(subcommand)]
        action: PoolCommands,
    },
    /// Apply the current minimal M3 board modification surface
    Modify {
        /// Path to board design file
        path: PathBuf,

        /// Delete one track by UUID
        #[arg(long = "delete-track")]
        delete_track: Vec<Uuid>,

        /// Delete one via by UUID
        #[arg(long = "delete-via")]
        delete_via: Vec<Uuid>,

        /// Delete one component by UUID
        #[arg(long = "delete-component")]
        delete_component: Vec<Uuid>,

        /// Load Eagle libraries into the in-memory pool before applying modify ops
        #[arg(long = "library")]
        libraries: Vec<PathBuf>,

        /// Move one component: <uuid>:<x_mm>:<y_mm>[:<rotation_deg>]
        #[arg(long = "move-component")]
        move_component: Vec<String>,

        /// Rotate one component: <uuid>:<rotation_deg>
        #[arg(long = "rotate-component")]
        rotate_component: Vec<String>,

        /// Set one component value: <uuid>:<value>
        #[arg(long = "set-value")]
        set_value: Vec<String>,

        /// Assign one component part: <uuid>:<part_uuid>
        #[arg(long = "assign-part")]
        assign_part: Vec<String>,

        /// Set one component package: <uuid>:<package_uuid>
        #[arg(long = "set-package")]
        set_package: Vec<String>,

        /// Set one component package with an explicit compatible part: <uuid>:<package_uuid>:<part_uuid>
        #[arg(long = "set-package-with-part")]
        set_package_with_part: Vec<String>,

        /// Replace one component with an explicit compatible part+package: <uuid>:<package_uuid>:<part_uuid>
        #[arg(long = "replace-component")]
        replace_component: Vec<String>,

        /// Apply replacement-plan selection: <uuid>:package:<package_uuid> | <uuid>:part:<part_uuid> | <uuid>:package:<package_uuid>:part:<part_uuid>
        #[arg(long = "apply-replacement-plan")]
        apply_replacement_plan: Vec<String>,

        /// Apply replacement policy: <uuid>:package | <uuid>:part
        #[arg(long = "apply-replacement-policy")]
        apply_replacement_policy: Vec<String>,

        /// Apply scoped replacement policy: package|part[:ref_prefix=<text>][:value=<text>][:package_uuid=<uuid>][:part_uuid=<uuid>]
        #[arg(long = "apply-scoped-replacement-policy")]
        apply_scoped_replacement_policy: Vec<String>,

        /// Set one net class: <net_uuid>:<class_name>:<clearance_nm>:<track_width_nm>:<via_drill_nm>:<via_diameter_nm>[:<diffpair_width_nm>:<diffpair_gap_nm>]
        #[arg(long = "set-net-class")]
        set_net_class: Vec<String>,

        /// Set one component reference: <uuid>:<reference>
        #[arg(long = "set-reference")]
        set_reference: Vec<String>,

        /// Undo the most recent transaction count times
        #[arg(long, default_value_t = 0)]
        undo: usize,

        /// Redo the most recent undone transaction count times
        #[arg(long, default_value_t = 0)]
        redo: usize,

        /// Save modifications to a new path
        #[arg(long)]
        save: Option<PathBuf>,

        /// Set the default all-scope copper clearance rule minimum in nm
        #[arg(long)]
        set_clearance_min_nm: Option<i64>,

        /// Save back to the original imported file path
        #[arg(long, default_value_t = false)]
        save_original: bool,
    },
}

#[derive(Subcommand)]
enum QueryCommands {
    /// Board summary (dimensions, counts)
    Summary,
    /// List all nets
    Nets,
    /// List all components
    Components,
    /// List schematic labels
    Labels,
    /// List schematic ports
    Ports,
    /// Show schematic hierarchy
    Hierarchy,
    /// Show schematic connectivity diagnostics
    Diagnostics,
    /// Show unrouted connections
    Unrouted,
    /// Show design rules
    DesignRules,
    /// Show package-change compatibility candidates for a component UUID
    PackageChangeCandidates {
        /// Component UUID
        uuid: Uuid,
        /// Load Eagle libraries into the in-memory pool before querying candidates
        #[arg(long = "library")]
        libraries: Vec<PathBuf>,
    },
    /// Show part-change compatibility candidates for a component UUID
    PartChangeCandidates {
        /// Component UUID
        uuid: Uuid,
        /// Load Eagle libraries into the in-memory pool before querying candidates
        #[arg(long = "library")]
        libraries: Vec<PathBuf>,
    },
    /// Show a unified replacement-planning report for a component UUID
    ComponentReplacementPlan {
        /// Component UUID
        uuid: Uuid,
        /// Load Eagle libraries into the in-memory pool before querying the plan
        #[arg(long = "library")]
        libraries: Vec<PathBuf>,
    },
    /// Show the resolved replacements a scoped policy would apply
    ScopedReplacementPlan {
        /// Replacement policy to resolve
        #[arg(value_enum)]
        policy: ReplacementPolicyArg,
        /// Restrict matches by current reference prefix
        #[arg(long = "ref-prefix")]
        ref_prefix: Option<String>,
        /// Restrict matches by current value
        #[arg(long = "value")]
        value: Option<String>,
        /// Restrict matches by current package UUID
        #[arg(long = "package-uuid")]
        package_uuid: Option<Uuid>,
        /// Restrict matches by current part UUID
        #[arg(long = "part-uuid")]
        part_uuid: Option<Uuid>,
        /// Load Eagle libraries into the in-memory pool before querying the plan
        #[arg(long = "library")]
        libraries: Vec<PathBuf>,
    },
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum ReplacementPolicyArg {
    Package,
    Part,
}

#[derive(Subcommand)]
enum PoolCommands {
    /// Search for parts
    Search {
        /// Search query
        query: String,

        /// Eagle library files to load into the in-memory pool for this search
        #[arg(long = "library", required = true)]
        libraries: Vec<PathBuf>,
    },
}

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

#[cfg(test)]
fn execute(cli: Cli) -> Result<String> {
    let (output, _) = execute_with_exit_code(cli)?;
    Ok(output)
}

fn execute_with_exit_code(cli: Cli) -> Result<(String, i32)> {
    command_exec::execute_with_exit_code(cli)
}

#[derive(Debug, Clone, Serialize)]
struct ModifyReportView {
    actions: Vec<String>,
    last_result: Option<OperationResult>,
    saved_path: Option<String>,
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

fn run_erc(path: &Path) -> Result<Vec<ErcFinding>> {
    if path.extension().and_then(|ext| ext.to_str()) != Some("kicad_sch") {
        bail!(
            "erc currently only accepts KiCad .kicad_sch inputs in M1: {}",
            path.display()
        );
    }

    let mut engine = Engine::new().context("failed to initialize engine")?;
    engine
        .import(path)
        .with_context(|| format!("failed to import schematic {}", path.display()))?;
    engine
        .run_erc_prechecks()
        .with_context(|| format!("failed to run ERC on {}", path.display()))
}

fn run_drc(path: &Path) -> Result<DrcReport> {
    if path.extension().and_then(|ext| ext.to_str()) != Some("kicad_pcb") {
        bail!(
            "drc currently only accepts KiCad .kicad_pcb inputs in M2 slice: {}",
            path.display()
        );
    }

    let mut engine = Engine::new().context("failed to initialize engine")?;
    engine
        .import(path)
        .with_context(|| format!("failed to import board {}", path.display()))?;
    engine
        .run_drc(&[
            RuleType::Connectivity,
            RuleType::ClearanceCopper,
            RuleType::TrackWidth,
            RuleType::ViaHole,
            RuleType::ViaAnnularRing,
            RuleType::SilkClearance,
        ])
        .with_context(|| format!("failed to run DRC on {}", path.display()))
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
fn modify_board(
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
) -> Result<ModifyReportView> {
    modify_board_with_plan(
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
        &[],
        &[],
        &[],
    )
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
    )
}

fn run_check(path: &Path) -> Result<CheckReport> {
    let mut engine = Engine::new().context("failed to initialize engine")?;
    engine
        .import(path)
        .with_context(|| format!("failed to import design {}", path.display()))?;
    engine
        .get_check_report()
        .with_context(|| format!("failed to build check report for {}", path.display()))
}

fn check_exit_code(report: &CheckReport, fail_on: Option<FailOn>) -> i32 {
    let status = match report {
        CheckReport::Board { summary, .. } => summary.status,
        CheckReport::Schematic { summary, .. } => summary.status,
    };

    let threshold = fail_on.unwrap_or(FailOn::Error);
    if status_rank(status) >= fail_on_rank(threshold) {
        1
    } else {
        0
    }
}

fn status_rank(status: CheckStatus) -> u8 {
    match status {
        CheckStatus::Ok => 0,
        CheckStatus::Info => 1,
        CheckStatus::Warning => 2,
        CheckStatus::Error => 3,
    }
}

fn fail_on_rank(level: FailOn) -> u8 {
    match level {
        FailOn::Info => 1,
        FailOn::Warning => 2,
        FailOn::Error => 3,
    }
}

fn render_check_report_text(report: &CheckReport) -> String {
    match report {
        CheckReport::Board {
            summary,
            diagnostics,
        } => {
            let mut lines = vec![format!(
                "board check: status={} errors={} warnings={} infos={} waived={}",
                render_status(summary.status),
                summary.errors,
                summary.warnings,
                summary.infos,
                summary.waived
            )];
            if !summary.by_code.is_empty() {
                lines.push("counts:".into());
                for entry in &summary.by_code {
                    lines.push(format!("  {} x{}", entry.code, entry.count));
                }
            }
            if !diagnostics.is_empty() {
                lines.push("diagnostics:".into());
                for diagnostic in diagnostics {
                    lines.push(format!(
                        "  [{}] {}",
                        diagnostic.severity, diagnostic.message
                    ));
                }
            }
            lines.join("\n")
        }
        CheckReport::Schematic {
            summary,
            diagnostics,
            erc,
        } => {
            let mut lines = vec![format!(
                "schematic check: status={} errors={} warnings={} infos={} waived={}",
                render_status(summary.status),
                summary.errors,
                summary.warnings,
                summary.infos,
                summary.waived
            )];
            if !summary.by_code.is_empty() {
                lines.push("counts:".into());
                for entry in &summary.by_code {
                    lines.push(format!("  {} x{}", entry.code, entry.count));
                }
            }
            if !diagnostics.is_empty() {
                lines.push("diagnostics:".into());
                for diagnostic in diagnostics {
                    lines.push(format!(
                        "  [{}] {}",
                        diagnostic.severity, diagnostic.message
                    ));
                }
            }
            if !erc.is_empty() {
                lines.push("erc:".into());
                for finding in erc {
                    let waived = if finding.waived { " (waived)" } else { "" };
                    lines.push(format!(
                        "  [{}] {}: {}{}",
                        render_erc_severity(&finding.severity),
                        finding.code,
                        finding.message,
                        waived
                    ));
                }
            }
            lines.join("\n")
        }
    }
}

fn render_status(status: CheckStatus) -> &'static str {
    match status {
        CheckStatus::Ok => "ok",
        CheckStatus::Info => "info",
        CheckStatus::Warning => "warning",
        CheckStatus::Error => "error",
    }
}

fn render_erc_severity(severity: &eda_engine::erc::ErcSeverity) -> &'static str {
    match severity {
        eda_engine::erc::ErcSeverity::Error => "error",
        eda_engine::erc::ErcSeverity::Warning => "warning",
        eda_engine::erc::ErcSeverity::Info => "info",
    }
}

fn render_drc_report_text(report: &DrcReport) -> String {
    let mut lines = vec![format!(
        "drc: passed={} errors={} warnings={}",
        report.passed, report.summary.errors, report.summary.warnings
    )];
    if !report.violations.is_empty() {
        lines.push("violations:".into());
        for violation in &report.violations {
            let location = violation
                .location
                .as_ref()
                .map(|loc| format!(" @({}, {}) L{:?}", loc.x_nm, loc.y_nm, loc.layer))
                .unwrap_or_default();
            lines.push(format!(
                "  [{}] {}: {}{}",
                render_drc_severity(violation.severity),
                violation.code,
                violation.message,
                location
            ));
        }
    }
    lines.join("\n")
}

fn render_drc_severity(severity: eda_engine::drc::DrcSeverity) -> &'static str {
    match severity {
        eda_engine::drc::DrcSeverity::Error => "error",
        eda_engine::drc::DrcSeverity::Warning => "warning",
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
enum SummaryView {
    Board {
        name: String,
        layers: usize,
        components: usize,
        nets: usize,
    },
    Schematic {
        sheets: usize,
        symbols: usize,
        labels: usize,
        ports: usize,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
enum NetListView {
    Board { nets: Vec<BoardNetInfo> },
    Schematic { nets: Vec<SchematicNetInfo> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
enum ComponentListView {
    Board { components: Vec<ComponentInfo> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
enum LabelListView {
    Schematic { labels: Vec<LabelInfo> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
enum PortListView {
    Schematic { ports: Vec<PortInfo> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
enum HierarchyView {
    Schematic { hierarchy: HierarchyInfo },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
enum DiagnosticsView {
    Board {
        diagnostics: Vec<ConnectivityDiagnosticInfo>,
    },
    Schematic {
        diagnostics: Vec<ConnectivityDiagnosticInfo>,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
enum UnroutedView {
    Board { airwires: Vec<Airwire> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "domain", rename_all = "snake_case")]
enum DesignRuleListView {
    Board { rules: Vec<Rule> },
}

fn import_design_for_query(path: &Path) -> Result<Engine> {
    import_design_for_query_with_libraries(path, &[])
}

fn import_design_for_query_with_libraries(path: &Path, libraries: &[PathBuf]) -> Result<Engine> {
    let mut engine = Engine::new().context("failed to initialize engine")?;
    for library in libraries {
        engine
            .import_eagle_library(library)
            .with_context(|| format!("failed to import library {}", library.display()))?;
    }
    engine
        .import(path)
        .with_context(|| format!("failed to import design {}", path.display()))?;
    Ok(engine)
}

fn query_summary(path: &Path) -> Result<SummaryView> {
    let engine = import_design_for_query(path)?;
    match engine.get_board_summary() {
        Ok(summary) => Ok(SummaryView::Board {
            name: summary.name,
            layers: summary.layer_count,
            components: summary.component_count,
            nets: summary.net_count,
        }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => {
            let summary = engine.get_schematic_summary()?;
            Ok(SummaryView::Schematic {
                sheets: summary.sheet_count,
                symbols: summary.symbol_count,
                labels: summary.net_label_count,
                ports: summary.port_count,
            })
        }
        Err(err) => Err(err.into()),
    }
}

fn query_nets(path: &Path) -> Result<NetListView> {
    let engine = import_design_for_query(path)?;
    match engine.get_net_info() {
        Ok(nets) => Ok(NetListView::Board { nets }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => Ok(NetListView::Schematic {
            nets: engine.get_schematic_net_info()?,
        }),
        Err(err) => Err(err.into()),
    }
}

fn query_components(path: &Path) -> Result<ComponentListView> {
    let engine = import_design_for_query(path)?;
    match engine.get_components() {
        Ok(components) => Ok(ComponentListView::Board { components }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "component query is currently only implemented for boards in M1: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

fn require_schematic(engine: &Engine, path: &Path) -> Result<()> {
    match engine.get_schematic_summary() {
        Ok(_) => Ok(()),
        Err(EngineError::NotFound {
            object_type: "schematic",
            ..
        }) => bail!(
            "query is currently only implemented for schematics for this subcommand in M1: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

fn query_labels(path: &Path) -> Result<LabelListView> {
    let engine = import_design_for_query(path)?;
    require_schematic(&engine, path)?;
    Ok(LabelListView::Schematic {
        labels: engine.get_labels(None)?,
    })
}

fn query_ports(path: &Path) -> Result<PortListView> {
    let engine = import_design_for_query(path)?;
    require_schematic(&engine, path)?;
    Ok(PortListView::Schematic {
        ports: engine.get_ports(None)?,
    })
}

fn query_hierarchy(path: &Path) -> Result<HierarchyView> {
    let engine = import_design_for_query(path)?;
    require_schematic(&engine, path)?;
    Ok(HierarchyView::Schematic {
        hierarchy: engine.get_hierarchy()?,
    })
}

fn query_diagnostics(path: &Path) -> Result<DiagnosticsView> {
    let engine = import_design_for_query(path)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(DiagnosticsView::Board {
            diagnostics: engine.get_connectivity_diagnostics()?,
        }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => {
            require_schematic(&engine, path)?;
            Ok(DiagnosticsView::Schematic {
                diagnostics: engine.get_connectivity_diagnostics()?,
            })
        }
        Err(err) => Err(err.into()),
    }
}

fn query_unrouted(path: &Path) -> Result<UnroutedView> {
    let engine = import_design_for_query(path)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(UnroutedView::Board {
            airwires: engine.get_unrouted()?,
        }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "query unrouted is currently only implemented for boards in M1: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

fn query_design_rules(path: &Path) -> Result<DesignRuleListView> {
    let engine = import_design_for_query(path)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(DesignRuleListView::Board {
            rules: engine.get_design_rules()?,
        }),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "query design-rules is currently only implemented for boards in M3: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

fn query_package_change_candidates(
    path: &Path,
    uuid: &Uuid,
    libraries: &[PathBuf],
) -> Result<PackageChangeCompatibilityReport> {
    let engine = import_design_for_query_with_libraries(path, libraries)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(engine.get_package_change_candidates(uuid)?),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "query package-change-candidates is currently only implemented for boards in M3: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

fn query_part_change_candidates(
    path: &Path,
    uuid: &Uuid,
    libraries: &[PathBuf],
) -> Result<PartChangeCompatibilityReport> {
    let engine = import_design_for_query_with_libraries(path, libraries)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(engine.get_part_change_candidates(uuid)?),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "query part-change-candidates is currently only implemented for boards in M3: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

fn query_component_replacement_plan(
    path: &Path,
    uuid: &Uuid,
    libraries: &[PathBuf],
) -> Result<ComponentReplacementPlan> {
    let engine = import_design_for_query_with_libraries(path, libraries)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(engine.get_component_replacement_plan(uuid)?),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "query component-replacement-plan is currently only implemented for boards in M3: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

fn query_scoped_component_replacement_plan(
    path: &Path,
    input: ScopedComponentReplacementPolicyInput,
    libraries: &[PathBuf],
) -> Result<ScopedComponentReplacementPlan> {
    let engine = import_design_for_query_with_libraries(path, libraries)?;
    match engine.get_board_summary() {
        Ok(_) => Ok(engine.get_scoped_component_replacement_plan(input)?),
        Err(EngineError::NotFound {
            object_type: "board",
            ..
        }) => bail!(
            "query scoped-replacement-plan is currently only implemented for boards in M3: {}",
            path.display()
        ),
        Err(err) => Err(err.into()),
    }
}

fn render_output<T: Serialize>(format: &OutputFormat, value: &T) -> String {
    match format {
        OutputFormat::Text => render_text(value),
        OutputFormat::Json => {
            serde_json::to_string_pretty(value).expect("CLI JSON serialization must succeed")
        }
    }
}

fn render_text<T: Serialize>(value: &T) -> String {
    let json = serde_json::to_value(value).expect("CLI text formatting serialization must succeed");
    match json {
        serde_json::Value::Array(items) => items
            .into_iter()
            .map(|item| {
                serde_json::to_string_pretty(&item)
                    .expect("CLI text formatting item serialization must succeed")
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => serde_json::to_string_pretty(value)
            .expect("CLI text formatting serialization must succeed"),
    }
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
mod tests {
    use super::*;
    use eda_engine::import::ImportKind;

    fn eagle_fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../engine/testdata/import/eagle")
            .join(name)
    }

    fn kicad_fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../engine/testdata/import/kicad")
            .join(name)
    }

    #[test]
    fn import_path_supports_eagle_libraries() {
        let report =
            import_path(&eagle_fixture_path("simple-opamp.lbr")).expect("fixture should import");
        assert!(matches!(report.kind, ImportKind::EagleLibrary));
        assert_eq!(report.counts.parts, 2);
        assert_eq!(
            report.metadata.get("library_name").map(String::as_str),
            Some("demo-analog")
        );
    }

    #[test]
    fn search_pool_loads_multiple_libraries() {
        let results = search_pool(
            "SOT23",
            &[
                eagle_fixture_path("simple-opamp.lbr"),
                eagle_fixture_path("bjt-sot23.lbr"),
            ],
        )
        .expect("search should succeed");

        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|part| part.package == "SOT23"));
        assert!(results.iter().any(|part| part.package == "SOT23-5"));
    }

    #[test]
    fn search_pool_rejects_non_lbr_inputs() {
        let err = search_pool("x", &[PathBuf::from("not-a-library.txt")])
            .expect_err("non-lbr input must fail");
        let msg = format!("{err:#}");
        assert!(msg.contains("only accepts Eagle .lbr inputs"), "{msg}");
    }

    #[test]
    fn render_output_json_formats_structured_data() {
        let report = ImportReportView::from(
            import_path(&eagle_fixture_path("simple-opamp.lbr")).expect("fixture should import"),
        );
        let output = render_output(&OutputFormat::Json, &report);
        assert!(output.contains("\"kind\": \"eagle_library\""));
        assert!(output.contains("\"library_name\": \"demo-analog\""));
    }

    #[test]
    fn render_output_text_joins_array_items() {
        let results = search_pool("SOT23", &[eagle_fixture_path("bjt-sot23.lbr")])
            .expect("search should succeed");
        let output = render_output(&OutputFormat::Text, &results);
        assert!(output.contains("\"package\": \"SOT23\""));
    }

    #[test]
    fn clap_parses_import_command_with_global_format_before_subcommand() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "import",
            eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
        ])
        .expect("CLI should parse");

        match cli.command {
            Commands::Import { path } => assert!(path.ends_with("simple-opamp.lbr")),
            _ => panic!("expected import command"),
        }
        assert!(matches!(cli.format, OutputFormat::Json));
    }

    #[test]
    fn execute_import_command_returns_report_output() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "import",
            eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("import command should succeed");
        assert!(output.contains("\"kind\": \"eagle_library\""));
        assert!(output.contains("\"parts\": 2"));
    }

    #[test]
    fn execute_query_package_change_candidates_returns_report_output() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            kicad_fixture_path("partial-route-demo.kicad_pcb")
                .to_str()
                .unwrap(),
            "package-change-candidates",
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "--library",
            eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("candidate query should succeed");
        assert!(output.contains("\"status\": \"no_known_part\""));
    }

    #[test]
    fn execute_query_part_change_candidates_returns_report_output() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-query-part-change-candidates.kicad_pcb",
            Uuid::new_v4()
        ));
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        let lmv321_part_uuid = engine
            .search_pool("LMV321")
            .expect("search should succeed")
            .first()
            .map(|part| part.uuid)
            .expect("LMV321 part should exist");
        modify_board(
            &source,
            &[],
            &[],
            &[],
            &[eagle_fixture_path("simple-opamp.lbr")],
            &[],
            &[],
            &[],
            &[AssignPartInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                part_uuid: lmv321_part_uuid,
            }],
            &[],
            &[],
            &[],
            &[],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify assign_part save should succeed");

        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            target.to_str().unwrap(),
            "part-change-candidates",
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "--library",
            eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("part-change candidate query should succeed");
        assert!(output.contains("\"status\": \"candidates_available\""));
        assert!(output.contains("\"package_name\": \"ALT-3\""));
        assert!(output.contains("\"value\": \"ALTAMP\""));

        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.parts.json",
            target.file_name().unwrap().to_string_lossy()
        )));
    }

    #[test]
    fn execute_query_component_replacement_plan_returns_combined_report_output() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-query-component-replacement-plan.kicad_pcb",
            Uuid::new_v4()
        ));
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        let lmv321_part_uuid = engine
            .search_pool("LMV321")
            .expect("search should succeed")
            .first()
            .map(|part| part.uuid)
            .expect("LMV321 part should exist");
        modify_board(
            &source,
            &[],
            &[],
            &[],
            &[eagle_fixture_path("simple-opamp.lbr")],
            &[],
            &[],
            &[],
            &[AssignPartInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                part_uuid: lmv321_part_uuid,
            }],
            &[],
            &[],
            &[],
            &[],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify assign_part save should succeed");

        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            target.to_str().unwrap(),
            "component-replacement-plan",
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "--library",
            eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("component replacement plan query should succeed");
        assert!(output.contains("\"current_reference\": \"R1\""));
        assert!(output.contains("\"package_change\""));
        assert!(output.contains("\"part_change\""));
        assert!(output.contains("\"status\": \"candidates_available\""));

        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.parts.json",
            target.file_name().unwrap().to_string_lossy()
        )));
    }

    #[test]
    fn execute_query_scoped_replacement_plan_returns_resolved_preview_output() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-query-scoped-replacement-plan.kicad_pcb",
            Uuid::new_v4()
        ));
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        let lmv321_part_uuid = engine
            .search_pool("LMV321")
            .expect("search should succeed")
            .first()
            .map(|part| part.uuid)
            .expect("LMV321 part should exist");
        modify_board(
            &source,
            &[],
            &[],
            &[],
            &[eagle_fixture_path("simple-opamp.lbr")],
            &[],
            &[],
            &[],
            &[
                AssignPartInput {
                    uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                    part_uuid: lmv321_part_uuid,
                },
                AssignPartInput {
                    uuid: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                    part_uuid: lmv321_part_uuid,
                },
            ],
            &[],
            &[],
            &[],
            &[],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify assign_part save should succeed");

        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            target.to_str().unwrap(),
            "scoped-replacement-plan",
            "package",
            "--ref-prefix",
            "R",
            "--value",
            "LMV321",
            "--library",
            eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("scoped replacement preview query should succeed");
        assert!(output.contains("\"policy\": \"best_compatible_package\""));
        assert!(output.contains("\"current_reference\": \"R1\""));
        assert!(output.contains("\"target_package_name\": \"ALT-3\""));
        assert!(output.contains("\"target_value\": \"ALTAMP\""));

        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.parts.json",
            target.file_name().unwrap().to_string_lossy()
        )));
    }

    #[test]
    fn execute_pool_search_command_returns_matches() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "pool",
            "search",
            "SOT23",
            "--library",
            eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
            "--library",
            eagle_fixture_path("bjt-sot23.lbr").to_str().unwrap(),
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("pool search should succeed");
        assert!(output.contains("\"package\": \"SOT23\""));
        assert!(output.contains("\"package\": \"SOT23-5\""));
    }

    #[test]
    fn execute_query_summary_command_returns_board_output() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            kicad_fixture_path("simple-demo.kicad_pcb")
                .to_str()
                .unwrap(),
            "summary",
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("board summary query should succeed");
        assert!(output.contains("\"domain\": \"board\""));
        assert!(output.contains("\"name\": \"simple-demo\""));
        assert!(output.contains("\"components\": 1"));
    }

    #[test]
    fn execute_query_nets_command_returns_schematic_output() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            kicad_fixture_path("simple-demo.kicad_sch")
                .to_str()
                .unwrap(),
            "nets",
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("schematic net query should succeed");
        assert!(output.contains("\"domain\": \"schematic\""));
        assert!(output.contains("\"name\": \"SCL\""));
        assert!(output.contains("\"name\": \"VCC\""));
    }

    #[test]
    fn execute_query_components_command_rejects_schematic_inputs() {
        let cli = Cli::try_parse_from([
            "eda",
            "query",
            kicad_fixture_path("simple-demo.kicad_sch")
                .to_str()
                .unwrap(),
            "components",
        ])
        .expect("CLI should parse");

        let err = execute(cli).expect_err("schematic components query must fail");
        let msg = format!("{err:#}");
        assert!(msg.contains("only implemented for boards in M1"), "{msg}");
    }

    #[test]
    fn execute_query_labels_command_returns_schematic_output() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            kicad_fixture_path("simple-demo.kicad_sch")
                .to_str()
                .unwrap(),
            "labels",
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("schematic labels query should succeed");
        assert!(output.contains("\"domain\": \"schematic\""));
        assert!(output.contains("\"name\": \"SCL\""));
        assert!(output.contains("\"name\": \"VCC\""));
        assert!(output.contains("\"name\": \"SUB_IN\""));
    }

    #[test]
    fn execute_query_ports_command_returns_schematic_output() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            kicad_fixture_path("simple-demo.kicad_sch")
                .to_str()
                .unwrap(),
            "ports",
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("schematic ports query should succeed");
        assert!(output.contains("\"domain\": \"schematic\""));
        assert!(output.contains("\"name\": \"SUB_IN\""));
    }

    #[test]
    fn execute_query_hierarchy_command_returns_schematic_output() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            kicad_fixture_path("simple-demo.kicad_sch")
                .to_str()
                .unwrap(),
            "hierarchy",
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("schematic hierarchy query should succeed");
        assert!(output.contains("\"domain\": \"schematic\""));
        assert!(output.contains("\"name\": \"Sub\""));
    }

    #[test]
    fn execute_query_diagnostics_command_returns_schematic_output() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            kicad_fixture_path("simple-demo.kicad_sch")
                .to_str()
                .unwrap(),
            "diagnostics",
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("schematic diagnostics query should succeed");
        assert!(output.contains("\"domain\": \"schematic\""));
        assert!(output.contains("\"kind\": \"dangling_component_pin\""));
    }

    #[test]
    fn execute_query_diagnostics_command_returns_board_output() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            kicad_fixture_path("simple-demo.kicad_pcb")
                .to_str()
                .unwrap(),
            "diagnostics",
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("board diagnostics query should succeed");
        assert!(output.contains("\"domain\": \"board\""));
        assert!(output.contains("\"kind\": \"net_without_copper\""));
    }

    #[test]
    fn execute_query_diagnostics_command_returns_partial_route_board_output() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            kicad_fixture_path("partial-route-demo.kicad_pcb")
                .to_str()
                .unwrap(),
            "diagnostics",
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("partial-route diagnostics query should succeed");
        assert!(output.contains("\"domain\": \"board\""));
        assert!(output.contains("\"kind\": \"partially_routed_net\""));
    }

    #[test]
    fn execute_query_unrouted_command_returns_board_output() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            kicad_fixture_path("airwire-demo.kicad_pcb")
                .to_str()
                .unwrap(),
            "unrouted",
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("board unrouted query should succeed");
        assert!(output.contains("\"domain\": \"board\""));
        assert!(output.contains("\"net_name\": \"SIG\""));
        assert!(output.contains("\"component\": \"R1\""));
        assert!(output.contains("\"component\": \"R2\""));
    }

    #[test]
    fn execute_query_labels_command_rejects_board_inputs() {
        let cli = Cli::try_parse_from([
            "eda",
            "query",
            kicad_fixture_path("simple-demo.kicad_pcb")
                .to_str()
                .unwrap(),
            "labels",
        ])
        .expect("CLI should parse");

        let err = execute(cli).expect_err("board labels query must fail");
        let msg = format!("{err:#}");
        assert!(msg.contains("only implemented for schematics"), "{msg}");
    }

    #[test]
    fn execute_query_unrouted_command_rejects_schematic_inputs() {
        let cli = Cli::try_parse_from([
            "eda",
            "query",
            kicad_fixture_path("simple-demo.kicad_sch")
                .to_str()
                .unwrap(),
            "unrouted",
        ])
        .expect("CLI should parse");

        let err = execute(cli).expect_err("schematic unrouted query must fail");
        let msg = format!("{err:#}");
        assert!(msg.contains("only implemented for boards in M1"), "{msg}");
    }

    #[test]
    fn execute_query_design_rules_command_returns_board_rules() {
        let source = kicad_fixture_path("simple-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-query-design-rules.kicad_pcb",
            Uuid::new_v4()
        ));
        modify_board(
            &source,
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            Some(125_000),
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify rule save should succeed");

        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            target.to_str().unwrap(),
            "design-rules",
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("design-rules query should succeed");
        let payload: serde_json::Value =
            serde_json::from_str(&output).expect("output should be valid JSON");
        assert_eq!(payload["domain"], "board");
        let rules = payload["rules"]
            .as_array()
            .expect("rules should be an array");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0]["name"], "default clearance");

        let sidecar = target.with_file_name(format!(
            "{}.rules.json",
            target.file_name().unwrap().to_string_lossy()
        ));
        let _ = std::fs::remove_file(target);
        let _ = std::fs::remove_file(sidecar);
    }

    #[test]
    fn execute_query_design_rules_command_rejects_schematic_inputs() {
        let cli = Cli::try_parse_from([
            "eda",
            "query",
            kicad_fixture_path("simple-demo.kicad_sch")
                .to_str()
                .unwrap(),
            "design-rules",
        ])
        .expect("CLI should parse");

        let err = execute(cli).expect_err("schematic design-rules query must fail");
        let msg = format!("{err:#}");
        assert!(msg.contains("only implemented for boards in M3"), "{msg}");
    }

    #[test]
    fn run_erc_supports_kicad_schematic() {
        let findings =
            run_erc(&kicad_fixture_path("simple-demo.kicad_sch")).expect("erc should succeed");
        assert_eq!(findings.len(), 2);
        assert!(
            findings
                .iter()
                .any(|finding| finding.code == "unconnected_component_pin")
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.code == "undriven_power_net")
        );
    }

    #[test]
    fn run_erc_rejects_non_schematic_inputs() {
        let err = run_erc(&eagle_fixture_path("simple-opamp.lbr"))
            .expect_err("non schematic input must fail");
        let msg = format!("{err:#}");
        assert!(msg.contains("only accepts KiCad .kicad_sch"), "{msg}");
    }

    #[test]
    fn run_drc_supports_kicad_board() {
        let report =
            run_drc(&kicad_fixture_path("partial-route-demo.kicad_pcb")).expect("drc should run");
        assert!(!report.passed);
        assert!(
            report
                .violations
                .iter()
                .any(|violation| violation.code == "connectivity_unrouted_net")
        );
    }

    #[test]
    fn run_drc_rejects_non_board_inputs() {
        let err =
            run_drc(&kicad_fixture_path("simple-demo.kicad_sch")).expect_err("non board must fail");
        let msg = format!("{err:#}");
        assert!(msg.contains("only accepts KiCad .kicad_pcb"), "{msg}");
    }

    #[test]
    fn modify_board_supports_save_slice() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target =
            std::env::temp_dir().join(format!("{}-cli-save-simple-demo.kicad_pcb", Uuid::new_v4()));
        let deleted_uuid =
            Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("uuid should parse");
        let report = modify_board(
            &source,
            &[deleted_uuid],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
        assert!(target.exists());
        let saved = std::fs::read_to_string(&target).expect("saved file should read");
        assert!(!saved.contains(&deleted_uuid.to_string()));
        let _ = std::fs::remove_file(target);
    }

    #[test]
    fn modify_board_supports_delete_via_save_slice() {
        let source = kicad_fixture_path("simple-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-save-simple-demo-via.kicad_pcb",
            Uuid::new_v4()
        ));
        let deleted_uuid =
            Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("uuid should parse");
        let report = modify_board(
            &source,
            &[],
            &[deleted_uuid],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify via save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
        assert!(target.exists());
        let saved = std::fs::read_to_string(&target).expect("saved file should read");
        assert!(!saved.contains(&deleted_uuid.to_string()));
        let _ = std::fs::remove_file(target);
    }

    #[test]
    fn modify_board_supports_set_design_rule_slice() {
        let source = kicad_fixture_path("simple-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-save-simple-demo-rule.kicad_pcb",
            Uuid::new_v4()
        ));
        let report = modify_board(
            &source,
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            Some(125_000),
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify rule save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
        assert!(
            report
                .actions
                .contains(&"set_design_rule clearance_copper 125000".to_string())
        );
        let sidecar = target.with_file_name(format!(
            "{}.rules.json",
            target.file_name().unwrap().to_string_lossy()
        ));
        assert!(sidecar.exists());
        let _ = std::fs::remove_file(target);
        let _ = std::fs::remove_file(sidecar);
    }

    #[test]
    fn modify_board_supports_set_value_slice() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-save-partial-route-value.kicad_pcb",
            Uuid::new_v4()
        ));
        let report = modify_board(
            &source,
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[SetValueInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                value: "22k".to_string(),
            }],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify set_value save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
        let saved = std::fs::read_to_string(&target).expect("saved file should read");
        assert!(saved.contains("(property \"Value\" \"22k\""));
        let _ = std::fs::remove_file(target);
    }

    #[test]
    fn modify_board_supports_move_component_slice() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-save-partial-route-move.kicad_pcb",
            Uuid::new_v4()
        ));
        let report = modify_board(
            &source,
            &[],
            &[],
            &[],
            &[],
            &[MoveComponentInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                position: eda_engine::ir::geometry::Point::new(15_000_000, 12_000_000),
                rotation: Some(90),
            }],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify move save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
        let saved = std::fs::read_to_string(&target).expect("saved file should read");
        assert!(saved.contains("(at 15 12 90)"));
        let _ = std::fs::remove_file(target);
    }

    #[test]
    fn modify_board_supports_set_reference_slice() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-save-partial-route-reference.kicad_pcb",
            Uuid::new_v4()
        ));
        let report = modify_board(
            &source,
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[SetReferenceInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                reference: "R10".to_string(),
            }],
            None,
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify set_reference save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
        let saved = std::fs::read_to_string(&target).expect("saved file should read");
        assert!(saved.contains("(property \"Reference\" \"R10\""));
        let _ = std::fs::remove_file(target);
    }

    #[test]
    fn modify_board_supports_delete_component_slice() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-save-partial-route-delete-component.kicad_pcb",
            Uuid::new_v4()
        ));
        let report = modify_board(
            &source,
            &[],
            &[],
            &[Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap()],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify delete_component save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
        let saved = std::fs::read_to_string(&target).expect("saved file should read");
        assert!(!saved.contains("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"));
        let _ = std::fs::remove_file(target);
    }

    #[test]
    fn modify_board_supports_assign_part_slice() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-save-partial-route-assign-part.kicad_pcb",
            Uuid::new_v4()
        ));
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        let part_uuid = engine
            .search_pool("ALTAMP")
            .expect("search should succeed")
            .first()
            .map(|part| part.uuid)
            .expect("ALTAMP part should exist");

        let report = modify_board(
            &source,
            &[],
            &[],
            &[],
            &[eagle_fixture_path("simple-opamp.lbr")],
            &[],
            &[],
            &[],
            &[AssignPartInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                part_uuid,
            }],
            &[],
            &[],
            &[],
            &[],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify assign_part save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
        let saved = std::fs::read_to_string(&target).expect("saved file should read");
        assert!(saved.contains("(property \"Value\" \"ALTAMP\""));
        assert!(saved.contains("(footprint \"ALT-3\""));
        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.parts.json",
            target.file_name().unwrap().to_string_lossy()
        )));
    }

    #[test]
    fn modify_board_assign_part_preserves_logical_nets_across_known_part_remap() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-save-partial-route-assign-part-remap.kicad_pcb",
            Uuid::new_v4()
        ));
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        let lmv321_part_uuid = engine
            .search_pool("LMV321")
            .expect("search should succeed")
            .first()
            .map(|part| part.uuid)
            .expect("LMV321 part should exist");
        let altamp_part_uuid = engine
            .search_pool("ALTAMP")
            .expect("search should succeed")
            .first()
            .map(|part| part.uuid)
            .expect("ALTAMP part should exist");

        let report = modify_board(
            &source,
            &[],
            &[],
            &[],
            &[eagle_fixture_path("simple-opamp.lbr")],
            &[],
            &[],
            &[],
            &[
                AssignPartInput {
                    uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                    part_uuid: lmv321_part_uuid,
                },
                AssignPartInput {
                    uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                    part_uuid: altamp_part_uuid,
                },
            ],
            &[],
            &[],
            &[],
            &[],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify assign_part remap save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));

        let mut reloaded = Engine::new().expect("engine should initialize");
        reloaded
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        reloaded.import(&target).expect("saved board should reimport");
        let sig = reloaded
            .get_net_info()
            .expect("net info should query")
            .into_iter()
            .find(|net| net.name == "SIG")
            .expect("SIG net should exist");
        assert_eq!(sig.pins.len(), 2);

        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.parts.json",
            target.file_name().unwrap().to_string_lossy()
        )));
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.packages.json",
            target.file_name().unwrap().to_string_lossy()
        )));
    }

    #[test]
    fn modify_board_supports_set_package_slice() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-save-partial-route-set-package.kicad_pcb",
            Uuid::new_v4()
        ));
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        let package_uuid = engine
            .search_pool("ALTAMP")
            .expect("search should succeed")
            .first()
            .map(|part| part.package_uuid)
            .expect("ALTAMP package should exist");

        let report = modify_board(
            &source,
            &[],
            &[],
            &[],
            &[eagle_fixture_path("simple-opamp.lbr")],
            &[],
            &[],
            &[],
            &[],
            &[SetPackageInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                package_uuid,
            }],
            &[],
            &[],
            &[],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify set_package save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
        let updated = match query_components(&target).expect("saved components should query") {
            ComponentListView::Board { components } => components
                .into_iter()
                .find(|component| component.uuid == Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
                .expect("target component should exist"),
        };
        assert_eq!(updated.package_uuid, package_uuid);
        let sig = match query_nets(&target).expect("saved nets should query") {
            NetListView::Board { nets } => nets
                .into_iter()
                .find(|net| net.name == "SIG")
                .expect("SIG net should exist"),
            NetListView::Schematic { .. } => panic!("expected board net list"),
        };
        assert_eq!(sig.pins.len(), 1);
        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.packages.json",
            target.file_name().unwrap().to_string_lossy()
        )));
    }

    #[test]
    fn modify_board_set_package_preserves_logical_nets_across_known_part_remap() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-save-partial-route-set-package-remap.kicad_pcb",
            Uuid::new_v4()
        ));
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        let lmv321_part_uuid = engine
            .search_pool("LMV321")
            .expect("search should succeed")
            .first()
            .map(|part| part.uuid)
            .expect("LMV321 part should exist");
        let altamp_package_uuid = engine
            .search_pool("ALTAMP")
            .expect("search should succeed")
            .first()
            .map(|part| part.package_uuid)
            .expect("ALTAMP package should exist");

        let report = modify_board(
            &source,
            &[],
            &[],
            &[],
            &[eagle_fixture_path("simple-opamp.lbr")],
            &[],
            &[],
            &[],
            &[AssignPartInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                part_uuid: lmv321_part_uuid,
            }],
            &[SetPackageInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                package_uuid: altamp_package_uuid,
            }],
            &[],
            &[],
            &[],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify set_package remap save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));

        let mut reloaded = Engine::new().expect("engine should initialize");
        reloaded
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        reloaded.import(&target).expect("saved board should reimport");
        let sig = reloaded
            .get_net_info()
            .expect("net info should query")
            .into_iter()
            .find(|net| net.name == "SIG")
            .expect("SIG net should exist");
        assert_eq!(sig.pins.len(), 2);

        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.parts.json",
            target.file_name().unwrap().to_string_lossy()
        )));
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.packages.json",
            target.file_name().unwrap().to_string_lossy()
        )));
    }

    #[test]
    fn modify_board_supports_set_package_with_part_slice() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-save-partial-route-set-package-with-part.kicad_pcb",
            Uuid::new_v4()
        ));
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        let lmv321_part_uuid = engine
            .search_pool("LMV321")
            .expect("search should succeed")
            .first()
            .map(|part| part.uuid)
            .expect("LMV321 part should exist");
        let altamp = engine
            .search_pool("ALTAMP")
            .expect("search should succeed")
            .first()
            .cloned()
            .expect("ALTAMP part should exist");

        let report = modify_board(
            &source,
            &[],
            &[],
            &[],
            &[eagle_fixture_path("simple-opamp.lbr")],
            &[],
            &[],
            &[],
            &[AssignPartInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                part_uuid: lmv321_part_uuid,
            }],
            &[],
            &[SetPackageWithPartInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                package_uuid: altamp.package_uuid,
                part_uuid: altamp.uuid,
            }],
            &[],
            &[],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify set_package_with_part save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));

        let mut reloaded = Engine::new().expect("engine should initialize");
        reloaded
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        reloaded.import(&target).expect("saved board should reimport");
        let sig = reloaded
            .get_net_info()
            .expect("net info should query")
            .into_iter()
            .find(|net| net.name == "SIG")
            .expect("SIG net should exist");
        let component = match query_components(&target).expect("saved components should query") {
            ComponentListView::Board { components } => components
                .into_iter()
                .find(|component| component.uuid == Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
                .expect("target component should exist"),
        };
        assert_eq!(component.package_uuid, altamp.package_uuid);
        assert_eq!(component.value, "ALTAMP");
        assert_eq!(sig.pins.len(), 2);

        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.parts.json",
            target.file_name().unwrap().to_string_lossy()
        )));
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.packages.json",
            target.file_name().unwrap().to_string_lossy()
        )));
    }

    #[test]
    fn modify_board_supports_replace_component_slice() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-save-partial-route-replace-component.kicad_pcb",
            Uuid::new_v4()
        ));
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        let lmv321_part_uuid = engine
            .search_pool("LMV321")
            .expect("search should succeed")
            .first()
            .map(|part| part.uuid)
            .expect("LMV321 part should exist");
        let altamp = engine
            .search_pool("ALTAMP")
            .expect("search should succeed")
            .first()
            .cloned()
            .expect("ALTAMP part should exist");

        let report = modify_board(
            &source,
            &[],
            &[],
            &[],
            &[eagle_fixture_path("simple-opamp.lbr")],
            &[],
            &[],
            &[],
            &[AssignPartInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                part_uuid: lmv321_part_uuid,
            }],
            &[],
            &[],
            &[ReplaceComponentInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                package_uuid: altamp.package_uuid,
                part_uuid: altamp.uuid,
            }],
            &[],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify replace_component save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
        assert!(
            report
                .actions
                .iter()
                .any(|action| action.starts_with("replace_component "))
        );

        let mut reloaded = Engine::new().expect("engine should initialize");
        reloaded
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        reloaded.import(&target).expect("saved board should reimport");
        let sig = reloaded
            .get_net_info()
            .expect("net info should query")
            .into_iter()
            .find(|net| net.name == "SIG")
            .expect("SIG net should exist");
        let component = match query_components(&target).expect("saved components should query") {
            ComponentListView::Board { components } => components
                .into_iter()
                .find(|component| component.uuid == Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
                .expect("target component should exist"),
        };
        assert_eq!(component.package_uuid, altamp.package_uuid);
        assert_eq!(component.value, "ALTAMP");
        assert_eq!(sig.pins.len(), 2);

        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.parts.json",
            target.file_name().unwrap().to_string_lossy()
        )));
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.packages.json",
            target.file_name().unwrap().to_string_lossy()
        )));
    }

    #[test]
    fn modify_board_batches_replace_component_inputs_into_one_undo_step() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-save-partial-route-replace-components-batch.kicad_pcb",
            Uuid::new_v4()
        ));
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        let altamp = engine
            .search_pool("ALTAMP")
            .expect("search should succeed")
            .first()
            .cloned()
            .expect("ALTAMP part should exist");

        let report = modify_board(
            &source,
            &[],
            &[],
            &[],
            &[eagle_fixture_path("simple-opamp.lbr")],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[
                ReplaceComponentInput {
                    uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                    package_uuid: altamp.package_uuid,
                    part_uuid: altamp.uuid,
                },
                ReplaceComponentInput {
                    uuid: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                    package_uuid: altamp.package_uuid,
                    part_uuid: altamp.uuid,
                },
            ],
            &[],
            &[],
            None,
            1,
            0,
            Some(&target),
            false,
        )
        .expect("modify batched replace_component undo save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
        assert!(report.actions.contains(&"undo".to_string()));

        let components = match query_components(&target).expect("saved components should query") {
            ComponentListView::Board { components } => components,
        };
        assert_eq!(
            components
                .iter()
                .filter(|component| component.value == "10k")
                .count(),
            2
        );

        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.parts.json",
            target.file_name().unwrap().to_string_lossy()
        )));
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.packages.json",
            target.file_name().unwrap().to_string_lossy()
        )));
    }

    #[test]
    fn modify_board_with_plan_resolves_package_and_part_selectors() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-save-partial-route-apply-replacement-plan.kicad_pcb",
            Uuid::new_v4()
        ));
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        let lmv321 = engine
            .search_pool("LMV321")
            .expect("search should succeed")
            .first()
            .cloned()
            .expect("LMV321 part should exist");
        let altamp = engine
            .search_pool("ALTAMP")
            .expect("search should succeed")
            .first()
            .cloned()
            .expect("ALTAMP part should exist");

        let report = modify_board_with_plan(
            &source,
            &[],
            &[],
            &[],
            &[eagle_fixture_path("simple-opamp.lbr")],
            &[],
            &[],
            &[],
            &[
                AssignPartInput {
                    uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                    part_uuid: lmv321.uuid,
                },
                AssignPartInput {
                    uuid: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                    part_uuid: lmv321.uuid,
                },
            ],
            &[],
            &[],
            &[],
            &[],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
            &[
                PlannedComponentReplacementInput {
                    uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                    package_uuid: Some(altamp.package_uuid),
                    part_uuid: None,
                },
                PlannedComponentReplacementInput {
                    uuid: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                    package_uuid: None,
                    part_uuid: Some(altamp.uuid),
                },
            ],
            &[],
            &[],
        )
        .expect("modify apply_replacement_plan save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));

        let components = match query_components(&target).expect("saved components should query") {
            ComponentListView::Board { components } => components,
        };
        assert_eq!(
            components
                .iter()
                .filter(|component| component.value == "ALTAMP")
                .count(),
            2
        );

        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.parts.json",
            target.file_name().unwrap().to_string_lossy()
        )));
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.packages.json",
            target.file_name().unwrap().to_string_lossy()
        )));
    }

    #[test]
    fn modify_board_with_plan_resolves_best_policy_candidates() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-save-partial-route-apply-replacement-policy.kicad_pcb",
            Uuid::new_v4()
        ));
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        let lmv321 = engine
            .search_pool("LMV321")
            .expect("search should succeed")
            .first()
            .cloned()
            .expect("LMV321 part should exist");

        let report = modify_board_with_plan(
            &source,
            &[],
            &[],
            &[],
            &[eagle_fixture_path("simple-opamp.lbr")],
            &[],
            &[],
            &[],
            &[
                AssignPartInput {
                    uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                    part_uuid: lmv321.uuid,
                },
                AssignPartInput {
                    uuid: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                    part_uuid: lmv321.uuid,
                },
            ],
            &[],
            &[],
            &[],
            &[],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
            &[],
            &[
                PolicyDrivenComponentReplacementInput {
                    uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                    policy: ComponentReplacementPolicy::BestCompatiblePackage,
                },
                PolicyDrivenComponentReplacementInput {
                    uuid: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                    policy: ComponentReplacementPolicy::BestCompatiblePart,
                },
            ],
            &[],
        )
        .expect("modify apply_replacement_policy save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));

        let components = match query_components(&target).expect("saved components should query") {
            ComponentListView::Board { components } => components,
        };
        assert_eq!(
            components
                .iter()
                .filter(|component| component.value == "ALTAMP")
                .count(),
            2
        );

        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.parts.json",
            target.file_name().unwrap().to_string_lossy()
        )));
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.packages.json",
            target.file_name().unwrap().to_string_lossy()
        )));
    }

    #[test]
    fn modify_board_with_plan_applies_scoped_replacement_policy() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-save-partial-route-apply-scoped-replacement-policy.kicad_pcb",
            Uuid::new_v4()
        ));
        let mut engine = Engine::new().expect("engine should initialize");
        engine
            .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
            .expect("library import should succeed");
        let lmv321 = engine
            .search_pool("LMV321")
            .expect("search should succeed")
            .first()
            .cloned()
            .expect("LMV321 part should exist");

        let report = modify_board_with_plan(
            &source,
            &[],
            &[],
            &[],
            &[eagle_fixture_path("simple-opamp.lbr")],
            &[],
            &[],
            &[],
            &[
                AssignPartInput {
                    uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                    part_uuid: lmv321.uuid,
                },
                AssignPartInput {
                    uuid: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap(),
                    part_uuid: lmv321.uuid,
                },
            ],
            &[],
            &[],
            &[],
            &[],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
            &[],
            &[],
            &[ScopedComponentReplacementPolicyInput {
                scope: ComponentReplacementScope {
                    reference_prefix: Some("R".to_string()),
                    value_equals: Some("LMV321".to_string()),
                    current_package_uuid: None,
                    current_part_uuid: None,
                },
                policy: ComponentReplacementPolicy::BestCompatiblePackage,
            }],
        )
        .expect("modify apply_scoped_replacement_policy save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));

        let components = match query_components(&target).expect("saved components should query") {
            ComponentListView::Board { components } => components,
        };
        assert_eq!(
            components
                .iter()
                .filter(|component| component.value == "ALTAMP")
                .count(),
            2
        );

        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.parts.json",
            target.file_name().unwrap().to_string_lossy()
        )));
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.packages.json",
            target.file_name().unwrap().to_string_lossy()
        )));
    }

    #[test]
    fn modify_board_supports_set_net_class_slice() {
        let source = kicad_fixture_path("simple-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-save-simple-demo-net-class.kicad_pcb",
            Uuid::new_v4()
        ));
        let net_uuid = match query_nets(&source).expect("nets should query") {
            NetListView::Board { nets } => nets
                .into_iter()
                .find(|net| net.name == "GND")
                .expect("GND net should exist")
                .uuid,
            NetListView::Schematic { .. } => panic!("expected board net list"),
        };

        let report = modify_board(
            &source,
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[SetNetClassInput {
                net_uuid,
                class_name: "power".to_string(),
                clearance: 125_000,
                track_width: 250_000,
                via_drill: 300_000,
                via_diameter: 600_000,
                diffpair_width: 0,
                diffpair_gap: 0,
            }],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify set_net_class save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
        let nets = query_nets(&target).expect("saved nets should query");
        let gnd = match nets {
            NetListView::Board { nets } => nets
                .into_iter()
                .find(|net| net.uuid == net_uuid)
                .expect("updated GND net should exist"),
            NetListView::Schematic { .. } => panic!("expected board net list"),
        };
        assert_eq!(gnd.class, "power");
        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_file(target.with_file_name(format!(
            "{}.net-classes.json",
            target.file_name().unwrap().to_string_lossy()
        )));
    }

    #[test]
    fn modify_board_supports_rotate_component_slice() {
        let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
        let target = std::env::temp_dir().join(format!(
            "{}-cli-save-partial-route-rotate.kicad_pcb",
            Uuid::new_v4()
        ));
        let report = modify_board(
            &source,
            &[],
            &[],
            &[],
            &[],
            &[],
            &[RotateComponentInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                rotation: 180,
            }],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            None,
            0,
            0,
            Some(&target),
            false,
        )
        .expect("modify rotate_component save should succeed");
        assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
        let saved = std::fs::read_to_string(&target).expect("saved file should read");
        assert!(saved.contains("(at 10 10 180)"));
        let _ = std::fs::remove_file(target);
    }

    #[test]
    fn run_check_supports_board_and_schematic_inputs() {
        let board = run_check(&kicad_fixture_path("simple-demo.kicad_pcb"))
            .expect("board check should succeed");
        match board {
            CheckReport::Board {
                summary,
                diagnostics,
            } => {
                assert_eq!(summary.status, eda_engine::api::CheckStatus::Info);
                assert_eq!(summary.infos, 1);
                assert_eq!(summary.by_code.len(), 1);
                assert_eq!(summary.by_code[0].code, "net_without_copper");
                assert_eq!(summary.by_code[0].count, 1);
                assert_eq!(diagnostics.len(), 1);
                assert_eq!(diagnostics[0].kind, "net_without_copper");
            }
            other => panic!("expected board report, got {other:?}"),
        }

        let partial_board = run_check(&kicad_fixture_path("partial-route-demo.kicad_pcb"))
            .expect("partial-route board check should succeed");
        match partial_board {
            CheckReport::Board {
                summary,
                diagnostics,
            } => {
                assert_eq!(summary.status, eda_engine::api::CheckStatus::Warning);
                assert_eq!(summary.warnings, 1);
                assert_eq!(summary.infos, 1);
                assert!(
                    summary
                        .by_code
                        .iter()
                        .any(|entry| entry.code == "partially_routed_net" && entry.count == 1)
                );
                assert!(
                    diagnostics
                        .iter()
                        .any(|diagnostic| diagnostic.kind == "partially_routed_net")
                );
            }
            other => panic!("expected board report, got {other:?}"),
        }

        let schematic = run_check(&kicad_fixture_path("simple-demo.kicad_sch"))
            .expect("schematic check should succeed");
        match schematic {
            CheckReport::Schematic {
                summary,
                diagnostics,
                erc,
            } => {
                assert_eq!(summary.status, eda_engine::api::CheckStatus::Warning);
                assert_eq!(summary.warnings, 3);
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
            other => panic!("expected schematic report, got {other:?}"),
        }
    }

    #[test]
    fn execute_erc_command_returns_finding_output() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "erc",
            kicad_fixture_path("simple-demo.kicad_sch")
                .to_str()
                .unwrap(),
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("erc command should succeed");
        assert!(output.contains("\"code\": \"unconnected_component_pin\""));
        assert!(output.contains("\"code\": \"undriven_power_net\""));
        assert!(output.contains("\"waived\": false"));
    }

    #[test]
    fn execute_drc_command_returns_report_output() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "drc",
            kicad_fixture_path("partial-route-demo.kicad_pcb")
                .to_str()
                .unwrap(),
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("drc command should succeed");
        assert!(output.contains("\"passed\": false"));
        assert!(output.contains("\"code\": \"connectivity_unrouted_net\""));
    }

    #[test]
    fn execute_check_command_returns_schematic_report_output() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            kicad_fixture_path("simple-demo.kicad_sch")
                .to_str()
                .unwrap(),
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("check command should succeed");
        assert!(output.contains("\"domain\": \"schematic\""));
        assert!(output.contains("\"status\": \"warning\""));
        assert!(output.contains("\"kind\": \"dangling_component_pin\""));
        assert!(output.contains("\"code\": \"unconnected_component_pin\""));
    }

    #[test]
    fn execute_check_command_returns_board_report_output() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            kicad_fixture_path("simple-demo.kicad_pcb")
                .to_str()
                .unwrap(),
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("check command should succeed");
        assert!(output.contains("\"domain\": \"board\""));
        assert!(output.contains("\"status\": \"info\""));
        assert!(output.contains("\"by_code\""));
        assert!(output.contains("\"kind\": \"net_without_copper\""));
    }

    #[test]
    fn execute_check_command_returns_partial_route_board_report_output() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            kicad_fixture_path("partial-route-demo.kicad_pcb")
                .to_str()
                .unwrap(),
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("check command should succeed");
        assert!(output.contains("\"domain\": \"board\""));
        assert!(output.contains("\"status\": \"warning\""));
        assert!(output.contains("\"kind\": \"partially_routed_net\""));
    }

    #[test]
    fn execute_check_command_text_output_is_compact_for_schematic() {
        let cli = Cli::try_parse_from([
            "eda",
            "check",
            kicad_fixture_path("simple-demo.kicad_sch")
                .to_str()
                .unwrap(),
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("check command should succeed");
        assert!(output.contains("schematic check: status=warning"));
        assert!(output.contains("counts:"));
        assert!(output.contains("dangling_component_pin x1"));
        assert!(output.contains("erc:"));
        assert!(output.contains("[warning] unconnected_component_pin:"));
    }

    #[test]
    fn execute_check_command_text_output_is_compact_for_board() {
        let cli = Cli::try_parse_from([
            "eda",
            "check",
            kicad_fixture_path("simple-demo.kicad_pcb")
                .to_str()
                .unwrap(),
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("check command should succeed");
        assert!(output.contains("board check: status=info"));
        assert!(output.contains("counts:"));
        assert!(output.contains("net_without_copper x1"));
        assert!(output.contains("diagnostics:"));
    }

    #[test]
    fn execute_check_command_text_output_is_compact_for_partial_route_board() {
        let cli = Cli::try_parse_from([
            "eda",
            "check",
            kicad_fixture_path("partial-route-demo.kicad_pcb")
                .to_str()
                .unwrap(),
        ])
        .expect("CLI should parse");

        let output = execute(cli).expect("check command should succeed");
        assert!(output.contains("board check: status=warning"));
        assert!(output.contains("partially_routed_net x1"));
        assert!(output.contains("net_without_copper x1"));
    }

    #[test]
    fn render_check_report_text_includes_input_without_explicit_driver() {
        let test_uuid =
            eda_engine::ir::ids::import_uuid(&eda_engine::ir::ids::namespace_kicad(), "test-pin");
        let report = CheckReport::Schematic {
            summary: eda_engine::api::CheckSummary {
                status: CheckStatus::Info,
                errors: 0,
                warnings: 0,
                infos: 1,
                waived: 0,
                by_code: vec![eda_engine::api::CheckCodeCount {
                    code: "input_without_explicit_driver".into(),
                    count: 1,
                }],
            },
            diagnostics: Vec::new(),
            erc: vec![ErcFinding {
                id: test_uuid,
                code: "input_without_explicit_driver",
                severity: eda_engine::erc::ErcSeverity::Info,
                message: "input pins on net IN_P have no explicit driver".into(),
                net_name: Some("IN_P".into()),
                component: None,
                pin: None,
                objects: vec![eda_engine::erc::ErcObjectRef {
                    kind: "pin",
                    key: "Q1.1".into(),
                }],
                object_uuids: vec![test_uuid],
                waived: false,
            }],
        };

        let output = render_check_report_text(&report);
        assert!(output.contains("schematic check: status=info"));
        assert!(output.contains("input_without_explicit_driver x1"));
        assert!(output.contains("[info] input_without_explicit_driver:"));
    }

    #[test]
    fn execute_check_command_honors_fail_on_threshold() {
        let cli = Cli::try_parse_from([
            "eda",
            "check",
            kicad_fixture_path("simple-demo.kicad_sch")
                .to_str()
                .unwrap(),
            "--fail-on",
            "warning",
        ])
        .expect("CLI should parse");

        let (_output, exit_code) =
            execute_with_exit_code(cli).expect("check command should run successfully");
        assert_eq!(exit_code, 1);
    }

    #[test]
    fn execute_check_command_allows_higher_fail_on_threshold() {
        let cli = Cli::try_parse_from([
            "eda",
            "check",
            kicad_fixture_path("simple-demo.kicad_sch")
                .to_str()
                .unwrap(),
            "--fail-on",
            "error",
        ])
        .expect("CLI should parse");

        let (output, exit_code) =
            execute_with_exit_code(cli).expect("check command should run successfully");
        assert_eq!(exit_code, 0);
        assert!(output.contains("schematic check: status=warning"));
    }

    #[test]
    fn execute_drc_command_uses_violation_exit_code() {
        let cli = Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "drc",
            kicad_fixture_path("partial-route-demo.kicad_pcb")
                .to_str()
                .unwrap(),
        ])
        .expect("CLI should parse");

        let (_output, exit_code) =
            execute_with_exit_code(cli).expect("drc command should run successfully");
        assert_eq!(exit_code, 1);
    }
}
