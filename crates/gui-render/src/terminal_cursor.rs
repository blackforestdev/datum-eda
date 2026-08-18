//! Terminal cursor presentation (decision 024, TF-04).
//!
//! The child application's DECSCUSR shape and DEC visibility remain
//! authoritative. Datum projects only keyboard ownership: focused shapes are
//! filled; unfocused shapes are hollow. Geometry-backed rendering avoids
//! depending on font coverage for cursor glyphs.

use datum_gui_protocol::TerminalLaneState;
use datum_gui_viewport::{TERMINAL_CELL_HEIGHT_PX, TERMINAL_CELL_WIDTH_PX};

use super::{Quad, RectPx, TEXT_ACCENT, TEXT_MUTED, push_rect_border};
use crate::design_tokens;

const CURSOR_STROKE_PX: f32 = 1.0;
// Keep the painted cursor visibly inside its logical cell. JetBrains Mono's
// right side bearing is narrower than one device pixel at the governed terminal
// size, so painting from the exact cell boundary visually fuses a block/bar to
// the preceding glyph even though the parser column is correct.
const CURSOR_HORIZONTAL_INSET_PX: f32 = 1.0;
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
    let inset_x = x + CURSOR_HORIZONTAL_INSET_PX;
    let inset_width = TERMINAL_CELL_WIDTH_PX - 2.0 * CURSOR_HORIZONTAL_INSET_PX;
    match cursor_shape(style) {
        CursorShape::Block => RectPx {
            x: inset_x,
            y,
            width: inset_width,
            height: TERMINAL_CELL_HEIGHT_PX,
        },
        CursorShape::Underline => RectPx {
            x: inset_x,
            y: y + TERMINAL_CELL_HEIGHT_PX - CURSOR_UNDERLINE_HEIGHT_PX,
            width: inset_width,
            height: CURSOR_UNDERLINE_HEIGHT_PX,
        },
        CursorShape::Bar => RectPx {
            x: inset_x,
            y,
            width: CURSOR_BAR_WIDTH_PX,
            height: TERMINAL_CELL_HEIGHT_PX,
        },
    }
}

pub(super) fn render_terminal_cursor(
    terminal: &TerminalLaneState,
    has_keyboard_focus: bool,
    origin_x: f32,
    y: f32,
    over_selection: bool,
    quads: &mut Vec<Quad>,
) {
    let x = origin_x + terminal.screen_cursor_col as f32 * TERMINAL_CELL_WIDTH_PX;
    let rect = cursor_rect(terminal.screen_cursor_style.as_deref(), x, y);
    let color = if over_selection {
        design_tokens::chrome::TEXT_PRIMARY
    } else if has_keyboard_focus {
        TEXT_ACCENT
    } else {
        TEXT_MUTED
    };
    if has_keyboard_focus {
        quads.push(Quad::from_rect(rect, color));
    } else {
        push_rect_border(quads, rect, color, CURSOR_STROKE_PX);
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
        render_terminal_cursor(&terminal, false, 10.0, 20.0, false, &mut unfocused);
        assert_eq!(unfocused.len(), 4, "unfocused cursor must be hollow");
        assert!(unfocused.iter().all(|quad| quad.color == TEXT_MUTED));

        let mut focused = Vec::new();
        render_terminal_cursor(&terminal, true, 10.0, 20.0, false, &mut focused);
        assert_eq!(focused, vec![Quad::from_rect(expected, TEXT_ACCENT)]);

        let mut selected = Vec::new();
        render_terminal_cursor(&terminal, true, 10.0, 20.0, true, &mut selected);
        assert_eq!(
            selected,
            vec![Quad::from_rect(
                expected,
                design_tokens::chrome::TEXT_PRIMARY
            )]
        );
    }

    #[test]
    fn decscusr_shapes_keep_distinct_geometry() {
        let block = cursor_rect(None, 0.0, 0.0);
        let underline = cursor_rect(Some("steady_underline"), 0.0, 0.0);
        let bar = cursor_rect(Some("blinking_bar"), 0.0, 0.0);

        assert_eq!(
            block.width,
            TERMINAL_CELL_WIDTH_PX - 2.0 * CURSOR_HORIZONTAL_INSET_PX
        );
        assert_eq!(block.height, TERMINAL_CELL_HEIGHT_PX);
        assert_eq!(underline.width, block.width);
        assert_eq!(underline.height, CURSOR_UNDERLINE_HEIGHT_PX);
        assert_eq!(bar.width, CURSOR_BAR_WIDTH_PX);
        assert_eq!(bar.height, TERMINAL_CELL_HEIGHT_PX);
    }

    #[test]
    fn trailing_slash_cursor_paint_stays_inside_the_next_logical_cell() {
        let prompt = "bfadmin@debian3520:/tmp/datum-eda/gui-imports/DOA2526-5825f90fe7490128$ cd ~/Documents/datum-eda/";
        let origin_x = 94.0;
        let logical_cell_x = origin_x + prompt.chars().count() as f32 * TERMINAL_CELL_WIDTH_PX;
        let block = cursor_rect(None, logical_cell_x, 802.0);

        assert_eq!(block.x, logical_cell_x + CURSOR_HORIZONTAL_INSET_PX);
        assert_eq!(
            block.x + block.width,
            logical_cell_x + TERMINAL_CELL_WIDTH_PX - CURSOR_HORIZONTAL_INSET_PX
        );
        assert!(
            block.x > logical_cell_x,
            "painted cursor must leave a visible gutter after the trailing slash"
        );
    }
}
