//! Menu-strip brand and revision paint with shared ancestor clipping.
use super::*;

/// Brand and revision belong to the menu strip, independent of pane chrome.
pub(super) fn render_shell_identity(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) {
    let starts = (panel_quads.len(), text_runs.len());
    // Brand wordmark: three runs on one baseline — "Datum" / accent middot /
    // "EDA" — advancing x by each measured run width so the middot is truly
    // colored and kerned, not a full "Datum EDA" string.
    let brand_size = 14.0;
    let brand_y = layout.top_menu_bar.y + design_tokens::spacing::SP_03;
    let mut brand_x = layout.top_menu_bar.x + design_tokens::spacing::SP_04;
    for (run, color) in [
        ("Datum", TEXT_PRIMARY),
        ("\u{00B7}", TEXT_ACCENT),
        ("EDA", TEXT_PRIMARY),
    ] {
        draw_text(
            run,
            brand_x,
            brand_y,
            brand_size,
            color,
            TextFace::UiStrong,
            text_runs,
        );
        brand_x += estimated_text_run_width_px(run, brand_size, TextFace::UiStrong) - 16.0;
    }
    // Rev pill: "{project} · rev {short-revision}" in a SURFACE_01 quad with a
    // BORDER_SUBTLE border, right-aligned to the menubar right edge.
    let short_rev: String = state.scene.source_revision.chars().take(6).collect();
    let rev_label = if short_rev.is_empty() {
        truncate_text(&state.scene.project_name, 30)
    } else {
        format!(
            "{} \u{00B7} rev {}",
            truncate_text(&state.scene.project_name, 24),
            short_rev
        )
    };
    let rev_text_w = estimated_text_run_width_px(
        &rev_label,
        design_tokens::typography::DATA_SIZE,
        TextFace::Mono,
    ) - 16.0;
    let pill_pad_x = design_tokens::spacing::SP_03;
    let pill_pad_y = design_tokens::spacing::SP_02;
    let pill_h = design_tokens::typography::DATA_SIZE + pill_pad_y * 2.0;
    let pill_w = rev_text_w + pill_pad_x * 2.0;
    let pill_x = (layout.top_menu_bar.x + layout.top_menu_bar.width
        - design_tokens::spacing::SP_03
        - pill_w)
        .max(layout.top_menu_bar.x);
    let pill_y = layout.top_menu_bar.y + (layout.top_menu_bar.height - pill_h) * 0.5;
    let pill_rect = RectPx {
        x: pill_x,
        y: pill_y,
        width: pill_w,
        height: pill_h,
    };
    panel_quads.push(Quad::from_rect(pill_rect, PANEL_BG));
    push_rect_border(panel_quads, pill_rect, PANEL_CARD_BORDER, 1.0);
    draw_text(
        &rev_label,
        pill_x + pill_pad_x,
        pill_y + pill_pad_y,
        design_tokens::typography::DATA_SIZE,
        TEXT_MUTED,
        TextFace::Mono,
        text_runs,
    );

    hit_clipping::clip_content(
        panel_quads,
        text_runs,
        &mut Vec::new(),
        starts.0,
        starts.1,
        0,
        layout.top_menu_bar,
    );
}

#[cfg(test)]
mod shell_identity_tests {
    use super::*;

    #[test]
    fn brand_and_long_revision_stay_inside_the_menu_strip() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.scene.project_name = "A deliberately long project name spanning a narrow menu".into();
        for width in [1, 25, 100, 1280] {
            let layout = ShellLayout::for_surface(width, 800, 1.0, None);
            let mut quads = Vec::new();
            let mut text = Vec::new();
            render_shell_identity(&state, &layout, &mut quads, &mut text);
            let bounds = layout.top_menu_bar;
            for quad in &quads {
                assert!(quad.points.iter().all(|&(x, y)| bounds.contains(x, y)));
            }
            assert!(!text.is_empty());
            for run in &text {
                assert_eq!(run.clip_bounds, Some(bounds));
            }
        }
    }
}
