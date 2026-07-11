//! Per-frame GPU vertex-buffer synchronization.
//!
//! Keeping the buffer inventory together makes it harder to add an immediate
//! overlay without also defining its upload lifetime. This is a real child
//! module, rather than another textual inclusion into the renderer root.

use super::*;

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn upload_frame_vertices(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        panel: &[Vertex],
        viewport_underlay: &[Vertex],
        viewport_overlay: &[Vertex],
        board_interaction: &[Vertex],
        menu_overlay: &[Vertex],
        world: &[Vertex],
        schematic_world: Option<&RetainedScene>,
        schematic_underlay: &[Vertex],
        schematic_overlay: &[Vertex],
    ) {
        Self::upload_vertices(
            device,
            queue,
            &mut self.panel_vertex_buffer,
            &mut self.panel_vertex_capacity,
            "datum-gui-render-panel-vertex-buffer",
            panel,
        );
        Self::upload_vertices(
            device,
            queue,
            &mut self.viewport_underlay_vertex_buffer,
            &mut self.viewport_underlay_vertex_capacity,
            "datum-gui-render-viewport-underlay-vertex-buffer",
            viewport_underlay,
        );
        Self::upload_vertices(
            device,
            queue,
            &mut self.viewport_overlay_vertex_buffer,
            &mut self.viewport_overlay_vertex_capacity,
            "datum-gui-render-viewport-overlay-vertex-buffer",
            viewport_overlay,
        );
        Self::upload_vertices(
            device,
            queue,
            &mut self.board_interaction_vertex_buffer,
            &mut self.board_interaction_vertex_capacity,
            "datum-gui-render-board-interaction-vertex-buffer",
            board_interaction,
        );
        Self::upload_vertices(
            device,
            queue,
            &mut self.menu_overlay_vertex_buffer,
            &mut self.menu_overlay_vertex_capacity,
            "datum-gui-render-menu-overlay-vertex-buffer",
            menu_overlay,
        );
        self.sync_world_vertices(device, queue, world);

        if let Some(scene) = schematic_world {
            Self::upload_vertices(
                device,
                queue,
                &mut self.schematic_world_vertex_buffer,
                &mut self.schematic_world_vertex_capacity,
                "datum-gui-render-schematic-world-vertex-buffer",
                scene.world_vertices(),
            );
        }
        if !schematic_underlay.is_empty() {
            Self::upload_vertices(
                device,
                queue,
                &mut self.schematic_underlay_vertex_buffer,
                &mut self.schematic_underlay_vertex_capacity,
                "datum-gui-render-schematic-underlay-vertex-buffer",
                schematic_underlay,
            );
        }
        if !schematic_overlay.is_empty() {
            Self::upload_vertices(
                device,
                queue,
                &mut self.schematic_overlay_vertex_buffer,
                &mut self.schematic_overlay_vertex_capacity,
                "datum-gui-render-schematic-overlay-vertex-buffer",
                schematic_overlay,
            );
        }
    }
}
