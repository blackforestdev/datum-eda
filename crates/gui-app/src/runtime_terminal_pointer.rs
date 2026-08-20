use datum_gui_protocol::ApplicationFocus;
use datum_terminal_core::{
    KeyModifiers, LogicalPoint, MouseAction, MouseButton as CoreMouseButton, MouseInput,
    MousePosition, SelectionScope,
};
use winit::event::{ElementState, MouseButton};

use crate::{Runtime, keyboard_focus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TerminalSelectionPoint {
    logical: LogicalPoint,
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
        let _ = self.terminal_sessions.clear_active_selection();
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
        let selection = (focus != anchor).then_some((anchor.logical, focus.logical));
        if let Some((anchor, focus)) = selection {
            let _ = self.terminal_sessions.set_active_selection(
                anchor,
                focus,
                SelectionScope::Grapheme,
            );
        } else {
            let _ = self.terminal_sessions.clear_active_selection();
        }
        {
            let terminal = &mut self.session.workspace_mut().ui.terminal;
            terminal.clear_text_selection();
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
        self.terminal_sessions
            .active_logical_point_at(
                usize::from(geometry.rows),
                self.workspace().ui.terminal.scroll_offset,
                usize::from(visible_row),
                usize::from(column),
            )
            .ok()
            .flatten()
            .map(|logical| TerminalSelectionPoint { logical })
    }

    pub(super) fn report_terminal_mouse_button(
        &mut self,
        button: MouseButton,
        state: ElementState,
    ) -> bool {
        if !self.terminal_mouse_reporting_active() {
            return false;
        }
        let Some(position) = self.terminal_mouse_position() else {
            return false;
        };
        let Some(core_button) = core_mouse_button(button) else {
            return false;
        };
        let action = if state == ElementState::Pressed {
            MouseAction::Press(core_button)
        } else {
            MouseAction::Release(core_button)
        };
        if !self.write_terminal_mouse_input(action, position) {
            return false;
        }
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
        let Some(position) = self.terminal_mouse_position() else {
            return false;
        };
        self.write_terminal_mouse_input(
            MouseAction::Move(self.terminal_mouse_button.and_then(core_mouse_button)),
            position,
        )
    }

    pub(super) fn report_terminal_mouse_wheel(&mut self, scroll_lines: f32) -> bool {
        if !self.terminal_mouse_reporting_active() {
            return false;
        }
        let Some(position) = self.terminal_mouse_position() else {
            return false;
        };
        let action = if scroll_lines < 0.0 {
            MouseAction::WheelDown
        } else {
            MouseAction::WheelUp
        };
        self.write_terminal_mouse_input(action, position)
    }

    fn terminal_mouse_reporting_active(&self) -> bool {
        !self.modifiers.shift_key()
            && keyboard_focus::terminal_mouse_report_allowed(
                self.application_focus(),
                true,
                self.terminal_sessions.active_attached(),
                self.last_cursor_pos
                    .and_then(|(x, y)| self.terminal_screen_cell_at(x, y))
                    .is_some(),
            )
    }

    fn terminal_mouse_position(&self) -> Option<MousePosition> {
        let (x, y) = self.last_cursor_pos?;
        let geometry = self.terminal_screen_geometry();
        let (column, row) = geometry.cell_at(x, y)?;
        Some(MousePosition {
            column: i64::from(column),
            row: i64::from(row),
            pixel_x: (x - geometry.screen.x).round() as i64,
            pixel_y: (y - geometry.screen.y).round() as i64,
        })
    }

    fn write_terminal_mouse_input(&mut self, action: MouseAction, position: MousePosition) -> bool {
        let input = MouseInput {
            action,
            position,
            modifiers: KeyModifiers {
                shift: self.modifiers.shift_key(),
                alt: self.modifiers.alt_key(),
                control: self.modifiers.control_key(),
                super_key: self.modifiers.super_key(),
                hyper: false,
                meta: false,
            },
            local_override: self.modifiers.shift_key(),
        };
        match self.terminal_sessions.encode_active_mouse(input) {
            Ok(Some(bytes)) => match self.terminal_sessions.active().write_bytes(&bytes) {
                Ok(()) => true,
                Err(err) => {
                    self.log_review_event(format!("terminal mouse report failed: {err}"));
                    true
                }
            },
            Ok(None) => false,
            Err(err) => {
                self.log_review_event(format!("terminal mouse encoding failed: {err}"));
                true
            }
        }
    }
}

fn core_mouse_button(button: MouseButton) -> Option<CoreMouseButton> {
    match button {
        MouseButton::Left => Some(CoreMouseButton::Left),
        MouseButton::Middle => Some(CoreMouseButton::Middle),
        MouseButton::Right => Some(CoreMouseButton::Right),
        _ => None,
    }
}

#[cfg(test)]
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
        logical: LogicalPoint {
            line: datum_terminal_core::LogicalLineId::new(
                (tail_start + visible_row.min(last_visible_row)) as u64,
            ),
            cluster: column as u32,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_rows_follow_the_visible_scrollback_window() {
        assert_eq!(
            terminal_grid_point(40, 10, 0, 7, 2),
            Some(TerminalSelectionPoint {
                logical: LogicalPoint {
                    line: datum_terminal_core::LogicalLineId::new(32),
                    cluster: 7
                }
            })
        );
        assert_eq!(
            terminal_grid_point(40, 10, 5, 3, 9),
            Some(TerminalSelectionPoint {
                logical: LogicalPoint {
                    line: datum_terminal_core::LogicalLineId::new(34),
                    cluster: 3
                }
            })
        );
        assert_eq!(terminal_grid_point(0, 10, 0, 0, 0), None);
    }

    #[test]
    fn pointer_rows_clamp_to_the_last_real_line_below_short_content() {
        assert_eq!(
            terminal_grid_point(2, 10, 0, 4, 8),
            Some(TerminalSelectionPoint {
                logical: LogicalPoint {
                    line: datum_terminal_core::LogicalLineId::new(1),
                    cluster: 4
                }
            })
        );
    }
}
