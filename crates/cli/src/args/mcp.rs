use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub(crate) enum McpCommands {
    /// Serve the standard MCP protocol over stdin/stdout
    Serve(McpServeArgs),
}

#[derive(Args)]
pub(crate) struct McpServeArgs {
    /// Protected Datum agent discovery or terminal context document
    #[arg(long)]
    pub(crate) discovery: PathBuf,
}
