use super::Runtime;
use crate::terminal_transport::GLOBAL_SHUTDOWN_MS;
use std::time::{Duration, Instant};

impl Runtime {
    pub(super) fn begin_application_terminal_shutdown(&mut self) {
        if self.application_shutdown_started.is_some() {
            return;
        }
        let started = Instant::now();
        let deadline = started + Duration::from_millis(GLOBAL_SHUTDOWN_MS);
        self.application_shutdown_started = Some(started);
        self.application_shutdown_blocked = false;
        self.session
            .workspace_mut()
            .ui
            .terminal
            .application_shutdown_blocked = None;
        self.terminal_sessions.terminate_all_by(deadline);
        self.session.workspace_mut().ui.terminal.status =
            "closing Datum: terminating all owned terminal sessions".to_string();
        self.sync_terminal_tabs();
        self.invalidate_frame();
    }

    pub(super) fn poll_application_terminal_shutdown(&mut self) -> bool {
        let Some(started) = self.application_shutdown_started else {
            return false;
        };
        if self.terminal_sessions.all_sessions_closed() {
            return false;
        }
        if !self.application_shutdown_blocked
            && started.elapsed() >= Duration::from_millis(GLOBAL_SHUTDOWN_MS)
        {
            self.application_shutdown_blocked = true;
            let failures = self.terminal_sessions.shutdown_failure_summary();
            let status = format!(
                "shutdown blocked by terminal teardown; RETRY or CANCEL SHUTDOWN{}{}",
                if failures.is_empty() { "" } else { ": " },
                failures
            );
            self.log_review_event(status.clone());
            self.session
                .workspace_mut()
                .ui
                .terminal
                .application_shutdown_blocked = Some(status);
            self.sync_terminal_tabs();
            self.invalidate_frame();
            return true;
        }
        false
    }

    pub(super) fn application_terminal_shutdown_complete(&self) -> bool {
        self.application_shutdown_started.is_some()
            && !self.application_shutdown_blocked
            && self.terminal_sessions.all_sessions_closed()
    }

    pub(super) fn retry_application_terminal_shutdown(&mut self) {
        if self.application_shutdown_started.is_none() {
            return;
        }
        let started = Instant::now();
        let deadline = started + Duration::from_millis(GLOBAL_SHUTDOWN_MS);
        self.terminal_sessions
            .retry_nonclosed_terminations_by(deadline);
        self.application_shutdown_started = Some(started);
        self.application_shutdown_blocked = false;
        self.session
            .workspace_mut()
            .ui
            .terminal
            .application_shutdown_blocked = None;
    }

    pub(super) fn cancel_application_terminal_shutdown(&mut self) {
        self.application_shutdown_started = None;
        self.application_shutdown_blocked = false;
        self.session
            .workspace_mut()
            .ui
            .terminal
            .application_shutdown_blocked = None;
        self.session.workspace_mut().ui.terminal.status =
            "shutdown canceled; already-started terminal teardown continues".to_string();
        self.sync_terminal_tabs();
        self.invalidate_frame();
    }
}
