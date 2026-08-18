use datum_gui_protocol::ApplicationFocus;
use winit::event::{ElementState, MouseButton};

use crate::terminal_input::{
    terminal_sgr_mouse_button_sequence, terminal_sgr_mouse_motion_sequence,
    terminal_sgr_mouse_wheel_sequence, terminal_urxvt_mouse_button_sequence,
    terminal_urxvt_mouse_motion_sequence, terminal_urxvt_mouse_wheel_sequence,
    terminal_utf8_mouse_button_sequence, terminal_utf8_mouse_motion_sequence,
    terminal_utf8_mouse_wheel_sequence, terminal_x10_mouse_button_sequence,
    terminal_x10_mouse_motion_sequence, terminal_x10_mouse_wheel_sequence,
};
use crate::{Runtime, keyboard_focus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TerminalSelectionPoint {
    row: usize,
    column: usize,
}

impl Runtime {
    pub(super) fn begin_terminal_text_selection(&mut self) -> bool {
        if self.workspace().ui.terminal.mouse_reporting_mode.is_some()
            && !self.modifiers.shift_key()
        {
            return false;
        }
        let Some(point) = self
            .last_cursor_pos
            .and_then(|pointer| self.terminal_text_point_at(pointer, false))
        else {
            return false;
        };
        if self.application_focus() != ApplicationFocus::Terminal {
            self.set_application_focus(ApplicationFocus::Terminal);
        }
        self.terminal_text_selection_drag = Some(point);
        self.session
            .workspace_mut()
            .ui
            .terminal
            .clear_text_selection();
        self.invalidate_frame();
        true
    }

    pub(super) fn advance_terminal_text_selection(&mut self, pointer: (f32, f32)) -> bool {
        let Some(anchor) = self.terminal_text_selection_drag else {
            return false;
        };
        let Some(focus) = self.terminal_text_point_at(pointer, true) else {
            return true;
        };
        let selection = (focus != anchor).then_some((anchor, focus));
        let current = self.workspace().ui.terminal.text_selection_ordered();
        let next = selection.map(|(anchor, focus)| {
            let a = (anchor.row, anchor.column);
            let f = (focus.row, focus.column);
            if a <= f { (a, f) } else { (f, a) }
        });
        if current != next {
            let terminal = &mut self.session.workspace_mut().ui.terminal;
            if let Some((anchor, focus)) = selection {
                terminal.set_text_selection((anchor.row, anchor.column), (focus.row, focus.column));
            } else {
                terminal.clear_text_selection();
            }
            self.invalidate_frame();
        }
        true
    }

    pub(super) fn finish_terminal_text_selection(&mut self) -> bool {
        self.terminal_text_selection_drag.take().is_some()
    }

    pub(super) fn cancel_terminal_text_selection_drag(&mut self) -> bool {
        self.terminal_text_selection_drag.take().is_some()
    }

    fn terminal_text_point_at(
        &self,
        pointer: (f32, f32),
        clamp_to_screen: bool,
    ) -> Option<TerminalSelectionPoint> {
        let geometry = self.terminal_screen_geometry();
        let (column, visible_row) = if clamp_to_screen {
            let screen = geometry.screen;
            let x = pointer
                .0
                .clamp(screen.x, screen.x + screen.width - f32::EPSILON);
            let y = pointer
                .1
                .clamp(screen.y, screen.y + screen.height - f32::EPSILON);
            geometry.cell_at(x, y)?
        } else {
            geometry.cell_at(pointer.0, pointer.1)?
        };
        terminal_grid_point(
            self.workspace().ui.terminal.grid_lines().len(),
            geometry.rows as usize,
            self.workspace().ui.terminal.scroll_offset,
            column as usize,
            visible_row as usize,
        )
    }

    pub(super) fn report_terminal_mouse_button(
        &mut self,
        button: MouseButton,
        state: ElementState,
    ) -> bool {
        if !self.terminal_mouse_reporting_active() {
            return false;
        }
        let Some((column, row)) = self.terminal_mouse_cell() else {
            return false;
        };
        let pressed = state == ElementState::Pressed;
        let Some(bytes) = self.terminal_mouse_encoding_sequence(|encoding| match encoding {
            Some("sgr") => terminal_sgr_mouse_button_sequence(button, pressed, column, row),
            Some("urxvt") => terminal_urxvt_mouse_button_sequence(button, pressed, column, row),
            Some("utf8") => terminal_utf8_mouse_button_sequence(button, pressed, column, row),
            None => terminal_x10_mouse_button_sequence(button, pressed, column, row),
            _ => None,
        }) else {
            return false;
        };
        self.write_terminal_mouse_report(&bytes);
        self.terminal_mouse_button = if state == ElementState::Pressed {
            Some(button)
        } else {
            None
        };
        true
    }

    pub(super) fn report_terminal_mouse_motion(&mut self) -> bool {
        if !self.terminal_mouse_reporting_active() {
            return false;
        }
        let terminal = &self.workspace().ui.terminal;
        let held_button = match terminal.mouse_reporting_mode.as_deref() {
            Some("any_event") => self.terminal_mouse_button,
            Some("button_event") => {
                let Some(button) = self.terminal_mouse_button else {
                    return false;
                };
                Some(button)
            }
            _ => return false,
        };
        let Some((column, row)) = self.terminal_mouse_cell() else {
            return false;
        };
        let Some(bytes) = self.terminal_mouse_encoding_sequence(|encoding| match encoding {
            Some("sgr") => terminal_sgr_mouse_motion_sequence(held_button, column, row),
            Some("urxvt") => held_button
                .and_then(|button| terminal_urxvt_mouse_motion_sequence(button, column, row)),
            Some("utf8") => held_button
                .and_then(|button| terminal_utf8_mouse_motion_sequence(button, column, row)),
            None => held_button
                .and_then(|button| terminal_x10_mouse_motion_sequence(button, column, row)),
            _ => None,
        }) else {
            return false;
        };
        self.write_terminal_mouse_report(&bytes);
        true
    }

    pub(super) fn report_terminal_mouse_wheel(&mut self, scroll_lines: f32) -> bool {
        if !self.terminal_mouse_reporting_active() {
            return false;
        }
        let Some((column, row)) = self.terminal_mouse_cell() else {
            return false;
        };
        let Some(bytes) = self.terminal_mouse_encoding_sequence(|encoding| match encoding {
            Some("sgr") => terminal_sgr_mouse_wheel_sequence(scroll_lines, column, row),
            Some("urxvt") => terminal_urxvt_mouse_wheel_sequence(scroll_lines, column, row),
            Some("utf8") => terminal_utf8_mouse_wheel_sequence(scroll_lines, column, row),
            None => terminal_x10_mouse_wheel_sequence(scroll_lines, column, row),
            _ => None,
        }) else {
            return false;
        };
        self.write_terminal_mouse_report(&bytes);
        true
    }

    fn terminal_mouse_reporting_active(&self) -> bool {
        let terminal = &self.workspace().ui.terminal;
        !self.modifiers.shift_key()
            && keyboard_focus::terminal_mouse_report_allowed(
                self.application_focus(),
                terminal.mouse_reporting_mode.is_some(),
                self.terminal_sessions.active_attached(),
                self.last_cursor_pos
                    .and_then(|(x, y)| self.terminal_screen_cell_at(x, y))
                    .is_some(),
            )
    }

    fn terminal_mouse_encoding_sequence(
        &self,
        sequence: impl FnOnce(Option<&str>) -> Option<Vec<u8>>,
    ) -> Option<Vec<u8>> {
        sequence(
            self.workspace()
                .ui
                .terminal
                .mouse_coordinate_encoding
                .as_deref(),
        )
    }

    fn terminal_mouse_cell(&self) -> Option<(u16, u16)> {
        let (x, y) = self.last_cursor_pos?;
        self.terminal_screen_cell_at(x, y)
            .map(|(column, row)| (column.saturating_add(1), row.saturating_add(1)))
    }

    fn write_terminal_mouse_report(&mut self, bytes: &[u8]) {
        if let Err(err) = self.terminal_sessions.active().write_bytes(bytes) {
            self.log_review_event(format!("terminal mouse report failed: {err}"));
        }
    }
}

fn terminal_grid_point(
    total_lines: usize,
    visible_rows: usize,
    scroll_offset: usize,
    column: usize,
    visible_row: usize,
) -> Option<TerminalSelectionPoint> {
    if total_lines == 0 {
        return None;
    }
    let scroll = scroll_offset.min(total_lines.saturating_sub(visible_rows));
    let tail_start = total_lines.saturating_sub(visible_rows + scroll);
    let last_visible_row = total_lines.saturating_sub(tail_start + 1);
    Some(TerminalSelectionPoint {
        row: tail_start + visible_row.min(last_visible_row),
        column,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_rows_follow_the_visible_scrollback_window() {
        assert_eq!(
            terminal_grid_point(40, 10, 0, 7, 2),
            Some(TerminalSelectionPoint { row: 32, column: 7 })
        );
        assert_eq!(
            terminal_grid_point(40, 10, 5, 3, 9),
            Some(TerminalSelectionPoint { row: 34, column: 3 })
        );
        assert_eq!(terminal_grid_point(0, 10, 0, 0, 0), None);
    }

    #[test]
    fn pointer_rows_clamp_to_the_last_real_line_below_short_content() {
        assert_eq!(
            terminal_grid_point(2, 10, 0, 4, 8),
            Some(TerminalSelectionPoint { row: 1, column: 4 })
        );
    }
}
