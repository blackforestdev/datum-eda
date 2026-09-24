//! Buffer and hit-region access owned by the prepared-scene projection.
//!
//! Keeping these accessors together separates prepared-frame consumption from
//! scene construction while preserving one `PreparedScene` authority.

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedScene {
    pub(super) consumers: crate::resource_consumers::FrameConsumers,
    pub layout: ShellLayout,
    pub hit_regions: Vec<HitRegion>,
    pub scene_viewport: RectPx,
    pub(super) surface_passes: Vec<PreparedSurfacePass>,
    pub(super) board_pane_id: datum_gui_protocol::PaneId,
    pub(super) scene_bounds: datum_gui_protocol::SceneBounds,
    pub(super) camera: CameraState,
    pub(super) panel_vertices: Vec<Vertex>,
    pub(super) menu_overlay_vertices: Vec<Vertex>,
    pub(super) menu_overlay_text_runs: Vec<TextRun>,
    pub(super) viewport_underlay_vertices: Vec<Vertex>,
    pub(super) viewport_overlay_vertices: Vec<Vertex>,
    pub(super) board_interaction_vertices: Vec<Vertex>,
    pub(super) console_overlay_vertices: Vec<Vertex>,
    pub(super) console_overlay_layout: Option<ConsoleOverlayLayout>,
    pub(super) visible_draw_commands: Vec<RetainedDrawCommand>,
    pub(super) text_runs: Vec<TextRun>,
    pub(super) terminal_graphics: Vec<PreparedTerminalGraphic>,
    // P2.2a bounded second-scene descriptor: the STATIC companion schematic pass.
    // `Some` only when the layout has a Schematic pane AND the workspace carries a
    // projected `schematic_scene`; gates the additive second world GPU pass. The
    // camera is a fixed fit-to-schematic-bounds (no interactive pan/zoom on pane B
    // this slice). Ranges for that pass are derived in gpu.rs from the threaded
    // schematic RetainedScene (render() has no `state`), so they are not stored
    // here — the schematic renders all of its batches (its layers are always
    // visible, not board-layer-toggle governed).
    pub(super) schematic_scene_viewport: Option<RectPx>,
    pub(super) schematic_pane_id: Option<datum_gui_protocol::PaneId>,
    pub(super) schematic_bounds: datum_gui_protocol::SceneBounds,
    pub(super) schematic_camera: CameraState,
    // S4 (HoverEngine): the immediate screen-space interaction overlays are class-A
    // `ScreenConstant` chrome — driven by live hover, so they are empty in the
    // offscreen visual-test capture, keeping the board frame byte-identical. Board
    // hover folds straight into `viewport_overlay_vertices` at construction; the
    // schematic hover rides `schematic_underlay_vertices`, rebuilt when the warm
    // schematic camera is applied (`set_schematic_camera`) — hence the hovered
    // symbol's world bbox is retained here to re-project it against that camera.
    pub(super) schematic_hover_bounds_nm: Option<datum_gui_protocol::RectNm>,
    // S4 cursor crosshair (decision 023 UVT-005): the live cursor in device-pixel
    // SCREEN space and the user-selected style, retained so `set_schematic_camera`
    // (which has no `state`) can rebuild the schematic underlay crosshair against
    // the warm camera. `None` cursor in the offscreen capture keeps the frame
    // byte-identical; both the board and schematic panes read these.
    pub(super) crosshair_cursor_screen: Option<(f32, f32)>,
    pub(super) crosshair_style: datum_gui_protocol::CrosshairStyle,
    pub(super) schematic_underlay_vertices: Vec<Vertex>,
    pub(super) schematic_overlay_vertices: Vec<Vertex>,
}

impl PreparedScene {
    /// The immediate pre-world schematic grid underlay. S4 interaction chrome
    /// uses a separate post-world buffer with the same pane scissor.
    pub(super) fn schematic_underlay_vertices(&self) -> &[Vertex] {
        &self.schematic_underlay_vertices
    }

    pub(super) fn schematic_overlay_vertices(&self) -> &[Vertex] {
        &self.schematic_overlay_vertices
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<&HitTarget> {
        self.hit_regions
            .iter()
            .rev()
            .find(|region| region.rect.contains(x, y))
            .map(|region| &region.target)
    }

    // `world_point_at_screen` (per-pane screen->world resolve, UVT-004) lives in
    // the `coordinate_hit` module alongside the world hit-test.

    pub(super) fn panel_vertices(&self) -> &[Vertex] {
        &self.panel_vertices
    }

    /// Top-overlay quads for menus and application dialogs, composited after
    /// scissored viewport passes so work-pane content cannot overpaint them.
    pub(super) fn menu_overlay_vertices(&self) -> &[Vertex] {
        &self.menu_overlay_vertices
    }

    /// Text belonging to the top overlay, rendered after its opaque cards.
    pub(super) fn menu_overlay_text_runs(&self) -> &[TextRun] {
        &self.menu_overlay_text_runs
    }

    pub(super) fn viewport_underlay_vertices(&self) -> &[Vertex] {
        &self.viewport_underlay_vertices
    }

    pub(super) fn viewport_overlay_vertices(&self) -> &[Vertex] {
        &self.viewport_overlay_vertices
    }

    pub(super) fn board_interaction_vertices(&self) -> &[Vertex] {
        &self.board_interaction_vertices
    }

    pub(super) fn visible_draw_commands(&self) -> &[RetainedDrawCommand] {
        &self.visible_draw_commands
    }
}
