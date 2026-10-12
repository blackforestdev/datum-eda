//! Actual retained painter replay: unselected material occludes; drill voids clear.
use super::*;
use crate::renderer_state::damage::clip::Clip;
impl Renderer {
    pub(crate) fn encode_selection_mask(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        prepared: &PreparedScene,
        extent: [u32; 2],
        timestamp_writes: Option<wgpu::RenderPassTimestampWrites<'_>>,
    ) -> anyhow::Result<()> {
        if !self.selection_mask.active() {
            return Ok(());
        }
        let origin = self.selection_mask.active_band.get() as u32 * BAND;
        let pipelines = self
            .selection_mask
            .pipelines
            .get()
            .expect("mask pipelines prepared");
        let view = self
            .selection_mask
            .images
            .current_sampled_view()
            .expect("mask attachment prepared");
        let color_view = self
            .selection_mask
            .colors
            .current_sampled_view()
            .expect("mask material attachment prepared");
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("datum-complete-selection-mask"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            timestamp_writes,
            ..Default::default()
        });
        pass.set_viewport(
            0.0,
            GUARD as f32 - origin as f32,
            extent[0] as f32,
            extent[1] as f32,
            0.0,
            1.0,
        );
        let top = origin.saturating_sub(GUARD);
        let region = Clip::translated(
            [0, top, extent[0], (origin + BAND + GUARD).min(extent[1])],
            [0, GUARD + top - origin],
        );
        for (index, surface) in prepared.surface_passes().iter().enumerate() {
            let Some(binding) = self
                .selection_mask
                .bindings
                .get(index)
                .and_then(Option::as_ref)
            else {
                continue;
            };
            if !region.set(
                &mut pass,
                immediate_admission::screen_admission::scissor(surface.scene_viewport),
            ) {
                continue;
            }
            pass.set_bind_group(0, &self.surface_scene_uniforms[index].bind_group, &[]);
            pass.set_bind_group(1, &binding.table, &[]);
            let (quads, strokes) = match surface.surface {
                SceneSurface::Board => (&self.world_vertices_gpu, &self.world_strokes_gpu),
                SceneSurface::Schematic => (
                    &self.schematic_world_vertices_gpu,
                    &self.schematic_world_strokes_gpu,
                ),
            };
            let (glyphs, materials) = match surface.surface {
                SceneSurface::Board => (
                    &self.world_glyph_points_gpu,
                    &self.world_glyph_materials_gpu,
                ),
                SceneSurface::Schematic => (
                    &self.schematic_glyph_points_gpu,
                    &self.schematic_glyph_materials_gpu,
                ),
            };
            for (kind, range) in self.surface_world_bundles[index].painter_ranges() {
                match kind {
                    crate::gpu_surface_pass::DrawKind::Quads => {
                        if let Some(buffer) = quads.buffer() {
                            pass.set_pipeline(&pipelines.mask_quad);
                            pass.set_vertex_buffer(0, buffer.slice(..));
                            pass.draw(range.clone(), 0..1);
                        }
                    }
                    crate::gpu_surface_pass::DrawKind::Strokes => {
                        if let Some(buffer) = strokes.buffer() {
                            pass.set_pipeline(&pipelines.mask_stroke);
                            pass.set_vertex_buffer(0, buffer.slice(..));
                            pass.draw(0..6, range.clone());
                        }
                    }
                    crate::gpu_surface_pass::DrawKind::Glyphs(material) => {
                        if let (Some(buffer), Some(colors)) = (glyphs.buffer(), materials.buffer())
                        {
                            pass.set_pipeline(&pipelines.mask_glyph);
                            pass.set_vertex_buffer(0, buffer.slice(..));
                            pass.set_vertex_buffer(1, colors.slice(..));
                            pass.draw(range, material..material + 1);
                        }
                    }
                }
            }
            if binding.overflow {
                if let Some(request) = prepared
                    .selection_masks
                    .iter()
                    .find_map(|(pane, _, request)| (*pane == surface.pane_id).then_some(request))
                    && let Some(buffer) = self.viewport_overlay_gpu.buffer()
                {
                    pass.set_pipeline(&pipelines.mask_screen_quad);
                    gpu_vertex_upload::screen_quads::bind(
                        &mut pass,
                        buffer,
                        prepared.viewport_overlay_vertices().len() / 6,
                    );
                    for span in request.screen() {
                        pass.draw(0..6, span.range.start / 6..span.range.end / 6);
                    }
                }
                // Preserve the current band's viewport and pane scissor. Glyphs
                // use the same full-window coordinates as retained screen quads.
                if prepared.has_workspace_text() {
                    self.text_renderer.render_selection_mask(
                        &self.atlas,
                        &mut pass,
                        surface.pane_id,
                    )?;
                }
            }
        }
        Ok(())
    }
    pub(crate) fn draw_selection_mask_lift<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        prepared: &PreparedScene,
        damage: &wgpu::BindGroup,
        clip: Clip,
    ) {
        let Some(pipelines) = self
            .selection_mask
            .pipelines
            .get()
            .filter(|_| self.selection_mask.active())
        else {
            return;
        };
        pass.set_pipeline(&pipelines.lift);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_bind_group(1, damage, &[]);
        for (index, surface) in prepared.surface_passes().iter().enumerate() {
            let Some(binding) = self
                .selection_mask
                .bindings
                .get(index)
                .and_then(Option::as_ref)
            else {
                continue;
            };
            if !clip.set(
                pass,
                immediate_admission::screen_admission::scissor(surface.scene_viewport),
            ) {
                continue;
            }
            pass.set_bind_group(
                2,
                &binding.images[self.selection_mask.active_band.get()],
                &[],
            );
            pass.draw(0..3, 0..1);
        }
    }
    pub(crate) fn draw_selection_mask_cue<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        prepared: &PreparedScene,
        damage: &wgpu::BindGroup,
        clip: Clip,
    ) {
        let Some(pipelines) = self
            .selection_mask
            .pipelines
            .get()
            .filter(|_| self.selection_mask.active())
        else {
            return;
        };
        pass.set_pipeline(&pipelines.cue);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_bind_group(1, damage, &[]);
        for (index, surface) in prepared.surface_passes().iter().enumerate() {
            let Some(binding) = self
                .selection_mask
                .bindings
                .get(index)
                .and_then(Option::as_ref)
            else {
                continue;
            };
            if !clip.set(
                pass,
                immediate_admission::screen_admission::scissor(surface.scene_viewport),
            ) {
                continue;
            }
            pass.set_bind_group(
                2,
                &binding.images[self.selection_mask.active_band.get()],
                &[],
            );
            pass.draw(0..3, 0..1);
        }
    }
}
