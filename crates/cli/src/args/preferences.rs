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
}
