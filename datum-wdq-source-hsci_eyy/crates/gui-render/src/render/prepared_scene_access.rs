//! Buffer and hit-region access owned by the prepared-scene projection.
//!
//! Keeping these accessors together separates prepared-frame consumption from
//! scene construction while preserving one `PreparedScene` authority.

use super::*;

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
