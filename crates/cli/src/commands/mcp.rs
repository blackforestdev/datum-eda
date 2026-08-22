use std::{
    env, fs,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};

use crate::{McpCommands, McpServeArgs, OutputFormat};

const MCP_SERVER_OVERRIDE: &str = "DATUM_MCP_SERVER_SCRIPT";

pub(crate) fn execute_mcp_command(
    _format: &OutputFormat,
    command: McpCommands,
) -> Result<(String, i32)> {
    match command {
        McpCommands::Serve(args) => serve(args),
    }
}

fn serve(args: McpServeArgs) -> Result<(String, i32)> {
    let discovery = fs::canonicalize(&args.discovery).with_context(|| {
        format!(
            "MCP discovery document is unavailable: {}",
            args.discovery.display()
        )
    })?;
    if !discovery.is_file() {
        bail!(
            "MCP discovery document is not a regular file: {}",
            discovery.display()
        );
    }
    let server = resolve_server_script()?;
    let error = Command::new("python3")
        .arg(&server)
        .arg("--discovery")
        .arg(&discovery)
        .env("DATUM_AGENT_DISCOVERY", &discovery)
        .exec();
    Err(error).with_context(|| format!("failed to execute MCP server {}", server.display()))
}

fn resolve_server_script() -> Result<PathBuf> {
    if let Some(override_path) = env::var_os(MCP_SERVER_OVERRIDE) {
        return validate_server_script(PathBuf::from(override_path));
    }
    let mut candidates = Vec::new();
    if let Ok(executable) = env::current_exe()
        && let Some(bin_dir) = executable.parent()
    {
        candidates.push(
            bin_dir
                .join("..")
                .join("lib")
                .join("datum-eda")
                .join("mcp-server")
                .join("server.py"),
        );
    }
    candidates.push(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("mcp-server")
            .join("server.py"),
    );
    candidates
        .into_iter()
        .find_map(|path| validate_server_script(path).ok())
        .context(
            "Datum MCP server.py was not found; install the Datum runtime files or set DATUM_MCP_SERVER_SCRIPT",
        )
}

fn validate_server_script(path: PathBuf) -> Result<PathBuf> {
    let canonical = fs::canonicalize(&path)
        .with_context(|| format!("MCP server script is unavailable: {}", path.display()))?;
    if !canonical.is_file() {
        bail!(
            "MCP server script is not a regular file: {}",
            canonical.display()
        );
    }
    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_server_script_is_resolvable() {
        let path = resolve_server_script().expect("workspace MCP server");
        assert_eq!(
            path.file_name().and_then(|name| name.to_str()),
            Some("server.py")
        );
        assert!(path.ends_with("mcp-server/server.py"));
    }
}
