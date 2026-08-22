//! Candidate-A output-only Datum Console overlay (decision 033).

use super::{
    ConsoleOverlayLayout, HitRegion, HitTarget, Quad, RectPx, ReviewWorkspaceState, ShellLayout,
    TextFace, TextRun, design_tokens, draw_text_clipped, estimated_text_run_width_px,
};
use datum_gui_protocol::{ConsoleFeedbackCategory, ConsoleFeedbackSeverity, ConsoleHistoryFilter};

const MAX_PANE_WIDTH_FRACTION: f32 = 0.72;
const TEXT_SIZE: f32 = 12.0;

pub(super) fn render_datum_console(
    state: &ReviewWorkspaceState,
    shell: &ShellLayout,
    scale: f32,
    quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) -> Option<ConsoleOverlayLayout> {
    let record = state.ui.console.latest()?;
    let panes = shell.viewport_panes(&state.ui.layout);
    let focused = panes.focused_pane();
    let body = focused.rect.body();

    let left = 12.0 * scale;
    let bottom = 10.0 * scale;
    let pad_x = 12.0 * scale;
    let pad_y = 5.0 * scale;
    let row_height = 16.0 * scale;
    let strip_height = row_height + pad_y * 2.0;
    let max_width = body.width * MAX_PANE_WIDTH_FRACTION;
    if body.width < 96.0 * scale || body.height < strip_height + bottom || max_width <= pad_x * 2.0
    {
        return None;
    }

    let (glyph, semantic_prefix, text_color) = presentation(record.category, record.severity);
    let visible_text = format!("{glyph} {semantic_prefix}{}", record.message);
    let natural_text_width =
        estimated_text_run_width_px(&visible_text, TEXT_SIZE * scale, TextFace::Mono) - 16.0;
    let strip_width = (natural_text_width + pad_x * 2.0)
        .min(max_width)
        .max((pad_x * 2.0 + 1.0).min(max_width));
    let strip = RectPx {
        x: body.x + left,
        y: body.y + body.height - bottom - strip_height,
        width: strip_width,
        height: strip_height,
    };
    let text_clip = RectPx {
        x: strip.x + pad_x,
        y: strip.y + pad_y,
        width: (strip.width - pad_x * 2.0).max(1.0),
        height: row_height,
    };

    push_card(quads, strip, record.severity, scale);
    draw_text_clipped(
        &visible_text,
        text_clip.x,
        text_clip.y,
        TEXT_SIZE,
        text_color,
        TextFace::Mono,
        text_clip,
        text_runs,
    );
    hit_regions.push(HitRegion {
        target: HitTarget::ConsoleHistoryToggle,
        rect: strip,
    });

    let history_panel = if state.ui.console.history_expanded() {
        Some(render_history(
            state,
            body,
            strip,
            scale,
            quads,
            text_runs,
            hit_regions,
        ))
    } else {
        None
    };

    Some(ConsoleOverlayLayout {
        pane_id: focused.id,
        pane_body: body,
        strip,
        text_clip,
        history_panel,
    })
}

#[allow(clippy::too_many_arguments)]
fn render_history(
    state: &ReviewWorkspaceState,
    body: RectPx,
    strip: RectPx,
    scale: f32,
    quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) -> RectPx {
    let gap = 6.0 * scale;
    let width = (520.0 * scale).min((body.width - 24.0 * scale).max(1.0));
    let max_height = 170.0 * scale;
    let available = (strip.y - body.y - gap).max(1.0);
    let height = max_height.min(available);
    let panel = RectPx {
        x: strip.x,
        y: strip.y - gap - height,
        width,
        height,
    };
    quads.push(Quad::from_rect(panel, design_tokens::chrome::SURFACE_01));
    let header_h = 26.0 * scale;
    let header = RectPx {
        height: header_h,
        ..panel
    };
    quads.push(Quad::from_rect(header, design_tokens::chrome::SURFACE_02));
    push_border(quads, panel, design_tokens::chrome::BORDER_STRONG, scale);
    draw_text_clipped(
        "SESSION HISTORY",
        panel.x + 11.0 * scale,
        panel.y + 6.0 * scale,
        10.5,
        design_tokens::chrome::TEXT_SECONDARY,
        TextFace::UiStrong,
        header,
        text_runs,
    );
    hit_regions.push(HitRegion {
        target: HitTarget::ConsoleHistoryToggle,
        rect: header,
    });

    let chip_w = 48.0 * scale;
    let chip_gap = 5.0 * scale;
    let chips = [
        (ConsoleHistoryFilter::All, "all"),
        (ConsoleHistoryFilter::Operations, "ops"),
        (ConsoleHistoryFilter::Errors, "errors"),
    ];
    let mut chip_x = panel.x + panel.width - 11.0 * scale - chip_w * 3.0 - chip_gap * 2.0;
    for (filter, label) in chips {
        let chip = RectPx {
            x: chip_x,
            y: panel.y + 5.0 * scale,
            width: chip_w,
            height: 16.0 * scale,
        };
        let selected = state.ui.console.history_filter() == filter;
        push_border(
            quads,
            chip,
            if selected {
                design_tokens::chrome::ACCENT
            } else {
                design_tokens::chrome::BORDER_SUBTLE
            },
            scale,
        );
        draw_text_clipped(
            label,
            chip.x + 6.0 * scale,
            chip.y + 1.0 * scale,
            9.5,
            if selected {
                design_tokens::chrome::ACCENT
            } else {
                design_tokens::chrome::TEXT_MUTED
            },
            TextFace::Mono,
            chip,
            text_runs,
        );
        hit_regions.push(HitRegion {
            target: HitTarget::ConsoleHistoryFilter(filter),
            rect: chip,
        });
        chip_x += chip_w + chip_gap;
    }

    let rows_clip = RectPx {
        x: panel.x + 1.0 * scale,
        y: panel.y + header_h,
        width: panel.width - 2.0 * scale,
        height: panel.height - header_h - 1.0 * scale,
    };
    let rows = history_rows(state);
    let row_h = 19.0 * scale;
    let visible_count = (rows_clip.height / row_h).floor().max(0.0) as usize;
    let scroll = state.ui.console.history_scroll_offset();
    let end = rows.len().saturating_sub(scroll.min(rows.len()));
    let start = end.saturating_sub(visible_count);
    let mut y = rows_clip.y + 3.0 * scale;
    for row in &rows[start..end] {
        draw_text_clipped(
            &row.text,
            rows_clip.x + 10.0 * scale,
            y,
            11.5,
            row.color,
            TextFace::Mono,
            rows_clip,
            text_runs,
        );
        y += row_h;
    }
    panel
}

struct HistoryRow {
    text: String,
    color: [f32; 3],
}

fn history_rows(state: &ReviewWorkspaceState) -> Vec<HistoryRow> {
    let filter = state.ui.console.history_filter();
    let mut rows = Vec::new();
    if filter != ConsoleHistoryFilter::Operations {
        if state.ui.console.dropped_count() > 0 {
            rows.push(HistoryRow {
                text: format!(
                    "… {} earlier feedback records omitted",
                    state.ui.console.dropped_count()
                ),
                color: design_tokens::chrome::TEXT_MUTED,
            });
        }
        rows.extend(state.ui.console.records().filter_map(|record| {
            if filter == ConsoleHistoryFilter::Errors
                && !matches!(
                    record.severity,
                    ConsoleFeedbackSeverity::Warning | ConsoleFeedbackSeverity::Error
                )
            {
                return None;
            }
            let seconds = (record.occurred_unix_ms / 1_000) % 86_400;
            Some(HistoryRow {
                text: format!(
                    "{:02}:{:02}:{:02}  {}  gui",
                    seconds / 3_600,
                    (seconds / 60) % 60,
                    seconds % 60,
                    record.message
                ),
                color: if record.severity == ConsoleFeedbackSeverity::Error {
                    design_tokens::chrome::TEXT_PRIMARY
                } else {
                    design_tokens::chrome::TEXT_SECONDARY
                },
            })
        }));
    }
    if filter != ConsoleHistoryFilter::Errors {
        if state.ui.console_journal.omitted_session_record_count() > 0 {
            rows.push(HistoryRow {
                text: format!(
                    "… {} earlier session operations omitted",
                    state.ui.console_journal.omitted_session_record_count()
                ),
                color: design_tokens::chrome::TEXT_MUTED,
            });
        }
        rows.extend(state.ui.console_journal.records().map(|record| HistoryRow {
            text: format!(
                "#{}  {}  op·journal #{}",
                record.journal_ordinal, record.reason, record.journal_ordinal
            ),
            color: design_tokens::chrome::ACCENT_HOVER,
        }));
    }
    rows
}

fn presentation(
    category: ConsoleFeedbackCategory,
    severity: ConsoleFeedbackSeverity,
) -> (&'static str, &'static str, [f32; 3]) {
    match category {
        ConsoleFeedbackCategory::ActionEcho => (
            "·",
            "",
            if severity == ConsoleFeedbackSeverity::Success {
                design_tokens::chrome::STATUS_SUCCESS
            } else {
                design_tokens::chrome::TEXT_SECONDARY
            },
        ),
        ConsoleFeedbackCategory::ToolPrompt => ("◇", "Tool: ", design_tokens::chrome::TEXT_PRIMARY),
        ConsoleFeedbackCategory::ActionRefusal => (
            "!",
            "Refused: ",
            if severity == ConsoleFeedbackSeverity::Warning {
                design_tokens::chrome::STATUS_WARN
            } else {
                design_tokens::chrome::TEXT_PRIMARY
            },
        ),
    }
}

fn push_card(quads: &mut Vec<Quad>, rect: RectPx, severity: ConsoleFeedbackSeverity, scale: f32) {
    quads.push(Quad::from_rect(rect, design_tokens::chrome::SURFACE_01));
    let border = if severity == ConsoleFeedbackSeverity::Error {
        design_tokens::chrome::STATUS_ERROR
    } else {
        design_tokens::chrome::BORDER_SUBTLE
    };
    let hairline = scale.max(1.0);
    for edge in [
        RectPx {
            width: rect.width,
            height: hairline,
            ..rect
        },
        RectPx {
            y: rect.y + rect.height - hairline,
            width: rect.width,
            height: hairline,
            ..rect
        },
        RectPx {
            width: hairline,
            ..rect
        },
        RectPx {
            x: rect.x + rect.width - hairline,
            width: hairline,
            ..rect
        },
    ] {
        quads.push(Quad::from_rect(edge, border));
    }
    if severity == ConsoleFeedbackSeverity::Error {
        quads.push(Quad::from_rect(
            RectPx {
                width: 2.0 * scale,
                ..rect
            },
            design_tokens::chrome::STATUS_ERROR,
        ));
    }
}

fn push_border(quads: &mut Vec<Quad>, rect: RectPx, color: [f32; 3], scale: f32) {
    let hairline = scale.max(1.0);
    for edge in [
        RectPx {
            width: rect.width,
            height: hairline,
            ..rect
        },
        RectPx {
            y: rect.y + rect.height - hairline,
            width: rect.width,
            height: hairline,
            ..rect
        },
        RectPx {
            width: hairline,
            ..rect
        },
        RectPx {
            x: rect.x + rect.width - hairline,
            width: hairline,
            ..rect
        },
    ] {
        quads.push(Quad::from_rect(edge, color));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datum_gui_protocol::{
        ConsoleFeedbackDraft, ConsoleFeedbackSource, ConsoleJournalProjectionRecord, DockTab,
        PaneId,
    };

    fn state_with(draft: ConsoleFeedbackDraft) -> ReviewWorkspaceState {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.ui.console.publish(draft);
        state
    }

    fn assert_inside(inner: RectPx, outer: RectPx) {
        assert!(inner.x >= outer.x);
        assert!(inner.y >= outer.y);
        assert!(inner.x + inner.width <= outer.x + outer.width + 0.01);
        assert!(inner.y + inner.height <= outer.y + outer.height + 0.01);
    }

    #[test]
    fn overlay_follows_exactly_one_focused_pane_across_scale_matrix() {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let width = (1280.0 * scale) as u32;
            let height = (800.0 * scale) as u32;
            for focused in [PaneId(0), PaneId(1)] {
                let mut state = state_with(ConsoleFeedbackDraft::action_echo(
                    ConsoleFeedbackSource::Viewport,
                    42,
                    "fit board",
                ));
                state.ui.layout.focused = focused;
                let shell = ShellLayout::for_surface(width, height, scale, None);
                let mut quads = Vec::new();
                let mut text = Vec::new();
                let layout = render_datum_console(
                    &state,
                    &shell,
                    scale,
                    &mut quads,
                    &mut text,
                    &mut Vec::new(),
                )
                .expect("visible record renders");

                assert_eq!(layout.pane_id, focused);
                assert_inside(layout.strip, layout.pane_body);
                assert!(layout.strip.width <= layout.pane_body.width * 0.72 + 0.01);
                assert_eq!(text.len(), 1);
                assert!(text[0].text.starts_with("· "));
            }
        }
    }

    #[test]
    fn maximized_terminal_open_and_narrow_layouts_preserve_body_anchor() {
        let mut state = state_with(ConsoleFeedbackDraft::tool_prompt(
            ConsoleFeedbackSource::Tool,
            42,
            "Move — select a component",
        ));
        state.ui.layout.focused = PaneId(1);
        state.ui.layout.zoomed = Some(PaneId(1));
        state.ui.active_dock_tab = Some(DockTab::Terminal);
        let shell = ShellLayout::for_surface(900, 700, 1.0, Some(220));
        let shell_before = shell.clone();
        let mut quads = Vec::new();
        let mut text = Vec::new();
        let layout =
            render_datum_console(&state, &shell, 1.0, &mut quads, &mut text, &mut Vec::new())
                .unwrap();

        assert_eq!(shell, shell_before, "Console reserves no shell geometry");
        assert_eq!(layout.pane_id, PaneId(1));
        assert_inside(layout.strip, layout.pane_body);
        assert!(layout.strip.y + layout.strip.height <= shell.bottom_strip.y);
        assert!(layout.strip.width <= layout.pane_body.width * 0.72 + 0.01);
        assert_eq!(text[0].clip_bounds, Some(layout.text_clip));
        assert!(!text[0].text.contains('\n'));
        assert!(text[0].text.starts_with("◇ Tool: "));
    }

    #[test]
    fn refusal_has_textual_semantics_and_empty_state_emits_nothing() {
        let shell = ShellLayout::for_surface(1280, 800, 1.0, None);
        let empty = datum_gui_protocol::load_fixture_workspace_state();
        assert!(
            render_datum_console(
                &empty,
                &shell,
                1.0,
                &mut Vec::new(),
                &mut Vec::new(),
                &mut Vec::new(),
            )
            .is_none()
        );

        let state = state_with(ConsoleFeedbackDraft::action_refusal(
            ConsoleFeedbackSource::Menu,
            42,
            "Rotate is unavailable",
        ));
        let mut quads = Vec::new();
        let mut text = Vec::new();
        render_datum_console(&state, &shell, 1.0, &mut quads, &mut text, &mut Vec::new()).unwrap();
        assert!(text[0].text.starts_with("! Refused: "));
        assert!(quads.len() >= 6, "error card carries border and left rule");
    }

    #[test]
    fn prepared_scene_threads_console_geometry_and_clipped_text() {
        let state = state_with(ConsoleFeedbackDraft::action_echo(
            ConsoleFeedbackSource::Selection,
            42,
            "selected U1",
        ));
        let retained = super::super::RetainedScene::from_workspace(&state, 1280, 800);
        let prepared = super::super::PreparedScene::from_workspace(
            &state,
            1280,
            800,
            super::super::CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
        );

        let layout = prepared
            .console_overlay_layout()
            .expect("prepared scene retains Console placement");
        assert_eq!(layout.pane_id, state.ui.layout.focused);
        assert!(!prepared.console_overlay_vertices().is_empty());
        assert!(prepared.text_runs.iter().any(|run| {
            run.text == "· selected U1" && run.clip_bounds == Some(layout.text_clip)
        }));
    }

    #[test]
    fn expanded_history_distinguishes_feedback_from_ordinal_journal_truth() {
        let mut state = state_with(ConsoleFeedbackDraft::action_refusal(
            ConsoleFeedbackSource::Tool,
            50_000,
            "no board text selected",
        ));
        state.ui.console.set_history_expanded(true);
        state.ui.console_journal.begin_session(40);
        state.ui.console_journal.reconcile(
            &[ConsoleJournalProjectionRecord {
                journal_ordinal: 41,
                transaction_id: "tx-41".to_string(),
                transaction_kind: "normal".to_string(),
                commit_source: "manual".to_string(),
                reason: "move U4".to_string(),
                operation_count: 1,
                created_count: 0,
                modified_count: 1,
                deleted_count: 0,
            }],
            41,
        );
        let shell = ShellLayout::for_surface(1280, 800, 1.0, None);
        let mut quads = Vec::new();
        let mut text = Vec::new();
        let mut hits = Vec::new();
        let layout =
            render_datum_console(&state, &shell, 1.0, &mut quads, &mut text, &mut hits).unwrap();

        assert!(layout.history_panel.is_some());
        assert!(text.iter().any(|run| run.text.contains("gui")));
        assert!(
            text.iter()
                .any(|run| run.text == "#41  move U4  op·journal #41")
        );
        assert!(hits.iter().any(|hit| {
            matches!(
                hit.target,
                HitTarget::ConsoleHistoryFilter(ConsoleHistoryFilter::Operations)
            )
        }));
    }
}
