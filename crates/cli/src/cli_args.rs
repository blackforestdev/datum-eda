use std::path::PathBuf;

use clap::{Parser, Subcommand};
use eda_engine::api::ScopedComponentReplacementPlan;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[path = "cli_args_board_component.rs"]
mod cli_args_board_component;
#[path = "cli_args_board_dimension.rs"]
mod cli_args_board_dimension;
#[path = "cli_args_commands.rs"]
mod cli_args_commands;
#[path = "cli_args_drill.rs"]
mod cli_args_drill;
#[path = "cli_args_gerber_plan.rs"]
mod cli_args_gerber_plan;
#[path = "cli_args_inventory.rs"]
mod cli_args_inventory;
#[path = "cli_args_manufacturing.rs"]
mod cli_args_manufacturing;
#[path = "cli_args_native_support.rs"]
mod cli_args_native_support;
#[path = "cli_args_output.rs"]
mod cli_args_output;
#[path = "cli_args_pool.rs"]
mod cli_args_pool;
#[path = "cli_args_project_command_args_artifacts.rs"]
mod cli_args_project_command_args_artifacts;
#[path = "cli_args_project_command_args_board.rs"]
mod cli_args_project_command_args_board;
#[path = "cli_args_project_command_args_schematic_connectivity.rs"]
mod cli_args_project_command_args_schematic_connectivity;
#[path = "cli_args_project_command_args_schematic_symbols.rs"]
mod cli_args_project_command_args_schematic_symbols;
#[path = "cli_args_project_commands.rs"]
mod cli_args_project_commands;
#[path = "cli_args_project_query_plan.rs"]
mod cli_args_project_query_plan;

pub(crate) use self::cli_args_board_component::{
    BoardComponentMechanicalArgs, BoardComponentModels3dArgs, BoardComponentPadsArgs,
    BoardComponentSilkscreenArgs, SetBoardComponentLayerArgs, SetBoardComponentPackageArgs,
    SetBoardComponentPartArgs, SetBoardComponentReferenceArgs, SetBoardComponentValueArgs,
};
use self::cli_args_board_dimension::{EditBoardDimensionArgs, PlaceBoardDimensionArgs};
pub(crate) use self::cli_args_commands::{Cli, Commands, QueryCommands};
pub(crate) use self::cli_args_drill::{
    CompareDrillArgs, CompareExcellonDrillArgs, ExportDrillArgs, ExportExcellonDrillArgs,
    InspectDrillArgs, ReportDrillHoleClassesArgs, ValidateDrillArgs, ValidateExcellonDrillArgs,
};
pub(crate) use self::cli_args_gerber_plan::{
    CompareGerberExportPlanArgs, CompareGerberSetArgs, ExportGerberSetArgs, PlanGerberExportArgs,
    ValidateGerberSetArgs,
};
pub(crate) use self::cli_args_inventory::{
    CompareBomArgs, ComparePnpArgs, ExportBomArgs, ExportPnpArgs, InspectBomArgs, InspectPnpArgs,
    ValidateBomArgs, ValidatePnpArgs,
};
pub(crate) use self::cli_args_manufacturing::{
    CompareManufacturingSetArgs, ExportManufacturingSetArgs, InspectManufacturingSetArgs,
    ManifestManufacturingSetArgs, ReportManufacturingArgs, ValidateManufacturingSetArgs,
};
pub(crate) use self::cli_args_native_support::{
    NativeHiddenPowerBehaviorArg, NativePortDirectionArg, NativeSymbolDisplayModeArg,
};
pub(crate) use self::cli_args_output::{FailOn, OutputFormat};
pub(crate) use self::cli_args_pool::{PoolCommands, ReplacementPolicyArg};
pub(crate) use self::cli_args_project_command_args_artifacts::*;
pub(crate) use self::cli_args_project_command_args_board::*;
pub(crate) use self::cli_args_project_command_args_schematic_connectivity::*;
pub(crate) use self::cli_args_project_command_args_schematic_symbols::*;
pub(crate) use self::cli_args_project_commands::ProjectCommands;
pub(crate) use self::cli_args_project_query_plan::{NativeProjectQueryCommands, PlanCommands};

#[derive(Clone, clap::ValueEnum)]
pub(crate) enum NativeLabelKindArg {
    Local,
    Global,
    Hierarchical,
    Power,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ManifestFileFingerprint {
    pub(crate) path: PathBuf,
    pub(crate) source_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ScopedReplacementPlanManifest {
    pub(crate) kind: String,
    pub(crate) version: u32,
    pub(crate) board_path: PathBuf,
    pub(crate) board_source_hash: String,
    pub(crate) libraries: Vec<ManifestFileFingerprint>,
    pub(crate) plan: ScopedComponentReplacementPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ManifestDriftStatus {
    Match,
    Drifted,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ManifestFileInspection {
    pub(crate) path: PathBuf,
    pub(crate) recorded_source_hash: String,
    pub(crate) current_source_hash: Option<String>,
    pub(crate) status: ManifestDriftStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ScopedReplacementPlanManifestInspection {
    pub(crate) manifest_path: PathBuf,
    pub(crate) kind: String,
    pub(crate) source_version: u32,
    pub(crate) version: u32,
    pub(crate) migration_applied: bool,
    pub(crate) replacements: usize,
    pub(crate) all_inputs_match: bool,
    pub(crate) board: ManifestFileInspection,
    pub(crate) libraries: Vec<ManifestFileInspection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ScopedReplacementPlanManifestUpgradeReport {
    pub(crate) input_path: PathBuf,
    pub(crate) output_path: PathBuf,
    pub(crate) kind: String,
    pub(crate) source_version: u32,
    pub(crate) version: u32,
    pub(crate) migration_applied: bool,
    pub(crate) replacements: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ScopedReplacementPlanManifestValidationReport {
    pub(crate) manifest_path: PathBuf,
    pub(crate) source_version: u32,
    pub(crate) version: u32,
    pub(crate) migration_applied: bool,
    pub(crate) all_inputs_match: bool,
    pub(crate) board_status: ManifestDriftStatus,
    pub(crate) drifted_libraries: usize,
    pub(crate) missing_libraries: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ScopedReplacementPlanManifestValidationSummary {
    pub(crate) manifests_checked: usize,
    pub(crate) manifests_passing: usize,
    pub(crate) manifests_failing: usize,
    pub(crate) reports: Vec<ScopedReplacementPlanManifestValidationReport>,
}
