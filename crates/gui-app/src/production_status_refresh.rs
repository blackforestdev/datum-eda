use anyhow::Result;
use datum_gui_protocol::{
    DatumToolSessionLifecycle, LiveDesignSession, load_board_editor_workspace_state,
    load_live_workspace_state, refresh_check_run_review_state, refresh_production_status,
    refresh_source_shard_status,
};
use std::sync::mpsc::TryRecvError;
use std::time::{Duration, Instant};
use winit::event_loop::{ActiveEventLoop, ControlFlow};

use super::terminal_session::{TerminalEvent, mark_terminal_session_lifecycle};
use super::terminal_session_events::{
    record_terminal_lifecycle_event, record_terminal_output_event,
};
use super::{App, Runtime, append_gui_diagnostic_line};

const TERMINAL_PRODUCTION_REFRESH_DELAY: Duration = Duration::from_millis(500);
const TERMINAL_PRODUCTION_REFRESH_RETRY_LIMIT: u8 = 8;
const TERMINAL_EVENT_DRAIN_LIMIT: usize = 128;

pub(super) enum ProductionStatusRefresh {
    Changed,
    Unchanged,
}

pub(super) fn refresh_after_terminal_output(
    session: &mut LiveDesignSession,
    production_pending: &mut bool,
    workspace_pending: &mut bool,
    include_review: bool,
) -> Result<ProductionStatusRefresh> {
    if !*production_pending && !*workspace_pending {
        return Ok(ProductionStatusRefresh::Unchanged);
    }
    let Some(backing) = session.workspace().backing.clone() else {
        *production_pending = false;
        *workspace_pending = false;
        return Ok(ProductionStatusRefresh::Unchanged);
    };
    if *workspace_pending {
        return refresh_workspace_after_terminal_output(
            session,
            production_pending,
            workspace_pending,
            include_review,
            &backing.request,
        );
    }
    let before_production = session.workspace().production.clone();
    let before_checks = session.workspace().checks.clone();
    let before_source_shards = session.workspace().source_shards.clone();
    let next_production = refresh_production_status(&backing.request)?;
    let next_checks = refresh_check_run_review_state(&backing.request)?;
    let next_source_shards = refresh_source_shard_status(&backing.request)?;
    if next_production == before_production
        && next_checks == before_checks
        && next_source_shards == before_source_shards
    {
        return Ok(ProductionStatusRefresh::Unchanged);
    }
    let workspace = session.workspace_mut();
    workspace.production = next_production;
    workspace.checks = next_checks;
    workspace.source_shards = next_source_shards;
    *production_pending = false;
    Ok(ProductionStatusRefresh::Changed)
}

fn refresh_workspace_after_terminal_output(
    session: &mut LiveDesignSession,
    production_pending: &mut bool,
    workspace_pending: &mut bool,
    include_review: bool,
    request: &datum_gui_protocol::LiveReviewRequest,
) -> Result<ProductionStatusRefresh> {
    let before = session.workspace().clone();
    let next = if include_review {
        load_live_workspace_state(request)?
    } else {
        load_board_editor_workspace_state(request)?
    };
    if next.scene == before.scene
        && next.review == before.review
        && next.production == before.production
        && next.checks == before.checks
        && next.source_shards == before.source_shards
    {
        return Ok(ProductionStatusRefresh::Unchanged);
    }
    let workspace = session.workspace_mut();
    workspace.scene = next.scene;
    workspace.review = next.review;
    workspace.production = next.production;
    workspace.source_shards = next.source_shards;
    workspace.checks = next.checks;
    workspace.active_review_target_id = next.active_review_target_id;
    workspace.backing = next.backing;
    *production_pending = false;
    *workspace_pending = false;
    Ok(ProductionStatusRefresh::Changed)
}

impl App {
    pub(super) fn poll_background_work(&mut self, event_loop: &ActiveEventLoop) {
        let mut changed = false;
        let mut next_refresh_due = None;
        if let Some(runtime) = &mut self.runtime {
            changed |= runtime.poll_terminal_output();
            changed |= runtime.poll_scheduled_production_refresh();
            next_refresh_due = runtime.next_production_refresh_due();
        }
        if changed {
            self.request_redraw_if_needed();
        }
        if let Some(next_refresh_due) = next_refresh_due {
            event_loop.set_control_flow(ControlFlow::WaitUntil(next_refresh_due));
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}

impl Runtime {
    pub(super) fn mark_terminal_production_refresh_pending(&mut self) {
        self.terminal_production_refresh_pending = true;
        self.terminal_production_refresh_attempts = 0;
        self.terminal_production_refresh_due =
            Some(Instant::now() + TERMINAL_PRODUCTION_REFRESH_DELAY);
    }

    pub(super) fn mark_terminal_workspace_refresh_pending(&mut self) {
        self.terminal_workspace_refresh_pending = true;
        self.terminal_production_refresh_attempts = 0;
        self.terminal_production_refresh_due =
            Some(Instant::now() + TERMINAL_PRODUCTION_REFRESH_DELAY);
    }

    pub(super) fn next_production_refresh_due(&self) -> Option<Instant> {
        if self.terminal_production_refresh_pending || self.terminal_workspace_refresh_pending {
            self.terminal_production_refresh_due
        } else {
            None
        }
    }

    pub(super) fn poll_scheduled_production_refresh(&mut self) -> bool {
        let Some(due) = self.next_production_refresh_due() else {
            return false;
        };
        if Instant::now() < due {
            return false;
        }
        self.terminal_production_refresh_attempts =
            self.terminal_production_refresh_attempts.saturating_add(1);
        match refresh_after_terminal_output(
            &mut self.session,
            &mut self.terminal_production_refresh_pending,
            &mut self.terminal_workspace_refresh_pending,
            self.workspace_include_review,
        ) {
            Ok(ProductionStatusRefresh::Changed) => {
                self.terminal_production_refresh_due = None;
                self.terminal_production_refresh_attempts = 0;
                self.invalidate_scene();
                self.push_terminal_line("workspace scene/status refreshed".to_string());
                true
            }
            Ok(ProductionStatusRefresh::Unchanged) => {
                if self.terminal_production_refresh_attempts
                    >= TERMINAL_PRODUCTION_REFRESH_RETRY_LIMIT
                {
                    self.terminal_production_refresh_pending = false;
                    self.terminal_workspace_refresh_pending = false;
                    self.terminal_production_refresh_due = None;
                    self.terminal_production_refresh_attempts = 0;
                } else {
                    self.terminal_production_refresh_due =
                        Some(Instant::now() + TERMINAL_PRODUCTION_REFRESH_DELAY);
                }
                false
            }
            Err(err) => {
                self.terminal_production_refresh_pending = false;
                self.terminal_workspace_refresh_pending = false;
                self.terminal_production_refresh_due = None;
                self.terminal_production_refresh_attempts = 0;
                self.push_terminal_line(format!("production status refresh failed: {err}"));
                true
            }
        }
    }

    pub(super) fn poll_terminal_output(&mut self) -> bool {
        let mut changed = false;
        for _ in 0..TERMINAL_EVENT_DRAIN_LIMIT {
            match self.terminal_sessions.active().rx.try_recv() {
                Ok(TerminalEvent::Output(bytes)) => {
                    let _ = record_terminal_output_event(self.terminal_sessions.active(), &bytes);
                    self.refresh_terminal_activity_summary();
                    let responses = self
                        .terminal_sessions
                        .active_screen_mut()
                        .apply_bytes_with_responses(
                            &mut self.session.workspace_mut().ui.terminal,
                            &bytes,
                        );
                    for response in responses {
                        if let Err(err) = self.terminal_sessions.active().write_bytes(&response) {
                            self.push_terminal_line(format!(
                                "terminal status response failed: {err}"
                            ));
                        }
                    }
                    if (self.terminal_production_refresh_pending
                        || self.terminal_workspace_refresh_pending)
                        && self.terminal_production_refresh_due.is_none()
                    {
                        self.terminal_production_refresh_due =
                            Some(Instant::now() + TERMINAL_PRODUCTION_REFRESH_DELAY);
                    }
                    self.invalidate_frame();
                    changed = true;
                }
                Ok(TerminalEvent::Exited(code)) => {
                    let _ = mark_terminal_session_lifecycle(
                        self.terminal_sessions.active(),
                        DatumToolSessionLifecycle::Exited,
                        code,
                    );
                    let _ = record_terminal_lifecycle_event(
                        self.terminal_sessions.active(),
                        DatumToolSessionLifecycle::Exited,
                        code,
                    );
                    self.refresh_terminal_activity_summary();
                    let status = code.map_or_else(
                        || "terminated by signal".to_string(),
                        |code| format!("exited {code}"),
                    );
                    self.session.workspace_mut().ui.terminal.status = status.clone();
                    self.sync_terminal_tabs();
                    self.push_terminal_line(format!("terminal {status}"));
                    if (self.terminal_production_refresh_pending
                        || self.terminal_workspace_refresh_pending)
                        && self.terminal_production_refresh_due.is_none()
                    {
                        self.terminal_production_refresh_due =
                            Some(Instant::now() + TERMINAL_PRODUCTION_REFRESH_DELAY);
                    }
                    changed = true;
                }
                Err(TryRecvError::Empty) => return changed,
                Err(TryRecvError::Disconnected) => {
                    if self.terminal_disconnected_reported {
                        return false;
                    }
                    self.terminal_disconnected_reported = true;
                    self.push_terminal_line("terminal session ended".to_string());
                    return true;
                }
            }
        }
        append_gui_diagnostic_line(format!(
            "terminal output drain capped at {TERMINAL_EVENT_DRAIN_LIMIT} events"
        ));
        changed
    }
}
