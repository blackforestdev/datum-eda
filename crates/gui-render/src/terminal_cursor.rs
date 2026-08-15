//! Terminal cursor presentation (decision 024, TF-04).
//!
//! The child application's DECSCUSR shape and DEC visibility remain
//! authoritative. Datum projects only keyboard ownership: focused shapes are
//! filled; unfocused shapes are hollow. Geometry-backed rendering avoids
//! depending on font coverage for cursor glyphs.

use datum_gui_protocol::TerminalLaneState;
use datum_gui_viewport::{TERMINAL_CELL_HEIGHT_PX, TERMINAL_CELL_WIDTH_PX};

use super::{Quad, RectPx, TEXT_ACCENT, TEXT_MUTED, push_rect_border};

const CURSOR_STROKE_PX: f32 = 1.0;
const CURSOR_BAR_WIDTH_PX: f32 = 3.0;
const CURSOR_UNDERLINE_HEIGHT_PX: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CursorShape {
    Block,
    Underline,
    Bar,
}

fn cursor_shape(style: Option<&str>) -> CursorShape {
    match style {
        Some("blinking_underline" | "steady_underline") => CursorShape::Underline,
        Some("blinking_bar" | "steady_bar") => CursorShape::Bar,
        _ => CursorShape::Block,
    }
}

fn cursor_rect(style: Option<&str>, x: f32, y: f32) -> RectPx {
    match cursor_shape(style) {
        CursorShape::Block => RectPx {
            x,
            y,
            width: TERMINAL_CELL_WIDTH_PX,
            height: TERMINAL_CELL_HEIGHT_PX,
        },
        CursorShape::Underline => RectPx {
            x,
            y: y + TERMINAL_CELL_HEIGHT_PX - CURSOR_UNDERLINE_HEIGHT_PX,
            width: TERMINAL_CELL_WIDTH_PX,
            height: CURSOR_UNDERLINE_HEIGHT_PX,
        },
        CursorShape::Bar => RectPx {
            x,
            y,
            width: CURSOR_BAR_WIDTH_PX,
            height: TERMINAL_CELL_HEIGHT_PX,
        },
    }
}

pub(super) fn render_terminal_cursor(
    terminal: &TerminalLaneState,
    origin_x: f32,
    y: f32,
    quads: &mut Vec<Quad>,
) {
    let x = origin_x + terminal.screen_cursor_col as f32 * TERMINAL_CELL_WIDTH_PX;
    let rect = cursor_rect(terminal.screen_cursor_style.as_deref(), x, y);
    if terminal.has_keyboard_focus {
        quads.push(Quad::from_rect(rect, TEXT_ACCENT));
    } else {
        push_rect_border(quads, rect, TEXT_MUTED, CURSOR_STROKE_PX);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_changes_fill_without_replacing_child_shape() {
        let mut terminal = TerminalLaneState::default();
        terminal.screen_cursor_col = 2;
        terminal.screen_cursor_style = Some("steady_bar".to_string());
        let expected = cursor_rect(
            terminal.screen_cursor_style.as_deref(),
            10.0 + 2.0 * TERMINAL_CELL_WIDTH_PX,
            20.0,
        );

        let mut unfocused = Vec::new();
        render_terminal_cursor(&terminal, 10.0, 20.0, &mut unfocused);
        assert_eq!(unfocused.len(), 4, "unfocused cursor must be hollow");
        assert!(unfocused.iter().all(|quad| quad.color == TEXT_MUTED));

        terminal.has_keyboard_focus = true;
        let mut focused = Vec::new();
        render_terminal_cursor(&terminal, 10.0, 20.0, &mut focused);
        assert_eq!(focused, vec![Quad::from_rect(expected, TEXT_ACCENT)]);
    }

    #[test]
    fn decscusr_shapes_keep_distinct_geometry() {
        let block = cursor_rect(None, 0.0, 0.0);
        let underline = cursor_rect(Some("steady_underline"), 0.0, 0.0);
        let bar = cursor_rect(Some("blinking_bar"), 0.0, 0.0);

        assert_eq!(block.width, TERMINAL_CELL_WIDTH_PX);
        assert_eq!(block.height, TERMINAL_CELL_HEIGHT_PX);
        assert_eq!(underline.width, TERMINAL_CELL_WIDTH_PX);
        assert_eq!(underline.height, CURSOR_UNDERLINE_HEIGHT_PX);
        assert_eq!(bar.width, CURSOR_BAR_WIDTH_PX);
        assert_eq!(bar.height, TERMINAL_CELL_HEIGHT_PX);
    }
}
