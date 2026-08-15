use crate::{
    terminal_activity_snapshot::TerminalActivitySummaryCache,
    terminal_context::{
        TerminalContext, read_session_created_unix_ms, tool_session_event_log_path, unix_time_ms,
        update_terminal_lifecycle_file, write_terminal_context_files,
    },
    terminal_process::{
        TerminalWakeGate, spawn_terminal_process, terminal_transport::INITIAL_TERMINAL_SIZE,
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
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use winit::event_loop::EventLoopProxy;

pub(super) enum TerminalEvent {
    Output(Vec<u8>),
    Exited(Option<i32>),
}

pub(super) struct TerminalSession {
    pub(super) stdin: Arc<Mutex<Box<dyn Write + Send>>>,
    pub(super) rx: Receiver<TerminalEvent>,
    pub(super) context_path: PathBuf,
    pub(super) latest_context_path: PathBuf,
    pub(super) session_path: PathBuf,
    pub(super) session_id: String,
    pub(super) context_id: String,
    pub(super) master: Box<dyn portable_pty::MasterPty + Send>,
    pub(super) process_group_id: libc::pid_t,
    pub(super) active_execution_id: Arc<Mutex<Option<String>>>,
    /// Byte offset of the next unscanned event-log line for the
    /// command-finished check (terminal performance slice): the log is
    /// append-only, so each region is scanned at most once instead of
    /// re-reading the whole file per drained output chunk.
    pub(super) finished_scan_offset: std::cell::Cell<u64>,
}

pub(super) struct TerminalSessionRegistry {
    sessions: Vec<TerminalSessionSlot>,
    active_index: usize,
    terminal_wake: TerminalWakeGate,
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
    /// Incremental activity summary/event-count over this session's event log
    /// (terminal performance slice): O(new bytes) per refresh instead of a
    /// full-log reload. Self-resets when the slot's log path changes (restart).
    activity: TerminalActivitySummaryCache,
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
    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn spawn(context: &TerminalLaunchContext) -> Result<Self> {
        Self::spawn_with_proxy(context, None)
    }

    pub(super) fn spawn_with_proxy(
        context: &TerminalLaunchContext,
        terminal_event_proxy: Option<EventLoopProxy<()>>,
    ) -> Result<Self> {
        let terminal_wake = TerminalWakeGate::new(terminal_event_proxy);
        let session = spawn_terminal_session_with_wake(context, terminal_wake.clone())?;
        Ok(Self {
            sessions: vec![TerminalSessionSlot {
                session,
                screen: TerminalScreen::default(),
                label: "shell 1".to_string(),
                status: "running".to_string(),
                attached: true,
                previous_session_id: None,
                restart_count: 0,
                columns: INITIAL_TERMINAL_SIZE.cols,
                rows: INITIAL_TERMINAL_SIZE.rows,
                activity: TerminalActivitySummaryCache::default(),
            }],
            active_index: 0,
            terminal_wake,
        })
    }

    #[allow(dead_code)]
    pub(super) fn spawn_and_activate(&mut self, context: &TerminalLaunchContext) -> Result<&str> {
        let previous_active_index = self.active_index;
        let session = spawn_terminal_session_with_wake(context, self.terminal_wake.clone())?;
        self.sessions.push(TerminalSessionSlot {
            session,
            screen: TerminalScreen::default(),
            label: format!("shell {}", self.sessions.len() + 1),
            status: "running".to_string(),
            attached: true,
            previous_session_id: None,
            restart_count: 0,
            columns: INITIAL_TERMINAL_SIZE.cols,
            rows: INITIAL_TERMINAL_SIZE.rows,
            activity: TerminalActivitySummaryCache::default(),
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
        restart_terminal_session(
            &mut slot.session,
            &mut slot.screen,
            state,
            context,
            self.terminal_wake.clone(),
        )?;
        slot.status = state.status.clone();
        slot.attached = true;
        slot.previous_session_id = Some(previous_session_id);
        slot.restart_count += 1;
        slot.session.resize(slot.columns, slot.rows)?;
        slot.screen.resize_grid(slot.columns, slot.rows);
        self.sync_lane_tabs(state);
        Ok(())
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn active_event_log_path(&self) -> PathBuf {
        self.active().event_log_path()
    }

    pub(super) fn request_output_poll(&self) {
        self.terminal_wake.request();
    }

    pub(super) fn acknowledge_output_poll(&self) {
        self.terminal_wake.acknowledge();
    }

    /// Refresh the active session's incremental activity summary and return
    /// the formatted window — the production summary read
    /// (`Runtime::refresh_terminal_activity_summary`), O(new log bytes) per
    /// call (terminal performance slice).
    pub(super) fn active_activity_summary_lines(
        &mut self,
        max_spans: usize,
    ) -> Result<Vec<String>> {
        let slot = &mut self.sessions[self.active_index];
        let event_log_path = slot.session.event_log_path();
        slot.activity.refresh(&event_log_path);
        slot.activity.summary_lines(max_spans)
    }

    pub(super) fn sync_lane_tabs(&mut self, state: &mut TerminalLaneState) {
        let active_index = self.active_index;
        state.active_session_id = Some(self.active().session_id().to_string());
        let tabs = self
            .sessions
            .iter_mut()
            .enumerate()
            .map(|(index, slot)| {
                if index == active_index {
                    slot.status = state.status.clone();
                }
                let event_log_path = slot.session.event_log_path();
                slot.activity.refresh(&event_log_path);
                TerminalTabState {
                    session_id: slot.session.session_id().to_string(),
                    previous_session_id: slot.previous_session_id.clone(),
                    label: slot.label.clone(),
                    event_log_path: event_log_path.display().to_string(),
                    activity_event_count: slot.activity.event_count(),
                    activity_summary: slot.activity.summary_lines(2).unwrap_or_else(|err| {
                        vec![format!(
                            "activity summary unavailable for {}: {err}",
                            event_log_path.display()
                        )]
                    }),
                    active: index == active_index,
                    attached: slot.attached,
                    status: slot.status.clone(),
                    restart_count: slot.restart_count,
                }
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
            hovered_object_id: state
                .ui
                .hovered_object
                .as_ref()
                .map(|hover| hover.object_id.clone()),
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

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn spawn_terminal_session(context: &TerminalLaunchContext) -> Result<TerminalSession> {
    spawn_terminal_session_with_proxy(context, None)
}

fn spawn_terminal_session_with_proxy(
    context: &TerminalLaunchContext,
    terminal_event_proxy: Option<EventLoopProxy<()>>,
) -> Result<TerminalSession> {
    spawn_terminal_session_with_wake(context, TerminalWakeGate::new(terminal_event_proxy))
}

fn spawn_terminal_session_with_wake(
    context: &TerminalLaunchContext,
    terminal_wake: TerminalWakeGate,
) -> Result<TerminalSession> {
    spawn_terminal_process(context, terminal_wake)
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
        process_group_id: Some(session.process_group_id),
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
    terminal_wake: TerminalWakeGate,
) -> Result<()> {
    mark_terminal_session_lifecycle(session, DatumToolSessionLifecycle::Restarted, None)?;
    record_terminal_lifecycle_event(session, DatumToolSessionLifecycle::Restarted, None)?;
    *session = spawn_terminal_session_with_wake(context, terminal_wake)?;
    *screen = TerminalScreen::default();
    state.status = "running".to_string();
    // T0-C01 / decision 027 FT-001: restart is a lifecycle event. It must not
    // write a notice row into the terminal grid — the grid holds only PTY
    // output. Session status is visible through chrome (`state.status`); the
    // narration goes to the console sink at the Runtime call site.
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
        Some(session.process_group_id),
    )?;
    update_terminal_lifecycle_file(
        &session.latest_context_path,
        lifecycle,
        process_exit_code,
        Some(session.process_group_id),
    )?;
    update_terminal_lifecycle_file(
        &session.session_path,
        lifecycle,
        process_exit_code,
        Some(session.process_group_id),
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
    pub(super) fn finished_scan_offset(&self) -> u64 {
        self.finished_scan_offset.get()
    }
    pub(super) fn set_finished_scan_offset(&self, offset: u64) {
        self.finished_scan_offset.set(offset);
    }

    pub(super) fn interrupt(&self) -> Result<()> {
        // A terminal Ctrl-C is input, not an out-of-band signal to the shell's
        // original process group. The PTY line discipline delivers SIGINT to
        // whichever foreground pipeline currently owns the terminal.
        self.write_bytes(b"\x03")
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
        self.master
            .resize(portable_pty::PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("resize portable terminal PTY")
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}

#[cfg(test)]
#[path = "terminal_regression_boundary_tests.rs"]
mod terminal_regression_boundary_tests;
#[cfg(test)]
#[path = "terminal_screen_authority_tests.rs"]
mod terminal_screen_authority_tests;
#[cfg(test)]
#[path = "terminal_session_context_tests.rs"]
mod terminal_session_context_tests;
#[cfg(test)]
#[path = "terminal_session_tests.rs"]
mod terminal_session_tests;
