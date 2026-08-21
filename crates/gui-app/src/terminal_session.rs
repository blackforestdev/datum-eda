use crate::{
    terminal_activity_snapshot::TerminalActivitySummaryCache,
    terminal_context::{
        TerminalContext, read_session_created_unix_ms, unix_time_ms,
        update_terminal_lifecycle_file, update_terminal_lifecycle_file_exact,
        write_terminal_context_files,
    },
    terminal_core_adapter::TerminalCoreSessionAdapter,
    terminal_process::spawn_terminal_process,
    terminal_session_context::{TerminalSessionContextSummary, dock_tab_name, workspace_tool_name},
    terminal_session_events::record_terminal_lifecycle_event,
    terminal_transport::{MAX_LIVE_SESSIONS, TerminalTransportSession, TerminalWakeGate},
};
use anyhow::Result;
use datum_gui_protocol::{
    CheckRunReviewState, DatumCursorContext, DatumProjectionContext, DatumSceneBoundsContext,
    DatumSelectionContext, DatumToolSessionLifecycle, ProductionStatus, ReviewWorkspaceState,
    TerminalLaneState, TerminalTabState,
};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc::Receiver};
use winit::event_loop::EventLoopProxy;

pub(super) use crate::terminal_transport::TerminalTransportEvent as TerminalEvent;

pub(super) struct TerminalSession {
    transport: TerminalTransportSession,
    pub(super) context_path: PathBuf,
    pub(super) latest_context_path: PathBuf,
    pub(super) session_path: PathBuf,
    pub(super) session_id: String,
    pub(super) context_id: String,
    pub(super) active_execution_id: Arc<Mutex<Option<String>>>,
    /// Byte offset of the next unscanned event-log line for the
    /// command-finished check (terminal performance slice): the log is
    /// append-only, so each region is scanned at most once instead of
    /// re-reading the whole file per drained output chunk.
    pub(super) finished_scan_offset: std::cell::Cell<u64>,
}

pub(super) struct TerminalSessionRegistry {
    sessions: Vec<TerminalSessionSlot>,
    pending_spawns: Vec<PendingTerminalSpawn>,
    active_pending_id: Option<String>,
    active_index: usize,
    next_session_ordinal: usize,
    terminal_wake: TerminalWakeGate,
    next_drain_index: usize,
    projection_managed: bool,
}

struct PendingTerminalSpawn {
    pending_id: String,
    label: String,
    result: Receiver<std::result::Result<TerminalSession, String>>,
    canceled: bool,
}

struct TerminalSessionSlot {
    session: TerminalSession,
    core: TerminalCoreSessionAdapter,
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
    parked_lane: TerminalLaneState,
    disconnected_reported: bool,
    termination_failure_reported: bool,
    close_confirmation_armed: bool,
    pending_restart: bool,
    remove_when_closed: bool,
    hidden_after_close: bool,
    exact_exit_status: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct TerminalLaunchContext {
    pub(super) project_root: PathBuf,
    /// Directory inherited by the launched shell. This is intentionally
    /// separate from `project_root`, which remains Datum's stable project and
    /// context identity when a new tab follows the active shell's OSC 7 CWD.
    pub(super) launch_working_directory: PathBuf,
    pub(super) project_id: Option<String>,
    pub(super) project_name: Option<String>,
    pub(super) board_id: Option<String>,
    pub(super) board_name: Option<String>,
    pub(super) scene_id: Option<String>,
    pub(super) source_revision: Option<String>,
    pub(super) accepted_transaction_tip: Option<String>,
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
        let core = TerminalCoreSessionAdapter::new(
            session.session_id.clone(),
            session.context_id.clone(),
            80,
            24,
        )?;
        Ok(Self {
            sessions: vec![TerminalSessionSlot {
                session,
                core,
                label: "shell 1".to_string(),
                status: "running".to_string(),
                attached: true,
                previous_session_id: None,
                restart_count: 0,
                columns: 80,
                rows: 24,
                activity: TerminalActivitySummaryCache::default(),
                parked_lane: TerminalLaneState::default(),
                disconnected_reported: false,
                termination_failure_reported: false,
                close_confirmation_armed: false,
                pending_restart: false,
                remove_when_closed: false,
                hidden_after_close: false,
                exact_exit_status: None,
            }],
            pending_spawns: Vec::new(),
            active_pending_id: None,
            active_index: 0,
            next_session_ordinal: 2,
            terminal_wake,
            next_drain_index: 0,
            projection_managed: false,
        })
    }

    #[allow(dead_code)]
    pub(super) fn activate(&mut self, session_id: &str) -> Result<()> {
        let index = self
            .sessions
            .iter()
            .position(|slot| slot.session.session_id() == session_id)
            .ok_or_else(|| anyhow::anyhow!("terminal session not found: {session_id}"))?;
        if index == self.active_index {
            return Ok(());
        }
        self.active_index = index;
        Ok(())
    }

    pub(super) fn activate_with_lane(
        &mut self,
        session_id: &str,
        lane: &mut TerminalLaneState,
    ) -> Result<()> {
        if self
            .pending_spawns
            .iter()
            .any(|pending| pending.pending_id == session_id)
        {
            if self.active_pending_id.is_none() {
                lane.swap_session_projection(&mut self.sessions[self.active_index].parked_lane);
            }
            self.active_pending_id = Some(session_id.to_string());
            lane.status = "starting terminal session".to_string();
            self.projection_managed = true;
            return Ok(());
        }
        let previous = self.active_index;
        let target = self
            .sessions
            .iter()
            .position(|slot| slot.session.session_id() == session_id)
            .ok_or_else(|| anyhow::anyhow!("terminal session not found: {session_id}"))?;
        self.activate(session_id)?;
        if self.active_pending_id.take().is_some() {
            lane.swap_session_projection(&mut self.sessions[target].parked_lane);
        } else if target != previous {
            lane.swap_session_projection(&mut self.sessions[previous].parked_lane);
            lane.swap_session_projection(&mut self.sessions[target].parked_lane);
        }
        self.projection_managed = true;
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

    pub(super) fn active_attached(&self) -> bool {
        self.active_pending_id.is_none() && self.sessions[self.active_index].attached
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn active_core_mut(&mut self) -> &mut TerminalCoreSessionAdapter {
        &mut self.sessions[self.active_index].core
    }

    pub(super) fn restart_active(
        &mut self,
        state: &mut TerminalLaneState,
        context: &TerminalLaunchContext,
    ) -> Result<()> {
        if self.active_pending_id.is_some() {
            anyhow::bail!("terminal session is still starting");
        }
        let slot = &mut self.sessions[self.active_index];
        let phase = slot
            .session
            .shutdown_snapshot()
            .map(|snapshot| snapshot.phase);
        if phase != Some(crate::terminal_transport::ShutdownPhase::Closed)
            || !slot.session.presentation_complete()
        {
            slot.pending_restart = true;
            return terminate_terminal_session(&slot.session, state);
        }
        Self::replace_slot_session(slot, state, context, self.terminal_wake.clone())
    }

    fn replace_slot_session(
        slot: &mut TerminalSessionSlot,
        state: &mut TerminalLaneState,
        context: &TerminalLaunchContext,
        terminal_wake: TerminalWakeGate,
    ) -> Result<()> {
        let previous_session_id = slot.session.session_id().to_string();
        restart_terminal_session(&mut slot.session, state, context, terminal_wake)?;
        slot.core = TerminalCoreSessionAdapter::new(
            slot.session.session_id.clone(),
            slot.session.context_id.clone(),
            slot.columns,
            slot.rows,
        )?;
        slot.status = state.status.clone();
        slot.attached = true;
        slot.previous_session_id = Some(previous_session_id);
        slot.restart_count += 1;
        slot.pending_restart = false;
        slot.remove_when_closed = false;
        slot.hidden_after_close = false;
        slot.exact_exit_status = None;
        slot.session.resize(slot.columns, slot.rows)?;
        Ok(())
    }

    pub(super) fn complete_pending_restarts(
        &mut self,
        state: &mut TerminalLaneState,
        context: &TerminalLaunchContext,
    ) -> Result<bool> {
        let mut changed = false;
        for index in 0..self.sessions.len() {
            let ready = self.sessions[index].pending_restart
                && self.sessions[index]
                    .session
                    .shutdown_snapshot()
                    .is_some_and(|snapshot| {
                        snapshot.phase == crate::terminal_transport::ShutdownPhase::Closed
                    })
                && self.sessions[index].session.presentation_complete();
            if !ready {
                continue;
            }
            if index == self.active_index {
                Self::replace_slot_session(
                    &mut self.sessions[index],
                    state,
                    context,
                    self.terminal_wake.clone(),
                )?;
            } else {
                let mut lane = std::mem::take(&mut self.sessions[index].parked_lane);
                Self::replace_slot_session(
                    &mut self.sessions[index],
                    &mut lane,
                    context,
                    self.terminal_wake.clone(),
                )?;
                self.sessions[index].parked_lane = lane;
            }
            changed = true;
        }
        if changed {
            self.sync_lane_tabs(state);
        }
        Ok(changed)
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
        if self.active_pending_id.is_some() {
            return Ok(vec!["starting terminal session".to_string()]);
        }
        let slot = &mut self.sessions[self.active_index];
        let event_log_path = slot.session.event_log_path();
        slot.activity.refresh(&event_log_path);
        slot.activity.summary_lines(max_spans)
    }

    pub(super) fn sync_lane_tabs(&mut self, state: &mut TerminalLaneState) {
        let active_index = self.active_index;
        state.active_session_id = self.active_pending_id.clone().or_else(|| {
            (!self.sessions[self.active_index].hidden_after_close)
                .then(|| self.active().session_id().to_string())
        });
        let tabs = self
            .sessions
            .iter_mut()
            .enumerate()
            .filter(|(_, slot)| !slot.hidden_after_close)
            .map(|(index, slot)| {
                if self.active_pending_id.is_none() && index == active_index {
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
                    active: self.active_pending_id.is_none() && index == active_index,
                    attached: slot.attached,
                    status: slot.status.clone(),
                    restart_count: slot.restart_count,
                }
            })
            .chain(
                self.pending_spawns
                    .iter()
                    .filter(|pending| !pending.canceled)
                    .map(|pending| TerminalTabState {
                        session_id: pending.pending_id.clone(),
                        previous_session_id: None,
                        label: pending.label.clone(),
                        event_log_path: String::new(),
                        activity_event_count: 0,
                        activity_summary: vec!["starting terminal session".to_string()],
                        active: self.active_pending_id.as_deref() == Some(&pending.pending_id),
                        attached: true,
                        status: "starting".to_string(),
                        restart_count: 0,
                    }),
            )
            .collect::<Vec<_>>();
        if let Some(active_tab) = tabs.iter().find(|tab| tab.active) {
            state.activity_summary = active_tab.activity_summary.clone();
        }
        if self.active_pending_id.is_none() {
            let active_slot = &self.sessions[self.active_index];
            state.columns = active_slot.columns;
            state.rows = active_slot.rows;
        }
        state.tabs = tabs;
    }

    #[allow(dead_code)]
    pub(super) fn len(&self) -> usize {
        self.sessions.len()
    }
}

fn ensure_session_capacity(live_sessions: usize) -> Result<()> {
    if live_sessions >= MAX_LIVE_SESSIONS {
        anyhow::bail!("terminal session limit reached ({MAX_LIVE_SESSIONS})");
    }
    Ok(())
}

#[path = "terminal_session_drain.rs"]
mod drain;
#[path = "terminal_session_interaction.rs"]
mod interaction;
#[path = "terminal_session_lifecycle.rs"]
mod lifecycle;
#[path = "terminal_session_render.rs"]
mod render;
#[path = "terminal_session_spawn.rs"]
mod spawn;

pub(super) fn terminal_launch_context_from_state(
    project_root: &Path,
    state: &ReviewWorkspaceState,
) -> TerminalLaunchContext {
    TerminalLaunchContext {
        project_root: project_root.to_path_buf(),
        launch_working_directory: project_root.to_path_buf(),
        project_id: Some(state.scene.project_uuid.clone()),
        project_name: Some(state.scene.project_name.clone()),
        board_id: Some(state.scene.board_uuid.clone()),
        board_name: Some(state.scene.board_name.clone()),
        scene_id: Some(state.scene.scene_id.clone()),
        source_revision: Some(state.scene.source_revision.clone()),
        accepted_transaction_tip: state.supervision.journal.accepted_transaction_tip.clone(),
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
        context_path: session.context_path.clone(),
        latest_context_path: session.latest_context_path.clone(),
        session_path: session.session_path.clone(),
        context_id: session.context_id.clone(),
        session_id: session.session_id.clone(),
        project_id: context.project_id.clone(),
        model_revision: context.source_revision.clone(),
        created_unix_ms: read_session_created_unix_ms(&session.session_path)
            .unwrap_or_else(|| unix_time_ms().unwrap_or(0)),
        process_group_id: Some(session.process_group_id()),
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
    state: &mut TerminalLaneState,
    context: &TerminalLaunchContext,
    terminal_wake: TerminalWakeGate,
) -> Result<()> {
    let replacement = spawn_terminal_session_with_wake(context, terminal_wake)?;
    mark_terminal_session_lifecycle(session, DatumToolSessionLifecycle::Restarted, None)?;
    record_terminal_lifecycle_event(session, DatumToolSessionLifecycle::Restarted, None)?;
    *session = replacement;
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
        Some(session.process_group_id()),
    )?;
    update_terminal_lifecycle_file(
        &session.latest_context_path,
        lifecycle,
        process_exit_code,
        Some(session.process_group_id()),
    )?;
    update_terminal_lifecycle_file(
        &session.session_path,
        lifecycle,
        process_exit_code,
        Some(session.process_group_id()),
    )
}

pub(super) fn mark_terminal_session_exit(
    session: &TerminalSession,
    status: crate::terminal_transport::TerminalExitStatus,
) -> Result<()> {
    let (code, signal, core_dumped) = match status {
        crate::terminal_transport::TerminalExitStatus::Code(code) => (Some(code), None, None),
        crate::terminal_transport::TerminalExitStatus::Signal {
            signal,
            core_dumped,
        } => (None, Some(signal), Some(core_dumped)),
    };
    for path in [
        &session.context_path,
        &session.latest_context_path,
        &session.session_path,
    ] {
        update_terminal_lifecycle_file_exact(
            path,
            DatumToolSessionLifecycle::Exited,
            code,
            signal,
            core_dumped,
            Some(session.process_group_id()),
        )?;
    }
    Ok(())
}

#[cfg(test)]
impl TerminalSessionRegistry {
    fn test_session_text(&self, index: usize) -> String {
        self.sessions[index].core.test_plain_text()
    }

    fn test_active_text(&self) -> String {
        self.test_session_text(self.active_index)
    }
}

#[path = "terminal_session_handle.rs"]
mod handle;
#[path = "terminal_session_reorder.rs"]
mod reorder;

#[cfg(test)]
#[path = "terminal_job_control_tests.rs"]
mod terminal_job_control_tests;
#[cfg(test)]
#[path = "terminal_regression_boundary_tests.rs"]
mod terminal_regression_boundary_tests;
#[cfg(test)]
#[path = "terminal_session_close_tests.rs"]
mod terminal_session_close_tests;
#[cfg(test)]
#[path = "terminal_session_context_tests.rs"]
mod terminal_session_context_tests;
#[cfg(test)]
#[path = "terminal_session_naming_tests.rs"]
mod terminal_session_naming_tests;
#[cfg(test)]
#[path = "terminal_session_p06_gui_measurement_tests.rs"]
mod terminal_session_p06_gui_measurement_tests;
#[cfg(test)]
#[path = "terminal_session_p06_isolation_tests.rs"]
mod terminal_session_p06_isolation_tests;
#[cfg(test)]
#[path = "terminal_session_p06_lifecycle_measurement_tests.rs"]
mod terminal_session_p06_lifecycle_measurement_tests;
#[cfg(test)]
#[path = "terminal_session_p06_measurement_tests.rs"]
mod terminal_session_p06_measurement_tests;
#[cfg(test)]
#[path = "terminal_session_p06_soak_tests.rs"]
mod terminal_session_p06_soak_tests;
#[cfg(test)]
#[path = "terminal_session_p06_stress_tests.rs"]
mod terminal_session_p06_stress_tests;
#[cfg(test)]
#[path = "terminal_session_p06_throughput_tests.rs"]
mod terminal_session_p06_throughput_tests;
#[cfg(test)]
#[path = "terminal_session_tests.rs"]
mod terminal_session_tests;

#[cfg(test)]
static P06_REAL_PTY_TEST_LOCK: Mutex<()> = Mutex::new(());
