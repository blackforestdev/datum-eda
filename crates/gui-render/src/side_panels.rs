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
        // Flush stacked panel: body and header strip are the SAME SURFACE_01
        // material; the 28px header is distinguished only by its uppercase title
        // and a single BORDER_SUBTLE bottom divider (Design Book panel-hd). No
        // per-card box border — panels read as a contiguous stack, not widgets.
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
    // One BORDER_SUBTLE divider separating the stacked PROJECT and LAYERS panels
    // in the left column (their bodies are contiguous SURFACE_01).
    push_section_divider(
        panel_quads,
        project_rect.x,
        project_rect.y + project_rect.height - 1.0,
        project_rect.width,
        PANEL_CARD_BORDER,
    );
    // Single outer edge border per column (left column right edge, right column
    // left edge) — the column's only chrome outline, matching `.col` borders.
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
