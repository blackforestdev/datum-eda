use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::{
    AgentCommands, AgentDoctorArgs, AgentLaunchArgs, OutputFormat,
    agent_adapters::{
        AGENT_ADAPTERS, AgentAdapter, EphemeralConfigStrategy, ResumeArguments, agent_adapter,
    },
    render_output,
};

#[derive(Debug, Serialize)]
struct AgentListReport {
    schema: &'static str,
    adapters: &'static [AgentAdapter],
}

#[derive(Debug, Serialize)]
struct AgentDoctorReport {
    schema: &'static str,
    adapter_id: &'static str,
    executable: Option<PathBuf>,
    version: Option<String>,
    available: bool,
    launch_ready: bool,
    diagnostics: Vec<String>,
}

#[derive(Debug, Serialize)]
struct AgentLaunchReport {
    schema: &'static str,
    adapter_id: &'static str,
    executable: PathBuf,
    cwd: PathBuf,
    discovery: PathBuf,
    launch_id: String,
    exit_code: Option<i32>,
    cleanup_complete: bool,
}

pub(crate) fn execute_agent_command(
    format: &OutputFormat,
    command: AgentCommands,
) -> Result<(String, i32)> {
    match command {
        AgentCommands::List => {
            let report = AgentListReport {
                schema: "datum_agent_adapter_list_v1",
                adapters: AGENT_ADAPTERS,
            };
            Ok((render_agent_list(format, &report), 0))
        }
        AgentCommands::Doctor(args) => {
            let report = doctor_agent(&args)?;
            let exit_code = i32::from(!report.launch_ready);
            Ok((render_agent_doctor(format, &report), exit_code))
        }
        AgentCommands::Launch(args) => launch_agent(format, args),
    }
}

fn adapter_for(id: &str) -> Result<&'static AgentAdapter> {
    agent_adapter(id).with_context(|| {
        format!(
            "unknown agent adapter {id:?}; expected codex, claude-code, cursor-cli, or local-generic"
        )
    })
}

fn doctor_agent(args: &AgentDoctorArgs) -> Result<AgentDoctorReport> {
    let adapter = adapter_for(&args.adapter)?;
    let executable = resolve_executable(adapter, args.binary.as_deref());
    let mut diagnostics = Vec::new();
    let version = match executable.as_deref() {
        Some(path) if !adapter.launch.version_probe.args.is_empty() => {
            match Command::new(path)
                .args(adapter.launch.version_probe.args)
                .stdin(Stdio::null())
                .output()
            {
                Ok(output) if output.status.success() => Some(
                    String::from_utf8_lossy(if output.stdout.is_empty() {
                        &output.stderr
                    } else {
                        &output.stdout
                    })
                    .trim()
                    .to_string(),
                ),
                Ok(output) => {
                    diagnostics.push(format!("version probe exited with {}", output.status));
                    None
                }
                Err(error) => {
                    diagnostics.push(format!("version probe failed: {error}"));
                    None
                }
            }
        }
        Some(_) => None,
        None => {
            diagnostics.push("client executable was not found".to_string());
            None
        }
    };
    let available = executable.is_some();
    let launch_ready =
        available && (adapter.launch.version_probe.args.is_empty() || version.is_some());
    Ok(AgentDoctorReport {
        schema: "datum_agent_doctor_v1",
        adapter_id: adapter.id,
        executable,
        version,
        available,
        launch_ready,
        diagnostics,
    })
}

fn resolve_executable(adapter: &AgentAdapter, explicit: Option<&Path>) -> Option<PathBuf> {
    if let Some(path) = explicit {
        return is_executable_file(path).then(|| path.to_path_buf());
    }
    let path = env::var_os("PATH")?;
    for candidate in adapter.launch.binary_candidates {
        for directory in env::split_paths(&path) {
            let path = directory.join(candidate);
            if is_executable_file(&path) {
                return Some(path);
            }
        }
    }
    None
}

fn is_executable_file(path: &Path) -> bool {
    fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

fn launch_agent(format: &OutputFormat, args: AgentLaunchArgs) -> Result<(String, i32)> {
    let adapter = adapter_for(&args.adapter)?;
    let discovery = args
        .discovery
        .clone()
        .or_else(|| env::var_os("DATUM_AGENT_DISCOVERY").map(PathBuf::from))
        .context("agent launch requires --discovery or DATUM_AGENT_DISCOVERY")?;
    if !args.project_root.is_dir() {
        bail!(
            "agent project root is not a directory: {}",
            args.project_root.display()
        );
    }
    if !discovery.is_file() {
        bail!(
            "agent discovery document does not exist: {}",
            discovery.display()
        );
    }
    let executable = resolve_executable(adapter, args.binary.as_deref())
        .with_context(|| format!("agent executable for {} was not found", adapter.id))?;
    let launch_id = Uuid::new_v4().to_string();
    let runtime_root = args
        .project_root
        .join(".datum/runtime")
        .join(format!("agent-launch-{launch_id}"));
    let mut runtime = AgentRuntime::create(runtime_root)?;
    let mut command = Command::new(&executable);
    command
        .current_dir(&args.project_root)
        .env("DATUM_AGENT_DISCOVERY", &discovery)
        .env("DATUM_PROJECT_ROOT", &args.project_root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    apply_resume(adapter, &args, &mut command)?;
    configure_mcp(adapter, &args, &discovery, &mut runtime, &mut command)?;
    command.args(&args.native_args);

    eprintln!(
        "launching {} as {} in {}; MCP server datum; capabilities inspect+propose; config lifetime child process",
        adapter.id,
        executable.display(),
        args.project_root.display()
    );
    let status = command.status().with_context(|| {
        format!(
            "failed to launch {} via {}",
            adapter.id,
            executable.display()
        )
    })?;
    runtime.cleanup()?;
    let report = AgentLaunchReport {
        schema: "datum_agent_launch_v1",
        adapter_id: adapter.id,
        executable,
        cwd: args.project_root,
        discovery,
        launch_id,
        exit_code: status.code(),
        cleanup_complete: true,
    };
    Ok((
        render_agent_launch(format, &report),
        status.code().unwrap_or(1),
    ))
}

fn apply_resume(
    adapter: &AgentAdapter,
    args: &AgentLaunchArgs,
    command: &mut Command,
) -> Result<()> {
    let resume = if let Some(identity) = &args.resume_id {
        match adapter.launch.resume_identity {
            ResumeArguments::IdentityPrefix(prefix) => {
                command.args(prefix).arg(identity);
                return Ok(());
            }
            ResumeArguments::Unsupported => bail!("{} does not support native resume", adapter.id),
            ResumeArguments::Latest(_) => unreachable!("identity resume must use a prefix"),
        }
    } else if args.resume {
        adapter.launch.resume_latest
    } else {
        command.args(adapter.launch.interactive_args);
        return Ok(());
    };
    match resume {
        ResumeArguments::Latest(native_args) => {
            command.args(native_args);
            Ok(())
        }
        ResumeArguments::Unsupported => bail!("{} does not support native resume", adapter.id),
        ResumeArguments::IdentityPrefix(_) => unreachable!("latest resume must be complete args"),
    }
}

fn configure_mcp(
    adapter: &AgentAdapter,
    args: &AgentLaunchArgs,
    discovery_path: &Path,
    runtime: &mut AgentRuntime,
    command: &mut Command,
) -> Result<()> {
    let discovery = discovery_path.to_string_lossy();
    let broker_args = ["mcp", "serve", "--discovery", discovery.as_ref()];
    match adapter.mcp.ephemeral {
        EphemeralConfigStrategy::CommandLineOverrides { flag } => {
            let command_value = serde_json::to_string("datum-eda")?;
            let args_value = serde_json::to_string(&broker_args)?;
            command
                .arg(flag)
                .arg(format!("mcp_servers.datum.command={command_value}"))
                .arg(flag)
                .arg(format!("mcp_servers.datum.args={args_value}"));
        }
        EphemeralConfigStrategy::CommandLineFile { flags } => {
            let config = runtime.write_json(
                "mcp.json",
                &json!({"mcpServers": {"datum": {"command": "datum-eda", "args": broker_args}}}),
            )?;
            for flag in flags {
                command.arg(flag);
                if *flag == "--mcp-config" {
                    command.arg(&config);
                }
            }
        }
        EphemeralConfigStrategy::ReviewedProjectOverlay { relative_path } => {
            if !args.approve_project_config {
                bail!(
                    "{} requires --approve-project-config to temporarily overlay and restore {}",
                    adapter.id,
                    relative_path
                );
            }
            runtime.install_project_overlay(
                args.project_root.join(relative_path),
                json!({"mcpServers": {"datum": {"command": "datum-eda", "args": broker_args}}}),
            )?;
        }
        EphemeralConfigStrategy::DiscoveryOnly => {
            eprintln!(
                "standard Datum MCP command: datum-eda mcp serve --discovery {}",
                discovery_path.display()
            );
        }
        EphemeralConfigStrategy::IsolatedConfigHome { .. } => {
            bail!("isolated config-home launch strategy is not enabled")
        }
    }
    Ok(())
}

struct ProjectOverlay {
    path: PathBuf,
    original: Option<Vec<u8>>,
    created_parent: Option<PathBuf>,
}

struct AgentRuntime {
    root: PathBuf,
    overlays: Vec<ProjectOverlay>,
    cleaned: bool,
}

impl AgentRuntime {
    fn create(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(&root)
            .with_context(|| format!("failed to create agent runtime {}", root.display()))?;
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
        Ok(Self {
            root,
            overlays: Vec::new(),
            cleaned: false,
        })
    }

    fn write_json(&self, name: &str, value: &Value) -> Result<PathBuf> {
        let path = self.root.join(name);
        fs::write(&path, serde_json::to_vec_pretty(value)?)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        Ok(path)
    }

    fn install_project_overlay(&mut self, path: PathBuf, datum: Value) -> Result<()> {
        let original = fs::read(&path).ok();
        let created_parent = path
            .parent()
            .filter(|parent| !parent.exists())
            .map(Path::to_path_buf);
        let mut root = match original.as_deref() {
            Some(bytes) => serde_json::from_slice::<Value>(bytes)
                .with_context(|| format!("failed to parse existing {}", path.display()))?,
            None => json!({}),
        };
        let servers = root
            .as_object_mut()
            .context("Cursor project MCP config root must be a JSON object")?
            .entry("mcpServers")
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .context("Cursor mcpServers must be a JSON object")?;
        let datum_server = datum
            .get("mcpServers")
            .and_then(|value| value.get("datum"))
            .cloned()
            .context("generated Datum MCP server is missing")?;
        if servers.contains_key("datum") {
            bail!("{} already defines mcpServers.datum", path.display());
        }
        servers.insert("datum".to_string(), datum_server);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, serde_json::to_vec_pretty(&root)?)?;
        self.overlays.push(ProjectOverlay {
            path,
            original,
            created_parent,
        });
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        while let Some(overlay) = self.overlays.pop() {
            match overlay.original {
                Some(bytes) => fs::write(&overlay.path, bytes)?,
                None if overlay.path.exists() => fs::remove_file(&overlay.path)?,
                None => {}
            }
            if let Some(parent) = overlay.created_parent {
                match fs::remove_dir(&parent) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                }
            }
        }
        if self.root.exists() {
            fs::remove_dir_all(&self.root)?;
        }
        self.cleaned = true;
        Ok(())
    }
}

impl Drop for AgentRuntime {
    fn drop(&mut self) {
        if !self.cleaned {
            let _ = self.cleanup();
        }
    }
}

fn render_agent_list(format: &OutputFormat, report: &AgentListReport) -> String {
    match format {
        OutputFormat::Json => render_output(format, report),
        OutputFormat::Text => report
            .adapters
            .iter()
            .map(|adapter| format!("{}\t{}", adapter.id, adapter.display_name))
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

fn render_agent_doctor(format: &OutputFormat, report: &AgentDoctorReport) -> String {
    match format {
        OutputFormat::Json => render_output(format, report),
        OutputFormat::Text => format!(
            "adapter: {}\navailable: {}\nlaunch_ready: {}\nexecutable: {}\nversion: {}\ndiagnostics: {}",
            report.adapter_id,
            report.available,
            report.launch_ready,
            report
                .executable
                .as_deref()
                .map_or_else(|| "-".to_string(), |path| path.display().to_string()),
            report.version.as_deref().unwrap_or("-"),
            if report.diagnostics.is_empty() {
                "-".to_string()
            } else {
                report.diagnostics.join("; ")
            }
        ),
    }
}

fn render_agent_launch(format: &OutputFormat, report: &AgentLaunchReport) -> String {
    match format {
        OutputFormat::Json => render_output(format, report),
        OutputFormat::Text => format!(
            "adapter: {}\nlaunch_id: {}\nexit_code: {}\ncleanup_complete: {}",
            report.adapter_id,
            report.launch_id,
            report
                .exit_code
                .map_or_else(|| "signal".to_string(), |code| code.to_string()),
            report.cleanup_complete
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use clap::Parser;

    use super::*;
    use crate::{Cli, Commands};

    fn temp_root(label: &str) -> PathBuf {
        let root = env::temp_dir().join(format!("datum-agent-{label}-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).expect("temp root");
        root
    }

    #[test]
    fn agent_cli_parses_list_doctor_and_native_launch_args() {
        for argv in [
            vec!["datum-eda", "agent", "list"],
            vec!["datum-eda", "agent", "doctor", "codex"],
            vec![
                "datum-eda",
                "agent",
                "launch",
                "claude-code",
                "--project-root",
                "/project",
                "--discovery",
                "/runtime/discovery.json",
                "--",
                "--model",
                "sonnet",
            ],
        ] {
            let cli = Cli::try_parse_from(argv).expect("agent CLI must parse");
            assert!(matches!(cli.command, Commands::Agent { .. }));
        }
    }

    #[test]
    fn launch_runtime_is_private_and_cleanup_restores_project_config() {
        let project = temp_root("overlay");
        let runtime_root = project.join(".datum/runtime/fixture");
        let config = project.join(".cursor/mcp.json");
        fs::create_dir_all(config.parent().expect("config parent")).expect("config parent");
        let original = br#"{"mcpServers":{"owner":{"command":"owner"}}}"#;
        fs::write(&config, original).expect("original config");
        let mut runtime = AgentRuntime::create(runtime_root.clone()).expect("runtime");
        assert_eq!(
            fs::metadata(&runtime_root)
                .expect("runtime metadata")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        runtime
            .install_project_overlay(
                config.clone(),
                json!({"mcpServers":{"datum":{"command":"datum-eda"}}}),
            )
            .expect("install overlay");
        let installed: Value =
            serde_json::from_slice(&fs::read(&config).expect("installed config"))
                .expect("valid installed config");
        assert_eq!(installed["mcpServers"]["owner"]["command"], "owner");
        assert_eq!(installed["mcpServers"]["datum"]["command"], "datum-eda");
        runtime.cleanup().expect("cleanup");
        assert_eq!(fs::read(&config).expect("restored config"), original);
        assert!(!runtime_root.exists());
        fs::remove_dir_all(project).expect("remove fixture");
    }

    #[test]
    fn doctor_uses_explicit_binary_without_global_configuration() {
        let report = doctor_agent(&AgentDoctorArgs {
            adapter: "local-generic".to_string(),
            binary: Some(PathBuf::from("/bin/sh")),
        })
        .expect("doctor");
        assert!(report.available);
        assert!(report.launch_ready);
        assert!(report.version.is_none());
    }

    #[test]
    fn generic_launch_preserves_cwd_and_discovery_then_removes_runtime() {
        let project = temp_root("generic-launch");
        let discovery = project.join("discovery.json");
        fs::write(&discovery, b"{}").expect("discovery");
        let assertion = format!(
            "test \"$PWD\" = '{}' && test \"$DATUM_AGENT_DISCOVERY\" = '{}'",
            project.display(),
            discovery.display()
        );
        let (report, exit_code) = launch_agent(
            &OutputFormat::Text,
            AgentLaunchArgs {
                adapter: "local-generic".to_string(),
                project_root: project.clone(),
                discovery: Some(discovery),
                binary: Some(PathBuf::from("/bin/sh")),
                resume: false,
                resume_id: None,
                approve_project_config: false,
                native_args: vec!["-c".to_string(), assertion],
            },
        )
        .expect("generic launch");
        assert_eq!(exit_code, 0);
        assert!(report.contains("cleanup_complete: true"));
        let runtime = project.join(".datum/runtime");
        assert!(
            !runtime.exists()
                || fs::read_dir(runtime)
                    .expect("runtime directory")
                    .next()
                    .is_none()
        );
        fs::remove_dir_all(project).expect("remove fixture");
    }

    #[test]
    fn cursor_overlay_requires_explicit_approval() {
        let project = temp_root("approval");
        let discovery = project.join("discovery.json");
        fs::write(&discovery, b"{}").expect("discovery");
        let adapter = agent_adapter("cursor-cli").expect("cursor adapter");
        let args = AgentLaunchArgs {
            adapter: adapter.id.to_string(),
            project_root: project.clone(),
            discovery: Some(discovery.clone()),
            binary: Some(PathBuf::from("/bin/true")),
            resume: false,
            resume_id: None,
            approve_project_config: false,
            native_args: Vec::new(),
        };
        let mut runtime =
            AgentRuntime::create(project.join(".datum/runtime/fixture")).expect("runtime");
        let mut command = Command::new("/bin/true");
        let error = configure_mcp(adapter, &args, &discovery, &mut runtime, &mut command)
            .expect_err("approval must be required");
        assert!(error.to_string().contains("--approve-project-config"));
        runtime.cleanup().expect("cleanup");
        fs::remove_dir_all(project).expect("remove fixture");
    }
}
