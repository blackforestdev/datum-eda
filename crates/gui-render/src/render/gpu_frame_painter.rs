//! One ordered painter implementation for full and retained-prefix graphs.
use super::*;
impl Renderer {
    pub(super) fn draw_frame_prefix<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        prepared: &PreparedScene,
        width: u32,
        height: u32,
        measurement: &mut Option<gpu_measurements::FrameQueries>,
    ) -> anyhow::Result<()> {
        let panel_vertices = prepared.panel_vertices();
        let viewport_underlay_vertices = prepared.viewport_underlay_vertices();
        pass.set_scissor_rect(0, 0, width, height);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        if !panel_vertices.is_empty() {
            pass.set_vertex_buffer(
                0,
                self.panel_gpu
                    .buffer()
                    .expect("panel vertex buffer should exist")
                    .slice(..),
            );
            pass.draw(0..panel_vertices.len() as u32, 0..1);
            self.observe_screen_draw(
                immediate_admission::screen_admission::ScreenGroup::Panel,
                panel_vertices.len() as u32,
                [0, 0, width, height],
            );
        }
        if prepared.surface_passes().is_empty() && !viewport_underlay_vertices.is_empty() {
            pass.set_scissor_rect(
                prepared.scene_viewport.x.max(0.0).floor() as u32,
                prepared.scene_viewport.y.max(0.0).floor() as u32,
                prepared.scene_viewport.width.max(1.0).ceil() as u32,
                prepared.scene_viewport.height.max(1.0).ceil() as u32,
            );
            pass.set_vertex_buffer(
                0,
                self.viewport_underlay_gpu
                    .buffer()
                    .expect("viewport underlay vertex buffer should exist")
                    .slice(..),
            );
            pass.draw(0..viewport_underlay_vertices.len() as u32, 0..1);
            self.observe_screen_draw(
                immediate_admission::screen_admission::ScreenGroup::Underlay,
                viewport_underlay_vertices.len() as u32,
                immediate_admission::screen_admission::scissor(prepared.scene_viewport),
            );
        }
        if !prepared.surface_passes().is_empty()
            && let Some(m) = measurement
        {
            m.mark_scene(pass, 0)?;
        }
        self.draw_surface_grids(pass, &self.surface_grids.batches);
        if !prepared.surface_passes().is_empty()
            && let Some(m) = measurement
        {
            m.mark_scene(pass, 1)?;
        }
        self.draw_surface_world_passes(pass, prepared);
        if !prepared.surface_passes().is_empty()
            && let Some(m) = measurement
        {
            m.mark_scene(pass, 2)?;
        }
        Ok(())
    }
    pub(super) fn draw_frame_suffix<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        prepared: &PreparedScene,
        width: u32,
        height: u32,
    ) -> anyhow::Result<std::time::Duration> {
        let schematic_overlay_vertices = prepared.schematic_overlay_vertices();
        let viewport_overlay_vertices = prepared.viewport_overlay_vertices();
        let board_interaction_vertices = prepared.board_interaction_vertices();
        let console_overlay_vertices = prepared.console_overlay_vertices();
        let menu_overlay_vertices = prepared.menu_overlay_vertices();
        pass.set_scissor_rect(0, 0, width, height);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        // Interaction chrome stays above schematic world geometry.
        if !schematic_overlay_vertices.is_empty()
            && let Some(scene_viewport) = prepared.interaction_viewport(SceneSurface::Schematic)
            && let Some(buffer) = self.schematic_overlay_gpu.buffer()
        {
            pass.set_scissor_rect(
                scene_viewport.x.max(0.0).floor() as u32,
                scene_viewport.y.max(0.0).floor() as u32,
                scene_viewport.width.max(1.0).ceil() as u32,
                scene_viewport.height.max(1.0).ceil() as u32,
            );
            pass.set_vertex_buffer(0, buffer.slice(..));
            pass.draw(0..schematic_overlay_vertices.len() as u32, 0..1);
            self.observe_screen_draw(
                immediate_admission::screen_admission::ScreenGroup::SchematicOverlay,
                schematic_overlay_vertices.len() as u32,
                immediate_admission::screen_admission::scissor(scene_viewport),
            );
        }
        if !viewport_overlay_vertices.is_empty() {
            pass.set_scissor_rect(
                prepared.scene_viewport.x.max(0.0).floor() as u32,
                prepared.scene_viewport.y.max(0.0).floor() as u32,
                prepared.scene_viewport.width.max(1.0).ceil() as u32,
                prepared.scene_viewport.height.max(1.0).ceil() as u32,
            );
            pass.set_vertex_buffer(
                0,
                self.viewport_overlay_gpu
                    .buffer()
                    .expect("viewport overlay vertex buffer should exist")
                    .slice(..),
            );
            pass.draw(0..viewport_overlay_vertices.len() as u32, 0..1);
            self.observe_screen_draw(
                immediate_admission::screen_admission::ScreenGroup::Overlay,
                viewport_overlay_vertices.len() as u32,
                immediate_admission::screen_admission::scissor(prepared.scene_viewport),
            );
        }
        if !board_interaction_vertices.is_empty() {
            let interaction_viewport = prepared
                .interaction_viewport(SceneSurface::Board)
                .unwrap_or(prepared.scene_viewport);
            pass.set_scissor_rect(
                interaction_viewport.x.max(0.0).floor() as u32,
                interaction_viewport.y.max(0.0).floor() as u32,
                interaction_viewport.width.max(1.0).ceil() as u32,
                interaction_viewport.height.max(1.0).ceil() as u32,
            );
            pass.set_vertex_buffer(
                0,
                self.board_interaction_gpu
                    .buffer()
                    .expect("board interaction vertex buffer should exist")
                    .slice(..),
            );
            pass.draw(0..board_interaction_vertices.len() as u32, 0..1);
            self.observe_screen_draw(
                immediate_admission::screen_admission::ScreenGroup::BoardInteraction,
                board_interaction_vertices.len() as u32,
                immediate_admission::screen_admission::scissor(interaction_viewport),
            );
        }
        self.terminal_graphics
            .draw_layer(pass, &self.uniform_bind_group, false);
        let text_encode_started = std::time::Instant::now();
        if prepared.has_workspace_text() {
            pass.set_scissor_rect(0, 0, width, height);
            self.text_renderer
                .render_layer(&self.atlas, pass, Some(TextLayer::Workspace))
                .map_err(|error| anyhow::anyhow!("render GUI text: {error}"))?;
        }
        let mut text_encode_elapsed = text_encode_started.elapsed();
        self.draw_console(pass, console_overlay_vertices, prepared);
        if prepared.has_workspace_text() {
            let foreground_started = std::time::Instant::now();
            pass.set_scissor_rect(0, 0, width, height);
            self.text_renderer
                .render_layer(&self.atlas, pass, Some(TextLayer::Foreground))
                .map_err(|error| anyhow::anyhow!("render foreground GUI text: {error}"))?;
            text_encode_elapsed += foreground_started.elapsed();
        }
        self.terminal_graphics
            .draw_layer(pass, &self.uniform_bind_group, true);
        // The card must occlude workspace text as well as geometry.
        if !menu_overlay_vertices.is_empty() {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            pass.set_scissor_rect(0, 0, width, height);
            pass.set_vertex_buffer(
                0,
                self.menu_overlay_gpu
                    .buffer()
                    .expect("menu overlay vertex buffer should exist")
                    .slice(..),
            );
            pass.draw(0..menu_overlay_vertices.len() as u32, 0..1);
            self.observe_screen_draw(
                immediate_admission::screen_admission::ScreenGroup::Menu,
                menu_overlay_vertices.len() as u32,
                [0, 0, width, height],
            );
            if prepared.has_overlay_text() {
                pass.set_scissor_rect(0, 0, width, height);
                self.menu_overlay_text_renderer
                    .render(&self.atlas, pass)
                    .map_err(|error| anyhow::anyhow!("render menu overlay text: {error}"))?;
            }
        }
        Ok(text_encode_elapsed)
    }
}
