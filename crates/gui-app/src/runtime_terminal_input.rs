//! Exclusive runtime terminal input-mode authority (TI-02).
//!
//! Attached PTY input, detached read-only observation, and chrome-local tab
//! rename are mutually exclusive recipients. This module owns that selection
//! and composed-text routing; byte encoding remains in `terminal_input`.

use super::*;

impl Runtime {
    pub(super) fn terminal_input_owner(&self) -> keyboard_focus::TerminalInputOwner {
        keyboard_focus::terminal_input_owner(
            self.keyboard_focus,
            matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal)),
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
            keyboard_focus::TerminalInputOwner::DetachedReadOnly => true,
            keyboard_focus::TerminalInputOwner::Unowned => false,
        }
    }
}
