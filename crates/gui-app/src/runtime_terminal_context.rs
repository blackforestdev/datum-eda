use super::*;
use crate::terminal_session::refresh_terminal_session_context_from_state;

impl Runtime {
    pub(super) fn refresh_terminal_context_snapshot(&mut self) {
        let started = std::time::Instant::now();
        append_gui_verbose_diagnostic_line(|| "terminal context refresh begin");
        let origin = self
            .terminal_sessions
            .active_origin_context()
            .unwrap_or(&self.terminal_launch_context)
            .clone();
        let current_project = origin.project_root == self.terminal_launch_context.project_root;
        let result = if current_project {
            refresh_terminal_session_context_from_state(
                self.terminal_sessions.active(),
                &self.terminal_launch_context,
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
                    self.terminal_launch_context = context;
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
