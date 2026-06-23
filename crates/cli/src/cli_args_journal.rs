use super::*;

#[derive(Subcommand)]
pub(crate) enum JournalCommands {
    /// Return the native project transaction journal summary
    List(JournalListArgs),
    /// Show one transaction journal record by UUID
    Show(JournalShowArgs),
    /// Undo the latest journaled transaction by appending a compensating transaction
    Undo(ProjectUndoArgs),
    /// Redo the latest journaled undo by appending a compensating transaction
    Redo(ProjectRedoArgs),
}

#[derive(clap::Args)]
pub(crate) struct JournalListArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
}

#[derive(clap::Args)]
pub(crate) struct JournalShowArgs {
    /// Project root directory
    pub(crate) path: PathBuf,
    /// Transaction UUID
    #[arg(long = "transaction")]
    pub(crate) transaction: Uuid,
}
