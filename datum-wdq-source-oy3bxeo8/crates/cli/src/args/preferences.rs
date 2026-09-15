use crate::*;

#[derive(Clone, clap::ValueEnum)]
pub(crate) enum PreferencesSeedSource {
    Global,
    Factory,
}

#[derive(Subcommand)]
pub(crate) enum PreferencesCommands {
    /// Describe active sections and typed controls
    Describe,
    /// List active preference values
    List {
        #[arg(long)]
        section: Option<String>,
    },
    /// Get one active preference
    Get { key: String },
    /// Search active preference labels, descriptions, keys, and aliases
    Search { query: String },
    /// Explain one active preference resolution
    Explain { key: String },
    /// Preview the exact eight-key Units seed for Project creation
    #[command(name = "preview-project-units-seed")]
    PreviewProjectUnitsSeed {
        #[arg(long, value_enum)]
        source: PreferencesSeedSource,
        #[arg(long)]
        expected: Option<String>,
    },
    /// Set one active Global Preference after foreground-TTY confirmation
    Set {
        key: String,
        #[arg(long)]
        value_json: String,
        #[arg(long)]
        expected: String,
        #[arg(long)]
        reason: String,
        #[arg(long)]
        request_id: Uuid,
    },
    /// Reset one active Global Preference after foreground-TTY confirmation
    Reset {
        key: String,
        #[arg(long)]
        expected: String,
        #[arg(long)]
        reason: String,
        #[arg(long)]
        request_id: Uuid,
    },
    /// Prepare, validate, accept, or reject portable preference proposals
    Proposal {
        #[command(subcommand)]
        action: PreferencesProposalCommands,
    },
}

#[derive(Subcommand)]
pub(crate) enum PreferencesProposalCommands {
    /// Prepare one proposal from a mutation request JSON file or stdin (`-`)
    Prepare {
        #[arg(long)]
        request_json: PathBuf,
        #[arg(long)]
        rationale: String,
    },
    /// Validate one proposal JSON file or stdin (`-`)
    Validate {
        #[arg(long)]
        proposal_json: PathBuf,
    },
    /// Apply one proposal after foreground-TTY confirmation
    #[command(name = "accept-apply")]
    AcceptApply {
        #[arg(long)]
        proposal_json: PathBuf,
    },
    /// Authorize the named MCP session's next matching apply after TTY review
    #[command(name = "authorize-mcp")]
    AuthorizeMcp {
        #[arg(long)]
        proposal_json: PathBuf,
        #[arg(long)]
        mcp_session: String,
    },
    /// Reject one proposal without persistent state
    Reject {
        #[arg(long)]
        proposal_json: PathBuf,
    },
}
