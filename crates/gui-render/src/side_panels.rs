use super::*;

mod inspector_dispatch;
mod layer_scroll;
mod panel_chrome;
include!("side_panels/layout.rs");
mod inspector_chrome;
mod render_inspector;
mod render_project_filters;
include!("side_panels/helpers.rs");
pub(super) fn render_side_panels(
    state: &ReviewWorkspaceState,
    layout: &ShellLayout,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) -> anyhow::Result<()> {
    let left = layout.left_sidebar;
    let right = layout.right_sidebar;

    let project_layout = solve_project_panel_layout_with_taffy(state, left)
        .unwrap_or_else(|| fallback_project_panel_layout(state, left));
    let project_rect = project_layout.project_rect;
    let filters_rect = project_layout.filters_rect;
    let right_layout = solve_right_panel_layout_with_taffy(state, right)
        .unwrap_or_else(|| fallback_right_panel_layout(state, right));
    let inspector_rect = right_layout.inspector_rect;
    panel_chrome::render(
        project_rect,
        filters_rect,
        inspector_rect,
        left,
        right,
        panel_quads,
        text_runs,
    );
    render_project_filters::render_project_and_filters_panel(
        state,
        &project_layout,
        project_rect,
        filters_rect,
        panel_quads,
        text_runs,
        hit_regions,
    );
    inspector_dispatch::render_active_inspector(
        state,
        inspector_rect,
        panel_quads,
        text_runs,
        hit_regions,
    )?;

    Ok(())
}
