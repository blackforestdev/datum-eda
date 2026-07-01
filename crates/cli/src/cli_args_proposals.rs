pub(crate) use super::cli_args_proposal_library::{
    ProposalAddPoolFootprintSilkscreenCircleArgs, ProposalAddPoolFootprintSilkscreenLineArgs,
    ProposalAddPoolFootprintSilkscreenPolygonArgs, ProposalAddPoolFootprintSilkscreenRectArgs,
    ProposalCreatePoolEntityArgs, ProposalCreatePoolFootprintArgs,
    ProposalCreatePoolLibraryObjectArgs, ProposalCreatePoolPackageArgs,
    ProposalCreatePoolPadstackArgs, ProposalCreatePoolPinPadMapArgs, ProposalCreatePoolSymbolArgs,
    ProposalCreatePoolUnitArgs, ProposalSetPoolFootprintCourtyardPolygonArgs,
    ProposalSetPoolFootprintCourtyardRectArgs, ProposalSetPoolFootprintPadArgs,
    ProposalSetPoolPackageCourtyardPolygonArgs, ProposalSetPoolPackageCourtyardRectArgs,
    ProposalSetPoolPackagePadArgs, ProposalSetPoolPinPadMapArgs,
};
use super::*;

#[derive(Subcommand)]
pub(crate) enum ProposalCommands {
    /// Create a draft proposal from an OperationBatch JSON file
    Create(ProjectCreateProposalArgs),
    /// Create a draft proposal to place a schematic label
    CreatePlaceLabel(ProposalPlaceLabelArgs),
    /// Create a draft proposal to place a schematic symbol
    CreatePlaceSymbol(ProposalPlaceSymbolArgs),
    /// Create a draft proposal to draw a schematic wire
    CreateDrawWire(ProposalDrawWireArgs),
    /// Create a draft proposal to replace one board component package/part/value
    CreateBoardComponentReplacement(ProposalCreateBoardComponentReplacementArgs),
    /// Create one draft proposal to replace multiple board component package/part/value sets
    CreateBoardComponentReplacements(ProposalCreateBoardComponentReplacementsArgs),
    /// Create one draft proposal from replacement-plan shaped selections
    CreateBoardComponentReplacementPlan(ProposalCreateBoardComponentReplacementPlanArgs),
    /// Create a draft proposal to author a raw native pool-library object
    CreatePoolLibraryObject(ProposalCreatePoolLibraryObjectArgs),
    /// Create a draft proposal to author a native pool unit
    CreatePoolUnit(ProposalCreatePoolUnitArgs),
    /// Create a draft proposal to author a native pool symbol
    CreatePoolSymbol(ProposalCreatePoolSymbolArgs),
    /// Create a draft proposal to author a native pool entity
    CreatePoolEntity(ProposalCreatePoolEntityArgs),
    /// Create a draft proposal to author a native pool padstack
    CreatePoolPadstack(ProposalCreatePoolPadstackArgs),
    /// Create a draft proposal to author a native pool package
    CreatePoolPackage(ProposalCreatePoolPackageArgs),
    /// Create a draft proposal to author a native pool footprint
    CreatePoolFootprint(ProposalCreatePoolFootprintArgs),
    /// Create a draft proposal to author a native pool PinPadMap
    CreatePoolPinPadMap(ProposalCreatePoolPinPadMapArgs),
    /// Create a draft proposal to update native pool PinPadMap mappings
    SetPoolPinPadMap(ProposalSetPoolPinPadMapArgs),
    /// Create a draft proposal to set one pad on a native pool footprint
    SetPoolFootprintPad(ProposalSetPoolFootprintPadArgs),
    /// Create a draft proposal to set rectangular footprint courtyard geometry
    SetPoolFootprintCourtyardRect(ProposalSetPoolFootprintCourtyardRectArgs),
    /// Create a draft proposal to set polygon footprint courtyard geometry
    SetPoolFootprintCourtyardPolygon(ProposalSetPoolFootprintCourtyardPolygonArgs),
    /// Create a draft proposal to append one footprint silkscreen line
    AddPoolFootprintSilkscreenLine(ProposalAddPoolFootprintSilkscreenLineArgs),
    /// Create a draft proposal to append one footprint silkscreen rectangle
    AddPoolFootprintSilkscreenRect(ProposalAddPoolFootprintSilkscreenRectArgs),
    /// Create a draft proposal to append one footprint silkscreen circle
    AddPoolFootprintSilkscreenCircle(ProposalAddPoolFootprintSilkscreenCircleArgs),
    /// Create a draft proposal to append one footprint silkscreen polygon/polyline
    AddPoolFootprintSilkscreenPolygon(ProposalAddPoolFootprintSilkscreenPolygonArgs),
    /// Create a draft proposal to add one pad to a native pool package
    SetPoolPackagePad(ProposalSetPoolPackagePadArgs),
    /// Create a draft proposal to set rectangular package courtyard geometry
    SetPoolPackageCourtyardRect(ProposalSetPoolPackageCourtyardRectArgs),
    /// Create a draft proposal to set polygon package courtyard geometry
    SetPoolPackageCourtyardPolygon(ProposalSetPoolPackageCourtyardPolygonArgs),
    /// Create a draft proposal to author an OutputJob
    CreateOutputJob(ProposalCreateOutputJobArgs),
    /// Create a draft proposal to update an OutputJob
    UpdateOutputJob(ProposalUpdateOutputJobArgs),
    /// Create a draft proposal to delete an OutputJob
    DeleteOutputJob(ProposalDeleteOutputJobArgs),
    /// Create a draft proposal to author a ManufacturingPlan
    CreateManufacturingPlan(ProposalCreateManufacturingPlanArgs),
    /// Create a draft proposal to update a ManufacturingPlan
    UpdateManufacturingPlan(ProposalUpdateManufacturingPlanArgs),
    /// Create a draft proposal to delete a ManufacturingPlan
    DeleteManufacturingPlan(ProposalDeleteManufacturingPlanArgs),
    /// Create a draft proposal to author a PanelProjection
    CreatePanelProjection(ProposalCreatePanelProjectionArgs),
    /// Create a draft proposal to update a PanelProjection
    UpdatePanelProjection(ProposalUpdatePanelProjectionArgs),
    /// Create a draft proposal to delete a PanelProjection
    DeletePanelProjection(ProposalDeletePanelProjectionArgs),
    /// List resolver-discovered native project proposals
    List(ProjectProposalListArgs),
    /// Show one persisted proposal plus validation state
    Show(ProjectShowProposalArgs),
    /// Preview one persisted proposal's classified diff without writing shards
    Preview(ProjectPreviewProposalArgs),
    /// Validate one persisted proposal against the current model revision
    Validate(ProjectValidateProposalArgs),
    /// Mark one proposal as accepted, deferred, or rejected
    Review(ProjectReviewProposalArgs),
    /// Defer one draft proposal without applying it
    Defer(ProjectDeferProposalArgs),
    /// Reject one draft proposal without applying it
    Reject(ProjectRejectProposalArgs),
    /// Accept one draft proposal and immediately apply it through the proposal gateway
    AcceptApply(ProjectApplyProposalArgs),
    /// Apply one accepted proposal through the proposal gateway
    Apply(ProjectApplyProposalArgs),
}

#[derive(clap::Args)]
pub(crate) struct ProjectProposalListArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct ProposalPlaceLabelArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Target sheet UUID
    #[arg(long)]
    pub(crate) sheet: Uuid,
    /// Label name
    #[arg(long)]
    pub(crate) name: String,
    /// Label kind
    #[arg(long, value_enum, default_value = "local")]
    pub(crate) kind: NativeLabelKindArg,
    /// X coordinate in nm
    #[arg(long)]
    pub(crate) x_nm: i64,
    /// Y coordinate in nm
    #[arg(long)]
    pub(crate) y_nm: i64,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalPlaceSymbolArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Target sheet UUID
    #[arg(long)]
    pub(crate) sheet: Uuid,
    /// Reference designator
    #[arg(long)]
    pub(crate) reference: String,
    /// Display value
    #[arg(long)]
    pub(crate) value: String,
    /// Optional library identifier for future resolution
    #[arg(long = "lib-id")]
    pub(crate) lib_id: Option<String>,
    /// X coordinate in nm
    #[arg(long)]
    pub(crate) x_nm: i64,
    /// Y coordinate in nm
    #[arg(long)]
    pub(crate) y_nm: i64,
    /// Rotation in degrees
    #[arg(long = "rotation-deg", default_value_t = 0)]
    pub(crate) rotation_deg: i32,
    /// Mirror the symbol about its local Y axis
    #[arg(long, default_value_t = false)]
    pub(crate) mirrored: bool,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalDrawWireArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Target sheet UUID
    #[arg(long)]
    pub(crate) sheet: Uuid,
    /// Start X coordinate in nm
    #[arg(long)]
    pub(crate) from_x_nm: i64,
    /// Start Y coordinate in nm
    #[arg(long)]
    pub(crate) from_y_nm: i64,
    /// End X coordinate in nm
    #[arg(long)]
    pub(crate) to_x_nm: i64,
    /// End Y coordinate in nm
    #[arg(long)]
    pub(crate) to_y_nm: i64,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalCreateBoardComponentReplacementArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Board component UUID
    #[arg(long = "component")]
    pub(crate) component: Uuid,
    /// Replacement package UUID
    #[arg(long = "package")]
    pub(crate) package: Option<Uuid>,
    /// Replacement part UUID
    #[arg(long = "part")]
    pub(crate) part: Option<Uuid>,
    /// Replacement component value; defaults to preserving current value
    #[arg(long = "value")]
    pub(crate) value: Option<String>,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalCreateBoardComponentReplacementsArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Replacement JSON object; repeat for each component.
    ///
    /// Example: {"component":"<uuid>","package":"<uuid>","part":"<uuid>","value":"10k"}
    #[arg(long = "replacement", required = true)]
    pub(crate) replacements: Vec<String>,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalCreateBoardComponentReplacementPlanArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Replacement plan JSON object; repeat for each component.
    ///
    /// Example: {"uuid":"<component>","package_uuid":"<package>","part_uuid":"<part>","value":"10k"}
    #[arg(long = "selection", required = true)]
    pub(crate) selections: Vec<String>,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalCreateOutputJobArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Deterministic output prefix this job will generate
    #[arg(long)]
    pub(crate) prefix: String,
    /// Artifact include scopes: comma-separated gerber-set, manufacturing-set, bom, pnp, drill, or all
    #[arg(long)]
    pub(crate) include: String,
    /// Preferred output directory for generated artifacts
    #[arg(long = "output-dir")]
    pub(crate) output_dir: Option<PathBuf>,
    /// Human-readable output job name
    #[arg(long)]
    pub(crate) name: Option<String>,
    /// Manufacturing plan UUID this output job executes
    #[arg(long = "manufacturing-plan")]
    pub(crate) manufacturing_plan: Option<Uuid>,
    /// Variant overlay UUID this output job targets
    #[arg(long)]
    pub(crate) variant: Option<Uuid>,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalUpdateOutputJobArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// OutputJob UUID
    #[arg(long = "output-job")]
    pub(crate) output_job: Uuid,
    /// Replacement human-readable output job name
    #[arg(long)]
    pub(crate) name: Option<String>,
    /// Replacement preferred output directory for generated artifacts
    #[arg(long = "output-dir", conflicts_with = "clear_output_dir")]
    pub(crate) output_dir: Option<PathBuf>,
    /// Replacement manufacturing plan UUID this output job executes
    #[arg(
        long = "manufacturing-plan",
        conflicts_with = "clear_manufacturing_plan"
    )]
    pub(crate) manufacturing_plan: Option<Uuid>,
    /// Replacement variant overlay UUID this output job targets
    #[arg(long, conflicts_with = "clear_variant")]
    pub(crate) variant: Option<Uuid>,
    /// Clear any linked manufacturing plan
    #[arg(long = "clear-manufacturing-plan")]
    pub(crate) clear_manufacturing_plan: bool,
    /// Clear any linked variant
    #[arg(long = "clear-variant")]
    pub(crate) clear_variant: bool,
    /// Clear any stored output directory so launchers use their default
    #[arg(long = "clear-output-dir")]
    pub(crate) clear_output_dir: bool,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalDeleteOutputJobArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// OutputJob UUID
    #[arg(long = "output-job")]
    pub(crate) output_job: Uuid,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalCreateManufacturingPlanArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Deterministic manufacturing artifact filename prefix
    #[arg(long)]
    pub(crate) prefix: String,
    /// Human-readable manufacturing plan name
    #[arg(long)]
    pub(crate) name: Option<String>,
    /// Optional variant UUID this plan targets
    #[arg(long)]
    pub(crate) variant: Option<Uuid>,
    /// Optional panel projection UUID this plan targets instead of the board
    #[arg(long = "panel-projection")]
    pub(crate) panel_projection: Option<Uuid>,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalUpdateManufacturingPlanArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// ManufacturingPlan UUID to update
    #[arg(long = "manufacturing-plan")]
    pub(crate) manufacturing_plan: Uuid,
    /// Replacement human-readable manufacturing plan name
    #[arg(long)]
    pub(crate) name: Option<String>,
    /// Replacement deterministic manufacturing artifact filename prefix
    #[arg(long)]
    pub(crate) prefix: Option<String>,
    /// Replacement variant UUID this plan targets
    #[arg(long)]
    pub(crate) variant: Option<Uuid>,
    /// Clear the variant target
    #[arg(long = "clear-variant")]
    pub(crate) clear_variant: bool,
    /// Replacement panel projection UUID this plan targets
    #[arg(long = "panel-projection")]
    pub(crate) panel_projection: Option<Uuid>,
    /// Clear the panel target and retarget the current board
    #[arg(long = "clear-panel-projection")]
    pub(crate) clear_panel_projection: bool,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalDeleteManufacturingPlanArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// ManufacturingPlan UUID to delete
    #[arg(long = "manufacturing-plan")]
    pub(crate) manufacturing_plan: Uuid,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalCreatePanelProjectionArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Deterministic panel key
    #[arg(long)]
    pub(crate) key: String,
    /// Human-readable panel projection name
    #[arg(long)]
    pub(crate) name: Option<String>,
    /// Optional board UUID for the first panel instance; defaults to current board
    #[arg(long)]
    pub(crate) board: Option<Uuid>,
    /// First board instance X offset in nanometers
    #[arg(long = "x-nm", default_value_t = 0)]
    pub(crate) x_nm: i64,
    /// First board instance Y offset in nanometers
    #[arg(long = "y-nm", default_value_t = 0)]
    pub(crate) y_nm: i64,
    /// First board instance rotation in degrees
    #[arg(long = "rotation-deg", default_value_t = 0)]
    pub(crate) rotation_deg: i32,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalUpdatePanelProjectionArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// PanelProjection UUID to update
    #[arg(long = "panel-projection")]
    pub(crate) panel_projection: Uuid,
    /// Replacement human-readable panel projection name
    #[arg(long)]
    pub(crate) name: Option<String>,
    /// Replacement board UUID for the first panel instance
    #[arg(long)]
    pub(crate) board: Option<Uuid>,
    /// Replacement first board instance X offset in nanometers
    #[arg(long = "x-nm")]
    pub(crate) x_nm: Option<i64>,
    /// Replacement first board instance Y offset in nanometers
    #[arg(long = "y-nm")]
    pub(crate) y_nm: Option<i64>,
    /// Replacement first board instance rotation in degrees
    #[arg(long = "rotation-deg")]
    pub(crate) rotation_deg: Option<i32>,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProposalDeletePanelProjectionArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// PanelProjection UUID to delete
    #[arg(long = "panel-projection")]
    pub(crate) panel_projection: Uuid,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectRejectProposalArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Persisted proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Uuid,
}
