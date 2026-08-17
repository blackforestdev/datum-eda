use datum_gui_protocol::{DockTab, ReviewWorkspaceState, TerminalStyledLine};
use datum_gui_viewport::{
    TERMINAL_CELL_HEIGHT_PX, TERMINAL_CELL_WIDTH_PX, TerminalScreenGeometry,
    terminal_screen_geometry,
};

use super::{
    HitRegion, HitTarget, PANEL_BG, PANEL_CARD_BORDER, Quad, RectPx, ShellLayout, TEXT_ACCENT,
    TEXT_MUTED, TEXT_PANEL_VALUE, TEXT_PRIMARY, TEXT_SECONDARY, TextFace, TextRun, TextRunSpan,
    design_tokens, draw_rich_text, draw_text, estimated_text_run_width_px, push_rect_border,
    truncate_text,
};
use crate::terminal_cursor::render_terminal_cursor;
use crate::terminal_session_chrome::render_terminal_session_controls;
use taffy::prelude::*;

/// Keep JetBrains Mono's ink comfortably inside the cell, then use the shaping
/// engine's explicit letter spacing to preserve the exact governed 7.9 px
/// advance. Enlarging the raw 0.6-em glyph advance to the full cell made
/// adjacent contrasting glyphs and the cursor visually fuse at raster scale.
pub(super) const TERMINAL_FONT_SIZE_PX: f32 = 12.0;
pub(super) const TERMINAL_LETTER_SPACING_EM: f32 =
    (TERMINAL_CELL_WIDTH_PX - TERMINAL_FONT_SIZE_PX * 0.6) / TERMINAL_FONT_SIZE_PX;

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
    let active = matches!(state.ui.active_dock_tab, Some(DockTab::Terminal));
    // Tab label is the session's own (lower/mixed-case) name, never an
    // uppercased constant. PTY doctrine: this lane is the real terminal.
    let label = state
        .ui
        .terminal
        .title
        .as_deref()
        .map(|title| truncate_text(title, 16))
        .unwrap_or_else(|| "terminal".to_string());
    // Seated tab sized to the measured label + padding, its bottom anchored to
    // the strip seam.
    let label_w = estimated_text_run_width_px(&label, 12.5, TextFace::Ui) - 16.0;
    let tab = RectPx {
        x: strip.x + 12.0,
        y: strip.y + 6.0,
        width: label_w + design_tokens::spacing::SP_04 * 2.0,
        height: (strip.height - 6.0).max(1.0),
    };
    if active {
        // SURFACE_01 fill + a 2px ACCENT top edge only — a seated tab, not a box.
        panel_quads.push(Quad::from_rect(tab, design_tokens::chrome::SURFACE_01));
        panel_quads.push(Quad::from_rect(
            RectPx {
                x: tab.x,
                y: tab.y,
                width: tab.width,
                height: 2.0,
            },
            TEXT_ACCENT,
        ));
    }
    draw_text(
        &label,
        tab.x + design_tokens::spacing::SP_04,
        tab.y + 8.0,
        12.5,
        if active { TEXT_PRIMARY } else { TEXT_MUTED },
        TextFace::Ui,
        text_runs,
    );
    hit_regions.push(HitRegion {
        target: HitTarget::TerminalTab,
        rect: tab,
    });
    // "+" add-terminal affordance seated after the tab.
    let plus = RectPx {
        x: tab.x + tab.width + design_tokens::spacing::SP_03,
        y: tab.y,
        width: 20.0,
        height: tab.height,
    };
    draw_text(
        "+",
        plus.x + 6.0,
        plus.y + 7.0,
        14.0,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    hit_regions.push(HitRegion {
        target: HitTarget::TerminalSessionNew,
        rect: plus,
    });
    // Right-aligned persistent dock hint.
    let hint = "Ctrl+Shift+T new terminal   \u{00B7}   Ctrl+K palette";
    let hint_w = estimated_text_run_width_px(hint, 11.5, TextFace::Mono) - 16.0;
    draw_text(
        hint,
        strip.x + strip.width - 12.0 - hint_w,
        tab.y + 8.0,
        11.5,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );

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
    if let Some(header) = geometry.header {
        render_terminal_header(state, header.into(), text_runs);
    }
    if let Some(sessions_row) = geometry.sessions_row {
        render_terminal_sessions_row(state, sessions_row.into(), text_runs, hit_regions);
    }
    render_terminal_screen(state, geometry, panel_quads, text_runs, hit_regions);
}

/// Terminal chrome header band: lane title plus PTY session status/meta and
/// the copy/scroll/paste shortcut hint. Chrome only — never terminal cells
/// (T0-C01/T0-C02; decision 027 FT-001).
fn render_terminal_header(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    text_runs: &mut Vec<TextRun>,
) {
    draw_text(
        "PROJECT TERMINAL",
        rect.x,
        rect.y,
        12.0,
        TEXT_SECONDARY,
        TextFace::Ui,
        text_runs,
    );
    let hint = "COPY SCROLLBACK CTRL+SHIFT+C  SCROLL SHIFT+PGUP/PGDN  PASTE CTRL+V";
    let hint_w = estimated_text_run_width_px(hint, 10.5, TextFace::Mono) - 16.0;
    draw_text(
        hint,
        rect.x + rect.width - hint_w,
        rect.y + 1.0,
        10.5,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    let mut session_label =
        if let Some(blocked) = state.ui.terminal.application_shutdown_blocked.as_deref() {
            format!("APPLICATION SHUTDOWN / {blocked}")
        } else if let Some(title) = state.ui.terminal.title.as_deref() {
            format!(
                "SHELL SESSION / {} / {}",
                state.ui.terminal.status.to_uppercase(),
                truncate_text(title, 48)
            )
        } else {
            format!(
                "SHELL SESSION / {}",
                state.ui.terminal.status.to_uppercase()
            )
        };
    if state.ui.terminal.bell_count > 0 {
        session_label.push_str(&format!(" / BELL {}", state.ui.terminal.bell_count));
    }
    if let Some(cwd) = state.ui.terminal.current_working_directory.as_deref() {
        session_label.push_str(&format!(" / CWD {}", truncate_text(cwd, 56)));
    }
    session_label.push_str(&format!(
        " / SIZE {}x{}",
        state.ui.terminal.columns, state.ui.terminal.rows
    ));
    if state.ui.terminal.focus_event_reporting {
        session_label.push_str(" / FOCUS EVENTS");
    }
    if state.ui.terminal.application_cursor_keys {
        session_label.push_str(" / APP CURSOR");
    }
    if state.ui.terminal.application_keypad {
        session_label.push_str(" / APP KEYPAD");
    }
    if let Some(mode) = state.ui.terminal.mouse_reporting_mode.as_deref() {
        session_label.push_str(&format!(" / MOUSE {}", mode.to_uppercase()));
    }
    if let Some(encoding) = state.ui.terminal.mouse_coordinate_encoding.as_deref() {
        session_label.push_str(&format!(" {}", encoding.to_uppercase()));
    }
    draw_text(
        &session_label,
        rect.x,
        rect.y + 16.0,
        10.5,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
}

/// Terminal chrome sessions band: session controls, tabs, and the inline
/// rename affordance. Application summaries (ACTIVITY SPANS) are gone from
/// the lane entirely — summaries belong to chrome/console surfaces and must
/// consume zero cell rows (T0-C02; owner directive on
/// dat-pan-trace-terminal-pollution-0j0).
fn render_terminal_sessions_row(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    if !state.ui.terminal.tabs.is_empty() {
        let y = rect.y + 2.0;
        draw_text(
            "SESSIONS",
            rect.x,
            y,
            10.5,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        let mut x = render_terminal_session_controls(
            rect,
            y,
            &state.ui.terminal.status,
            state.ui.terminal.application_shutdown_blocked.as_deref(),
            text_runs,
            hit_regions,
        );
        for tab in state.ui.terminal.tabs.iter().take(6) {
            let renaming = state
                .ui
                .terminal
                .rename_session_id
                .as_deref()
                .is_some_and(|session_id| session_id == tab.session_id);
            let label = if renaming {
                let (before, after) = split_at_cursor(
                    &state.ui.terminal.rename_input,
                    state.ui.terminal.rename_cursor,
                );
                format!(
                    "[{}|{}]",
                    truncate_text(before, 12),
                    truncate_text(after, 8)
                )
            } else if tab.active {
                let label = if tab.restart_count > 0 {
                    format!("{} R{}", truncate_text(&tab.label, 12), tab.restart_count)
                } else if tab.activity_event_count > 0 {
                    format!(
                        "{} A{}",
                        truncate_text(&tab.label, 12),
                        tab.activity_event_count
                    )
                } else {
                    truncate_text(&tab.label, 18)
                };
                format!("[{}]", label)
            } else if !tab.attached {
                format!("{}:DETACHED", truncate_text(&tab.label, 12))
            } else {
                truncate_text(&tab.label, 18)
            };
            draw_text(
                &label,
                x,
                y,
                10.5,
                if tab.active {
                    TEXT_PRIMARY
                } else {
                    TEXT_SECONDARY
                },
                TextFace::Mono,
                text_runs,
            );
            hit_regions.push(HitRegion {
                target: HitTarget::TerminalSessionTab(tab.session_id.clone()),
                rect: RectPx {
                    x: x - 4.0,
                    y: y - 2.0,
                    width: (label.len() as f32 * 7.0 + 8.0).max(24.0),
                    height: 14.0,
                },
            });
            x += (label.len() as f32 * 7.0 + 18.0).min(160.0);
            if x > rect.x + rect.width - 60.0 {
                break;
            }
        }
        if state.ui.terminal.rename_session_id.is_some() {
            let hint = "RENAMING TERMINAL TAB  ENTER SAVE  ESC CANCEL";
            let hint_w = estimated_text_run_width_px(hint, 10.5, TextFace::Mono) - 16.0;
            draw_text(
                hint,
                rect.x + rect.width - hint_w,
                y,
                10.5,
                TEXT_MUTED,
                TextFace::Mono,
                text_runs,
            );
        }
    }
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
        if state.ui.terminal.screen_cursor_visible
            && state.ui.terminal.screen_cursor_row == line_index
            && state.ui.terminal.screen_cursor_col < max_columns
        {
            render_terminal_cursor(
                &state.ui.terminal,
                state.ui.focus.is_terminal(),
                screen.x,
                y,
                panel_quads,
            );
        }
        y += TERMINAL_CELL_HEIGHT_PX;
    }
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
            terminal_span_color(
                span.fg.as_deref(),
                span.bg.as_deref(),
                span.bold,
                span.inverse,
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

fn terminal_span_color(fg: Option<&str>, bg: Option<&str>, bold: bool, inverse: bool) -> [f32; 3] {
    let effective_fg = if inverse { bg.or(Some("white")) } else { fg };
    match effective_fg {
        Some("black") => [0.25, 0.27, 0.30],
        Some("red") => [0.95, 0.32, 0.28],
        Some("green") => [0.45, 0.82, 0.48],
        Some("yellow") => [0.96, 0.78, 0.32],
        Some("blue") => [0.42, 0.62, 0.95],
        Some("magenta") => [0.82, 0.50, 0.90],
        Some("cyan") => [0.38, 0.82, 0.88],
        Some("white") => [0.90, 0.92, 0.94],
        Some("bright_black") => [0.48, 0.52, 0.58],
        Some("bright_red") => [1.00, 0.42, 0.36],
        Some("bright_green") => [0.58, 0.92, 0.56],
        Some("bright_yellow") => [1.00, 0.86, 0.42],
        Some("bright_blue") => [0.52, 0.72, 1.00],
        Some("bright_magenta") => [0.92, 0.62, 1.00],
        Some("bright_cyan") => [0.50, 0.92, 0.96],
        Some("bright_white") => [1.00, 1.00, 1.00],
        _ if bold => TEXT_PRIMARY,
        _ => TEXT_PANEL_VALUE,
    }
}

fn split_at_cursor(input: &str, cursor: usize) -> (&str, &str) {
    let byte_pos = input
        .char_indices()
        .nth(cursor)
        .map(|(i, _)| i)
        .unwrap_or(input.len());
    (&input[..byte_pos], &input[byte_pos..])
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
