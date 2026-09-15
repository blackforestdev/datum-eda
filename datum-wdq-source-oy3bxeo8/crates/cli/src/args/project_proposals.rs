use crate::*;
use clap::ValueEnum;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "snake_case")]
pub(crate) enum ProposalReviewStatusArg {
    Accepted,
    Deferred,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "snake_case")]
pub(crate) enum ProposalSourceArg {
    Manual,
    Cli,
    Tool,
    Assistant,
    Check,
    Import,
}

#[derive(clap::Args)]
pub(crate) struct ProjectCreateProposalArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// OperationBatch JSON file to wrap in a draft proposal
    #[arg(long = "batch")]
    pub(crate) batch: PathBuf,
    /// Human-readable rationale for review
    #[arg(long = "rationale")]
    pub(crate) rationale: String,
    /// Optional stable proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Option<Uuid>,
    /// Proposal source provenance
    #[arg(long = "source", value_enum, default_value_t = ProposalSourceArg::Cli)]
    pub(crate) source: ProposalSourceArg,
    /// Check run UUID associated with the proposal
    #[arg(long = "check-run")]
    pub(crate) checks_run: Vec<Uuid>,
    /// Check finding fingerprint associated with the proposal
    #[arg(long = "finding-fingerprint")]
    pub(crate) finding_fingerprints: Vec<String>,
}

#[derive(clap::Args)]
pub(crate) struct ProjectShowProposalArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Persisted proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectPreviewProposalArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Persisted proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectValidateProposalArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Persisted proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectDeferProposalArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Persisted proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Uuid,
}

#[derive(clap::Args)]
pub(crate) struct ProjectReviewProposalArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Persisted proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Uuid,
    /// Review status to persist
    #[arg(long = "status", value_enum)]
    pub(crate) status: ProposalReviewStatusArg,
}

#[derive(clap::Args)]
pub(crate) struct ProjectApplyProposalArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Accepted persisted proposal UUID
    #[arg(long = "proposal")]
    pub(crate) proposal: Uuid,
}
