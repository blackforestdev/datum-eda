pub(crate) use super::*;

pub(crate) use self::cli_args_artifact::{
    ArtifactCancelOutputJobRunArgs, ArtifactCommands, ArtifactCompareArgs, ArtifactFilesArgs,
    ArtifactGenerateArgs, ArtifactListArgs, ArtifactPreviewArgs, ArtifactShowArgs,
    ArtifactStartOutputJobRunArgs, ArtifactValidateArgs,
};
pub(crate) use self::cli_args_board_component::{
    BoardComponentMechanicalArgs, BoardComponentModels3dArgs, BoardComponentPadsArgs,
    BoardComponentSilkscreenArgs, SetBoardComponentLayerArgs, SetBoardComponentPackageArgs,
    SetBoardComponentPartArgs, SetBoardComponentReferenceArgs, SetBoardComponentValueArgs,
};
pub(crate) use self::cli_args_board_dimension::{EditBoardDimensionArgs, PlaceBoardDimensionArgs};
pub(crate) use self::cli_args_check::{
    CheckAcceptDeviationArgs, CheckCommands, CheckFillZonesArgs, CheckImportedArgs, CheckListArgs,
    CheckProfilesArgs, CheckRepairStandardsArgs, CheckRunArgs, CheckShowArgs, CheckWaiveArgs,
};
pub(crate) use self::cli_args_commands::{
    Cli, Commands, ImportedQueryCommandParser, ImportedQueryCommands, QueryCommands, QueryPathArgs,
};
pub(crate) use self::cli_args_context::{
    ContextCommands, ContextGetArgs, ContextSessionActivityArgs, ContextSessionEventsArgs,
};
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
pub(crate) use self::cli_args_journal::{JournalCommands, JournalListArgs, JournalShowArgs};
pub(crate) use self::cli_args_manufacturing::{
    CompareManufacturingSetArgs, ExportManufacturingSetArgs, InspectManufacturingSetArgs,
    ManifestManufacturingSetArgs, ProjectCreateManufacturingPlanArgs,
    ProjectCreatePanelProjectionArgs, ProjectDeleteManufacturingPlanArgs,
    ProjectDeletePanelProjectionArgs, ProjectUpdateManufacturingPlanArgs,
    ProjectUpdatePanelProjectionArgs, ReportManufacturingArgs, ValidateManufacturingSetArgs,
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
pub(crate) use self::cli_args_project_component_instances::{
    ProjectBindComponentInstanceArgs, ProjectDeleteComponentInstanceArgs,
    ProjectSetComponentInstanceArgs,
};
pub(crate) use self::cli_args_project_import::{
    ProjectImportEagleLibraryArgs, ProjectImportKiCadBoardArgs, ProjectImportKiCadFootprintArgs,
    ProjectImportKiCadSchematicArgs,
};
pub(crate) use self::cli_args_project_journal::{ProjectRedoArgs, ProjectUndoArgs};
pub(crate) use self::cli_args_project_library::{
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
    ProjectSetPoolSymbolPinAnchorArgs, ProjectSetPoolUnitPinArgs,
};
pub(crate) use self::cli_args_project_output_jobs::{
    ProjectCancelOutputJobRunArgs, ProjectCreateGerberOutputJobArgs, ProjectCreateOutputJobArgs,
    ProjectDeleteOutputJobArgs, ProjectRunOutputJobArgs, ProjectStartOutputJobRunArgs,
    ProjectUpdateOutputJobArgs,
};
pub(crate) use self::cli_args_project_proposals::{
    ProjectApplyProposalArgs, ProjectCreateProposalArgs, ProjectDeferProposalArgs,
    ProjectPreviewProposalArgs, ProjectReviewProposalArgs, ProjectShowProposalArgs,
    ProjectValidateProposalArgs, ProposalReviewStatusArg, ProposalSourceArg,
};
pub(crate) use self::cli_args_project_query_plan::{
    NativeProjectQueryCommands, NativeRoutePathCandidateAuthoredCopperGraphPolicy, PlanCommands,
};
pub(crate) use self::cli_args_project_waivers::{
    ProjectAcceptDeviationArgs, ProjectWaiveFindingArgs,
};
pub(crate) use self::cli_args_proposals::{
    ProjectProposalListArgs, ProjectRejectProposalArgs, ProposalCommands,
    ProposalCreateBoardComponentReplacementArgs, ProposalCreateBoardComponentReplacementPlanArgs,
    ProposalCreateBoardComponentReplacementsArgs, ProposalCreateManufacturingPlanArgs,
    ProposalCreateOutputJobArgs, ProposalCreatePanelProjectionArgs, ProposalCreatePoolEntityArgs,
    ProposalCreatePoolLibraryObjectArgs, ProposalCreatePoolPackageArgs,
    ProposalCreatePoolPadstackArgs, ProposalCreatePoolSymbolArgs, ProposalCreatePoolUnitArgs,
    ProposalDeleteManufacturingPlanArgs, ProposalDeleteOutputJobArgs,
    ProposalDeletePanelProjectionArgs, ProposalDrawWireArgs, ProposalPlaceLabelArgs,
    ProposalPlaceSymbolArgs, ProposalSetPoolPackageCourtyardPolygonArgs,
    ProposalSetPoolPackageCourtyardRectArgs, ProposalSetPoolPackagePadArgs,
    ProposalUpdateManufacturingPlanArgs, ProposalUpdateOutputJobArgs,
    ProposalUpdatePanelProjectionArgs,
};

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
