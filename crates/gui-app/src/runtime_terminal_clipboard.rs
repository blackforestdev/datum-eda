use datum_gui_protocol::{ApplicationFocus, TerminalClipboardMenuState};
use datum_gui_render::HitTarget;
use winit::event::{ElementState, MouseButton};

use crate::Runtime;

impl Runtime {
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
    use super::terminal_context_menu_should_open;
    use datum_gui_protocol::TerminalClipboardMenuState;
    use datum_gui_render::HitTarget;
    use winit::event::{ElementState, MouseButton};

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
