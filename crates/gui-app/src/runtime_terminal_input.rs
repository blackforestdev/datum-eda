//! Exclusive runtime terminal input-mode authority (TI-02).
//!
//! This module owns attached-PTY selection and composed-text routing; byte
//! encoding remains in `terminal_input`.

use super::*;

pub(super) fn write_attached_terminal_bytes(
    registry: &TerminalSessionRegistry,
    bytes: &[u8],
) -> Result<bool> {
    if !registry.active_attached() {
        return Ok(false);
    }
    registry.active().write_bytes(bytes)?;
    Ok(true)
}

fn follow_live_terminal_input(state: &mut datum_gui_protocol::TerminalLaneState) {
    // Native terminals return to the live screen as soon as the user types.
    // Leaving accepted input hidden behind scrollback makes a healthy PTY look
    // unfocused or frozen even though the child is receiving and echoing bytes.
    state.scroll_offset = 0;
    state.clear_text_selection();
}

impl Runtime {
    pub(super) fn write_foreign_shell_bytes(&mut self, bytes: &[u8]) -> bool {
        match write_attached_terminal_bytes(&self.terminal_sessions, bytes) {
            Ok(true) => {
                follow_live_terminal_input(&mut self.session.workspace_mut().ui.terminal);
                self.invalidate_frame();
            }
            Ok(false) => self.log_review_event(
                "terminal session is starting; input is not ready yet".to_string(),
            ),
            Err(err) => {
                let message = format!("terminal input refused: {err}");
                self.session.workspace_mut().ui.terminal.status = message.clone();
                self.log_review_event(message);
                self.invalidate_frame();
            }
        }
        true
    }

    pub(super) fn handle_close_confirmation_action(
        &mut self,
        action: &TerminalKeyAction,
    ) -> Option<bool> {
        if !self.terminal_sessions.active_close_confirmation_armed() {
            return None;
        }
        match action {
            TerminalKeyAction::Write(bytes) if bytes == &[0x03] => None,
            TerminalKeyAction::Write(bytes) => {
                self.terminal_sessions.handle_close_confirmation_input(
                    bytes,
                    &mut self.session.workspace_mut().ui.terminal,
                );
                self.sync_terminal_tabs();
                self.invalidate_frame();
                Some(true)
            }
            TerminalKeyAction::CloseSession | TerminalKeyAction::TerminateSession => {
                let _ = self
                    .terminal_sessions
                    .confirm_close_active(&mut self.session.workspace_mut().ui.terminal);
                Some(true)
            }
            _ => Some(true),
        }
    }

    pub(super) fn terminal_input_owner(&self) -> keyboard_focus::TerminalInputOwner {
        keyboard_focus::terminal_input_owner(
            self.application_focus(),
            matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal))
                && !self.workspace().ui.terminal.tabs.is_empty(),
        )
    }

    pub(super) fn terminal_owns_input(&self) -> bool {
        self.terminal_input_owner() != keyboard_focus::TerminalInputOwner::Unowned
    }

    pub(super) fn commit_terminal_ime_text(&mut self, text: &str) -> bool {
        match self.terminal_input_owner() {
            keyboard_focus::TerminalInputOwner::AttachedPty => {
                self.write_foreign_shell_bytes(text.as_bytes())
            }
            keyboard_focus::TerminalInputOwner::Unowned => false,
        }
    }
}

#[cfg(test)]
#[path = "terminal_input_mode_boundary_tests.rs"]
mod tests;
