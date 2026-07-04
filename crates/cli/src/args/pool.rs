use clap::Subcommand;
use std::path::PathBuf;

#[derive(Clone, Copy, clap::ValueEnum)]
pub(crate) enum ReplacementPolicyArg {
    Package,
    Part,
}

#[derive(Subcommand)]
pub(crate) enum PoolCommands {
    /// Search for parts
    Search {
        /// Search query
        query: String,

        /// Eagle library files to load into the in-memory pool for this search
        #[arg(long = "library", required = true)]
        libraries: Vec<PathBuf>,
    },
}
