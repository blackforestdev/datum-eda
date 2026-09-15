//! GPU resources and draw pass owned by the Datum Console overlay.

use super::{ConsoleOverlayLayout, PreparedScene, Renderer, Vertex};

#[derive(Default)]
pub(super) struct ConsoleGpuResources {
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_capacity: usize,
}

impl Renderer {
    pub(super) fn draw_console<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
        vertices: &[Vertex],
        prepared: &PreparedScene,
    ) {
        self.console_gpu
            .draw(pass, vertices, prepared.console_overlay_layout());
    }
}

impl ConsoleGpuResources {
    pub(super) fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: &[Vertex],
    ) {
        Renderer::upload_vertices(
            device,
            queue,
            &mut self.vertex_buffer,
            &mut self.vertex_capacity,
            "datum-gui-render-console-overlay-vertex-buffer",
            vertices,
        );
    }

    pub(super) fn draw<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
        vertices: &[Vertex],
        layout: Option<ConsoleOverlayLayout>,
    ) {
        if vertices.is_empty() {
            return;
        }
        let Some(layout) = layout else {
            return;
        };
        let Some(buffer) = self.vertex_buffer.as_ref() else {
            return;
        };

        pass.set_scissor_rect(
            layout.pane_body.x.max(0.0).floor() as u32,
            layout.pane_body.y.max(0.0).floor() as u32,
            layout.pane_body.width.max(1.0).ceil() as u32,
            layout.pane_body.height.max(1.0).ceil() as u32,
        );
        pass.set_vertex_buffer(0, buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }
}
