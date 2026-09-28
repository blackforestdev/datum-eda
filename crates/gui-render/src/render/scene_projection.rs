//! Board underlay and immediate overlay projection.
use super::*;

// Render helper threads many quad/text-run/hit-region sinks.
#[allow(clippy::too_many_arguments)]
pub(super) fn render_scene(
    state: &ReviewWorkspaceState,
    scene_viewport: RectPx,
    camera: CameraState,
    viewport_underlay_quads: &mut Vec<Quad>,
    viewport_overlay_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
    push_scene_underlay(
        viewport_underlay_quads,
        &state.scene,
        scene_viewport,
        camera,
    );
    let scene_hit_start = hit_regions.len();
    push_scene_overlay_and_hits(
        viewport_overlay_quads,
        &state.scene,
        scene_viewport,
        camera,
        state,
        text_runs,
        hit_regions,
    );
    hit_clipping::clip_new_hit_regions(hit_regions, scene_hit_start, scene_viewport);
    // The pane header and Project panel already name the document.
    // (Removed the "ACTIVE <action-id> / NET <name>" review-HUD overlay that
    // bled over the canvas top-left — internal selection state, not designed
    // board-pane chrome. The selection is reflected in the Inspector + status bar.)
    // (Removed the "F FIT / REVIEW NAV / CLICK SELECT / SCROLL ZOOM / ESC CLEAR"
    // keyboard-hint overlay that overflowed across the canvas top — not part of the
    // designed board pane; shortcuts belong in a proper help surface, not a HUD.)
    // (Removed the in-canvas TOOL/ZOOM/SEL status strip and the command-status
    // overlay that painted a PANEL_BG band across the bottom of the canvas.
    // These readouts belong in the global status bar (see M7), not floating on
    // the board field — the canvas stays a clean board surface.)
}

pub(super) fn push_scene_underlay(
    out: &mut Vec<Quad>,
    scene: &BoardReviewSceneV1,
    scene_viewport: RectPx,
    camera: CameraState,
) {
    // One uniform board substrate fills the ENTIRE viewport. Previously this was a
    // two-tone step — an outer CANVAS band around a 10px-inset InnerField
    // rectangle — whose boundary was only ever masked by the decorative gold edge
    // stroke. With that stroke removed (Bug A), the bare color step read as a
    // spurious grey border, so the field is now a single flat substrate.
    let board_field = inset_rect(scene_viewport, 10.0, 10.0, 10.0, 10.0);
    let projection = Projection::new(board_field, &scene.bounds, camera);
    out.push(Quad::from_rect(
        scene_viewport,
        board_surface_color(BoardSurfaceRole::InnerField),
    ));
    // No decorative board-edge stroke here: the only board outline is the REAL
    // projected Edge.Cuts, drawn from `scene.outline` in the retained world pass
    // (`push_retained_board_graphic_batches`). A fixed viewport-inset frame here
    // was not the true board bounds and read as spurious chrome. The board is still
    // projected into the 10px-inset `board_field` so it keeps a small margin.
    push_scene_grid(out, &projection);
}
