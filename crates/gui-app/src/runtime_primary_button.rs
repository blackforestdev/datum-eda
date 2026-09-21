//! Primary-button press routing, kept separate from the window event loop so
//! rail input can reject terminal and scene work before expensive hit testing.

use super::*;

impl App {
    pub(super) fn handle_primary_button_press(&mut self) {
        let Some(runtime) = &mut self.runtime else {
            return;
        };
        if runtime.terminal_clipboard_menu_active() {
            return;
        }
        if runtime.cursor_in_dock() {
            if runtime.modifiers.control_key() && runtime.arm_terminal_link_at_cursor() {
                self.request_workspace_redraw();
                return;
            }
            if runtime.begin_terminal_split_drag() {
                let icon = runtime
                    .last_cursor_pos
                    .and_then(|pointer| runtime.terminal_split_cursor_icon(pointer))
                    .unwrap_or(winit::window::CursorIcon::Default);
                self.apply_cursor_icon(icon);
                return;
            }
            if runtime.begin_terminal_tab_drag() {
                self.apply_cursor_icon(winit::window::CursorIcon::Grabbing);
                return;
            }
            runtime.focus_terminal_screen_before_mouse_report();
            if runtime.begin_terminal_text_selection() {
                self.apply_cursor_icon(winit::window::CursorIcon::Text);
                self.request_workspace_redraw();
                return;
            }
            if runtime.report_terminal_mouse_button(MouseButton::Left, ElementState::Pressed) {
                return;
            }
        }
        if runtime.begin_primary_pan() {
            if runtime.clear_interaction_overlay() {
                self.request_workspace_redraw();
            }
            return;
        }
        let Some((x, y)) = runtime.last_cursor_pos else {
            return;
        };
        if runtime.begin_dock_resize_drag((x, y)) {
            self.request_workspace_redraw();
            return;
        }
        // Divider drag precedes click-to-focus so a gutter never selects a pane.
        if runtime.begin_divider_drag(x, y) {
            self.request_workspace_redraw();
        }
    }
}
