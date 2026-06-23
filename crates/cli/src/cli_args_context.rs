use super::*;

#[derive(Subcommand)]
pub(crate) enum ContextCommands {
    /// Return the current Datum terminal/session context envelope
    Get(ContextGetArgs),
    /// Return the freshest available Datum terminal/session context envelope
    Refresh(ContextGetArgs),
    /// Return recorded tool-session events for the current terminal/session
    #[command(name = "session-events")]
    SessionEvents(ContextSessionEventsArgs),
    /// Return summarized tool-session activity for the current terminal/session
    #[command(name = "session-activity")]
    SessionActivity(ContextSessionActivityArgs),
}

#[derive(Parser)]
pub(crate) struct ContextGetArgs {
    /// Expected terminal/session id; rejects mismatched discovery envelopes
    #[arg(long)]
    pub(crate) session: Option<String>,

    /// Explicit context/discovery JSON path
    #[arg(long)]
    pub(crate) path: Option<PathBuf>,

    /// Project root containing .datum/gui-terminal-context.json
    #[arg(long = "project-root")]
    pub(crate) project_root: Option<PathBuf>,
}

#[derive(Parser)]
pub(crate) struct ContextSessionEventsArgs {
    /// Expected terminal/session id; rejects mismatched discovery envelopes
    #[arg(long)]
    pub(crate) session: Option<String>,

    /// Explicit context/discovery JSON path
    #[arg(long)]
    pub(crate) path: Option<PathBuf>,

    /// Project root containing .datum/gui-terminal-context.json
    #[arg(long = "project-root")]
    pub(crate) project_root: Option<PathBuf>,

    /// Exact-match filter for the JSONL event kind field
    #[arg(long = "event-kind")]
    pub(crate) event_kind: Option<String>,

    /// Exact-match filter for the event origin field
    #[arg(long)]
    pub(crate) origin: Option<String>,

    /// Exact-match filter for the event command_id field
    #[arg(long = "command-id")]
    pub(crate) command_id: Option<String>,

    /// Exact-match filter for the event execution_id field
    #[arg(long = "execution-id")]
    pub(crate) execution_id: Option<String>,

    /// Return only the newest N matching events
    #[arg(long)]
    pub(crate) limit: Option<usize>,
}

#[derive(Parser)]
pub(crate) struct ContextSessionActivityArgs {
    /// Expected terminal/session id; rejects mismatched discovery envelopes
    #[arg(long)]
    pub(crate) session: Option<String>,

    /// Explicit context/discovery JSON path
    #[arg(long)]
    pub(crate) path: Option<PathBuf>,

    /// Project root containing .datum/gui-terminal-context.json
    #[arg(long = "project-root")]
    pub(crate) project_root: Option<PathBuf>,

    /// Exact-match filter for the JSONL event kind field
    #[arg(long = "event-kind")]
    pub(crate) event_kind: Option<String>,

    /// Exact-match filter for the event origin field
    #[arg(long)]
    pub(crate) origin: Option<String>,

    /// Exact-match filter for the event command_id field
    #[arg(long = "command-id")]
    pub(crate) command_id: Option<String>,

    /// Exact-match filter for the event execution_id field
    #[arg(long = "execution-id")]
    pub(crate) execution_id: Option<String>,

    /// Summarize only the newest N matching events
    #[arg(long)]
    pub(crate) limit: Option<usize>,
}
