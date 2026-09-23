//! Per-frame GPU vertex-buffer synchronization.
//!
//! Keeping the buffer inventory together makes it harder to add an immediate
//! overlay without also defining its upload lifetime. This is a real child
//! module, rather than another textual inclusion into the renderer root.

use super::*;
#[path = "cold_world_upload.rs"]
mod cold_world;
pub(super) use cold_world::ColdWorldUploads;

// Shared inventories for plan collection, cancellation, consumption and retirement.
macro_rules! screen_streams {
    ($this:ident, $stream:ident, $action:expr $(, $mutability:ident)?) => {{
        let $stream = & $($mutability)? $this.panel_gpu;
        $action;
        let $stream = & $($mutability)? $this.viewport_underlay_gpu;
        $action;
        let $stream = & $($mutability)? $this.viewport_overlay_gpu;
        $action;
        let $stream = & $($mutability)? $this.board_interaction_gpu;
        $action;
        let $stream = & $($mutability)? $this.console_gpu.vertices;
        $action;
        let $stream = & $($mutability)? $this.menu_overlay_gpu;
        $action;
        let $stream = & $($mutability)? $this.schematic_underlay_gpu;
        $action;
        let $stream = & $($mutability)? $this.schematic_overlay_gpu;
        $action;
        let $stream = & $($mutability)? $this.surface_grid_gpu;
        $action;
    }};
}
pub(super) use screen_streams;

macro_rules! world_streams {
    ($this:ident, $stream:ident, $action:expr $(, $mutability:ident)?) => {{
        let $stream = & $($mutability)? $this.world_vertices_gpu;
        $action;
        let $stream = & $($mutability)? $this.world_strokes_gpu;
        $action;
        let $stream = & $($mutability)? $this.schematic_world_vertices_gpu;
        $action;
        let $stream = & $($mutability)? $this.schematic_world_strokes_gpu;
        $action;
    }};
}
impl Renderer {
    /// Retained dirty-range capacities and Datum headers, charged to staging.
    /// Includes empty reusable slots; snapshot payload accounting is separate.
    pub fn screen_upload_metadata_bytes(&self) -> u64 {
        let mut bytes = self.terminal_graphics.pending_vertex_metadata_bytes();
        screen_streams!(self, stream, bytes += stream.pending_metadata_bytes());
        bytes
    }

    pub(super) fn cancel_vertex_uploads(&mut self) {
        screen_streams!(self, stream, stream.cancel_uploads(), mut);
        world_streams!(self, stream, stream.cancel_uploads(), mut);
        self.terminal_graphics.cancel_uploads();
    }

    pub(super) fn finish_screen_uploads(&mut self) {
        screen_streams!(self, stream, stream.finish_uploads(), mut);
        self.terminal_graphics.finish_vertex_uploads();
    }

    pub(super) fn vertex_submission_refs(
        &mut self,
    ) -> Vec<crate::text_gpu::lifetime::SubmissionRef> {
        let mut refs = Vec::new();
        screen_streams!(self, stream, refs.extend(stream.submission_ref()));
        world_streams!(self, stream, refs.extend(stream.submission_ref()));
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
    ) -> anyhow::Result<()> {
        self.panel_gpu
            .sync(device, queue, "datum-gui-render-panel-vertex-buffer", panel)?;
        self.viewport_underlay_gpu.sync(
            device,
            queue,
            "datum-gui-render-viewport-underlay-vertex-buffer",
            viewport_underlay,
        )?;
        self.viewport_overlay_gpu.sync(
            device,
            queue,
            "datum-gui-render-viewport-overlay-vertex-buffer",
            viewport_overlay,
        )?;
        self.board_interaction_gpu.sync(
            device,
            queue,
            "datum-gui-render-board-interaction-vertex-buffer",
            board_interaction,
        )?;
        self.console_gpu.upload(device, queue, console_overlay)?;
        self.menu_overlay_gpu.sync(
            device,
            queue,
            "datum-gui-render-menu-overlay-vertex-buffer",
            menu_overlay,
        )?;
        self.world_vertices_gpu
            .sync(device, queue, "datum-world-vertices", world)?;
        if let Some(scene) = schematic_world {
            self.schematic_world_vertices_gpu.sync(
                device,
                queue,
                "datum-schematic-world-vertices",
                &scene.world_vertices,
            )?;
        } else {
            self.schematic_world_vertices_gpu.clear();
            self.schematic_world_strokes_gpu.clear();
        }
        self.schematic_underlay_gpu.sync(
            device,
            queue,
            "datum-gui-render-schematic-underlay-vertex-buffer",
            schematic_underlay,
        )?;
        self.schematic_overlay_gpu.sync(
            device,
            queue,
            "datum-gui-render-schematic-overlay-vertex-buffer",
            schematic_overlay,
        )?;
        Ok(())
    }
}
