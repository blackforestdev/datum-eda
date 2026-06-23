use crate::{
    terminal_context::{
        DATUM_CLI, DATUM_LEGACY_CLI, TerminalContext, read_session_created_unix_ms,
        tool_session_event_log_path, unix_time_ms, update_terminal_lifecycle_file,
        write_terminal_context, write_terminal_context_files,
    },
    terminal_screen::TerminalScreen,
    terminal_session_events::{record_terminal_input_event, record_terminal_lifecycle_event},
};
use anyhow::{Context, Result};
use datum_gui_protocol::{
    CheckRunReviewState, DatumCursorContext, DatumProjectionContext, DatumSceneBoundsContext,
    DatumSelectionContext, DatumToolSessionLifecycle, DockTab, ProductionStatus,
    ReviewWorkspaceState, TerminalLaneState, WorkspaceTool,
};
use std::io::{Read, Write};
use std::os::fd::{FromRawFd, RawFd};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::{ffi::CStr, fs::File, io};

pub(super) enum TerminalEvent {
    Output(Vec<u8>),
    Exited(Option<i32>),
}

pub(super) struct TerminalSession {
    pub(super) stdin: Arc<Mutex<File>>,
    pub(super) rx: Receiver<TerminalEvent>,
    pub(super) context_path: PathBuf,
    latest_context_path: PathBuf,
    session_path: PathBuf,
    session_id: String,
    context_id: String,
    master_fd: RawFd,
    process_group_id: libc::pid_t,
    active_execution_id: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Clone)]
pub(super) struct TerminalLaunchContext {
    pub(super) project_root: PathBuf,
    pub(super) project_id: Option<String>,
    pub(super) project_name: Option<String>,
    pub(super) board_id: Option<String>,
    pub(super) board_name: Option<String>,
    pub(super) scene_id: Option<String>,
    pub(super) source_revision: Option<String>,
    pub(super) production_status: ProductionStatus,
    pub(super) check_status: CheckRunReviewState,
    pub(super) selection_context: DatumSelectionContext,
    pub(super) cursor_context: DatumCursorContext,
    pub(super) projection_context: DatumProjectionContext,
}

pub(super) fn terminal_launch_context_from_state(
    project_root: &Path,
    state: &ReviewWorkspaceState,
) -> TerminalLaunchContext {
    TerminalLaunchContext {
        project_root: project_root.to_path_buf(),
        project_id: Some(state.scene.project_uuid.clone()),
        project_name: Some(state.scene.project_name.clone()),
        board_id: Some(state.scene.board_uuid.clone()),
        board_name: Some(state.scene.board_name.clone()),
        scene_id: Some(state.scene.scene_id.clone()),
        source_revision: Some(state.scene.source_revision.clone()),
        production_status: state.production.clone(),
        check_status: state.checks.clone(),
        selection_context: DatumSelectionContext::from_selection(&state.selection),
        cursor_context: DatumCursorContext {
            screen_px: None,
            hovered_object_id: state.ui.hovered_object_id.clone(),
            active_dock_tab: state
                .ui
                .active_dock_tab
                .map(dock_tab_name)
                .map(str::to_string),
            active_tool: workspace_tool_name(state.tool).to_string(),
        },
        projection_context: DatumProjectionContext {
            scene_id: state.scene.scene_id.clone(),
            board_id: Some(state.scene.board_uuid.clone()),
            board_name: Some(state.scene.board_name.clone()),
            scene_bounds_nm: Some(DatumSceneBoundsContext::from_bounds(&state.scene.bounds)),
            active_projection_id: None,
        },
    }
}

pub(super) fn terminal_launch_context_from_state_with_cursor(
    project_root: &Path,
    state: &ReviewWorkspaceState,
    cursor: Option<(f32, f32)>,
) -> TerminalLaunchContext {
    let mut context = terminal_launch_context_from_state(project_root, state);
    context.cursor_context.screen_px = cursor.map(|(x, y)| [x.round() as i32, y.round() as i32]);
    context
}

pub(super) fn spawn_terminal_session(context: &TerminalLaunchContext) -> Result<TerminalSession> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let mut terminal_context = write_terminal_context(context)?;
    let pty = open_pty_pair().context("open terminal PTY")?;
    let reader = pty
        .master
        .try_clone()
        .context("clone terminal PTY master for reader")?;
    let stdin = Arc::new(Mutex::new(pty.master));
    let slave_path = pty.slave_path;
    let master_fd = pty.master_fd;
    let mut command = Command::new(&shell);
    command
        .current_dir(&terminal_context.project_root)
        .env("TERM", "xterm-256color")
        .env("DATUM_PROJECT_ROOT", &context.project_root)
        .env("DATUM_CLI", DATUM_CLI)
        .env("DATUM_LEGACY_CLI", DATUM_LEGACY_CLI)
        .env("DATUM_CONTEXT_ID", &terminal_context.context_id)
        .env("DATUM_SESSION_ID", &terminal_context.session_id)
        .env("DATUM_DISCOVERY", &terminal_context.context_path)
        .env(
            "DATUM_TOOL_SESSION_EVENT_LOG",
            tool_session_event_log_path(&terminal_context.session_path),
        )
        .env(
            "DATUM_MODEL_REVISION",
            terminal_context.model_revision.as_deref().unwrap_or(""),
        )
        .env("DATUM_TERMINAL_CONTEXT", &terminal_context.context_path)
        .env("DATUM_TERMINAL_SESSION_ID", &terminal_context.session_id);
    if let Some(project_id) = &terminal_context.project_id {
        command.env("DATUM_PROJECT_ID", project_id);
    }
    if let Some(model_revision) = &terminal_context.model_revision {
        command.env("DATUM_SOURCE_REVISION", model_revision);
    }
    unsafe {
        command.pre_exec(move || configure_child_pty(&slave_path, master_fd));
    }
    let mut child = command.spawn().with_context(|| {
        format!(
            "spawn PTY terminal shell {shell} in {}",
            terminal_context.project_root.display()
        )
    })?;
    let process_group_id = child.id() as libc::pid_t;
    terminal_context.process_group_id = Some(process_group_id as i32);
    write_terminal_context_files(&terminal_context, context)?;
    let (tx, rx) = mpsc::channel();
    let reader_tx = tx.clone();
    thread::spawn(move || {
        let mut reader = reader;
        let mut buffer = [0_u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    let _ = reader_tx.send(TerminalEvent::Output(buffer[..count].to_vec()));
                }
                Err(_) => break,
            }
        }
    });
    thread::spawn(move || {
        let code = child.wait().ok().and_then(|status| status.code());
        let _ = tx.send(TerminalEvent::Exited(code));
    });
    Ok(TerminalSession {
        stdin,
        rx,
        context_path: terminal_context.context_path,
        latest_context_path: terminal_context.latest_context_path,
        session_path: terminal_context.session_path,
        session_id: terminal_context.session_id,
        context_id: terminal_context.context_id,
        master_fd,
        process_group_id,
        active_execution_id: Arc::new(Mutex::new(None)),
    })
}

pub(super) fn refresh_terminal_session_context(
    session: &TerminalSession,
    context: &TerminalLaunchContext,
) -> Result<()> {
    let terminal_context = TerminalContext {
        project_root: context.project_root.clone(),
        context_path: session.context_path.clone(),
        latest_context_path: session.latest_context_path.clone(),
        session_path: session.session_path.clone(),
        context_id: session.context_id.clone(),
        session_id: session.session_id.clone(),
        project_id: context.project_id.clone(),
        model_revision: context.source_revision.clone(),
        created_unix_ms: read_session_created_unix_ms(&session.session_path)
            .unwrap_or_else(|| unix_time_ms().unwrap_or(0)),
        process_group_id: Some(session.process_group_id as i32),
    };
    write_terminal_context_files(&terminal_context, context)
}

pub(super) fn refresh_terminal_session_context_from_state(
    session: &TerminalSession,
    base_context: &TerminalLaunchContext,
    state: &ReviewWorkspaceState,
    cursor: Option<(f32, f32)>,
) -> Result<TerminalLaunchContext> {
    let context =
        terminal_launch_context_from_state_with_cursor(&base_context.project_root, state, cursor);
    refresh_terminal_session_context(session, &context)?;
    Ok(context)
}

pub(super) fn terminate_terminal_session(
    session: &TerminalSession,
    state: &mut TerminalLaneState,
) -> Result<()> {
    mark_terminal_session_lifecycle(session, DatumToolSessionLifecycle::Terminating, None)?;
    record_terminal_lifecycle_event(session, DatumToolSessionLifecycle::Terminating, None)?;
    session.terminate()?;
    state.status = "terminating".to_string();
    Ok(())
}

pub(super) fn restart_terminal_session(
    session: &mut TerminalSession,
    screen: &mut TerminalScreen,
    state: &mut TerminalLaneState,
    context: &TerminalLaunchContext,
) -> Result<()> {
    mark_terminal_session_lifecycle(session, DatumToolSessionLifecycle::Restarted, None)?;
    record_terminal_lifecycle_event(session, DatumToolSessionLifecycle::Restarted, None)?;
    *session = spawn_terminal_session(context)?;
    *screen = TerminalScreen::default();
    state.status = "running".to_string();
    state.lines.push(format!(
        "terminal restarted; context {}",
        session.context_path.display()
    ));
    state.scroll_offset = 0;
    Ok(())
}

pub(super) fn mark_terminal_session_lifecycle(
    session: &TerminalSession,
    lifecycle: DatumToolSessionLifecycle,
    process_exit_code: Option<i32>,
) -> Result<()> {
    update_terminal_lifecycle_file(
        &session.context_path,
        lifecycle,
        process_exit_code,
        Some(session.process_group_id as i32),
    )?;
    update_terminal_lifecycle_file(
        &session.latest_context_path,
        lifecycle,
        process_exit_code,
        Some(session.process_group_id as i32),
    )?;
    update_terminal_lifecycle_file(
        &session.session_path,
        lifecycle,
        process_exit_code,
        Some(session.process_group_id as i32),
    )
}

impl TerminalSession {
    pub(super) fn write_bytes(&self, bytes: &[u8]) -> Result<()> {
        let mut stdin = self
            .stdin
            .lock()
            .map_err(|_| anyhow::anyhow!("lock terminal PTY master"))?;
        stdin.write_all(bytes).context("write terminal PTY input")?;
        stdin.flush().context("flush terminal PTY input")?;
        let _ = record_terminal_input_event(self, bytes);
        Ok(())
    }
    pub(super) fn session_id(&self) -> &str {
        &self.session_id
    }
    pub(super) fn event_log_path(&self) -> PathBuf {
        tool_session_event_log_path(&self.session_path)
    }
    pub(super) fn set_active_execution_id(&self, execution_id: String) {
        if let Ok(mut active) = self.active_execution_id.lock() {
            *active = Some(execution_id);
        }
    }
    pub(super) fn active_execution_id(&self) -> Option<String> {
        self.active_execution_id
            .lock()
            .ok()
            .and_then(|active| active.clone())
    }
    pub(super) fn clear_active_execution_id(&self, execution_id: &str) {
        if let Ok(mut active) = self.active_execution_id.lock()
            && active.as_deref() == Some(execution_id)
        {
            *active = None;
        }
    }

    pub(super) fn interrupt(&self) -> Result<()> {
        self.signal_process_group(libc::SIGINT, "interrupt terminal process group")
    }

    pub(super) fn terminate(&self) -> Result<()> {
        self.signal_process_group(libc::SIGTERM, "terminate terminal process group")
    }

    fn signal_process_group(&self, signal: libc::c_int, context: &str) -> Result<()> {
        let rc = unsafe { libc::kill(-self.process_group_id, signal) };
        if rc < 0 {
            return Err(io::Error::last_os_error()).context(context.to_string());
        }
        Ok(())
    }

    pub(super) fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        let size = libc::winsize {
            ws_row: rows.max(1),
            ws_col: cols.max(1),
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let rc = unsafe { libc::ioctl(self.master_fd, libc::TIOCSWINSZ, &size) };
        if rc < 0 {
            return Err(io::Error::last_os_error()).context("resize terminal PTY");
        }
        Ok(())
    }
}

fn workspace_tool_name(tool: WorkspaceTool) -> &'static str {
    match tool {
        WorkspaceTool::Select => "select",
    }
}

fn dock_tab_name(tab: DockTab) -> &'static str {
    match tab {
        DockTab::Terminal => "terminal",
        DockTab::Assistant => "assistant",
        DockTab::Outputs => "outputs",
        DockTab::Supervision => "supervision",
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}

struct PtyPair {
    master: File,
    master_fd: RawFd,
    slave_path: Vec<u8>,
}

fn open_pty_pair() -> Result<PtyPair> {
    let master_fd = unsafe { libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY) };
    if master_fd < 0 {
        return Err(io::Error::last_os_error()).context("posix_openpt");
    }
    if unsafe { libc::grantpt(master_fd) } != 0 {
        let error = io::Error::last_os_error();
        unsafe {
            libc::close(master_fd);
        }
        return Err(error).context("grantpt");
    }
    if unsafe { libc::unlockpt(master_fd) } != 0 {
        let error = io::Error::last_os_error();
        unsafe {
            libc::close(master_fd);
        }
        return Err(error).context("unlockpt");
    }
    let slave_path = slave_path(master_fd)?;
    let master = unsafe { File::from_raw_fd(master_fd) };
    Ok(PtyPair {
        master,
        master_fd,
        slave_path,
    })
}

fn slave_path(master_fd: RawFd) -> Result<Vec<u8>> {
    let mut buffer = [0 as libc::c_char; 128];
    let rc = unsafe { libc::ptsname_r(master_fd, buffer.as_mut_ptr(), buffer.len()) };
    if rc != 0 {
        return Err(io::Error::from_raw_os_error(rc)).context("ptsname_r");
    }
    let path = unsafe { CStr::from_ptr(buffer.as_ptr()) };
    Ok(path.to_bytes_with_nul().to_vec())
}

fn configure_child_pty(slave_path: &[u8], master_fd: RawFd) -> io::Result<()> {
    if unsafe { libc::setsid() } < 0 {
        return Err(io::Error::last_os_error());
    }
    let slave_fd = unsafe { libc::open(slave_path.as_ptr().cast(), libc::O_RDWR) };
    if slave_fd < 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::ioctl(slave_fd, libc::TIOCSCTTY, 0) } < 0 {
        let error = io::Error::last_os_error();
        unsafe {
            libc::close(slave_fd);
        }
        return Err(error);
    }
    for fd in [libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO] {
        if unsafe { libc::dup2(slave_fd, fd) } < 0 {
            let error = io::Error::last_os_error();
            unsafe {
                libc::close(slave_fd);
            }
            return Err(error);
        }
    }
    if slave_fd > libc::STDERR_FILENO {
        unsafe {
            libc::close(slave_fd);
        }
    }
    unsafe {
        libc::close(master_fd);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use datum_gui_protocol::TERMINAL_COMMAND_CATALOG_VERSION;
    use std::fs;
    use std::time::Duration;

    impl TerminalLaunchContext {
        fn for_project_root(project_root: &std::path::Path) -> Self {
            Self {
                project_root: project_root.to_path_buf(),
                project_id: None,
                project_name: None,
                board_id: None,
                board_name: None,
                scene_id: None,
                source_revision: None,
                production_status: ProductionStatus::default(),
                check_status: CheckRunReviewState::default(),
                selection_context: DatumSelectionContext {
                    kind: "none".to_string(),
                    id: None,
                },
                cursor_context: DatumCursorContext {
                    screen_px: None,
                    hovered_object_id: None,
                    active_dock_tab: None,
                    active_tool: "select".to_string(),
                },
                projection_context: DatumProjectionContext {
                    scene_id: "test-scene".to_string(),
                    board_id: None,
                    board_name: None,
                    scene_bounds_nm: None,
                    active_projection_id: None,
                },
            }
        }
    }

    #[test]
    fn terminal_session_spawns_real_pty_shell() {
        let root = std::env::temp_dir().join(format!("datum-terminal-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("terminal test root should create");
        let mut context = TerminalLaunchContext::for_project_root(&root);
        context.project_id = Some("project-test".to_string());
        context.source_revision = Some("source-rev-test".to_string());
        context.check_status = CheckRunReviewState {
            check_run_id: Some("check-run-visible".to_string()),
            finding_count: 1,
            findings: vec![datum_gui_protocol::CheckFindingSummary {
                fingerprint: "sha256:visible-finding".to_string(),
                rule_id: "zone_fill_state".to_string(),
                ..datum_gui_protocol::CheckFindingSummary::default()
            }],
            ..CheckRunReviewState::default()
        };
        let session = spawn_terminal_session(&context).expect("spawn PTY terminal session");
        assert!(session.context_path.exists());
        assert!(
            session
                .context_path
                .to_string_lossy()
                .contains(".datum/terminal-contexts/terminal-")
        );
        let latest_context_path = root.join(".datum/gui-terminal-context.json");
        assert!(latest_context_path.exists());
        let session_context: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&session.context_path).unwrap()).unwrap();
        let latest_context: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&latest_context_path).unwrap()).unwrap();
        assert_eq!(session_context["session_id"], latest_context["session_id"]);
        assert_eq!(
            session_context["storage"]["legacy_context_path"],
            latest_context_path.display().to_string()
        );
        assert_eq!(
            session_context["storage"]["latest_context_path"],
            latest_context_path.display().to_string()
        );
        assert_eq!(
            session_context["storage"]["compatibility_context_path"],
            latest_context_path.display().to_string()
        );
        assert_eq!(
            session_context["storage"]["event_log_path"],
            session.event_log_path().display().to_string()
        );
        assert!(session.session_path.exists());
        let session_metadata: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&session.session_path).unwrap()).unwrap();
        assert_eq!(
            session_metadata["session_id"],
            session_context["session_id"]
        );
        for key in [
            "create_gerber_output_job",
            "update_output_job",
            "delete_output_job",
            "create_manufacturing_plan",
            "update_manufacturing_plan",
            "delete_manufacturing_plan",
            "create_panel_projection",
            "update_panel_projection",
            "delete_panel_projection",
        ] {
            let command = session_context["production_commands"][key]
                .as_str()
                .expect("production command template should be a string");
            assert!(
                command.starts_with("datum-eda proposal "),
                "production command template {key} should use canonical proposal CLI: {command}"
            );
            assert!(
                !command.contains("--as-proposal"),
                "canonical proposal template {key} should not need --as-proposal: {command}"
            );
        }
        assert_eq!(
            session_context["command_catalog_version"],
            TERMINAL_COMMAND_CATALOG_VERSION
        );
        assert_eq!(
            session_context["handoff_commands"]["datum.artifact.generate"]["mcp_alias"],
            "datum.artifact.generate"
        );
        assert_eq!(
            session_context["handoff_commands"]["datum.proposal.accept_apply"]["cli_argv_template"],
            serde_json::json!([
                "datum-eda",
                "proposal",
                "accept-apply",
                "{project_root}",
                "--proposal",
                "{proposal}"
            ])
        );
        assert_eq!(
            session_context["proposal_commands"]["preview"],
            "datum-eda proposal preview \"$DATUM_PROJECT_ROOT\" --proposal <uuid>"
        );
        assert_eq!(
            session_context["visible_check_run_ids"],
            serde_json::json!(["check-run-visible"])
        );
        assert_eq!(
            session_context["visible_finding_fingerprints"],
            serde_json::json!(["sha256:visible-finding"])
        );
        assert_eq!(
            session_context["check_status"]["findings"][0]["rule_id"],
            "zone_fill_state"
        );

        context.selection_context = DatumSelectionContext {
            kind: "authored_object".to_string(),
            id: Some("object-live".to_string()),
        };
        context.cursor_context.screen_px = Some([42, 84]);
        context.cursor_context.hovered_object_id = Some("hover-live".to_string());
        refresh_terminal_session_context(&session, &context)
            .expect("refresh existing terminal context");
        let refreshed: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&session.context_path).unwrap()).unwrap();
        assert_eq!(refreshed["session_id"], session_context["session_id"]);
        assert_eq!(refreshed["selection_context"]["id"], "object-live");
        assert_eq!(
            refreshed["cursor_context"]["screen_px"],
            serde_json::json!([42, 84])
        );
        assert_eq!(
            refreshed["session"]["session_id"],
            session_context["session_id"]
        );
        assert_eq!(refreshed["contract"], "datum_terminal_context_v1");
        assert_eq!(refreshed["datum_cli"], "datum-eda");
        assert_eq!(refreshed["actor_type"], "ExternalAgent");
        assert_eq!(refreshed["selection_context"]["kind"], "authored_object");
        assert_eq!(
            refreshed["agent_commands"]["refresh_context"],
            "datum-eda context refresh --session \"$DATUM_SESSION_ID\""
        );
        assert!(
            refreshed["agent_commands"]["codex_with_context"]
                .as_str()
                .unwrap()
                .contains("$DATUM_DISCOVERY")
        );
        assert!(
            refreshed["agent_commands"]["context_prompt"]
                .as_str()
                .unwrap()
                .contains("context session-activity")
        );
        assert_eq!(
            refreshed["check_commands"]["run_current"],
            "datum-eda check run \"$DATUM_PROJECT_ROOT\""
        );
        assert_eq!(
            refreshed["proposal_commands"]["accept_apply"],
            "datum-eda proposal accept-apply \"$DATUM_PROJECT_ROOT\" --proposal <uuid>"
        );
        assert_eq!(
            refreshed["query_commands"]["import_map"],
            "datum-eda query import-map \"$DATUM_PROJECT_ROOT\""
        );
        assert_eq!(
            refreshed["query_commands"]["zone_fills"],
            "datum-eda query zone-fills \"$DATUM_PROJECT_ROOT\""
        );
        session.resize(100, 24).expect("resize PTY");
        session
            .write_bytes(
                b"printf 'datum-pty-ok:%s:%s:%s\\n' \"$DATUM_PROJECT_ROOT\" \"$DATUM_CLI\" \"$DATUM_SESSION_ID\"\nexit\n",
            )
            .expect("write command to PTY");
        let mut output = String::new();
        for _ in 0..80 {
            if let Ok(event) = session.rx.recv_timeout(Duration::from_millis(100)) {
                match event {
                    TerminalEvent::Output(bytes) => {
                        let _ = crate::terminal_session_events::record_terminal_output_event(
                            &session, &bytes,
                        );
                        output.push_str(&String::from_utf8_lossy(&bytes));
                        if output.contains("datum-pty-ok:") && output.contains("datum-eda") {
                            break;
                        }
                    }
                    TerminalEvent::Exited(code) => {
                        assert_eq!(code, Some(0));
                    }
                }
            }
        }
        for expected in ["datum-pty-ok:", "datum-eda"] {
            assert!(
                output.contains(expected),
                "missing {expected:?} in PTY output: {output}"
            );
        }
        let event_log =
            fs::read_to_string(session.event_log_path()).expect("read terminal event log");
        assert!(
            event_log.contains(r#""event":"terminal_io""#),
            "terminal event log should record I/O events: {event_log}"
        );
        assert!(
            event_log.contains(r#""direction":"input""#),
            "terminal event log should record PTY input: {event_log}"
        );
        assert!(
            event_log.contains(r#""direction":"output""#),
            "terminal event log should record PTY output: {event_log}"
        );
        assert!(
            event_log.contains(session.session_id()),
            "terminal event log should tie events to the session id: {event_log}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn terminal_session_terminate_reports_signal_exit() {
        let root =
            std::env::temp_dir().join(format!("datum-terminal-terminate-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("terminal terminate test root should create");
        let context = TerminalLaunchContext::for_project_root(&root);
        let session = spawn_terminal_session(&context).expect("spawn PTY terminal session");
        session
            .write_bytes(b"printf 'datum-terminate-ready\\n'\nexec sleep 10\n")
            .expect("start long command");
        let mut ready = false;
        for _ in 0..50 {
            if let Ok(TerminalEvent::Output(bytes)) =
                session.rx.recv_timeout(Duration::from_millis(100))
            {
                if String::from_utf8_lossy(&bytes).contains("datum-terminate-ready") {
                    ready = true;
                    break;
                }
            }
        }
        assert!(
            ready,
            "terminal should confirm command execution before termination"
        );
        session.terminate().expect("terminate PTY session");
        let mut observed_exit_code = None;
        for _ in 0..120 {
            if let Ok(TerminalEvent::Exited(code)) =
                session.rx.recv_timeout(Duration::from_millis(100))
            {
                observed_exit_code = Some(code);
                break;
            }
        }
        assert!(
            observed_exit_code.is_some(),
            "terminated terminal should emit exit event"
        );
        let observed_exit_code = observed_exit_code.flatten();
        mark_terminal_session_lifecycle(
            &session,
            DatumToolSessionLifecycle::Exited,
            observed_exit_code,
        )
        .expect("mark terminated session exited");
        let context: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&session.context_path).unwrap()).unwrap();
        assert_eq!(context["session_lifecycle"], "exited");
        assert_eq!(context["session"]["lifecycle"], "exited");
        assert_eq!(
            context["process_exit_code"],
            serde_json::to_value(observed_exit_code).unwrap()
        );
        let _ = fs::remove_dir_all(&root);
    }
}
