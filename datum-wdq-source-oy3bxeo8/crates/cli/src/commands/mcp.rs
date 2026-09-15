use std::{
    env, fs,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};

use crate::{McpCommands, McpServeArgs, McpServeHttpArgs, OutputFormat};

const MCP_SERVER_OVERRIDE: &str = "DATUM_MCP_SERVER_SCRIPT";

pub(crate) fn execute_mcp_command(
    _format: &OutputFormat,
    command: McpCommands,
) -> Result<(String, i32)> {
    match command {
        McpCommands::Serve(args) => serve(args),
        McpCommands::ServeHttp(args) => serve_http(args),
    }
}

fn serve(args: McpServeArgs) -> Result<(String, i32)> {
    let discovery = canonical_regular_file(&args.discovery, "MCP discovery document")?;
    exec_server(["--discovery".into(), discovery.into_os_string()])
}

fn serve_http(args: McpServeHttpArgs) -> Result<(String, i32)> {
    let discovery = canonical_regular_file(&args.discovery, "MCP discovery document")?;
    let token_file = canonical_regular_file(&args.token_file, "MCP token file")?;
    let mut server_args = vec![
        "--discovery".into(),
        discovery.into_os_string(),
        "--transport".into(),
        "http".into(),
        "--port".into(),
        args.port.to_string().into(),
        "--token-file".into(),
        token_file.into_os_string(),
    ];
    for origin in args.allowed_origins {
        server_args.push("--allow-origin".into());
        server_args.push(origin.into());
    }
    exec_server(server_args)
}

fn exec_server<I>(args: I) -> Result<(String, i32)>
where
    I: IntoIterator<Item = std::ffi::OsString>,
{
    let server = resolve_server_script()?;
    let error = Command::new("python3").arg(&server).args(args).exec();
    Err(error).with_context(|| format!("failed to execute MCP server {}", server.display()))
}

fn canonical_regular_file(path: &Path, label: &str) -> Result<PathBuf> {
    let canonical = fs::canonicalize(path)
        .with_context(|| format!("{label} is unavailable: {}", path.display()))?;
    if !canonical.is_file() {
        bail!("{label} is not a regular file: {}", canonical.display());
    }
    Ok(canonical)
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
    use clap::Parser;

    use super::*;
    use crate::Cli;

    #[test]
    fn workspace_server_script_is_resolvable() {
        let path = resolve_server_script().expect("workspace MCP server");
        assert_eq!(
            path.file_name().and_then(|name| name.to_str()),
            Some("server.py")
        );
        assert!(path.ends_with("mcp-server/server.py"));
    }

    #[test]
    fn cli_parses_explicit_loopback_http_authority() {
        let cli = Cli::try_parse_from([
            "datum-eda",
            "mcp",
            "serve-http",
            "--discovery",
            "/tmp/discovery.json",
            "--port",
            "8123",
            "--token-file",
            "/tmp/token",
            "--allow-origin",
            "http://127.0.0.1:3000",
        ])
        .expect("parse serve-http");
        let crate::Commands::Mcp {
            action: McpCommands::ServeHttp(args),
        } = cli.command
        else {
            panic!("expected mcp serve-http")
        };
        assert_eq!(args.port, 8123);
        assert_eq!(args.allowed_origins, ["http://127.0.0.1:3000"]);
    }
}
