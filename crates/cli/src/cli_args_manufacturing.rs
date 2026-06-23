use clap::Args;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Args)]
pub(crate) struct ProjectCreateManufacturingPlanArgs {
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
    #[arg(long = "as-proposal")]
    pub(crate) as_proposal: bool,
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(Args)]
pub(crate) struct ProjectDeleteManufacturingPlanArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// ManufacturingPlan UUID to delete
    #[arg(long = "manufacturing-plan")]
    pub(crate) manufacturing_plan: Uuid,
    #[arg(long = "as-proposal")]
    pub(crate) as_proposal: bool,
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(Args)]
pub(crate) struct ProjectUpdateManufacturingPlanArgs {
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
    #[arg(long = "as-proposal")]
    pub(crate) as_proposal: bool,
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(Args)]
pub(crate) struct ProjectCreatePanelProjectionArgs {
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
    #[arg(long = "as-proposal")]
    pub(crate) as_proposal: bool,
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(Args)]
pub(crate) struct ProjectDeletePanelProjectionArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// PanelProjection UUID to delete
    #[arg(long = "panel-projection")]
    pub(crate) panel_projection: Uuid,
    #[arg(long = "as-proposal")]
    pub(crate) as_proposal: bool,
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(Args)]
pub(crate) struct ProjectUpdatePanelProjectionArgs {
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
    #[arg(long = "as-proposal")]
    pub(crate) as_proposal: bool,
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(Args)]
pub(crate) struct ReportManufacturingArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Optional Gerber artifact filename prefix; defaults to the board name
    #[arg(long)]
    pub(crate) prefix: Option<String>,
}

#[derive(Args)]
pub(crate) struct ExportManufacturingSetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Directory to write the current supported manufacturing set into
    #[arg(long = "output-dir")]
    pub(crate) output_dir: PathBuf,
    /// Optional artifact filename prefix; defaults to the board name
    #[arg(long)]
    pub(crate) prefix: Option<String>,
    /// Optional stored OutputJob UUID whose prefix, variant, and include list are used by default
    #[arg(long = "output-job", conflicts_with = "job")]
    pub(crate) output_job: Option<Uuid>,
    /// Optional stored OutputJob name whose prefix, variant, and include list are used by default
    #[arg(long)]
    pub(crate) job: Option<String>,
    /// Optional artifact include scopes: comma-separated gerber-set, manufacturing-set, bom, pnp, drill, or all
    #[arg(long)]
    pub(crate) include: Option<String>,
    /// Optional variant overlay UUID used to filter assembly rows
    #[arg(long)]
    pub(crate) variant: Option<Uuid>,
}

#[derive(Args)]
pub(crate) struct ValidateManufacturingSetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Directory to validate against the current supported manufacturing set
    #[arg(long = "output-dir")]
    pub(crate) output_dir: PathBuf,
    /// Optional artifact filename prefix; defaults to the board name
    #[arg(long)]
    pub(crate) prefix: Option<String>,
    /// Optional stored OutputJob UUID whose prefix, variant, and include list are used by default
    #[arg(long = "output-job", conflicts_with = "job")]
    pub(crate) output_job: Option<Uuid>,
    /// Optional stored OutputJob name whose prefix, variant, and include list are used by default
    #[arg(long)]
    pub(crate) job: Option<String>,
    /// Optional artifact include scopes: comma-separated gerber-set, manufacturing-set, bom, pnp, drill, or all
    #[arg(long)]
    pub(crate) include: Option<String>,
    /// Optional variant overlay UUID used to filter assembly rows
    #[arg(long)]
    pub(crate) variant: Option<Uuid>,
}

#[derive(Args)]
pub(crate) struct CompareManufacturingSetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Directory to compare semantically against the current supported manufacturing set
    #[arg(long = "output-dir")]
    pub(crate) output_dir: PathBuf,
    /// Optional artifact filename prefix; defaults to the board name
    #[arg(long)]
    pub(crate) prefix: Option<String>,
    /// Optional stored OutputJob UUID whose prefix, variant, and include list are used by default
    #[arg(long = "output-job", conflicts_with = "job")]
    pub(crate) output_job: Option<Uuid>,
    /// Optional stored OutputJob name whose prefix, variant, and include list are used by default
    #[arg(long)]
    pub(crate) job: Option<String>,
    /// Optional artifact include scopes: comma-separated gerber-set, manufacturing-set, bom, pnp, drill, or all
    #[arg(long)]
    pub(crate) include: Option<String>,
    /// Optional variant overlay UUID used to filter assembly rows
    #[arg(long)]
    pub(crate) variant: Option<Uuid>,
}

#[derive(Args)]
pub(crate) struct ManifestManufacturingSetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Directory containing or intended to contain the current supported manufacturing set
    #[arg(long = "output-dir")]
    pub(crate) output_dir: PathBuf,
    /// Optional artifact filename prefix; defaults to the board name
    #[arg(long)]
    pub(crate) prefix: Option<String>,
    /// Optional stored OutputJob UUID whose prefix and include list are used by default
    #[arg(long = "output-job", conflicts_with = "job")]
    pub(crate) output_job: Option<Uuid>,
    /// Optional stored OutputJob name whose prefix and include list are used by default
    #[arg(long)]
    pub(crate) job: Option<String>,
    /// Optional artifact include scopes: comma-separated gerber-set, manufacturing-set, bom, pnp, drill, or all
    #[arg(long)]
    pub(crate) include: Option<String>,
}

#[derive(Args)]
pub(crate) struct InspectManufacturingSetArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Directory containing the current supported manufacturing set
    #[arg(long = "output-dir")]
    pub(crate) output_dir: PathBuf,
    /// Optional artifact filename prefix; defaults to the board name
    #[arg(long)]
    pub(crate) prefix: Option<String>,
    /// Optional stored OutputJob UUID whose prefix and include list are used by default
    #[arg(long = "output-job", conflicts_with = "job")]
    pub(crate) output_job: Option<Uuid>,
    /// Optional stored OutputJob name whose prefix and include list are used by default
    #[arg(long)]
    pub(crate) job: Option<String>,
    /// Optional artifact include scopes: comma-separated gerber-set, manufacturing-set, bom, pnp, drill, or all
    #[arg(long)]
    pub(crate) include: Option<String>,
}
