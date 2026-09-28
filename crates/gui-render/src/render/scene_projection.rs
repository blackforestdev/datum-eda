//! Board immediate overlay projection; ordered panes own backgrounds and grids.
use super::*;

// Render helper threads many quad/text-run/hit-region sinks.
#[allow(clippy::too_many_arguments)]
pub(super) fn render_scene(
    state: &ReviewWorkspaceState,
    scene_viewport: RectPx,
    camera: CameraState,
    viewport_overlay_quads: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    hit_regions: &mut Vec<HitRegion>,
) {
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
