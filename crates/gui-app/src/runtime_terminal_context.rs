use super::*;
use crate::terminal_session::refresh_terminal_session_context_from_state;

impl Runtime {
    pub(super) fn refresh_terminal_context_snapshot(&mut self) {
        let started = std::time::Instant::now();
        append_gui_verbose_diagnostic_line("terminal context refresh begin");
        match refresh_terminal_session_context_from_state(
            self.terminal_sessions.active(),
            &self.terminal_launch_context,
            self.workspace(),
            self.last_cursor_pos,
        ) {
            Ok(context) => {
                self.terminal_launch_context = context;
            }
            Err(err) => {
                self.push_terminal_line(format!("terminal context refresh failed: {err}"));
            }
        }
        let elapsed_ms = started.elapsed().as_millis();
        if elapsed_ms > 50 {
            append_gui_diagnostic_line(format!("terminal context refresh slow {elapsed_ms}ms"));
        } else {
            append_gui_verbose_diagnostic_line(format!(
                "terminal context refresh end {elapsed_ms}ms"
            ));
        }
    }
}
