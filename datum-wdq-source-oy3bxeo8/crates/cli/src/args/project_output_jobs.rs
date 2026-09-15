use crate::*;

#[derive(clap::Args)]
pub(crate) struct ProjectCreateGerberOutputJobArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Deterministic output prefix this job will generate
    #[arg(long)]
    pub(crate) prefix: String,
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
    /// Write a draft proposal instead of applying the create immediately
    #[arg(long = "as-proposal")]
    pub(crate) as_proposal: bool,
    /// Optional stable proposal UUID when --as-proposal is used
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale when --as-proposal is used
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCreateOutputJobArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Deterministic output prefix this job will generate
    #[arg(long)]
    pub(crate) prefix: String,
    /// Preferred output directory for generated artifacts
    #[arg(long = "output-dir")]
    pub(crate) output_dir: Option<PathBuf>,
    /// Artifact include scopes: comma-separated gerber-set, manufacturing-set, bom, pnp, drill, or all
    #[arg(long)]
    pub(crate) include: String,
    /// Human-readable output job name
    #[arg(long)]
    pub(crate) name: Option<String>,
    /// Manufacturing plan UUID this output job executes
    #[arg(long = "manufacturing-plan")]
    pub(crate) manufacturing_plan: Option<Uuid>,
    /// Variant overlay UUID this output job targets
    #[arg(long)]
    pub(crate) variant: Option<Uuid>,
    /// Write a draft proposal instead of applying the create immediately
    #[arg(long = "as-proposal")]
    pub(crate) as_proposal: bool,
    /// Optional stable proposal UUID when --as-proposal is used
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale when --as-proposal is used
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectUpdateOutputJobArgs {
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
    /// Write a draft proposal instead of applying the update immediately
    #[arg(long = "as-proposal")]
    pub(crate) as_proposal: bool,
    /// Optional stable proposal UUID when --as-proposal is used
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale when --as-proposal is used
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectRunOutputJobArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// OutputJob UUID to execute
    #[arg(long = "output-job")]
    pub(crate) output_job: Uuid,
    /// One-shot output directory override; stored OutputJob output_dir wins when omitted
    #[arg(long = "output-dir")]
    pub(crate) output_dir: Option<PathBuf>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectStartOutputJobRunArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// OutputJob UUID to mark running
    #[arg(long = "output-job")]
    pub(crate) output_job: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCancelOutputJobRunArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// OutputJobRun UUID to mark canceled
    #[arg(long = "run")]
    pub(crate) run: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeleteOutputJobArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// OutputJob UUID
    #[arg(long = "output-job")]
    pub(crate) output_job: Uuid,
    /// Write a draft proposal instead of applying the delete immediately
    #[arg(long = "as-proposal")]
    pub(crate) as_proposal: bool,
    /// Optional stable proposal UUID when --as-proposal is used
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale when --as-proposal is used
    #[arg(long = "rationale")]
    pub(crate) rationale: Option<String>,
}
