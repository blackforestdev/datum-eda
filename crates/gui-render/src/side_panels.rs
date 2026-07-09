use super::*;

include!("side_panels/layout.rs");
include!("side_panels/render_project_filters.rs");
include!("side_panels/render_inspector.rs");
include!("side_panels/helpers.rs");

pub(super) fn render_side_panels(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    let left = layout.left_sidebar;
    let right = layout.right_sidebar;

    let project_layout = solve_project_panel_layout_with_taffy(state, left)
        .unwrap_or_else(|| fallback_project_panel_layout(state, left));
    let project_rect = project_layout.project_rect;
    let filters_rect = project_layout.filters_rect;
    let right_layout = solve_right_panel_layout_with_taffy(state, right)
        .unwrap_or_else(|| fallback_right_panel_layout(state, right));
    let inspector_rect = right_layout.inspector_rect;
    for (rect, title) in [
        (project_rect, "PROJECT"),
        (filters_rect, "LAYERS"),
        (inspector_rect, "INSPECTOR"),
    ] {
        // Raised card body, then a recessed header strip so the panel title
        // reads as a header bar (Design Book panel-hd), not floating text.
        panel_quads.push(Quad::from_rect(rect, PANEL_CARD_BG));
        let header = RectPx {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: UI_CARD_DIVIDER_Y,
        };
        panel_quads.push(Quad::from_rect(header, PANEL_BG));
        push_rect_border(panel_quads, rect, PANEL_CARD_BORDER, 1.0);
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
            rect.x + UI_CARD_PADDING_X,
            rect.y + UI_CARD_DIVIDER_Y,
            rect.width - UI_CARD_PADDING_X * 2.0,
            PANEL_CARD_BORDER,
        );
    }

    render_project_and_filters_panel(
        state,
        &project_layout,
        project_rect,
        filters_rect,
        panel_quads,
        text_runs,
        hit_regions,
    );
    render_inspector_panel(state, inspector_rect, panel_quads, text_runs, hit_regions);
}
