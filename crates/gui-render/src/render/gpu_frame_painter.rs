//! One ordered painter implementation for full and retained-prefix graphs.
use super::*;
impl Renderer {
    fn draw_admitted_overlay<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        prepared: &PreparedScene,
        range: std::ops::Range<u32>,
    ) {
        pass.set_pipeline(&self.viewport_overlay_pipeline);
        gpu_vertex_upload::screen_quads::bind(
            pass,
            self.viewport_overlay_gpu
                .buffer()
                .expect("viewport quad buffer"),
            prepared.viewport_overlay_vertices().len() / 6,
        );
        let mut cursor = range.start;
        // Plans follow append order; their detail ranges reference the same
        // immutable global vertex stream consumed by this frame.
        for (pane, plan) in &prepared.pane_cues {
            if !self.selection_mask.detailed_overflow(*pane) {
                continue;
            }
            for detail in plan.detail.iter() {
                let start = detail.start.max(range.start);
                let end = detail.end.min(range.end);
                if start >= end {
                    continue;
                }
                if cursor < start {
                    pass.draw(0..6, cursor / 6..start / 6);
                }
                cursor = cursor.max(end);
            }
        }
        if cursor < range.end {
            pass.draw(0..6, cursor / 6..range.end / 6);
        }
        pass.set_pipeline(&self.pipeline);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_frame_prefix<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        prepared: &PreparedScene,
        width: u32,
        height: u32,
        clip: crate::renderer_state::damage::clip::Clip,
        measurement: &mut Option<gpu_measurements::FrameQueries>,
    ) -> anyhow::Result<()> {
        let panel_vertices = prepared.panel_vertices();
        let viewport_underlay_vertices = prepared.viewport_underlay_vertices();
        clip.set(pass, [0, 0, width, height]);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_bind_group(1, &self.damage_masks.unrestricted.group, &[]);
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
            clip.set(
                pass,
                [
                    prepared.scene_viewport.x.max(0.0).floor() as u32,
                    prepared.scene_viewport.y.max(0.0).floor() as u32,
                    prepared.scene_viewport.width.max(1.0).ceil() as u32,
                    prepared.scene_viewport.height.max(1.0).ceil() as u32,
                ],
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
        self.draw_surface_grids(pass, &self.surface_grids.batches, clip);
        if !prepared.surface_passes().is_empty()
            && let Some(m) = measurement
        {
            m.mark_scene(pass, 1)?;
        }
        self.draw_surface_world_passes(pass, prepared, clip);
        if !prepared.surface_passes().is_empty()
            && let Some(m) = measurement
        {
            m.mark_scene(pass, 2)?;
        }
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw_frame_suffix<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        prepared: &PreparedScene,
        width: u32,
        height: u32,
        damage: &wgpu::BindGroup,
        clip: crate::renderer_state::damage::clip::Clip,
    ) -> anyhow::Result<std::time::Duration> {
        let schematic_overlay_vertices = prepared.schematic_overlay_vertices();
        let viewport_overlay_vertices = prepared.viewport_overlay_vertices();
        let board_interaction_vertices = prepared.board_interaction_vertices();
        let console_overlay_vertices = prepared.console_overlay_vertices();
        let menu_overlay_vertices = prepared.menu_overlay_vertices();
        if !clip.set(pass, [0, 0, width, height]) {
            return Ok(std::time::Duration::ZERO);
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_bind_group(1, damage, &[]);
        // Interaction chrome stays above schematic world geometry.
        if !schematic_overlay_vertices.is_empty()
            && let Some(scene_viewport) =
                prepared.painted_interaction_viewport(SceneSurface::Schematic)
            && let Some(buffer) = self.schematic_overlay_gpu.buffer()
            && clip
                .intersect(immediate_admission::screen_admission::scissor(
                    scene_viewport,
                ))
                .is_some()
        {
            clip.set(
                pass,
                [
                    scene_viewport.x.max(0.0).floor() as u32,
                    scene_viewport.y.max(0.0).floor() as u32,
                    scene_viewport.width.max(1.0).ceil() as u32,
                    scene_viewport.height.max(1.0).ceil() as u32,
                ],
            );
            pass.set_vertex_buffer(0, buffer.slice(..));
            pass.draw(0..schematic_overlay_vertices.len() as u32, 0..1);
            self.observe_screen_draw(
                immediate_admission::screen_admission::ScreenGroup::SchematicOverlay,
                schematic_overlay_vertices.len() as u32,
                clip.intersect(immediate_admission::screen_admission::scissor(
                    scene_viewport,
                ))
                .expect("nonempty painter clip"),
            );
        }
        self.draw_selection_mask_lift(pass, prepared, damage, clip);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_bind_group(1, damage, &[]);
        if !prepared.selection_strokes.is_empty() && clip.set(pass, [0, 0, width, height]) {
            pass.set_pipeline(&self.selection_stroke_pipelines.lift);
            pass.set_bind_group(
                2,
                self.selection_stroke_clips
                    .binding
                    .as_ref()
                    .expect("selection exact clips"),
                &[],
            );
            pass.set_vertex_buffer(
                0,
                self.selection_stroke_gpu
                    .buffer()
                    .expect("selection instance buffer")
                    .slice(..),
            );
            pass.draw(0..6, 0..prepared.selection_strokes.len() as u32);
            pass.set_pipeline(&self.pipeline);
        }
        let split = prepared
            .selection_overlay_split
            .min(viewport_overlay_vertices.len());
        if split > 0
            && clip.set(
                pass,
                immediate_admission::screen_admission::scissor(prepared.scene_viewport),
            )
        {
            pass.set_vertex_buffer(
                0,
                self.viewport_overlay_gpu
                    .buffer()
                    .expect("viewport overlay buffer")
                    .slice(..),
            );
            self.draw_admitted_overlay(pass, prepared, 0..split as u32);
        }
        self.draw_selection_mask_cue(pass, prepared, damage, clip);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_bind_group(1, damage, &[]);
        // Each native contribution carries its own pane clip. The combined stream
        // serves board and schematic, independently of the board's viewport.
        if !prepared.selection_strokes.is_empty() && clip.set(pass, [0, 0, width, height]) {
            pass.set_pipeline(&self.selection_stroke_pipelines.cue);
            pass.set_bind_group(
                2,
                self.selection_stroke_clips
                    .binding
                    .as_ref()
                    .expect("selection exact clips"),
                &[],
            );
            pass.set_vertex_buffer(
                0,
                self.selection_stroke_gpu
                    .buffer()
                    .expect("selection instance buffer")
                    .slice(..),
            );
            pass.draw(0..6, 0..prepared.selection_strokes.len() as u32);
            self.observe_screen_draw(
                immediate_admission::screen_admission::ScreenGroup::Selection,
                prepared.selection_strokes.len() as u32 * 12,
                clip.intersect([0, 0, width, height])
                    .expect("nonempty selection clip"),
            );
            pass.set_pipeline(&self.pipeline);
        }
        if split < viewport_overlay_vertices.len() && clip.set(pass, [0, 0, width, height]) {
            pass.set_vertex_buffer(
                0,
                self.viewport_overlay_gpu
                    .buffer()
                    .expect("viewport overlay buffer")
                    .slice(..),
            );
            self.draw_admitted_overlay(
                pass,
                prepared,
                split as u32..viewport_overlay_vertices.len() as u32,
            );
        }
        if !viewport_overlay_vertices.is_empty() {
            // One logical stream can have prefix and pane-clipped native suffix
            // draws; observation counts their complete payload once.
            self.observe_screen_draw(
                immediate_admission::screen_admission::ScreenGroup::Overlay,
                viewport_overlay_vertices.len() as u32,
                [0, 0, width, height],
            );
        }
        if !board_interaction_vertices.is_empty()
            && prepared
                .painted_interaction_viewport(SceneSurface::Board)
                .is_some_and(|v| {
                    clip.intersect(immediate_admission::screen_admission::scissor(v))
                        .is_some()
                })
        {
            let interaction_viewport = prepared
                .painted_interaction_viewport(SceneSurface::Board)
                .expect("board painter always has a viewport");
            clip.set(
                pass,
                [
                    interaction_viewport.x.max(0.0).floor() as u32,
                    interaction_viewport.y.max(0.0).floor() as u32,
                    interaction_viewport.width.max(1.0).ceil() as u32,
                    interaction_viewport.height.max(1.0).ceil() as u32,
                ],
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
                clip.intersect(immediate_admission::screen_admission::scissor(
                    interaction_viewport,
                ))
                .expect("nonempty painter clip"),
            );
        }
        self.terminal_graphics
            .draw_layer(pass, &self.uniform_bind_group, false, damage, clip);
        let text_encode_started = std::time::Instant::now();
        if prepared.has_workspace_text() {
            clip.set(pass, [0, 0, width, height]);
            self.text_renderer
                .render_workspace(&self.atlas, pass, damage, |pane| {
                    self.selection_mask.detailed_overflow(pane)
                })
                .map_err(|error| anyhow::anyhow!("render GUI text: {error}"))?;
        }
        let mut text_encode_elapsed = text_encode_started.elapsed();
        self.draw_console(pass, console_overlay_vertices, prepared, damage, clip);
        if prepared.has_workspace_text() {
            let foreground_started = std::time::Instant::now();
            clip.set(pass, [0, 0, width, height]);
            self.text_renderer
                .render_layer(&self.atlas, pass, Some(TextLayer::Foreground), damage)
                .map_err(|error| anyhow::anyhow!("render foreground GUI text: {error}"))?;
            text_encode_elapsed += foreground_started.elapsed();
        }
        self.terminal_graphics
            .draw_layer(pass, &self.uniform_bind_group, true, damage, clip);
        // The card must occlude workspace text as well as geometry.
        if !menu_overlay_vertices.is_empty() {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            pass.set_bind_group(1, damage, &[]);
            clip.set(pass, [0, 0, width, height]);
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
                clip.intersect([0, 0, width, height])
                    .expect("nonempty painter clip"),
            );
            if prepared.has_overlay_text() {
                clip.set(pass, [0, 0, width, height]);
                self.menu_overlay_text_renderer
                    .render(&self.atlas, pass, damage)
                    .map_err(|error| anyhow::anyhow!("render menu overlay text: {error}"))?;
            }
        }
        Ok(text_encode_elapsed)
    }
}
