use super::*;

pub(super) fn render_active_inspector(
    state: &ReviewWorkspaceState,
    rect: RectPx,
    panel_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    if !revision_workspace::render_evidence_inspector(state, rect, panel_quads, text_runs) {
        render_inspector_panel(state, rect, panel_quads, text_runs, hit_regions);
    }
}
