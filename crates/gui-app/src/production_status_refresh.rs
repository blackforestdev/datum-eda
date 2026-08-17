use anyhow::Result;
use datum_gui_protocol::{
    LiveDesignSession, load_board_editor_workspace_state, load_live_workspace_state,
    refresh_check_run_review_state, refresh_production_status, refresh_source_shard_status,
};
use std::time::{Duration, Instant};
use winit::event_loop::{ActiveEventLoop, ControlFlow};

use super::{App, Runtime};

const TERMINAL_PRODUCTION_REFRESH_DELAY: Duration = Duration::from_millis(500);
const TERMINAL_PRODUCTION_REFRESH_RETRY_LIMIT: u8 = 8;

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
            if runtime.application_terminal_shutdown_complete() {
                event_loop.exit();
                return;
            }
            changed |= runtime.poll_application_terminal_shutdown();
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
    pub(super) fn handle_terminal_output_wake(&mut self) -> bool {
        self.terminal_sessions.acknowledge_output_poll();
        self.poll_terminal_output()
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
                self.log_review_event("workspace scene/status refreshed".to_string());
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
                self.log_review_event(format!("production status refresh failed: {err}"));
                true
            }
        }
    }

    pub(super) fn poll_terminal_output(&mut self) -> bool {
        let spawn_notices = self
            .terminal_sessions
            .complete_pending_spawns(&mut self.session.workspace_mut().ui.terminal);
        let spawned = !spawn_notices.is_empty();
        if spawned && let Some(started) = self.application_shutdown_started {
            self.terminal_sessions.terminate_all_by(
                started + Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS),
            );
        }
        let report = self
            .terminal_sessions
            .drain_all(&mut self.session.workspace_mut().ui.terminal);
        let restarted = self
            .terminal_sessions
            .complete_pending_restarts(
                &mut self.session.workspace_mut().ui.terminal,
                &self.terminal_launch_context,
            )
            .unwrap_or_else(|error| {
                self.log_review_event(format!("terminal restart completion failed: {error}"));
                false
            });
        if report.events == 0
            && !restarted
            && !spawned
            && !report.tabs_changed
            && !report.active_projection_changed
            && report.notices.is_empty()
        {
            return false;
        }
        if restarted {
            self.log_review_event("terminal session restarted after verified teardown");
        }
        for notice in spawn_notices {
            self.log_review_event(notice);
        }
        if (self.terminal_production_refresh_pending || self.terminal_workspace_refresh_pending)
            && self.terminal_production_refresh_due.is_none()
        {
            self.terminal_production_refresh_due =
                Some(Instant::now() + TERMINAL_PRODUCTION_REFRESH_DELAY);
        }
        for notice in report.notices {
            self.log_review_event(notice);
        }
        if report.tabs_changed || spawned {
            self.sync_terminal_tabs();
        }
        if spawned {
            self.resize_terminal_to_dock();
            self.invalidate_frame();
        }
        self.refresh_terminal_activity_summary();
        if report.active_projection_changed {
            self.invalidate_frame();
        }
        true
    }
}
