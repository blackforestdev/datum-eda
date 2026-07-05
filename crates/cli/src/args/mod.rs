// args/ — home for all CLI argument/parser types (clap structs and enums),
// one file per family.
//
// Wave 2 (args lane) complete: the legacy cli_args chain (cli_args.rs ->
// cli_args_root.rs -> cli_args_surface.rs) is dissolved. Every family file
// lives here (`cli_args_` prefix dropped; cli_args_commands.rs -> root.rs,
// cli_args_project_commands.rs -> project.rs) and this file is the single
// routing surface main.rs consumes (`use args::*;`). The named re-export
// lists below are ported verbatim from the old cli_args_surface.rs shim;
// shared leftover types from that shim live in surface.rs.

#[allow(unused_imports)] // Scope anchor: keeps crate-root names visible here.
use super::*;

mod artifact;
mod board_component;
mod board_dimension;
mod check;
mod context;
mod drill;
mod gerber_plan;
mod inventory;
mod journal;
mod manufacturing;
mod native_support;
mod output;
mod pool;
mod prelude;
mod project;
mod project_command_args_artifacts;
mod project_command_args_board;
mod project_command_args_board_handoff;
mod project_command_args_schematic_connectivity;
mod project_command_args_schematic_symbols;
mod project_component_instances;
mod project_import;
mod project_journal;
mod project_library;
mod project_library_footprint;
mod project_library_part_bindings;
mod project_library_pin_pad_map;
mod project_library_symbol_pin;
mod project_output_jobs;
mod project_proposals;
mod project_query_plan;
mod project_waivers;
mod proposal_library;
mod proposals;
mod root;
mod surface;

pub(crate) use self::prelude::*;

pub(crate) use self::artifact::{
    ArtifactCancelOutputJobRunArgs, ArtifactCommands, ArtifactCompareArgs, ArtifactFilesArgs,
    ArtifactGenerateArgs, ArtifactListArgs, ArtifactPreviewArgs, ArtifactShowArgs,
    ArtifactStartOutputJobRunArgs, ArtifactValidateArgs,
};
pub(crate) use self::board_component::{
    BoardComponentMechanicalArgs, BoardComponentModels3dArgs, BoardComponentPadsArgs,
    BoardComponentSilkscreenArgs, SetBoardComponentLayerArgs, SetBoardComponentPackageArgs,
    SetBoardComponentPartArgs, SetBoardComponentReferenceArgs, SetBoardComponentValueArgs,
};
pub(crate) use self::board_dimension::{EditBoardDimensionArgs, PlaceBoardDimensionArgs};
pub(crate) use self::check::{
    CheckAcceptDeviationArgs, CheckCommands, CheckFillZonesArgs, CheckImportedArgs, CheckListArgs,
    CheckProfilesArgs, CheckRepairStandardsArgs, CheckRunArgs, CheckShowArgs, CheckWaiveArgs,
};
pub(crate) use self::context::{
    ContextCommands, ContextGetArgs, ContextSessionActivityArgs, ContextSessionEventsArgs,
};
pub(crate) use self::drill::{
    CompareDrillArgs, CompareExcellonDrillArgs, ExportDrillArgs, ExportExcellonDrillArgs,
    InspectDrillArgs, ReportDrillHoleClassesArgs, ValidateDrillArgs, ValidateExcellonDrillArgs,
};
pub(crate) use self::gerber_plan::{
    CompareGerberExportPlanArgs, CompareGerberSetArgs, ExportGerberSetArgs, PlanGerberExportArgs,
    ValidateGerberSetArgs,
};
pub(crate) use self::inventory::{
    CompareBomArgs, ComparePnpArgs, ExportBomArgs, ExportPnpArgs, InspectBomArgs, InspectPnpArgs,
    ValidateBomArgs, ValidatePnpArgs,
};
pub(crate) use self::journal::{JournalCommands, JournalListArgs, JournalShowArgs};
pub(crate) use self::manufacturing::{
    CompareManufacturingSetArgs, ExportManufacturingSetArgs, InspectManufacturingSetArgs,
    ManifestManufacturingSetArgs, ProjectCreateManufacturingPlanArgs,
    ProjectCreatePanelProjectionArgs, ProjectDeleteManufacturingPlanArgs,
    ProjectDeletePanelProjectionArgs, ProjectUpdateManufacturingPlanArgs,
    ProjectUpdatePanelProjectionArgs, ReportManufacturingArgs, ValidateManufacturingSetArgs,
};
pub(crate) use self::native_support::{
    NativeHiddenPowerBehaviorArg, NativePortDirectionArg, NativeSymbolDisplayModeArg,
};
pub(crate) use self::output::{FailOn, OutputFormat};
pub(crate) use self::pool::{PoolCommands, ReplacementPolicyArg};
pub(crate) use self::project::ProjectCommands;
pub(crate) use self::project_command_args_artifacts::*;
pub(crate) use self::project_command_args_board::*;
pub(crate) use self::project_command_args_board_handoff::*;
pub(crate) use self::project_command_args_schematic_connectivity::*;
pub(crate) use self::project_command_args_schematic_symbols::*;
pub(crate) use self::project_component_instances::{
    ProjectBindComponentInstanceArgs, ProjectDeleteComponentInstanceArgs,
    ProjectSetComponentInstanceArgs,
};
pub(crate) use self::project_import::{
    ProjectImportEagleLibraryArgs, ProjectImportKiCadBoardArgs, ProjectImportKiCadFootprintArgs,
    ProjectImportKiCadSchematicArgs,
};
pub(crate) use self::project_journal::{ProjectRedoArgs, ProjectUndoArgs};
pub(crate) use self::project_library::{
    ProjectAddPoolPackageModel3dArgs, ProjectAddPoolPackageSilkscreenArcArgs,
    ProjectAddPoolPackageSilkscreenCircleArgs, ProjectAddPoolPackageSilkscreenLineArgs,
    ProjectAddPoolPackageSilkscreenPolygonArgs, ProjectAddPoolPackageSilkscreenRectArgs,
    ProjectAddPoolPackageSilkscreenTextArgs, ProjectAddPoolSymbolArcArgs,
    ProjectAddPoolSymbolCircleArgs, ProjectAddPoolSymbolLineArgs, ProjectAddPoolSymbolPolygonArgs,
    ProjectAddPoolSymbolRectArgs, ProjectAddPoolSymbolTextArgs, ProjectAttachPoolPartModelArgs,
    ProjectCreatePoolEntityArgs, ProjectCreatePoolLibraryObjectArgs, ProjectCreatePoolPackageArgs,
    ProjectCreatePoolPadstackArgs, ProjectCreatePoolPartArgs, ProjectCreatePoolSymbolArgs,
    ProjectCreatePoolUnitArgs, ProjectDeletePoolLibraryObjectArgs, ProjectDetachPoolPartModelArgs,
    ProjectGcPoolModelsArgs, ProjectSetPoolLibraryObjectArgs, ProjectSetPoolPackageBodyHeightsArgs,
    ProjectSetPoolPackageCourtyardPolygonArgs, ProjectSetPoolPackageCourtyardRectArgs,
    ProjectSetPoolPackagePadArgs, ProjectSetPoolPartBehaviouralModelsArgs,
    ProjectSetPoolPartMetadataArgs, ProjectSetPoolPartOrderableMpnsArgs,
    ProjectSetPoolPartPackagingOptionsArgs, ProjectSetPoolPartPadMapArgs,
    ProjectSetPoolPartPadMapEntryArgs, ProjectSetPoolPartParametricArgs,
    ProjectSetPoolPartSupplyChainArgs, ProjectSetPoolPartTagsArgs, ProjectSetPoolPartThermalArgs,
    ProjectSetPoolUnitPinArgs,
};
pub(crate) use self::project_library_footprint::{
    IpcDensityLevelArg, ProjectAddPoolFootprintSilkscreenCircleArgs,
    ProjectAddPoolFootprintSilkscreenLineArgs, ProjectAddPoolFootprintSilkscreenPolygonArgs,
    ProjectAddPoolFootprintSilkscreenRectArgs, ProjectCreatePoolFootprintArgs,
    ProjectGenerateIpc7351bSoicArgs, ProjectGenerateIpc7351bTwoTerminalChipArgs,
    ProjectSetPoolFootprintCourtyardPolygonArgs, ProjectSetPoolFootprintCourtyardRectArgs,
    ProjectSetPoolFootprintPadArgs,
};
pub(crate) use self::project_library_part_bindings::ProjectSetPoolPartBindingsArgs;
pub(crate) use self::project_library_pin_pad_map::{
    ProjectCreatePoolPinPadMapArgs, ProjectSetPoolPinPadMapArgs,
};
pub(crate) use self::project_library_symbol_pin::ProjectSetPoolSymbolPinAnchorArgs;
pub(crate) use self::project_output_jobs::{
    ProjectCancelOutputJobRunArgs, ProjectCreateGerberOutputJobArgs, ProjectCreateOutputJobArgs,
    ProjectDeleteOutputJobArgs, ProjectRunOutputJobArgs, ProjectStartOutputJobRunArgs,
    ProjectUpdateOutputJobArgs,
};
pub(crate) use self::project_proposals::{
    ProjectApplyProposalArgs, ProjectCreateProposalArgs, ProjectDeferProposalArgs,
    ProjectPreviewProposalArgs, ProjectReviewProposalArgs, ProjectShowProposalArgs,
    ProjectValidateProposalArgs, ProposalReviewStatusArg, ProposalSourceArg,
};
pub(crate) use self::project_query_plan::{
    NativeProjectQueryCommands, NativeRoutePathCandidateAuthoredCopperGraphPolicy, PlanCommands,
};
pub(crate) use self::project_waivers::{ProjectAcceptDeviationArgs, ProjectWaiveFindingArgs};
pub(crate) use self::proposal_library::{
    ProposalAddPoolFootprintSilkscreenCircleArgs, ProposalAddPoolFootprintSilkscreenLineArgs,
    ProposalAddPoolFootprintSilkscreenPolygonArgs, ProposalAddPoolFootprintSilkscreenRectArgs,
    ProposalCreatePoolEntityArgs, ProposalCreatePoolFootprintArgs,
    ProposalCreatePoolLibraryObjectArgs, ProposalCreatePoolPackageArgs,
    ProposalCreatePoolPadstackArgs, ProposalCreatePoolPinPadMapArgs, ProposalCreatePoolSymbolArgs,
    ProposalCreatePoolUnitArgs, ProposalGenerateIpc7351bSoicArgs,
    ProposalGenerateIpc7351bTwoTerminalChipArgs,
    ProposalSetPoolFootprintCourtyardPolygonArgs, ProposalSetPoolFootprintCourtyardRectArgs,
    ProposalSetPoolFootprintPadArgs, ProposalSetPoolPackageCourtyardPolygonArgs,
    ProposalSetPoolPackageCourtyardRectArgs, ProposalSetPoolPackagePadArgs,
    ProposalSetPoolPinPadMapArgs,
};
pub(crate) use self::proposals::{
    ProjectProposalListArgs, ProjectRejectProposalArgs, ProposalBindComponentInstanceArgs,
    ProposalCommands, ProposalCreateBoardComponentReplacementArgs,
    ProposalCreateBoardComponentReplacementPlanArgs, ProposalCreateBoardComponentReplacementsArgs,
    ProposalCreateManufacturingPlanArgs, ProposalCreateOutputJobArgs,
    ProposalCreatePanelProjectionArgs, ProposalDeleteComponentInstanceArgs,
    ProposalDeleteManufacturingPlanArgs, ProposalDeleteOutputJobArgs,
    ProposalDeletePanelProjectionArgs, ProposalDrawWireArgs, ProposalPlaceLabelArgs,
    ProposalPlaceSymbolArgs, ProposalSetComponentInstanceArgs, ProposalUpdateManufacturingPlanArgs,
    ProposalUpdateOutputJobArgs, ProposalUpdatePanelProjectionArgs,
};
pub(crate) use self::root::{
    Cli, Commands, ImportedQueryCommandParser, ImportedQueryCommands, QueryCommands, QueryPathArgs,
};
pub(crate) use self::surface::*;
