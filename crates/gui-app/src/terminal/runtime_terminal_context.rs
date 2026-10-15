use super::*;
use crate::terminal_session::refresh_terminal_session_context_from_state;

impl Runtime {
    /// Publish changed emitted context for an existing current-Project consumer.
    /// Capped transport equality does not assert full GUI selection identity.
    pub(super) fn refresh_terminal_selection_context_if_changed(&mut self) {
        if !self.terminal_sessions.active_attached() {
            return;
        }
        let Some(origin) = self.terminal_sessions.active_origin_context() else {
            return;
        };
        if !self
            .terminal_launch_context
            .as_ref()
            .is_some_and(|context| origin.project_root == context.project_root)
        {
            return;
        }
        let state = self.workspace();
        let selection = datum_gui_protocol::DatumSelectionContext::from_selection(&state.selection);
        if origin.selection_context == selection
            && origin.project_id.as_deref() == Some(state.scene.project_uuid.as_str())
            && origin.source_revision.as_deref() == Some(state.scene.source_revision.as_str())
            && origin.accepted_transaction_tip == state.supervision.journal.accepted_transaction_tip
        {
            return;
        }
        // The existing publisher advances the baseline only after success;
        // later semantic refresh or session activation retries a failed write.
        self.refresh_terminal_context_snapshot();
    }

    pub(super) fn refresh_terminal_context_snapshot(&mut self) {
        let Some(current) = self.terminal_launch_context.as_ref() else {
            return;
        };
        if !self.terminal_sessions.active_attached() {
            return;
        }
        let started = std::time::Instant::now();
        append_gui_verbose_diagnostic_line(|| "terminal context refresh begin");
        let origin = self
            .terminal_sessions
            .active_origin_context()
            .unwrap_or(current)
            .clone();
        let current_project = origin.project_root == current.project_root;
        let result = if current_project {
            refresh_terminal_session_context_from_state(
                self.terminal_sessions.active(),
                current,
                self.workspace(),
                self.last_cursor_pos,
            )
        } else if let Some(state) = self
            .terminal_sessions
            .origin_workspace(&origin.project_root)
        {
            let mut state = state.clone();
            state.ui.terminal = self.workspace().ui.terminal.clone();
            refresh_terminal_session_context_from_state(
                self.terminal_sessions.active(),
                &origin,
                &state,
                None,
            )
        } else {
            Err(anyhow::anyhow!(
                "Terminal origin Project snapshot unavailable"
            ))
        };
        match result {
            Ok(context) => {
                self.terminal_sessions
                    .update_active_origin_context(context.clone());
                if current_project {
                    self.terminal_launch_context = Some(context);
                }
            }
            Err(err) => {
                self.log_terminal_event(format!("terminal context refresh failed: {err}"));
            }
        }
        let elapsed_ms = started.elapsed().as_millis();
        if elapsed_ms > 50 {
            append_gui_diagnostic_line(format!("terminal context refresh slow {elapsed_ms}ms"));
        } else {
            append_gui_verbose_diagnostic_line(|| {
                format!("terminal context refresh end {elapsed_ms}ms")
            });
        }
    }
}
