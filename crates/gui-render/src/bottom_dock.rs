use datum_gui_protocol::{
    DockTab, ReviewWorkspaceState, SelectionTarget, TerminalCommandHandoff,
    render_terminal_command_handoff,
};

use super::outputs_lane::render_outputs_lane;
use super::{
    HitRegion, HitTarget, PANEL_CARD_BORDER, Quad, RectPx, ShellLayout, TEXT_MUTED,
    TEXT_PANEL_VALUE, TEXT_PRIMARY, TEXT_SECONDARY, TextFace, TextRun, draw_text, push_rect_border,
    suffix_id, truncate_text, workspace_tool_label,
};

pub(super) fn render_bottom_tabs(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let terminal_rect = RectPx {
        x: layout.bottom_strip.x + 12.0,
        y: layout.bottom_strip.y + 8.0,
        width: 120.0,
        height: layout.bottom_strip.height - 16.0,
    };
    let assistant_rect = RectPx {
        x: layout.bottom_strip.x + 140.0,
        y: layout.bottom_strip.y + 8.0,
        width: 120.0,
        height: layout.bottom_strip.height - 16.0,
    };
    let outputs_rect = RectPx {
        x: layout.bottom_strip.x + 268.0,
        y: layout.bottom_strip.y + 8.0,
        width: 120.0,
        height: layout.bottom_strip.height - 16.0,
    };
    for (rect, label, target, active) in [
        (
            terminal_rect,
            "TERMINAL",
            HitTarget::TerminalTab,
            matches!(state.ui.active_dock_tab, Some(DockTab::Terminal)),
        ),
        (
            assistant_rect,
            "AGENTS",
            HitTarget::AssistantTab,
            matches!(state.ui.active_dock_tab, Some(DockTab::Assistant)),
        ),
        (
            outputs_rect,
            "OUTPUTS",
            HitTarget::OutputsTab,
            matches!(state.ui.active_dock_tab, Some(DockTab::Outputs)),
        ),
    ] {
        panel_quads.push(Quad::from_rect(
            rect,
            if active {
                [0.20, 0.22, 0.27]
            } else {
                [0.15, 0.16, 0.19]
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
    let handle_rect = RectPx {
        x: layout.bottom_strip.x,
        y: layout.bottom_strip.y,
        width: layout.bottom_strip.width,
        height: 6.0,
    };
    panel_quads.push(Quad::from_rect(handle_rect, [0.24, 0.26, 0.30]));
    hit_regions.push(HitRegion {
        target: HitTarget::DockResizeHandle,
        rect: handle_rect,
    });
    let content_rect = RectPx {
        x: layout.bottom_strip.x + 12.0,
        y: layout.bottom_strip.y + 44.0,
        width: layout.bottom_strip.width - 24.0,
        height: (layout.bottom_strip.height - 56.0).max(0.0),
    };
    panel_quads.push(Quad::from_rect(content_rect, [0.11, 0.12, 0.15]));
    push_rect_border(panel_quads, content_rect, PANEL_CARD_BORDER, 1.0);
    match active_tab {
        DockTab::Terminal => render_terminal_lane(state, content_rect, text_runs, hit_regions),
        DockTab::Assistant => render_assistant_lane(state, content_rect, text_runs, hit_regions),
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
    draw_text(
        &format!(
            "SHELL SESSION / {}",
            state.ui.terminal.status.to_uppercase()
        ),
        rect.x + 12.0,
        rect.y + 28.0,
        10.5,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    draw_text(
        "COPY SCROLLBACK CTRL+SHIFT+C  PASTE CTRL+V",
        rect.x + 12.0,
        rect.y + 43.0,
        10.5,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    let mut y = rect.y + 62.0;
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
    for line in state
        .ui
        .terminal
        .lines
        .iter()
        .skip(tail_start)
        .take(max_lines)
    {
        draw_text(
            &truncate_text(line, 180),
            rect.x + 12.0,
            y,
            11.0,
            TEXT_PANEL_VALUE,
            TextFace::Mono,
            text_runs,
        );
        y += 16.0;
    }
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

fn render_assistant_lane(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    draw_text(
        "AGENT SESSION",
        rect.x + 12.0,
        rect.y + 12.0,
        12.0,
        TEXT_SECONDARY,
        TextFace::Ui,
        text_runs,
    );
    draw_text(
        &format!(
            "PREFERRED: TERMINAL-LAUNCHED AGENTS  TOOL {}  SELECTION {}",
            workspace_tool_label(state.tool),
            selection_summary(state)
        ),
        rect.x + 12.0,
        rect.y + 28.0,
        10.5,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );
    let mut transcript_y = rect.y + 48.0;
    if !state.ui.terminal.activity_summary.is_empty() {
        draw_text(
            "RECENT ACTIVITY",
            rect.x + 12.0,
            transcript_y,
            10.5,
            TEXT_MUTED,
            TextFace::Mono,
            text_runs,
        );
        transcript_y += 16.0;
        for line in state.ui.terminal.activity_summary.iter().take(3) {
            draw_text(
                &truncate_text(line, 160),
                rect.x + 12.0,
                transcript_y,
                10.5,
                TEXT_SECONDARY,
                TextFace::Mono,
                text_runs,
            );
            push_activity_hit_region(hit_regions, rect, transcript_y, line);
            transcript_y += 15.0;
        }
        transcript_y += 4.0;
    }
    let max_lines = ((rect.y + rect.height - transcript_y - 28.0) / 18.0)
        .floor()
        .max(1.0) as usize;
    let total = state.ui.assistant.transcript.len();
    let scroll = state
        .ui
        .assistant
        .scroll_offset
        .min(total.saturating_sub(max_lines));
    let tail_start = total.saturating_sub(max_lines + scroll);
    let mut y = transcript_y;
    for msg in state
        .ui
        .assistant
        .transcript
        .iter()
        .skip(tail_start)
        .take(max_lines)
    {
        draw_text(
            &truncate_text(
                &format!("{}: {}", msg.role.to_uppercase(), msg.content),
                160,
            ),
            rect.x + 12.0,
            y,
            11.0,
            if msg.role == "assistant" {
                TEXT_PANEL_VALUE
            } else {
                TEXT_PRIMARY
            },
            if msg.role == "assistant" {
                TextFace::Ui
            } else {
                TextFace::Mono
            },
            text_runs,
        );
        y += 18.0;
    }
    let display_input = if state.ui.assistant.awaiting_api_key {
        if state.ui.assistant.input.is_empty() {
            "[enter api key]".to_string()
        } else {
            "[hidden]".to_string()
        }
    } else {
        let (before, after) = split_at_cursor(&state.ui.assistant.input, state.ui.assistant.cursor);
        format!("{before}|{after}")
    };
    draw_text(
        &format!("> {display_input}"),
        rect.x + 12.0,
        rect.y + rect.height - 18.0,
        11.5,
        TEXT_PRIMARY,
        TextFace::Mono,
        text_runs,
    );
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

fn selection_summary(state: &ReviewWorkspaceState) -> String {
    match &state.selection {
        SelectionTarget::ReviewAction(action_id) => truncate_text(
            &format!("REVIEW {}", suffix_id(action_id).to_uppercase()),
            28,
        ),
        SelectionTarget::AuthoredObject(object_id) => truncate_text(
            &format!("OBJECT {}", suffix_id(object_id).to_uppercase()),
            28,
        ),
        SelectionTarget::CheckFinding(fingerprint) => truncate_text(
            &format!("FINDING {}", suffix_id(fingerprint).to_uppercase()),
            28,
        ),
        SelectionTarget::Finding(finding_id) => truncate_text(
            &format!("FINDING {}", suffix_id(finding_id).to_uppercase()),
            28,
        ),
        SelectionTarget::JournalEntry(transaction_id) => truncate_text(
            &format!("JOURNAL {}", suffix_id(transaction_id).to_uppercase()),
            28,
        ),
        SelectionTarget::Relationship(relationship_id) => truncate_text(
            &format!("RELATION {}", suffix_id(relationship_id).to_uppercase()),
            28,
        ),
        SelectionTarget::None => "NONE".to_string(),
    }
}
