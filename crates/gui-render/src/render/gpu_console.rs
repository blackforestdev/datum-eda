//! GPU resources and draw pass owned by the Datum Console overlay.

use super::{ConsoleOverlayLayout, PreparedScene, Renderer, Vertex};

#[derive(Default)]
pub(super) struct ConsoleGpuResources {
    pub(super) vertices: super::gpu_data::screen_buffer::ScreenBuffer,
}

impl Renderer {
    pub(super) fn draw_console<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
        vertices: &[Vertex],
        prepared: &PreparedScene,
        damage: &wgpu::BindGroup,
        clip: crate::renderer_state::damage::clip::Clip,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_bind_group(1, damage, &[]);
        if let Some(layout) =
            self.console_gpu
                .draw(pass, vertices, prepared.console_overlay_layout(), clip)
        {
            self.observe_screen_draw(
                super::immediate_admission::screen_admission::ScreenGroup::Console,
                vertices.len() as u32,
                clip.intersect(super::immediate_admission::screen_admission::scissor(
                    layout.pane_body,
                ))
                .expect("nonempty Console clip"),
            );
        }
    }
}

impl ConsoleGpuResources {
    pub(super) fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: &[Vertex],
    ) -> anyhow::Result<()> {
        self.vertices.sync(
            device,
            queue,
            "datum-gui-render-console-overlay-vertex-buffer",
            vertices,
        )?;
        Ok(())
    }

    pub(super) fn draw<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
        vertices: &[Vertex],
        layout: Option<ConsoleOverlayLayout>,
        clip: crate::renderer_state::damage::clip::Clip,
    ) -> Option<ConsoleOverlayLayout> {
        if vertices.is_empty() {
            return None;
        }
        let layout = layout?;
        let buffer = self.vertices.buffer()?;

        if !clip.set(
            pass,
            [
                layout.pane_body.x.max(0.0).floor() as u32,
                layout.pane_body.y.max(0.0).floor() as u32,
                layout.pane_body.width.max(1.0).ceil() as u32,
                layout.pane_body.height.max(1.0).ceil() as u32,
            ],
        ) {
            return None;
        }
        pass.set_vertex_buffer(0, buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
        Some(layout)
    }
}
