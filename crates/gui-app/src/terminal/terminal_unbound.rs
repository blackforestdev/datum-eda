//! A real terminal with session identity and runtime storage, without Project authority.
use super::*;
use crate::terminal_context::unix_time_ms;
use crate::terminal_context_io::atomic_write_texts;
use crate::terminal_profile::{ResolvedTerminalLaunchProfile, TerminalLaunchProfile};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Debug)]
pub(crate) struct UnboundRuntime {
    directory: PathBuf,
}

impl UnboundRuntime {
    fn create() -> Result<Arc<Self>> {
        let base = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .filter(|p| {
                p.is_absolute()
                    && fs::metadata(p).is_ok_and(|m| {
                        m.is_dir() && m.uid() == unsafe { libc::geteuid() } && m.mode() & 0o077 == 0
                    })
            })
            .unwrap_or_else(std::env::temp_dir);
        let directory = base.join(format!(
            "datum-terminal-{}-{}",
            std::process::id(),
            eda_engine::preferences::new_product_id()
        ));
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&directory)
            .with_context(|| {
                format!(
                    "create owned unbound terminal runtime {}",
                    directory.display()
                )
            })?;
        Ok(Arc::new(Self { directory }))
    }
}

impl Drop for UnboundRuntime {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

/// Actual launch cwd, with an explicit filesystem fallback only if it is unavailable.
pub(crate) fn launch_directory() -> Result<PathBuf> {
    std::env::current_dir()
        .ok()
        .filter(|p| p.is_dir())
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .filter(|p| p.is_absolute() && p.is_dir())
        })
        .or_else(|| Path::new("/").is_dir().then(|| PathBuf::from("/")))
        .context("No usable terminal launch directory")
}

impl TerminalLaunchContext {
    pub(crate) fn for_spawn(&self) -> Result<Self> {
        let mut actual = self.clone();
        if actual.project_root.is_none() {
            if !actual.launch_working_directory.is_dir() {
                actual.launch_working_directory = launch_directory()?;
            }
            actual.launch_working_directory = actual
                .terminal_profile
                .resolve_unbound(&actual.launch_working_directory)?
                .cwd;
        }
        Ok(actual)
    }

    pub(crate) fn unbound(cwd: PathBuf, profile: TerminalLaunchProfile) -> Self {
        let cwd = profile
            .resolve_unbound(&cwd)
            .map(|resolved| resolved.cwd)
            .unwrap_or(cwd);
        Self {
            project_root: None,
            launch_working_directory: cwd,
            terminal_profile: profile,
            project_id: None,
            project_name: None,
            board_id: None,
            board_name: None,
            scene_id: None,
            source_revision: None,
            accepted_transaction_tip: None,
            production_status: Default::default(),
            source_shard_status: Default::default(),
            check_status: Default::default(),
            selection_context: datum_gui_protocol::DatumSelectionContext::from_selection(
                &datum_gui_protocol::SelectionTarget::None,
            ),
            cursor_context: datum_gui_protocol::DatumCursorContext {
                screen_px: None,
                hovered_object_id: None,
                active_dock_tab: None,
                active_tool: "select".into(),
            },
            projection_context: datum_gui_protocol::DatumProjectionContext {
                scene_id: String::new(),
                board_id: None,
                board_name: None,
                scene_bounds_nm: None,
                active_projection_id: None,
            },
            terminal_sessions: Default::default(),
        }
    }
}

fn context(context: &TerminalLaunchContext) -> Result<TerminalContext> {
    let runtime = UnboundRuntime::create()?;
    let session_id = format!("terminal-{}", eda_engine::preferences::new_product_id());
    let result = TerminalContext {
        context_path: runtime.directory.join("context.json"),
        latest_context_path: runtime.directory.join("latest.json"),
        session_path: runtime.directory.join("session.json"),
        unbound_runtime: Some(runtime),
        context_id: format!(
            "session-context-{}",
            eda_engine::preferences::new_product_id()
        ),
        session_id,
        project_id: None,
        model_revision: None,
        created_unix_ms: unix_time_ms()?,
        process_group_id: None,
    };
    refresh(&result, context)?;
    Ok(result)
}

pub(crate) fn refresh(session: &TerminalContext, context: &TerminalLaunchContext) -> Result<()> {
    anyhow::ensure!(
        context.project_root.is_none() && session.unbound_runtime.is_some(),
        "Unbound context requires owned session runtime"
    );
    let text = serde_json::to_string_pretty(&serde_json::json!({
        "schema": "datum_unbound_terminal_session_v1", "session_id": session.session_id,
        "terminal_session_id": session.session_id, "context_kind": "unbound",
        "working_directory": context.launch_working_directory,
        "terminal_launch_profile": context.terminal_profile.name(),
        "session_lifecycle": "running", "created_unix_ms": session.created_unix_ms,
        "updated_unix_ms": unix_time_ms()?, "process_group_id": session.process_group_id
    }))?;
    atomic_write_texts(&[
        (&session.context_path, &text),
        (&session.latest_context_path, &text),
        (&session.session_path, &text),
    ])
}

/// Resolve beside the running GUI (or its test binary), never a stale global CLI.
pub(crate) fn cli_companion() -> Result<PathBuf> {
    let executable = std::env::current_exe().context("locate running Datum executable")?;
    let mut directory = executable
        .parent()
        .context("Datum executable has no directory")?;
    if directory.file_name().is_some_and(|n| n == "deps") {
        directory = directory
            .parent()
            .context("test executable has no profile directory")?;
    }
    let binary = directory.join("datum-eda");
    anyhow::ensure!(
        fs::metadata(&binary).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0),
        "Matching Datum CLI companion is unavailable: {}. Build or install the GUI, CLI and daemon together.",
        binary.display()
    );
    Ok(binary)
}

fn authoritative_environment_key(key: &std::ffi::OsStr) -> bool {
    let key = key.to_string_lossy();
    key.starts_with("DATUM_PROJECT_")
        || key.starts_with("DATUM_MCP_")
        || matches!(
            key.as_ref(),
            "DATUM_AGENT_PROVENANCE"
                | "DATUM_AGENT_DISCOVERY"
                | "DATUM_AGENT_CREDENTIAL_FILE"
                | "DATUM_AGENT_LAUNCH_ID"
                | "DATUM_AGENT_ADAPTER_ID"
                | "DATUM_CONTEXT_ID"
                | "DATUM_SESSION_ID"
                | "DATUM_DISCOVERY"
                | "DATUM_LIVE_CONTEXT"
                | "DATUM_PINNED_CONTEXT"
                | "DATUM_TOOL_SESSION_EVENT_LOG"
                | "DATUM_MODEL_REVISION"
                | "DATUM_SOURCE_REVISION"
                | "DATUM_TERMINAL_CONTEXT"
                | "DATUM_TERMINAL_SESSION_ID"
                | "DATUM_TERMINAL_PROFILE"
                | "DATUM_ENGINE_SOCKET"
                | "DATUM_COMMIT_SOURCE"
                | "DATUM_CLI"
                | "DATUM_CLI_BIN"
                | "EDA_ENGINE_SOCKET"
                | "EDA_CLI_BIN"
        )
}

/// Sanitize AFTER profile overrides; keep ordinary shell and agent authentication.
pub(crate) fn environment(
    mut request: TerminalTransportRequest,
    profile: &ResolvedTerminalLaunchProfile,
) -> Result<TerminalTransportRequest> {
    for key in std::env::vars_os()
        .map(|(k, _)| k)
        .chain(profile.environment.iter().map(|(k, _)| k.clone()))
    {
        if authoritative_environment_key(&key) {
            request = request.env_remove(key);
        }
    }
    cli_environment(request, profile, cli_companion())
}

fn cli_environment(
    request: TerminalTransportRequest,
    profile: &ResolvedTerminalLaunchProfile,
    companion: Result<PathBuf>,
) -> Result<TerminalTransportRequest> {
    let Ok(cli) = companion else {
        return Ok(request);
    };
    let inherited = profile
        .environment
        .iter()
        .rev()
        .find(|(key, _)| key == "PATH")
        .map(|(_, value)| value.clone())
        .unwrap_or_else(|| std::env::var_os("PATH"));
    let directory = cli.parent().expect("companion directory");
    let paths = std::iter::once(directory.to_path_buf()).chain(
        inherited
            .iter()
            .flat_map(|value| std::env::split_paths(value))
            .filter(|p| p != directory),
    );
    let path = std::env::join_paths(paths).context("construct child CLI PATH")?;
    Ok(request
        .env("PATH", path)
        .env("DATUM_CLI", DATUM_CLI)
        .env("DATUM_CLI_BIN", cli))
}

pub(super) fn spawn(
    context: &TerminalLaunchContext,
    wake: TerminalWakeGate,
    rejected: &mut Option<TerminalSession>,
) -> Result<TerminalSession> {
    let profile = context
        .terminal_profile
        .resolve_unbound(&context.launch_working_directory)?;
    let mut actual = context.clone();
    actual.launch_working_directory = profile.cwd.clone();
    let mut session = self::context(&actual)?;
    let terminfo = install_session_terminfo(&session.session_path)?;
    let mut request = TerminalTransportRequest::new(&profile.executable, profile.cwd.clone())
        .args(profile.args.clone());
    for (key, value) in &profile.environment {
        request = match value {
            Some(value) => request.env(key, value),
            None => request.env_remove(key),
        };
    }
    let request = apply_datum_terminal_identity(environment(request, &profile)?, &terminfo)
        .env("DATUM_SESSION_ID", &session.session_id)
        .env("DATUM_TERMINAL_SESSION_ID", &session.session_id)
        .env("DATUM_TERMINAL_PROFILE", &profile.name);
    let prepared = prepare_terminal_transport(request)?;
    session.process_group_id = Some(prepared.process_group_id() as i32);
    if let Err(error) = refresh(&session, &actual) {
        *rejected = Some(TerminalSession::from_transport(
            prepared.start(wake),
            session,
            context.terminal_profile.clone(),
            profile.cwd.clone(),
        ));
        return Err(error);
    }
    Ok(TerminalSession::from_transport(
        prepared.start(wake),
        session,
        context.terminal_profile.clone(),
        profile.cwd,
    ))
}

#[cfg(test)]
#[path = "terminal_unbound_tests.rs"]
mod tests;
