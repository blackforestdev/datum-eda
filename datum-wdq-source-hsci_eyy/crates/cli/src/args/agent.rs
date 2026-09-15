use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub(crate) enum AgentCommands {
    /// List governed terminal-agent adapters
    List,
    /// Probe one adapter's executable and native CLI contract
    Doctor(AgentDoctorArgs),
    /// Launch one agent with session-scoped Datum MCP configuration
    Launch(AgentLaunchArgs),
}

#[derive(Args)]
pub(crate) struct AgentDoctorArgs {
    /// Stable adapter id: codex, claude-code, cursor-cli, or local-generic
    pub(crate) adapter: String,
    /// Explicit executable for local-generic or to override vendor lookup
    #[arg(long)]
    pub(crate) binary: Option<PathBuf>,
}

#[derive(Args)]
pub(crate) struct AgentLaunchArgs {
    /// Stable adapter id: codex, claude-code, cursor-cli, or local-generic
    pub(crate) adapter: String,
    /// Datum project root used as the agent working directory
    #[arg(long = "project-root")]
    pub(crate) project_root: PathBuf,
    /// Protected datum_agent_discovery_v1 document
    #[arg(long)]
    pub(crate) discovery: Option<PathBuf>,
    /// Explicit executable for local-generic or to override vendor lookup
    #[arg(long)]
    pub(crate) binary: Option<PathBuf>,
    /// Resume the client's most recent native conversation
    #[arg(long, conflicts_with = "resume_id")]
    pub(crate) resume: bool,
    /// Resume one opaque client-native conversation identity
    #[arg(long = "resume-id", conflicts_with = "resume")]
    pub(crate) resume_id: Option<String>,
    /// Permit the temporary, byte-for-byte-restored .cursor/mcp.json overlay
    #[arg(long)]
    pub(crate) approve_project_config: bool,
    /// Arguments passed unchanged after Datum's native adapter arguments
    #[arg(last = true, allow_hyphen_values = true)]
    pub(crate) native_args: Vec<String>,
}
