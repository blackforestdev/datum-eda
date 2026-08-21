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
}

impl Runtime {
    pub(super) fn write_foreign_shell_bytes(&mut self, bytes: &[u8]) -> bool {
        match write_attached_terminal_bytes(&self.terminal_sessions, bytes) {
            Ok(true) => {
                let _ = self.terminal_sessions.clear_active_selection();
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
            TerminalKeyAction::CoreKey(input) => {
                let bytes = self
                    .terminal_sessions
                    .encode_active_key(input)
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                if bytes == [0x03] {
                    return None;
                }
                self.terminal_sessions.handle_close_confirmation_input(
                    &bytes,
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

    pub(super) fn handle_terminal_ime(&mut self, ime: &winit::event::Ime) -> bool {
        if self.terminal_input_owner() != keyboard_focus::TerminalInputOwner::AttachedPty {
            return false;
        }
        let input = match ime {
            winit::event::Ime::Preedit(text, _) => {
                datum_terminal_core::ImeInput::Preedit(text.into())
            }
            winit::event::Ime::Commit(text) => datum_terminal_core::ImeInput::Commit(text.into()),
            winit::event::Ime::Disabled => datum_terminal_core::ImeInput::Disabled,
            winit::event::Ime::Enabled => {
                self.session.workspace_mut().ui.terminal.ime_preedit = None;
                return true;
            }
        };
        match self.terminal_sessions.encode_active_ime(&input) {
            Ok(Some(bytes)) => {
                self.session.workspace_mut().ui.terminal.ime_preedit = None;
                self.write_foreign_shell_bytes(&bytes)
            }
            Ok(None) => {
                self.session.workspace_mut().ui.terminal.ime_preedit = match input {
                    datum_terminal_core::ImeInput::Preedit(text) if !text.is_empty() => Some(text),
                    _ => None,
                };
                self.invalidate_frame();
                true
            }
            Err(err) => {
                self.log_review_event(format!("terminal IME encoding failed: {err}"));
                true
            }
        }
    }

    pub(super) fn terminal_ime_cursor_rect(&self) -> (f64, f64, u32, u32) {
        let geometry = self.terminal_screen_geometry();
        let cursor = self.workspace().ui.terminal.screen_cursor_col as f32;
        let row = self.workspace().ui.terminal.screen_cursor_row as f32;
        (
            f64::from(geometry.screen.x + cursor * geometry.metrics.width),
            f64::from(geometry.screen.y + row * geometry.metrics.height),
            geometry.metrics.width.ceil() as u32,
            geometry.metrics.height.ceil() as u32,
        )
    }
}

#[cfg(test)]
#[path = "terminal_input_mode_boundary_tests.rs"]
mod tests;
