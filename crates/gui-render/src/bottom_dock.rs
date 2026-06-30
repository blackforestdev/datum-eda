use datum_gui_protocol::{
    DockTab, ReviewWorkspaceState, TerminalCommandHandoff, TerminalStyledLine,
    render_terminal_command_handoff,
};

use super::outputs_lane::render_outputs_lane;
use super::{
    HitRegion, HitTarget, PANEL_BG, PANEL_CARD_BG, PANEL_CARD_BORDER, Quad, REVIEW_ROW_BADGE,
    RectPx, ShellLayout, TEXT_MUTED, TEXT_PANEL_VALUE, TEXT_PRIMARY, TEXT_SECONDARY, TextFace,
    TextRun, draw_text, push_rect_border, truncate_text,
};
use taffy::prelude::*;

#[derive(Debug, Clone, Copy)]
struct BottomDockLayout {
    terminal_tab: RectPx,
    assistant_tab: RectPx,
    outputs_tab: RectPx,
    handle: RectPx,
    content: RectPx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BottomDockNode {
    Terminal,
    Assistant,
    Outputs,
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
    add_tab(BottomDockNode::Assistant)?;
    add_tab(BottomDockNode::Outputs)?;
    drop(add_tab);

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
        assistant_tab: rect_for(BottomDockNode::Assistant)?,
        outputs_tab: rect_for(BottomDockNode::Outputs)?,
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
        assistant_tab: RectPx {
            x: strip.x + 140.0,
            y: strip.y + 8.0,
            width: 120.0,
            height: strip.height - 16.0,
        },
        outputs_tab: RectPx {
            x: strip.x + 268.0,
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
    for (rect, label, target, active) in [
        (
            dock_layout.terminal_tab,
            "TERMINAL",
            HitTarget::TerminalTab,
            matches!(state.ui.active_dock_tab, Some(DockTab::Terminal)),
        ),
        (
            dock_layout.assistant_tab,
            "AGENTS",
            HitTarget::AssistantTab,
            false,
        ),
        (
            dock_layout.outputs_tab,
            "OUTPUTS",
            HitTarget::OutputsTab,
            matches!(state.ui.active_dock_tab, Some(DockTab::Outputs)),
        ),
    ] {
        panel_quads.push(Quad::from_rect(
            rect,
            if active {
                REVIEW_ROW_BADGE
            } else {
                PANEL_CARD_BG
            },
        ));
        push_rect_border(panel_quads, rect, PANEL_CARD_BORDER, 1.0);
        draw_text(
            label,
            rect.x + 12.0,
            rect.y + 10.0,
            12.0,
            if active { TEXT_PRIMARY } else { TEXT_SECONDARY },
            TextFace::Ui,
            text_runs,
        );
        hit_regions.push(HitRegion { target, rect });
    }

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
        DockTab::Terminal | DockTab::Assistant => {
            render_terminal_lane(state, content_rect, text_runs, hit_regions)
        }
        DockTab::Outputs => {
            render_outputs_lane(state, content_rect, panel_quads, text_runs, hit_regions)
        }
    }
}

fn render_terminal_lane(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    draw_text(
        "PROJECT TERMINAL",
        rect.x + 12.0,
        rect.y + 12.0,
        12.0,
        TEXT_SECONDARY,
        TextFace::Ui,
        text_runs,
    );
    let mut session_label = if let Some(title) = state.ui.terminal.title.as_deref() {
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
        rect.x + 12.0,
        rect.y + 28.0,
        10.5,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    draw_text(
        "COPY SCROLLBACK CTRL+SHIFT+C  SCROLL SHIFT+PGUP/PGDN  PASTE CTRL+V",
        rect.x + 12.0,
        rect.y + 43.0,
        10.5,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    draw_text(
        "AGENTS BUTTON PREFILLS CODEX/CLAUDE COMMANDS HERE; EXECUTION REMAINS TERMINAL-OWNED",
        rect.x + 12.0,
        rect.y + 58.0,
        10.5,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    let mut y = rect.y + 77.0;
    if !state.ui.terminal.tabs.is_empty() {
        draw_text(
            "SESSIONS",
            rect.x + 12.0,
            y,
            10.5,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        let mut x = render_terminal_session_controls(rect, y, text_runs, hit_regions);
        for tab in state.ui.terminal.tabs.iter().take(6) {
            let renaming = state
                .ui
                .terminal
                .rename_session_id
                .as_deref()
                .is_some_and(|session_id| session_id == tab.session_id);
            let label = if renaming {
                let (before, after) =
                    split_at_cursor(&state.ui.terminal.input, state.ui.terminal.cursor);
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
            if x > rect.x + rect.width - 72.0 {
                break;
            }
        }
        y += 18.0;
        if state.ui.terminal.rename_session_id.is_some() {
            draw_text(
                "RENAMING TERMINAL TAB  ENTER SAVE  ESC CANCEL",
                rect.x + 12.0,
                y,
                10.5,
                TEXT_MUTED,
                TextFace::Mono,
                text_runs,
            );
            y += 16.0;
        }
    }
    if let Some(next_y) = render_terminal_journal_commands(rect, y, text_runs, hit_regions) {
        y = next_y + 4.0;
    }
    if !state.ui.terminal.activity_summary.is_empty() {
        draw_text(
            "ACTIVITY SPANS",
            rect.x + 12.0,
            y,
            10.5,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        y += 16.0;
        for line in state.ui.terminal.activity_summary.iter().take(4) {
            draw_text(
                &truncate_text(line, 180),
                rect.x + 12.0,
                y,
                10.5,
                TEXT_SECONDARY,
                TextFace::Mono,
                text_runs,
            );
            push_activity_hit_region(hit_regions, rect, y, line);
            y += 15.0;
        }
        y += 4.0;
    }
    let max_lines = ((rect.y + rect.height - y - 10.0) / 16.0).floor().max(1.0) as usize;
    let total = state.ui.terminal.lines.len();
    let scroll = state
        .ui
        .terminal
        .scroll_offset
        .min(total.saturating_sub(max_lines));
    let tail_start = total.saturating_sub(max_lines + scroll);
    for (line_index, line) in state
        .ui
        .terminal
        .lines
        .iter()
        .enumerate()
        .skip(tail_start)
        .take(max_lines)
    {
        if let Some(styled_line) = state.ui.terminal.styled_lines.get(line_index) {
            render_terminal_styled_line(styled_line, line, rect.x + 12.0, y, text_runs);
        } else {
            draw_text(
                &truncate_text(line, 180),
                rect.x + 12.0,
                y,
                11.0,
                TEXT_PANEL_VALUE,
                TextFace::Mono,
                text_runs,
            );
        }
        if state.ui.terminal.screen_cursor_visible
            && state.ui.terminal.screen_cursor_row == line_index
            && state.ui.terminal.screen_cursor_col <= 180
        {
            render_terminal_screen_cursor(&state.ui.terminal, rect.x + 12.0, y, text_runs);
        }
        y += 16.0;
    }
}

fn render_terminal_screen_cursor(
    terminal: &datum_gui_protocol::TerminalLaneState,
    origin_x: f32,
    y: f32,
    text_runs: &mut Vec<TextRun>,
) {
    let glyph = match terminal.screen_cursor_style.as_deref() {
        Some("blinking_underline" | "steady_underline") => "_",
        Some("blinking_bar" | "steady_bar") => "|",
        _ => "█",
    };
    draw_text(
        glyph,
        origin_x + terminal.screen_cursor_col as f32 * 7.9,
        y,
        11.0,
        TEXT_PRIMARY,
        TextFace::Mono,
        text_runs,
    );
}

fn render_terminal_styled_line(
    styled_line: &TerminalStyledLine,
    fallback_line: &str,
    x: f32,
    y: f32,
    text_runs: &mut Vec<TextRun>,
) {
    let text = if styled_line.text.is_empty() {
        fallback_line
    } else {
        &styled_line.text
    };
    let visible_len = text.chars().count().min(180);
    if visible_len == 0 {
        draw_text("", x, y, 11.0, TEXT_PANEL_VALUE, TextFace::Mono, text_runs);
        return;
    }
    let mut cursor = 0;
    for span in styled_line
        .spans
        .iter()
        .filter(|span| span.start < visible_len && span.start < span.end)
    {
        let start = span.start.min(visible_len);
        let end = span.end.min(visible_len);
        if cursor < start {
            draw_terminal_fragment(text, cursor, start, x, y, TEXT_PANEL_VALUE, text_runs);
        }
        draw_terminal_fragment(
            text,
            start,
            end,
            x,
            y,
            terminal_span_color(
                span.fg.as_deref(),
                span.bg.as_deref(),
                span.bold,
                span.inverse,
            ),
            text_runs,
        );
        cursor = end;
    }
    if cursor < visible_len {
        draw_terminal_fragment(text, cursor, visible_len, x, y, TEXT_PANEL_VALUE, text_runs);
    }
}

fn draw_terminal_fragment(
    text: &str,
    start: usize,
    end: usize,
    origin_x: f32,
    y: f32,
    color: [f32; 3],
    text_runs: &mut Vec<TextRun>,
) {
    if start >= end {
        return;
    }
    let fragment = text
        .chars()
        .skip(start)
        .take(end - start)
        .collect::<String>();
    draw_text(
        &fragment,
        origin_x + start as f32 * 7.9,
        y,
        11.0,
        color,
        TextFace::Mono,
        text_runs,
    );
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

fn render_terminal_session_controls(
    rect: RectPx,
    y: f32,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) -> f32 {
    let mut x = rect.x + 78.0;
    for (label, target) in [
        ("+NEW", HitTarget::TerminalSessionNew),
        ("RENAME", HitTarget::TerminalSessionRenameActive),
        ("RESTART", HitTarget::TerminalSessionRestartActive),
        ("DETACH", HitTarget::TerminalSessionDetachActive),
        ("CLOSE", HitTarget::TerminalSessionCloseActive),
    ] {
        draw_text(label, x, y, 10.5, TEXT_MUTED, TextFace::Mono, text_runs);
        hit_regions.push(HitRegion {
            target,
            rect: RectPx {
                x: x - 4.0,
                y: y - 2.0,
                width: (label.len() as f32 * 7.0 + 8.0).max(24.0),
                height: 14.0,
            },
        });
        x += label.len() as f32 * 7.0 + 16.0;
    }
    x + 4.0
}

fn render_terminal_journal_commands(
    rect: RectPx,
    mut y: f32,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) -> Option<f32> {
    draw_text(
        "JOURNAL",
        rect.x + 12.0,
        y,
        10.5,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    y += 16.0;
    for (label, command_id) in [
        ("LIST", "datum.journal.list"),
        ("UNDO", "datum.journal.undo"),
        ("REDO", "datum.journal.redo"),
    ] {
        let command = render_terminal_command_handoff(
            command_id,
            &[("project_root", "$DATUM_PROJECT_ROOT")],
        )?;
        push_terminal_command_hit_region(hit_regions, rect, y, &command);
        draw_text(
            &format!("{label} {}", truncate_text(&command.command, 62)),
            rect.x + 24.0,
            y,
            10.5,
            TEXT_SECONDARY,
            TextFace::Mono,
            text_runs,
        );
        y += 15.0;
    }
    Some(y)
}

fn push_terminal_command_hit_region(
    hit_regions: &mut Vec<HitRegion>,
    rect: RectPx,
    y: f32,
    command: &TerminalCommandHandoff,
) {
    hit_regions.push(HitRegion {
        target: HitTarget::ProductionTerminalCommand(command.clone()),
        rect: RectPx {
            x: rect.x + 18.0,
            y: y - 2.0,
            width: (rect.width - 36.0).max(0.0),
            height: 14.0,
        },
    });
}

fn push_activity_hit_region(hit_regions: &mut Vec<HitRegion>, rect: RectPx, y: f32, summary: &str) {
    hit_regions.push(HitRegion {
        target: HitTarget::TerminalActivitySummary(summary.to_string()),
        rect: RectPx {
            x: rect.x + 12.0,
            y: y - 2.0,
            width: (rect.width - 24.0).max(0.0),
            height: 14.0,
        },
    });
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

        assert!(layout.terminal_tab.x < layout.assistant_tab.x);
        assert!(layout.assistant_tab.x < layout.outputs_tab.x);
        assert!(layout.terminal_tab.x + layout.terminal_tab.width <= layout.assistant_tab.x);
        assert!(layout.assistant_tab.x + layout.assistant_tab.width <= layout.outputs_tab.x);
        assert!(layout.content.y > layout.terminal_tab.y);
        assert!(layout.content.x >= shell.bottom_strip.x);
        assert!(
            layout.content.x + layout.content.width
                <= shell.bottom_strip.x + shell.bottom_strip.width
        );
    }
}
