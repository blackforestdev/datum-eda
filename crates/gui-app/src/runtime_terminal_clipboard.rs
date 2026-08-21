use anyhow::{Context, Result};
use datum_gui_protocol::{
    ApplicationFocus, DockTab, TerminalClipboardConfirmation, TerminalClipboardMenuState,
    TerminalClipboardSelection,
};
use datum_gui_render::HitTarget;
use datum_terminal_core::{Base64Limits, ClipboardSelection, CoreLimits, decode_base64};
use winit::{
    event::{ElementState, KeyEvent, MouseButton},
    keyboard::{Key, NamedKey},
};

use crate::{
    Runtime, terminal_core_adapter::PRODUCTION_CORE_LIMIT_VALUES,
    terminal_session::TerminalClipboardWriteRequest,
};

pub(super) struct PendingTerminalClipboardWrite {
    session_id: String,
    selection: TerminalClipboardSelection,
    text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClipboardConfirmationKeyAction {
    Unowned,
    Consume,
    Cancel,
    Confirm,
    FinishEscapeRelease,
}

impl Runtime {
    pub(super) fn handle_terminal_clipboard_write_request(
        &mut self,
        request: TerminalClipboardWriteRequest,
    ) -> bool {
        let active = self.workspace().ui.terminal.active_session_id.as_deref();
        if !clipboard_request_is_eligible(
            self.application_focus() == ApplicationFocus::Terminal
                && self.workspace().ui.active_dock_tab == Some(DockTab::Terminal),
            active == Some(request.session_id.as_str()),
            self.workspace().ui.terminal.status == "running",
            self.workspace().ui.terminal.link_confirmation.is_none()
                && self
                    .workspace()
                    .ui
                    .terminal
                    .application_shutdown_blocked
                    .is_none()
                && self.pending_terminal_clipboard_write.is_none(),
        ) {
            self.log_review_event("terminal clipboard write request denied".to_string());
            return false;
        }
        let selection = match request.selection {
            ClipboardSelection::Clipboard => TerminalClipboardSelection::Clipboard,
            ClipboardSelection::Primary => TerminalClipboardSelection::Primary,
            ClipboardSelection::Select => {
                self.log_review_event("unsupported terminal select-clipboard request denied");
                return false;
            }
        };
        let text = match decode_clipboard_text(&request.encoded_contents) {
            Ok(text) => text,
            Err(error) => {
                self.log_review_event(format!("terminal clipboard payload rejected: {error}"));
                return false;
            }
        };
        let byte_count = text.len();
        self.pending_terminal_clipboard_write = Some(PendingTerminalClipboardWrite {
            session_id: request.session_id,
            selection,
            text,
        });
        let terminal = &mut self.session.workspace_mut().ui.terminal;
        terminal.clipboard_confirmation = Some(TerminalClipboardConfirmation {
            selection,
            byte_count,
        });
        terminal.clipboard_escape_release_pending = false;
        self.invalidate_frame();
        true
    }

    pub(super) fn handle_terminal_clipboard_confirmation_key(&mut self, event: &KeyEvent) -> bool {
        if self
            .workspace()
            .ui
            .terminal
            .clipboard_confirmation
            .is_some()
            && (self.workspace().ui.terminal.status != "running"
                || self
                    .workspace()
                    .ui
                    .terminal
                    .application_shutdown_blocked
                    .is_some())
        {
            self.cancel_terminal_clipboard_write();
            return false;
        }
        let terminal = &self.workspace().ui.terminal;
        match clipboard_confirmation_key_action(
            terminal.clipboard_confirmation.is_some(),
            terminal.clipboard_escape_release_pending,
            event.state,
            &event.logical_key,
        ) {
            ClipboardConfirmationKeyAction::Unowned => false,
            ClipboardConfirmationKeyAction::FinishEscapeRelease => {
                self.session
                    .workspace_mut()
                    .ui
                    .terminal
                    .clipboard_escape_release_pending = false;
                true
            }
            ClipboardConfirmationKeyAction::Cancel => {
                self.session
                    .workspace_mut()
                    .ui
                    .terminal
                    .clipboard_escape_release_pending = true;
                self.cancel_terminal_clipboard_write();
                true
            }
            ClipboardConfirmationKeyAction::Confirm => {
                self.confirm_terminal_clipboard_write();
                true
            }
            ClipboardConfirmationKeyAction::Consume => true,
        }
    }

    pub(super) fn cancel_terminal_clipboard_write(&mut self) -> bool {
        self.pending_terminal_clipboard_write = None;
        let removed = self
            .session
            .workspace_mut()
            .ui
            .terminal
            .clipboard_confirmation
            .take()
            .is_some();
        if removed {
            self.invalidate_frame();
        }
        removed
    }

    pub(super) fn confirm_terminal_clipboard_write(&mut self) -> bool {
        let Some(pending) = self.pending_terminal_clipboard_write.take() else {
            return false;
        };
        let active = self.workspace().ui.terminal.active_session_id.as_deref();
        if self.application_focus() != ApplicationFocus::Terminal
            || self.workspace().ui.active_dock_tab != Some(DockTab::Terminal)
            || active != Some(pending.session_id.as_str())
            || self.workspace().ui.terminal.status != "running"
        {
            self.session
                .workspace_mut()
                .ui
                .terminal
                .clipboard_confirmation = None;
            self.invalidate_frame();
            return false;
        }
        let result = self.write_terminal_clipboard_text(pending.selection, &pending.text);
        self.session
            .workspace_mut()
            .ui
            .terminal
            .clipboard_confirmation = None;
        if let Err(error) = result {
            self.log_review_event(format!("terminal clipboard write failed: {error}"));
        } else {
            self.log_review_event("confirmed terminal clipboard write".to_string());
        }
        self.invalidate_frame();
        true
    }

    pub(super) fn open_terminal_clipboard_menu_at_cursor(&mut self) -> bool {
        let Some((x, y)) = self.last_cursor_pos else {
            return false;
        };
        if !terminal_context_menu_should_open(
            self.prepared_scene().hit_test(x, y),
            MouseButton::Right,
            ElementState::Pressed,
        ) {
            return false;
        }
        let link = self.terminal_link_target_at_cursor();
        let ui = &mut self.session.workspace_mut().ui;
        ui.active_menu = None;
        ui.marking_menu = None;
        ui.terminal_clipboard_menu = Some(TerminalClipboardMenuState {
            anchor_x: x,
            anchor_y: y,
            link,
        });
        self.set_application_focus(ApplicationFocus::Terminal);
        self.invalidate_frame();
        true
    }

    pub(super) fn terminal_clipboard_menu_active(&self) -> bool {
        self.workspace().ui.terminal_clipboard_menu.is_some()
    }

    pub(super) fn terminal_clipboard_link_target(
        &self,
    ) -> Option<datum_gui_protocol::TerminalLinkTarget> {
        self.workspace()
            .ui
            .terminal_clipboard_menu
            .as_ref()
            .and_then(|menu| menu.link.clone())
    }

    pub(super) fn dismiss_terminal_clipboard_menu(&mut self) -> bool {
        if self
            .session
            .workspace_mut()
            .ui
            .terminal_clipboard_menu
            .take()
            .is_none()
        {
            return false;
        }
        self.invalidate_frame();
        true
    }
}

fn decode_clipboard_text(encoded: &[u8]) -> Result<String> {
    let limits = CoreLimits::try_from(PRODUCTION_CORE_LIMIT_VALUES)
        .context("materialize terminal clipboard limits")?;
    let decoded = decode_base64(
        encoded,
        Base64Limits::clipboard(limits.clipboard_bytes, limits.parser_work),
    )
    .context("decode terminal clipboard base64")?;
    String::from_utf8(decoded).map_err(|_| anyhow::anyhow!("clipboard payload is not UTF-8 text"))
}

const fn clipboard_request_is_eligible(
    terminal_focused: bool,
    active_session: bool,
    running: bool,
    prompt_clear: bool,
) -> bool {
    terminal_focused && active_session && running && prompt_clear
}

fn clipboard_confirmation_key_action(
    active: bool,
    escape_release_pending: bool,
    state: ElementState,
    key: &Key,
) -> ClipboardConfirmationKeyAction {
    if !active {
        return if escape_release_pending
            && state == ElementState::Released
            && matches!(key, Key::Named(NamedKey::Escape))
        {
            ClipboardConfirmationKeyAction::FinishEscapeRelease
        } else {
            ClipboardConfirmationKeyAction::Unowned
        };
    }
    if state == ElementState::Released {
        return ClipboardConfirmationKeyAction::Consume;
    }
    match key {
        Key::Named(NamedKey::Escape) => ClipboardConfirmationKeyAction::Cancel,
        Key::Named(NamedKey::Enter) => ClipboardConfirmationKeyAction::Confirm,
        _ => ClipboardConfirmationKeyAction::Consume,
    }
}

fn terminal_context_menu_should_open(
    target: Option<&HitTarget>,
    button: MouseButton,
    state: ElementState,
) -> bool {
    button == MouseButton::Right
        && state == ElementState::Pressed
        && target == Some(&HitTarget::TerminalScreen)
}

#[cfg(test)]
mod tests {
    use super::{
        ClipboardConfirmationKeyAction, clipboard_confirmation_key_action,
        clipboard_request_is_eligible, decode_clipboard_text, terminal_context_menu_should_open,
    };
    use datum_gui_protocol::TerminalClipboardMenuState;
    use datum_gui_render::HitTarget;
    use winit::{
        event::{ElementState, MouseButton},
        keyboard::{Key, NamedKey},
    };

    #[test]
    fn osc52_payload_is_decoded_only_as_bounded_utf8_text() {
        assert_eq!(decode_clipboard_text(b"RGF0dW0=").unwrap(), "Datum");
        assert!(decode_clipboard_text(b"/w==").is_err());
        assert!(decode_clipboard_text(b"not-base64").is_err());
    }

    #[test]
    fn clipboard_confirmation_exclusively_owns_enter_escape_and_other_keys() {
        assert_eq!(
            clipboard_confirmation_key_action(
                true,
                false,
                ElementState::Pressed,
                &Key::Named(NamedKey::Enter),
            ),
            ClipboardConfirmationKeyAction::Confirm
        );
        assert_eq!(
            clipboard_confirmation_key_action(
                true,
                false,
                ElementState::Pressed,
                &Key::Named(NamedKey::Escape),
            ),
            ClipboardConfirmationKeyAction::Cancel
        );
        assert_eq!(
            clipboard_confirmation_key_action(
                true,
                false,
                ElementState::Pressed,
                &Key::Character("x".into()),
            ),
            ClipboardConfirmationKeyAction::Consume
        );
        assert_eq!(
            clipboard_confirmation_key_action(
                false,
                true,
                ElementState::Released,
                &Key::Named(NamedKey::Escape),
            ),
            ClipboardConfirmationKeyAction::FinishEscapeRelease
        );
    }

    #[test]
    fn only_a_focused_active_running_session_may_arm_osc52_confirmation() {
        assert!(clipboard_request_is_eligible(true, true, true, true));
        for eligibility in [
            (false, true, true, true),
            (true, false, true, true),
            (true, true, false, true),
            (true, true, true, false),
        ] {
            assert!(!clipboard_request_is_eligible(
                eligibility.0,
                eligibility.1,
                eligibility.2,
                eligibility.3,
            ));
        }
    }

    #[test]
    fn clipboard_menu_state_is_transient_screen_space() {
        let menu = TerminalClipboardMenuState {
            anchor_x: 120.0,
            anchor_y: 480.0,
            link: None,
        };
        assert_eq!(menu.anchor_x, 120.0);
        assert_eq!(menu.anchor_y, 480.0);
    }

    #[test]
    fn only_a_terminal_screen_secondary_press_opens_the_clipboard_menu() {
        assert!(terminal_context_menu_should_open(
            Some(&HitTarget::TerminalScreen),
            MouseButton::Right,
            ElementState::Pressed,
        ));
        for (target, button, state) in [
            (
                Some(&HitTarget::TerminalScreen),
                MouseButton::Right,
                ElementState::Released,
            ),
            (
                Some(&HitTarget::TerminalScreen),
                MouseButton::Left,
                ElementState::Pressed,
            ),
            (None, MouseButton::Right, ElementState::Pressed),
        ] {
            assert!(!terminal_context_menu_should_open(target, button, state));
        }
    }
}
