use super::{DockTab, Runtime};
use datum_gui_protocol::TerminalSplitDirection;
use datum_gui_render::HitTarget;

impl Runtime {
    pub(super) fn handle_terminal_lifecycle_target(&mut self, target: &HitTarget) -> bool {
        match target {
            HitTarget::TerminalSessionTerminateActive => {
                let _ = self
                    .terminal_sessions
                    .confirm_close_active(&mut self.session.workspace_mut().ui.terminal);
            }
            HitTarget::TerminalSessionForceKillActive => {
                self.terminal_sessions.force_kill_active();
                self.invalidate_frame();
            }
            HitTarget::TerminalSessionRetryTermination => {
                if self.application_shutdown_started.is_some() {
                    self.retry_application_terminal_shutdown();
                } else {
                    self.terminal_sessions.retry_failed_terminations();
                }
            }
            HitTarget::TerminalShutdownCancel if self.application_shutdown_started.is_some() => {
                self.cancel_application_terminal_shutdown();
            }
            HitTarget::TerminalShutdownCancel => {
                self.terminal_sessions.handle_close_confirmation_input(
                    b"\x1b",
                    &mut self.session.workspace_mut().ui.terminal,
                );
                self.sync_terminal_tabs();
                self.invalidate_frame();
            }
            _ => return false,
        }
        true
    }

    pub(super) fn refresh_terminal_activity_summary(&mut self) -> bool {
        // Incremental read: O(new event-log bytes) per refresh (terminal
        // performance slice) — never a full-log reload on the drain path.
        let next = match self.terminal_sessions.active_activity_summary_lines(4) {
            Ok(lines) => lines,
            Err(err) => vec![format!("activity summary unavailable: {err}")],
        };
        let ui = &mut self.session.workspace_mut().ui;
        if ui.terminal.activity_summary == next {
            return false;
        }
        ui.terminal.activity_summary = next;
        self.sync_terminal_tabs();
        self.invalidate_frame();
        true
    }

    pub(super) fn sync_terminal_tabs(&mut self) {
        self.terminal_sessions
            .sync_lane_tabs(&mut self.session.workspace_mut().ui.terminal);
    }

    pub(super) fn spawn_terminal_session_tab(&mut self) -> bool {
        let context = crate::terminal_working_directory::context_for_new_terminal(
            &self.terminal_launch_context,
            self.session
                .workspace()
                .ui
                .terminal
                .current_working_directory
                .as_deref(),
        );
        match self
            .terminal_sessions
            .begin_spawn_and_activate(&context, &mut self.session.workspace_mut().ui.terminal)
        {
            Ok(pending_id) => {
                self.log_review_event(format!("opening terminal session {pending_id}"));
                self.set_active_dock(DockTab::Terminal);
                self.sync_terminal_tabs();
                self.invalidate_frame();
            }
            Err(err) => {
                let message = format!("terminal session open failed: {err}");
                self.session.workspace_mut().ui.terminal.status = message.clone();
                self.log_review_event(message);
                self.invalidate_frame();
            }
        }
        true
    }

    pub(super) fn spawn_terminal_split(&mut self, direction: TerminalSplitDirection) -> bool {
        let context = crate::terminal_working_directory::context_for_new_terminal(
            &self.terminal_launch_context,
            self.session
                .workspace()
                .ui
                .terminal
                .current_working_directory
                .as_deref(),
        );
        match self.terminal_sessions.begin_split_and_activate(
            &context,
            &mut self.session.workspace_mut().ui.terminal,
            direction,
        ) {
            Ok(pending_id) => {
                self.log_review_event(format!("opening terminal split {pending_id}"));
                self.set_active_dock(DockTab::Terminal);
                self.sync_terminal_tabs();
                self.resize_terminal_to_dock();
                self.invalidate_frame();
            }
            Err(err) => {
                let message = format!("terminal split open failed: {err}");
                self.session.workspace_mut().ui.terminal.status = message.clone();
                self.log_review_event(message);
                self.invalidate_frame();
            }
        }
        true
    }

    pub(super) fn close_active_terminal_session(&mut self) -> bool {
        match self
            .terminal_sessions
            .close_active(&mut self.session.workspace_mut().ui.terminal)
        {
            Ok(()) => {
                self.refresh_terminal_context_snapshot();
                self.refresh_terminal_activity_summary();
                self.sync_terminal_tabs();
                self.resize_terminal_to_dock();
            }
            Err(err) => self.log_review_event(format!("terminal session close failed: {err}")),
        }
        true
    }

    pub(super) fn close_terminal_session(&mut self, session_id: &str) -> bool {
        match self
            .terminal_sessions
            .close_session(session_id, &mut self.session.workspace_mut().ui.terminal)
        {
            Ok(()) => {
                self.refresh_terminal_context_snapshot();
                self.refresh_terminal_activity_summary();
                self.sync_terminal_tabs();
                self.resize_terminal_to_dock();
                self.invalidate_frame();
            }
            Err(err) => {
                self.log_review_event(format!("terminal session {session_id} close failed: {err}"))
            }
        }
        true
    }
}
