use super::*;

impl Runtime {
    pub(super) fn refresh_terminal_context_snapshot(&mut self) {
        let _ = refresh_terminal_session_context_from_state(
            &self.terminal,
            &self.terminal_launch_context,
            self.workspace(),
            self.last_cursor_pos,
        );
    }
}
