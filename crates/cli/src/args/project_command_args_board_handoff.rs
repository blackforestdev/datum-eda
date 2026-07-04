use crate::*;

#[derive(clap::Args)]
pub(crate) struct ProjectGenerateBoardComponentsArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Commit generated board packages through the project journal
    #[arg(long, conflicts_with = "as_proposal")]
    pub(crate) apply: bool,
    /// Write generated board packages as a draft proposal instead of applying them
    #[arg(long = "as-proposal")]
    pub(crate) as_proposal: bool,
    /// Optional stable proposal UUID for --as-proposal
    #[arg(long = "proposal", requires = "as_proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal review rationale for --as-proposal
    #[arg(long = "rationale", requires = "as_proposal")]
    pub(crate) rationale: Option<String>,
    /// Initial X coordinate in nm
    #[arg(long = "origin-x-nm", default_value_t = 0)]
    pub(crate) origin_x_nm: i64,
    /// Initial Y coordinate in nm
    #[arg(long = "origin-y-nm", default_value_t = 0)]
    pub(crate) origin_y_nm: i64,
    /// X pitch between generated packages in nm
    #[arg(long = "pitch-nm", default_value_t = 5_000_000)]
    pub(crate) pitch_nm: i64,
    /// Layer identifier
    #[arg(long, default_value_t = 1)]
    pub(crate) layer: i32,
}
