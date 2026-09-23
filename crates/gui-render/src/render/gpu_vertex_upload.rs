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
        console_overlay: &[Vertex],
        menu_overlay: &[Vertex],
        world: &gpu_data::shared_geometry::SharedGeometry<Vertex>,
        schematic_world: Option<&RetainedScene>,
        schematic_underlay: &[Vertex],
        schematic_overlay: &[Vertex],
    ) {
        self.panel_gpu
            .sync(device, queue, "datum-gui-render-panel-vertex-buffer", panel);
        self.viewport_underlay_gpu.sync(
            device,
            queue,
            "datum-gui-render-viewport-underlay-vertex-buffer",
            viewport_underlay,
        );
        self.viewport_overlay_gpu.sync(
            device,
            queue,
            "datum-gui-render-viewport-overlay-vertex-buffer",
            viewport_overlay,
        );
        self.board_interaction_gpu.sync(
            device,
            queue,
            "datum-gui-render-board-interaction-vertex-buffer",
            board_interaction,
        );
        self.console_gpu.upload(device, queue, console_overlay);
        self.menu_overlay_gpu.sync(
            device,
            queue,
            "datum-gui-render-menu-overlay-vertex-buffer",
            menu_overlay,
        );
        self.world_vertices_gpu
            .sync(device, queue, "datum-world-vertices", world);
        if let Some(scene) = schematic_world {
            self.schematic_world_vertices_gpu.sync(
                device,
                queue,
                "datum-schematic-world-vertices",
                &scene.world_vertices,
            );
        } else {
            self.schematic_world_vertices_gpu.clear();
            self.schematic_world_strokes_gpu.clear();
        }
        self.schematic_underlay_gpu.sync(
            device,
            queue,
            "datum-gui-render-schematic-underlay-vertex-buffer",
            schematic_underlay,
        );
        self.schematic_overlay_gpu.sync(
            device,
            queue,
            "datum-gui-render-schematic-overlay-vertex-buffer",
            schematic_overlay,
        );
    }
}
