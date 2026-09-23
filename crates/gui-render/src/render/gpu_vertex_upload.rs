//! Per-frame GPU vertex-buffer synchronization.
//!
//! Keeping the buffer inventory together makes it harder to add an immediate
//! overlay without also defining its upload lifetime. This is a real child
//! module, rather than another textual inclusion into the renderer root.

use super::*;

// One inventory for cancellation, successful upload, and submission retention.
macro_rules! vertex_streams {
    ($this:ident, $stream:ident, $action:expr) => {{
        let $stream = &mut $this.panel_gpu;
        $action;
        let $stream = &mut $this.viewport_underlay_gpu;
        $action;
        let $stream = &mut $this.viewport_overlay_gpu;
        $action;
        let $stream = &mut $this.board_interaction_gpu;
        $action;
        let $stream = &mut $this.console_gpu.vertices;
        $action;
        let $stream = &mut $this.menu_overlay_gpu;
        $action;
        let $stream = &mut $this.schematic_underlay_gpu;
        $action;
        let $stream = &mut $this.schematic_overlay_gpu;
        $action;
        let $stream = &mut $this.surface_grid_gpu;
        $action;
        let $stream = &mut $this.world_vertices_gpu;
        $action;
        let $stream = &mut $this.world_strokes_gpu;
        $action;
        let $stream = &mut $this.schematic_world_vertices_gpu;
        $action;
        let $stream = &mut $this.schematic_world_strokes_gpu;
        $action;
    }};
}

impl Renderer {
    pub(super) fn cancel_vertex_uploads(&mut self) {
        vertex_streams!(self, stream, stream.cancel_uploads());
        self.terminal_graphics.cancel_uploads();
    }

    pub(super) fn flush_vertex_uploads(&mut self, queue: &wgpu::Queue) {
        vertex_streams!(self, stream, stream.flush_uploads(queue));
        self.terminal_graphics.flush_uploads(queue);
    }

    pub(super) fn vertex_submission_refs(
        &mut self,
    ) -> Vec<crate::text_gpu::lifetime::SubmissionRef> {
        let mut refs = Vec::new();
        vertex_streams!(self, stream, refs.extend(stream.submission_ref()));
        refs.extend(self.terminal_graphics.submission_refs());
        refs
    }

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
