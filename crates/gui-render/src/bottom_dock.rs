use datum_gui_protocol::{DockTab, ReviewWorkspaceState, TerminalStyledLine};
use datum_gui_viewport::{
    TERMINAL_CELL_HEIGHT_PX, TERMINAL_CELL_WIDTH_PX, TerminalScreenGeometry,
    terminal_screen_geometry,
};

use super::{
    HitRegion, HitTarget, PANEL_BG, PANEL_CARD_BORDER, Quad, RectPx, ShellLayout, TEXT_PANEL_VALUE,
    TextFace, TextRun, TextRunSpan, draw_rich_text, draw_text, draw_text_clipped, push_rect_border,
    truncate_text,
};
use crate::design_tokens;
use crate::terminal_cursor::render_terminal_cursor;
use crate::terminal_tab_strip::render_terminal_tab_strip;
use taffy::prelude::*;

#[path = "terminal_color.rs"]
mod terminal_color;
use terminal_color::{span_background, span_foreground};

/// Keep JetBrains Mono's ink comfortably inside the cell, then use the shaping
/// engine's explicit letter spacing to preserve the exact governed 7.9 px
/// advance. Enlarging the raw 0.6-em glyph advance to the full cell made
/// adjacent contrasting glyphs and the cursor visually fuse at raster scale.
pub(super) const TERMINAL_FONT_SIZE_PX: f32 = 12.0;
pub(super) const TERMINAL_LETTER_SPACING_EM: f32 =
    (TERMINAL_CELL_WIDTH_PX - TERMINAL_FONT_SIZE_PX * 0.6) / TERMINAL_FONT_SIZE_PX;
const TERMINAL_SELECTION_BG: [f32; 3] = design_tokens::chrome::TERMINAL_SELECTION;
const TERMINAL_SELECTION_FG: [f32; 3] = design_tokens::chrome::TEXT_PRIMARY;

#[derive(Debug, Clone, Copy)]
struct BottomDockLayout {
    // Retained for the solver contract test; the seated tab is now sized to its
    // measured label directly in render_bottom_tabs.
    #[allow(dead_code)]
    terminal_tab: RectPx,
    handle: RectPx,
    content: RectPx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BottomDockNode {
    Terminal,
}

fn solve_bottom_dock_layout_with_taffy(layout: &ShellLayout) -> Option<BottomDockLayout> {
    let strip = layout.bottom_strip;
    let tab_height = (strip.height - 16.0).max(1.0);
    let tab_width = 120.0_f32;
    let tab_gap = 8.0_f32;
    let row_x = strip.x + 12.0;
    let row_y = strip.y + 8.0;

    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let mut nodes = Vec::new();
    let mut add_tab = |kind: BottomDockNode| -> Option<()> {
        let node = taffy
            .new_leaf(Style {
                size: Size {
                    width: length(tab_width),
                    height: length(tab_height),
                },
                ..Default::default()
            })
            .ok()?;
        nodes.push((kind, node));
        Some(())
    };
    add_tab(BottomDockNode::Terminal)?;

    let children = nodes.iter().map(|(_, node)| *node).collect::<Vec<_>>();
    let root = taffy
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                gap: Size {
                    width: length(tab_gap),
                    height: length(0.0),
                },
                size: Size {
                    width: length((strip.width - 24.0).max(1.0)),
                    height: length(tab_height),
                },
                ..Default::default()
            },
            &children,
        )
        .ok()?;
    taffy.compute_layout(root, Size::MAX_CONTENT).ok()?;

    let rect_for = |kind: BottomDockNode| -> Option<RectPx> {
        let node = nodes.iter().find(|(node_kind, _)| *node_kind == kind)?.1;
        let solved = taffy.layout(node).ok()?;
        Some(RectPx {
            x: row_x + solved.location.x,
            y: row_y + solved.location.y,
            width: solved.size.width,
            height: solved.size.height,
        })
    };

    Some(BottomDockLayout {
        terminal_tab: rect_for(BottomDockNode::Terminal)?,
        handle: RectPx {
            x: strip.x,
            y: strip.y,
            width: strip.width,
            height: 6.0,
        },
        content: RectPx {
            x: strip.x + 12.0,
            y: strip.y + 44.0,
            width: (strip.width - 24.0).max(1.0),
            height: (strip.height - 56.0).max(0.0),
        },
    })
}

fn fallback_bottom_dock_layout(layout: &ShellLayout) -> BottomDockLayout {
    let strip = layout.bottom_strip;
    BottomDockLayout {
        terminal_tab: RectPx {
            x: strip.x + 12.0,
            y: strip.y + 8.0,
            width: 120.0,
            height: strip.height - 16.0,
        },
        handle: RectPx {
            x: strip.x,
            y: strip.y,
            width: strip.width,
            height: 6.0,
        },
        content: RectPx {
            x: strip.x + 12.0,
            y: strip.y + 44.0,
            width: strip.width - 24.0,
            height: (strip.height - 56.0).max(0.0),
        },
    }
}

pub(super) fn render_bottom_tabs(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let dock_layout = solve_bottom_dock_layout_with_taffy(layout)
        .unwrap_or_else(|| fallback_bottom_dock_layout(layout));
    let strip = layout.bottom_strip;
    // Single top-edge hairline on the dock strip.
    panel_quads.push(Quad::from_rect(
        RectPx {
            x: strip.x,
            y: strip.y,
            width: strip.width,
            height: 1.0,
        },
        PANEL_CARD_BORDER,
    ));
    render_terminal_tab_strip(state, strip, panel_quads, text_runs, hit_regions);

    let Some(active_tab) = state.ui.active_dock_tab else {
        return;
    };
    let handle_rect = dock_layout.handle;
    panel_quads.push(Quad::from_rect(handle_rect, PANEL_CARD_BORDER));
    hit_regions.push(HitRegion {
        target: HitTarget::DockResizeHandle,
        rect: handle_rect,
    });
    let content_rect = dock_layout.content;
    panel_quads.push(Quad::from_rect(content_rect, PANEL_BG));
    push_rect_border(panel_quads, content_rect, PANEL_CARD_BORDER, 1.0);
    match active_tab {
        DockTab::Terminal => {
            // T0-C02: the exact visible cell rectangle is the space and
            // row/column authority — the same shared geometry the PTY resize
            // path uses (datum_gui_viewport::terminal_screen_geometry), so the
            // rows drawn here always equal the rows the PTY was told.
            let geometry = terminal_screen_geometry(layout.bottom_strip.into());
            render_terminal_lane(state, &geometry, panel_quads, text_runs, hit_regions);
        }
    }
}

fn render_terminal_lane(
    state: &ReviewWorkspaceState,
    geometry: &TerminalScreenGeometry,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    render_terminal_screen(state, geometry, panel_quads, text_runs, hit_regions);
}

/// The terminal SCREEN: the exact visible cell rectangle, drawing precisely
/// `geometry.rows` grid rows of `geometry.columns` cells — the same numbers
/// the PTY was resized to — and exposing the rectangle as the dedicated
/// terminal-content hit target (T0-C02).
fn render_terminal_screen(
    state: &ReviewWorkspaceState,
    geometry: &TerminalScreenGeometry,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let screen: RectPx = geometry.screen.into();
    hit_regions.push(HitRegion {
        target: HitTarget::TerminalScreen,
        rect: screen,
    });
    let max_lines = geometry.rows as usize;
    let max_columns = geometry.columns as usize;
    let total = state.ui.terminal.grid_lines().len();
    let scroll = state
        .ui
        .terminal
        .scroll_offset
        .min(total.saturating_sub(max_lines));
    let tail_start = total.saturating_sub(max_lines + scroll);
    let mut y = screen.y;
    for (line_index, line) in state
        .ui
        .terminal
        .grid_lines()
        .iter()
        .enumerate()
        .skip(tail_start)
        .take(max_lines)
    {
        if let Some(styled_line) = state.ui.terminal.grid_styled_lines().get(line_index) {
            render_terminal_style_backgrounds(styled_line, screen.x, y, max_columns, panel_quads);
        }
        let selection_rect = render_terminal_selection_row(
            &state.ui.terminal,
            line_index,
            screen.x,
            y,
            max_columns,
            panel_quads,
        );
        if let Some(styled_line) = state.ui.terminal.grid_styled_lines().get(line_index) {
            render_terminal_styled_line(styled_line, line, screen.x, y, max_columns, text_runs);
        } else {
            draw_text(
                &truncate_text(line, max_columns),
                screen.x,
                y,
                TERMINAL_FONT_SIZE_PX,
                TEXT_PANEL_VALUE,
                TextFace::Terminal,
                text_runs,
            );
        }
        if let Some(clip_bounds) = selection_rect {
            render_terminal_selection_text(line, screen.x, y, max_columns, clip_bounds, text_runs);
        }
        if state.ui.terminal.screen_cursor_visible
            && state.ui.terminal.screen_cursor_row == line_index
            && state.ui.terminal.screen_cursor_col < max_columns
        {
            render_terminal_cursor(
                &state.ui.terminal,
                state.ui.focus.is_terminal(),
                screen.x,
                y,
                state
                    .ui
                    .terminal
                    .text_selection_span(line_index, max_columns)
                    .is_some_and(|(first, last)| {
                        (first..last).contains(&state.ui.terminal.screen_cursor_col)
                    }),
                panel_quads,
            );
        }
        y += TERMINAL_CELL_HEIGHT_PX;
    }
}

fn render_terminal_style_backgrounds(
    styled_line: &TerminalStyledLine,
    x: f32,
    y: f32,
    max_columns: usize,
    panel_quads: &mut Vec<Quad>,
) {
    for span in styled_line
        .spans
        .iter()
        .filter(|span| span.start < max_columns && span.start < span.end)
    {
        let start = span.start.min(max_columns);
        let end = span.end.min(max_columns);
        let Some(color) = span_background(span.fg.as_deref(), span.bg.as_deref(), span.inverse)
        else {
            continue;
        };
        panel_quads.push(Quad::from_rect(
            RectPx {
                x: x + start as f32 * TERMINAL_CELL_WIDTH_PX,
                y,
                width: (end - start) as f32 * TERMINAL_CELL_WIDTH_PX,
                height: TERMINAL_CELL_HEIGHT_PX,
            },
            color,
        ));
    }
}

fn render_terminal_selection_row(
    terminal: &datum_gui_protocol::TerminalLaneState,
    row: usize,
    x: f32,
    y: f32,
    max_columns: usize,
    panel_quads: &mut Vec<Quad>,
) -> Option<RectPx> {
    let (first, last) = terminal.text_selection_span(row, max_columns)?;
    let rect = RectPx {
        x: x + first as f32 * TERMINAL_CELL_WIDTH_PX,
        y,
        width: (last - first) as f32 * TERMINAL_CELL_WIDTH_PX,
        height: TERMINAL_CELL_HEIGHT_PX,
    };
    panel_quads.push(Quad::from_rect(rect, TERMINAL_SELECTION_BG));
    Some(rect)
}

fn render_terminal_selection_text(
    line: &str,
    x: f32,
    y: f32,
    max_columns: usize,
    clip_bounds: RectPx,
    text_runs: &mut Vec<TextRun>,
) {
    draw_text_clipped(
        &truncate_text(line, max_columns),
        x,
        y,
        TERMINAL_FONT_SIZE_PX,
        TERMINAL_SELECTION_FG,
        TextFace::Terminal,
        clip_bounds,
        text_runs,
    );
}

fn render_terminal_styled_line(
    styled_line: &TerminalStyledLine,
    fallback_line: &str,
    x: f32,
    y: f32,
    max_columns: usize,
    text_runs: &mut Vec<TextRun>,
) {
    let text = if styled_line.text.is_empty() {
        fallback_line
    } else {
        &styled_line.text
    };
    let visible_len = text.chars().count().min(max_columns);
    if visible_len == 0 {
        draw_text(
            "",
            x,
            y,
            TERMINAL_FONT_SIZE_PX,
            TEXT_PANEL_VALUE,
            TextFace::Terminal,
            text_runs,
        );
        return;
    }
    let visible_text = text.chars().take(visible_len).collect::<String>();
    let mut rich_spans = Vec::new();
    let mut cursor = 0;
    for span in styled_line
        .spans
        .iter()
        .filter(|span| span.start < visible_len && span.start < span.end)
    {
        let start = span.start.min(visible_len);
        let end = span.end.min(visible_len);
        if cursor < start {
            push_terminal_rich_span(
                &visible_text,
                cursor,
                start,
                TEXT_PANEL_VALUE,
                &mut rich_spans,
            );
        }
        let styled_start = start.max(cursor);
        push_terminal_rich_span(
            &visible_text,
            styled_start,
            end,
            span_foreground(
                span.fg.as_deref(),
                span.bg.as_deref(),
                span.bold,
                span.inverse,
                span.conceal,
            ),
            &mut rich_spans,
        );
        cursor = cursor.max(end);
    }
    if cursor < visible_len {
        push_terminal_rich_span(
            &visible_text,
            cursor,
            visible_len,
            TEXT_PANEL_VALUE,
            &mut rich_spans,
        );
    }
    draw_rich_text(
        &visible_text,
        rich_spans,
        x,
        y,
        TERMINAL_FONT_SIZE_PX,
        TEXT_PANEL_VALUE,
        TextFace::Terminal,
        text_runs,
    );
}

fn push_terminal_rich_span(
    text: &str,
    start: usize,
    end: usize,
    color: [f32; 3],
    spans: &mut Vec<TextRunSpan>,
) {
    if start >= end {
        return;
    }
    spans.push(TextRunSpan {
        text: text.chars().skip(start).take(end - start).collect(),
        color,
    });
}

#[cfg(test)]
mod selection_tests {
    use datum_gui_protocol::TerminalLaneState;

    use super::*;

    #[test]
    fn terminal_selection_highlight_uses_exact_cell_geometry_behind_text() {
        let mut terminal = TerminalLaneState::default();
        terminal.set_text_selection((3, 2), (3, 4));
        let mut quads = Vec::new();
        let expected = RectPx {
            x: 10.0 + 2.0 * TERMINAL_CELL_WIDTH_PX,
            y: 20.0,
            width: 3.0 * TERMINAL_CELL_WIDTH_PX,
            height: TERMINAL_CELL_HEIGHT_PX,
        };
        let selection = render_terminal_selection_row(&terminal, 3, 10.0, 20.0, 80, &mut quads);
        assert_eq!(quads.len(), 1);
        assert_eq!(
            quads[0].points[0],
            (10.0 + 2.0 * TERMINAL_CELL_WIDTH_PX, 20.0)
        );
        assert_eq!(
            quads[0].points[1],
            (10.0 + 5.0 * TERMINAL_CELL_WIDTH_PX, 20.0)
        );
        assert_eq!(quads[0].color, TERMINAL_SELECTION_BG);
        assert_eq!(selection, Some(expected));
        assert_eq!(quads[0], Quad::from_rect(expected, TERMINAL_SELECTION_BG));
        assert_eq!(
            TERMINAL_SELECTION_BG,
            design_tokens::chrome::TERMINAL_SELECTION
        );
        assert_eq!(TERMINAL_SELECTION_FG, design_tokens::chrome::TEXT_PRIMARY);

        let mut text_runs = Vec::new();
        render_terminal_selection_text("selected", 10.0, 20.0, 80, expected, &mut text_runs);
        assert_eq!(text_runs.len(), 1);
        assert_eq!(text_runs[0].text, "selected");
        assert_eq!(text_runs[0].color, TERMINAL_SELECTION_FG);
        assert_eq!(text_runs[0].clip_bounds, Some(expected));
    }

    #[test]
    fn terminal_truecolor_background_uses_exact_styled_cell_geometry() {
        let styled = TerminalStyledLine {
            text: "abcdef".to_string(),
            spans: vec![datum_gui_protocol::TerminalStyleSpan {
                start: 2,
                end: 5,
                fg: None,
                bg: Some("rgb:12:34:56".to_string()),
                bold: false,
                dim: false,
                italic: false,
                underline: false,
                overline: false,
                blink: false,
                strikethrough: false,
                conceal: false,
                inverse: false,
            }],
        };
        let mut quads = Vec::new();
        render_terminal_style_backgrounds(&styled, 10.0, 20.0, 80, &mut quads);

        assert_eq!(quads.len(), 1);
        assert_eq!(
            quads[0],
            Quad::from_rect(
                RectPx {
                    x: 10.0 + 2.0 * TERMINAL_CELL_WIDTH_PX,
                    y: 20.0,
                    width: 3.0 * TERMINAL_CELL_WIDTH_PX,
                    height: TERMINAL_CELL_HEIGHT_PX,
                },
                terminal_color::terminal_color("rgb:12:34:56").expect("valid fixture color"),
            )
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bottom_dock_tabs_are_solver_backed_and_non_overlapping() {
        let shell = ShellLayout::for_window(1280, 800, Some(220));
        let layout =
            solve_bottom_dock_layout_with_taffy(&shell).expect("bottom dock layout should solve");

        assert!(layout.content.y > layout.terminal_tab.y);
        assert!(layout.content.x >= shell.bottom_strip.x);
        assert!(
            layout.content.x + layout.content.width
                <= shell.bottom_strip.x + shell.bottom_strip.width
        );
    }

    #[test]
    fn shared_terminal_geometry_agrees_with_dock_content_rect() {
        // T0-C02 guard: the shared geometry derives the dock content rect from
        // the bottom strip with the same constants the dock solver uses; if
        // either side drifts, renderer and PTY budgets diverge again.
        for dock_height in [120, 220, 320] {
            let shell = ShellLayout::for_window(1280, 800, Some(dock_height));
            let solved = solve_bottom_dock_layout_with_taffy(&shell)
                .expect("bottom dock layout should solve");
            let geometry = terminal_screen_geometry(shell.bottom_strip.into());
            let content: RectPx = geometry.content.into();
            assert_eq!(content, solved.content, "dock {dock_height}px");
        }
    }
}
