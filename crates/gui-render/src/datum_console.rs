//! Candidate-A output-only Datum Console overlay (decision 033).

use super::{
    ConsoleOverlayLayout, Quad, RectPx, ReviewWorkspaceState, ShellLayout, TextFace, TextRun,
    design_tokens, draw_text_clipped, estimated_text_run_width_px,
};
use datum_gui_protocol::{ConsoleFeedbackCategory, ConsoleFeedbackSeverity};

const MAX_PANE_WIDTH_FRACTION: f32 = 0.72;
const TEXT_SIZE: f32 = 12.0;

pub(super) fn render_datum_console(
    state: &ReviewWorkspaceState,
    shell: &ShellLayout,
    scale: f32,
    quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
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

    Some(ConsoleOverlayLayout {
        pane_id: focused.id,
        pane_body: body,
        strip,
        text_clip,
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use datum_gui_protocol::{ConsoleFeedbackDraft, ConsoleFeedbackSource, DockTab, PaneId};

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
                let layout = render_datum_console(&state, &shell, scale, &mut quads, &mut text)
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
        let layout = render_datum_console(&state, &shell, 1.0, &mut quads, &mut text).unwrap();

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
            render_datum_console(&empty, &shell, 1.0, &mut Vec::new(), &mut Vec::new()).is_none()
        );

        let state = state_with(ConsoleFeedbackDraft::action_refusal(
            ConsoleFeedbackSource::Menu,
            42,
            "Rotate is unavailable",
        ));
        let mut quads = Vec::new();
        let mut text = Vec::new();
        render_datum_console(&state, &shell, 1.0, &mut quads, &mut text).unwrap();
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
}
