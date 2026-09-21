//! Shared side-panel headings, dividers and outer borders.
use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn render(
    project_rect: RectPx,
    filters_rect: RectPx,
    inspector_rect: RectPx,
    left: RectPx,
    right: RectPx,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
) {
    for (rect, title) in [
        (project_rect, "PROJECT"),
        (filters_rect, "LAYERS"),
        (inspector_rect, "INSPECTOR"),
    ] {
        panel_quads.push(Quad::from_rect(rect, PANEL_CARD_BG));
        draw_text(
            title,
            rect.x + UI_CARD_PADDING_X,
            rect.y + UI_CARD_TITLE_Y,
            design_tokens::typography::HEADER_SIZE,
            TEXT_SECONDARY,
            TextFace::UiStrong,
            text_runs,
        );
        push_section_divider(
            panel_quads,
            rect.x,
            rect.y + UI_CARD_DIVIDER_Y,
            rect.width,
            PANEL_CARD_BORDER,
        );
    }
    push_section_divider(
        panel_quads,
        project_rect.x,
        project_rect.y + project_rect.height - 1.0,
        project_rect.width,
        PANEL_CARD_BORDER,
    );
    panel_quads.push(Quad::from_rect(
        RectPx {
            x: left.x + left.width - 1.0,
            y: left.y,
            width: 1.0,
            height: left.height,
        },
        PANEL_CARD_BORDER,
    ));
    panel_quads.push(Quad::from_rect(
        RectPx {
            x: right.x,
            y: right.y,
            width: 1.0,
            height: right.height,
        },
        PANEL_CARD_BORDER,
    ));
}
