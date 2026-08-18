//! Cell-exact rendering for Unicode block elements used by terminal TUIs.
//!
//! Font glyph ink is intentionally smaller than the terminal's logical cell so
//! ordinary adjacent text remains readable. Block elements are different: their
//! contract is to tile the cell, and shaping them as text leaves visible seams
//! between rows. Render the geometric subset as exact cell quads and replace
//! those scalars with advance-preserving spaces in the shaped text run.

use datum_gui_protocol::TerminalStyledLine;
use datum_gui_viewport::{TERMINAL_CELL_HEIGHT_PX, TERMINAL_CELL_WIDTH_PX};

use super::terminal_color::span_foreground;
use super::{Quad, RectPx, TEXT_PANEL_VALUE};

pub(super) fn render_terminal_block_elements(
    styled_line: Option<&TerminalStyledLine>,
    fallback_line: &str,
    x: f32,
    y: f32,
    max_columns: usize,
    quads: &mut Vec<Quad>,
) {
    let text = styled_line
        .filter(|line| !line.text.is_empty())
        .map_or(fallback_line, |line| line.text.as_str());
    for (column, glyph) in text.chars().take(max_columns).enumerate() {
        let Some(parts) = block_parts(glyph) else {
            continue;
        };
        let color = styled_line
            .and_then(|line| {
                line.spans
                    .iter()
                    .find(|span| span.start <= column && column < span.end)
            })
            .map_or(TEXT_PANEL_VALUE, |span| {
                span_foreground(
                    span.fg.as_deref(),
                    span.bg.as_deref(),
                    span.bold,
                    span.inverse,
                    span.conceal,
                )
            });
        for &(part_x, part_y, width, height) in parts {
            quads.push(Quad::from_rect(
                RectPx {
                    x: x + (column as f32 + part_x) * TERMINAL_CELL_WIDTH_PX,
                    y: y + part_y * TERMINAL_CELL_HEIGHT_PX,
                    width: width * TERMINAL_CELL_WIDTH_PX,
                    height: height * TERMINAL_CELL_HEIGHT_PX,
                },
                color,
            ));
        }
    }
}

pub(super) fn text_without_geometric_blocks(text: &str) -> String {
    text.chars()
        .map(|glyph| {
            if block_parts(glyph).is_some() {
                ' '
            } else {
                glyph
            }
        })
        .collect()
}

type BlockParts = &'static [(f32, f32, f32, f32)];

fn block_parts(glyph: char) -> Option<BlockParts> {
    const FULL: BlockParts = &[(0.0, 0.0, 1.0, 1.0)];
    const UPPER_HALF: BlockParts = &[(0.0, 0.0, 1.0, 0.5)];
    const LOWER_HALF: BlockParts = &[(0.0, 0.5, 1.0, 0.5)];
    const LEFT_HALF: BlockParts = &[(0.0, 0.0, 0.5, 1.0)];
    const RIGHT_HALF: BlockParts = &[(0.5, 0.0, 0.5, 1.0)];
    const UPPER_LEFT: BlockParts = &[(0.0, 0.0, 0.5, 0.5)];
    const UPPER_RIGHT: BlockParts = &[(0.5, 0.0, 0.5, 0.5)];
    const LOWER_LEFT: BlockParts = &[(0.0, 0.5, 0.5, 0.5)];
    const LOWER_RIGHT: BlockParts = &[(0.5, 0.5, 0.5, 0.5)];
    const UPPER_LEFT_LOWER_RIGHT: BlockParts = &[(0.0, 0.0, 0.5, 0.5), (0.5, 0.5, 0.5, 0.5)];
    const UPPER_RIGHT_LOWER_LEFT: BlockParts = &[(0.5, 0.0, 0.5, 0.5), (0.0, 0.5, 0.5, 0.5)];
    const UPPER_AND_LOWER_LEFT: BlockParts = &[(0.0, 0.0, 1.0, 0.5), (0.0, 0.5, 0.5, 0.5)];
    const UPPER_AND_LOWER_RIGHT: BlockParts = &[(0.0, 0.0, 1.0, 0.5), (0.5, 0.5, 0.5, 0.5)];
    const LEFT_AND_LOWER_RIGHT: BlockParts = &[(0.0, 0.0, 0.5, 1.0), (0.5, 0.5, 0.5, 0.5)];
    const RIGHT_AND_LOWER_LEFT: BlockParts = &[(0.5, 0.0, 0.5, 1.0), (0.0, 0.5, 0.5, 0.5)];

    match glyph {
        '▀' => Some(UPPER_HALF),
        '▁' => Some(&[(0.0, 0.875, 1.0, 0.125)]),
        '▂' => Some(&[(0.0, 0.75, 1.0, 0.25)]),
        '▃' => Some(&[(0.0, 0.625, 1.0, 0.375)]),
        '▄' => Some(LOWER_HALF),
        '▅' => Some(&[(0.0, 0.375, 1.0, 0.625)]),
        '▆' => Some(&[(0.0, 0.25, 1.0, 0.75)]),
        '▇' => Some(&[(0.0, 0.125, 1.0, 0.875)]),
        '█' => Some(FULL),
        '▉' => Some(&[(0.0, 0.0, 0.875, 1.0)]),
        '▊' => Some(&[(0.0, 0.0, 0.75, 1.0)]),
        '▋' => Some(&[(0.0, 0.0, 0.625, 1.0)]),
        '▌' => Some(LEFT_HALF),
        '▍' => Some(&[(0.0, 0.0, 0.375, 1.0)]),
        '▎' => Some(&[(0.0, 0.0, 0.25, 1.0)]),
        '▏' => Some(&[(0.0, 0.0, 0.125, 1.0)]),
        '▐' => Some(RIGHT_HALF),
        '▔' => Some(&[(0.0, 0.0, 1.0, 0.125)]),
        '▕' => Some(&[(0.875, 0.0, 0.125, 1.0)]),
        '▖' => Some(LOWER_LEFT),
        '▗' => Some(LOWER_RIGHT),
        '▘' => Some(UPPER_LEFT),
        '▙' => Some(LEFT_AND_LOWER_RIGHT),
        '▚' => Some(UPPER_LEFT_LOWER_RIGHT),
        '▛' => Some(UPPER_AND_LOWER_LEFT),
        '▜' => Some(UPPER_AND_LOWER_RIGHT),
        '▝' => Some(UPPER_RIGHT),
        '▞' => Some(UPPER_RIGHT_LOWER_LEFT),
        '▟' => Some(RIGHT_AND_LOWER_LEFT),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_mascot_blocks_use_cell_exact_geometry_and_leave_text_advance() {
        let mut quads = Vec::new();
        render_terminal_block_elements(None, "▛███▜", 10.0, 20.0, 80, &mut quads);

        assert_eq!(
            text_without_geometric_blocks("▛███▜ Claude"),
            "      Claude"
        );
        assert!(
            quads
                .iter()
                .flat_map(|quad| quad.points)
                .all(|(_, point_y)| (20.0..=20.0 + TERMINAL_CELL_HEIGHT_PX).contains(&point_y))
        );
        let full_blocks = quads
            .iter()
            .filter(|quad| {
                let top = quad
                    .points
                    .iter()
                    .map(|(_, point_y)| *point_y)
                    .fold(f32::INFINITY, f32::min);
                let bottom = quad
                    .points
                    .iter()
                    .map(|(_, point_y)| *point_y)
                    .fold(f32::NEG_INFINITY, f32::max);
                bottom - top == TERMINAL_CELL_HEIGHT_PX
            })
            .count();
        assert_eq!(full_blocks, 3);
    }
}
