//! Exclusive runtime terminal input-mode authority (TI-02).
//!
//! PTY input and chrome-local tab rename are mutually exclusive recipients.
//! This module owns that selection
//! and composed-text routing; byte encoding remains in `terminal_input`.

use super::*;

pub(super) fn write_attached_terminal_bytes(
    registry: &TerminalSessionRegistry,
    bytes: &[u8],
) -> Result<bool> {
    registry.active().write_bytes(bytes)?;
    Ok(true)
}

impl Runtime {
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
            self.terminal_sessions.active_attached(),
            self.terminal_rename_session_id.is_some(),
        )
    }

    pub(super) fn terminal_owns_input(&self) -> bool {
        self.terminal_input_owner() != keyboard_focus::TerminalInputOwner::Unowned
    }

    pub(super) fn terminal_rename_accepts_text_input(&self) -> bool {
        self.terminal_input_owner() == keyboard_focus::TerminalInputOwner::RenameChrome
    }

    pub(super) fn commit_terminal_ime_text(&mut self, text: &str) -> bool {
        match self.terminal_input_owner() {
            keyboard_focus::TerminalInputOwner::AttachedPty => {
                self.write_foreign_shell_bytes(text.as_bytes())
            }
            keyboard_focus::TerminalInputOwner::RenameChrome => {
                self.append_terminal_rename_text(text)
            }
            keyboard_focus::TerminalInputOwner::Unowned => false,
        }
    }
}

#[cfg(test)]
#[path = "terminal_input_mode_boundary_tests.rs"]
mod tests;
