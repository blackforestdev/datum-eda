use crate::{
    terminal_activity_snapshot::load_terminal_activity_summary_lines,
    terminal_context::{
        DATUM_CLI, DATUM_LEGACY_CLI, TerminalContext, read_session_created_unix_ms,
        tool_session_event_log_path, unix_time_ms, update_terminal_lifecycle_file,
        write_terminal_context, write_terminal_context_files,
    },
    terminal_screen::TerminalScreen,
    terminal_session_context::{TerminalSessionContextSummary, dock_tab_name, workspace_tool_name},
    terminal_session_events::{record_terminal_input_event, record_terminal_lifecycle_event},
};
use anyhow::{Context, Result};
use datum_gui_protocol::{
    CheckRunReviewState, DatumCursorContext, DatumProjectionContext, DatumSceneBoundsContext,
    DatumSelectionContext, DatumToolSessionLifecycle, ProductionStatus, ReviewWorkspaceState,
    TerminalLaneState, TerminalTabState,
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

pub(super) struct TerminalSessionRegistry {
    sessions: Vec<TerminalSessionSlot>,
    active_index: usize,
}

struct TerminalSessionSlot {
    session: TerminalSession,
    screen: TerminalScreen,
    label: String,
    status: String,
    attached: bool,
    previous_session_id: Option<String>,
    restart_count: usize,
    columns: u16,
    rows: u16,
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
    pub(super) source_shard_status: datum_gui_protocol::SourceShardStatusSummary,
    pub(super) check_status: CheckRunReviewState,
    pub(super) selection_context: DatumSelectionContext,
    pub(super) cursor_context: DatumCursorContext,
    pub(super) projection_context: DatumProjectionContext,
    pub(super) terminal_sessions: TerminalSessionContextSummary,
}

impl TerminalSessionRegistry {
    pub(super) fn spawn(context: &TerminalLaunchContext) -> Result<Self> {
        let session = spawn_terminal_session(context)?;
        Ok(Self {
            sessions: vec![TerminalSessionSlot {
                session,
                screen: TerminalScreen::default(),
                label: "shell 1".to_string(),
                status: "running".to_string(),
                attached: true,
                previous_session_id: None,
                restart_count: 0,
                columns: 80,
                rows: 24,
            }],
            active_index: 0,
        })
    }

    #[allow(dead_code)]
    pub(super) fn spawn_and_activate(&mut self, context: &TerminalLaunchContext) -> Result<&str> {
        let previous_active_index = self.active_index;
        let session = spawn_terminal_session(context)?;
        self.sessions.push(TerminalSessionSlot {
            session,
            screen: TerminalScreen::default(),
            label: format!("shell {}", self.sessions.len() + 1),
            status: "running".to_string(),
            attached: true,
            previous_session_id: None,
            restart_count: 0,
            columns: 80,
            rows: 24,
        });
        self.sessions[previous_active_index].attached = false;
        mark_terminal_session_lifecycle(
            &self.sessions[previous_active_index].session,
            DatumToolSessionLifecycle::Detached,
            None,
        )?;
        record_terminal_lifecycle_event(
            &self.sessions[previous_active_index].session,
            DatumToolSessionLifecycle::Detached,
            None,
        )?;
        self.active_index = self.sessions.len() - 1;
        mark_terminal_session_lifecycle(self.active(), DatumToolSessionLifecycle::Attached, None)?;
        record_terminal_lifecycle_event(self.active(), DatumToolSessionLifecycle::Attached, None)?;
        Ok(self.active().session_id())
    }

    #[allow(dead_code)]
    pub(super) fn activate(&mut self, session_id: &str) -> Result<()> {
        let index = self
            .sessions
            .iter()
            .position(|slot| slot.session.session_id() == session_id)
            .ok_or_else(|| anyhow::anyhow!("terminal session not found: {session_id}"))?;
        if index == self.active_index && self.sessions[index].attached {
            return Ok(());
        }
        if index != self.active_index {
            let previous_active_index = self.active_index;
            self.sessions[previous_active_index].attached = false;
            mark_terminal_session_lifecycle(
                &self.sessions[previous_active_index].session,
                DatumToolSessionLifecycle::Detached,
                None,
            )?;
            record_terminal_lifecycle_event(
                &self.sessions[previous_active_index].session,
                DatumToolSessionLifecycle::Detached,
                None,
            )?;
        }
        self.active_index = index;
        self.sessions[self.active_index].attached = true;
        mark_terminal_session_lifecycle(self.active(), DatumToolSessionLifecycle::Attached, None)?;
        record_terminal_lifecycle_event(self.active(), DatumToolSessionLifecycle::Attached, None)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub(super) fn rename(&mut self, session_id: &str, label: impl Into<String>) -> Result<()> {
        let slot = self
            .sessions
            .iter_mut()
            .find(|slot| slot.session.session_id() == session_id)
            .ok_or_else(|| anyhow::anyhow!("terminal session not found: {session_id}"))?;
        let label = label.into();
        let trimmed = label.trim();
        if trimmed.is_empty() {
            anyhow::bail!("terminal session label must not be empty");
        }
        slot.label = trimmed.to_string();
        Ok(())
    }

    pub(super) fn active(&self) -> &TerminalSession {
        &self.sessions[self.active_index].session
    }

    pub(super) fn active_label(&self) -> &str {
        &self.sessions[self.active_index].label
    }

    pub(super) fn active_attached(&self) -> bool {
        self.sessions[self.active_index].attached
    }

    pub(super) fn active_screen_mut(&mut self) -> &mut TerminalScreen {
        &mut self.sessions[self.active_index].screen
    }

    pub(super) fn active_bracketed_paste_enabled(&self) -> bool {
        self.sessions[self.active_index]
            .screen
            .bracketed_paste_enabled()
    }

    pub(super) fn resize_active(&mut self, cols: u16, rows: u16) -> Result<()> {
        let slot = &mut self.sessions[self.active_index];
        let cols = cols.max(1);
        let rows = rows.max(1);
        if slot.columns == cols && slot.rows == rows {
            return Ok(());
        }
        slot.session.resize(cols, rows)?;
        slot.screen.resize_grid(cols, rows);
        slot.columns = cols;
        slot.rows = rows;
        Ok(())
    }

    pub(super) fn detach_active(&mut self, state: &mut TerminalLaneState) -> Result<()> {
        if !self.sessions[self.active_index].attached {
            self.sync_lane_tabs(state);
            return Ok(());
        }
        self.sessions[self.active_index].attached = false;
        mark_terminal_session_lifecycle(self.active(), DatumToolSessionLifecycle::Detached, None)?;
        record_terminal_lifecycle_event(self.active(), DatumToolSessionLifecycle::Detached, None)?;
        self.sync_lane_tabs(state);
        Ok(())
    }

    pub(super) fn terminate_active(&mut self, state: &mut TerminalLaneState) -> Result<()> {
        terminate_terminal_session(self.active(), state)?;
        self.sessions[self.active_index].status = state.status.clone();
        self.sync_lane_tabs(state);
        Ok(())
    }

    pub(super) fn close_active(&mut self, state: &mut TerminalLaneState) -> Result<()> {
        if self.sessions.len() <= 1 {
            anyhow::bail!("cannot close the only terminal session");
        }
        terminate_terminal_session(self.active(), state)?;
        self.sessions.remove(self.active_index);
        if self.active_index >= self.sessions.len() {
            self.active_index = self.sessions.len() - 1;
        }
        self.sessions[self.active_index].attached = true;
        state.status = self.sessions[self.active_index].status.clone();
        self.sync_lane_tabs(state);
        Ok(())
    }

    pub(super) fn restart_active(
        &mut self,
        state: &mut TerminalLaneState,
        context: &TerminalLaunchContext,
    ) -> Result<()> {
        let slot = &mut self.sessions[self.active_index];
        let previous_session_id = slot.session.session_id().to_string();
        restart_terminal_session(&mut slot.session, &mut slot.screen, state, context)?;
        slot.status = state.status.clone();
        slot.attached = true;
        slot.previous_session_id = Some(previous_session_id);
        slot.restart_count += 1;
        slot.session.resize(slot.columns, slot.rows)?;
        slot.screen.resize_grid(slot.columns, slot.rows);
        self.sync_lane_tabs(state);
        Ok(())
    }

    pub(super) fn active_event_log_path(&self) -> PathBuf {
        self.active().event_log_path()
    }

    pub(super) fn sync_lane_tabs(&mut self, state: &mut TerminalLaneState) {
        for (index, slot) in self.sessions.iter_mut().enumerate() {
            if index == self.active_index {
                slot.status = state.status.clone();
            }
        }
        state.active_session_id = Some(self.active().session_id().to_string());
        let tabs = self
            .sessions
            .iter()
            .enumerate()
            .map(|(index, slot)| TerminalTabState {
                session_id: slot.session.session_id().to_string(),
                previous_session_id: slot.previous_session_id.clone(),
                label: slot.label.clone(),
                event_log_path: slot.session.event_log_path().display().to_string(),
                activity_event_count: terminal_event_count(&slot.session.event_log_path()),
                activity_summary: terminal_activity_summary(&slot.session.event_log_path(), 2),
                active: index == self.active_index,
                attached: slot.attached,
                status: slot.status.clone(),
                restart_count: slot.restart_count,
            })
            .collect::<Vec<_>>();
        if let Some(active_tab) = tabs.iter().find(|tab| tab.active) {
            state.activity_summary = active_tab.activity_summary.clone();
        }
        let active_slot = &self.sessions[self.active_index];
        state.columns = active_slot.columns;
        state.rows = active_slot.rows;
        state.tabs = tabs;
    }

    #[allow(dead_code)]
    pub(super) fn len(&self) -> usize {
        self.sessions.len()
    }
}

fn terminal_event_count(path: &Path) -> usize {
    std::fs::read_to_string(path)
        .map(|text| text.lines().filter(|line| !line.trim().is_empty()).count())
        .unwrap_or(0)
}

fn terminal_activity_summary(path: &Path, max_spans: usize) -> Vec<String> {
    load_terminal_activity_summary_lines(path, max_spans).unwrap_or_else(|err| {
        vec![format!(
            "activity summary unavailable for {}: {err}",
            path.display()
        )]
    })
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
        source_shard_status: state.source_shards.clone(),
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
        terminal_sessions: TerminalSessionContextSummary::from_lane_state(&state.ui.terminal),
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
#[path = "terminal_session_context_tests.rs"]
mod terminal_session_context_tests;
#[cfg(test)]
#[path = "terminal_session_tests.rs"]
mod terminal_session_tests;
