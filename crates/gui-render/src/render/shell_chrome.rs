//! Shell chrome composition; world and terminal device geometry remain separate.
use super::*;

pub(super) fn render_phase1_shell_chrome(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panel_quads: &mut global_preferences_primitives::ControlPainter<'_>,
    text_runs: &mut Vec<TextRun>,
) -> anyhow::Result<crate::resource_consumers::Consumers> {
    // Menu bar carries only a bottom hairline (Design Book .menubar
    // border-bottom), never a boxed 4-sided outline.
    panel_quads.push(Quad::from_rect(
        RectPx {
            x: layout.top_menu_bar.x,
            y: layout.top_menu_bar.y + layout.top_menu_bar.height - 1.0,
            width: layout.top_menu_bar.width,
            height: 1.0,
        },
        if state.ui.global_preferences.high_contrast_noncolor {
            TEXT_PRIMARY
        } else {
            PANEL_CARD_BORDER
        },
    ));
    if state.ui.global_preferences.high_contrast_noncolor {
        push_rect_border(
            panel_quads,
            RectPx {
                x: layout.top_menu_bar.x,
                y: layout.top_menu_bar.y,
                width: layout.top_menu_bar.width,
                height: layout.status_bar.y + layout.status_bar.height,
            },
            TEXT_PRIMARY,
            2.0,
        );
        draw_text_clipped(
            "[HC] HIGH CONTRAST + NON-COLOR CUES",
            layout.top_menu_bar.x + layout.top_menu_bar.width - 430.0,
            layout.status_bar.y + design_tokens::spacing::SP_02,
            design_tokens::typography::CAPTION_SIZE,
            TEXT_PRIMARY,
            TextFace::Mono,
            layout.status_bar,
            text_runs,
        );
    }
    shell_identity::render_shell_identity(state, layout, panel_quads, text_runs)?;

    let panes = render_viewport_panes(
        layout,
        &state.ui.layout,
        state.schematic_scene.is_some(),
        panel_quads,
        text_runs,
    );
    status_bar::render_status_bar(state, layout, panel_quads, text_runs);
    Ok(panes)
}
