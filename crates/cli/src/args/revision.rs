use super::*;

#[derive(Subcommand)]
pub(crate) enum RevisionCommands {
    /// Print the engine-owned revision operation, query, refusal, proposal, and evidence inventory
    Catalog,
    /// Query current or historical revision authority from a native Project
    Query(RevisionQueryArgs),
}

#[derive(clap::Args)]
pub(crate) struct RevisionQueryArgs {
    /// Native Project root
    pub(crate) path: PathBuf,
    /// Stable query name from `revision catalog`
    pub(crate) query: String,
    /// Historical authority event sequence (inclusive)
    #[arg(long)]
    pub(crate) as_of_sequence: Option<u64>,
    /// Reject if the resolved Design model revision differs
    #[arg(long)]
    pub(crate) expected_model_revision: Option<String>,
}
