use super::*;

#[derive(clap::Args)]
pub(crate) struct ProjectWaiveFindingArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Stable CheckFinding fingerprint to waive
    #[arg(long)]
    pub(crate) fingerprint: String,
    /// Waiver rationale recorded in the authored project
    #[arg(long)]
    pub(crate) rationale: String,
    /// Optional actor/user recorded on the waiver
    #[arg(long = "created-by")]
    pub(crate) created_by: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectAcceptDeviationArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Stable CheckFinding fingerprint to accept as a deviation
    #[arg(long)]
    pub(crate) fingerprint: String,
    /// Deviation rationale recorded in the authored project
    #[arg(long)]
    pub(crate) rationale: String,
    /// Optional actor/user recorded as accepting the deviation
    #[arg(long = "accepted-by")]
    pub(crate) accepted_by: Option<String>,
}
