use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub(crate) enum McpCommands {
    /// Serve the standard MCP protocol over stdin/stdout
    Serve(McpServeArgs),
    /// Serve optional authenticated Streamable HTTP on loopback
    #[command(name = "serve-http")]
    ServeHttp(McpServeHttpArgs),
}

#[derive(Args)]
pub(crate) struct McpServeHttpArgs {
    /// Protected Datum agent discovery or terminal context document
    #[arg(long)]
    pub(crate) discovery: PathBuf,
    /// Loopback TCP port (zero requests an ephemeral port)
    #[arg(long)]
    pub(crate) port: u16,
    /// Protected file containing the bearer credential
    #[arg(long = "token-file")]
    pub(crate) token_file: PathBuf,
    /// Exact browser Origin allowed to call the endpoint; repeatable
    #[arg(long = "allow-origin", required = true)]
    pub(crate) allowed_origins: Vec<String>,
}

#[derive(Args)]
pub(crate) struct McpServeArgs {
    /// Protected Datum agent discovery or terminal context document
    #[arg(long)]
    pub(crate) discovery: PathBuf,
}
