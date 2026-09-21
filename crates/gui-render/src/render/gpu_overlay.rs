//! A dialog-only frame needs one render pass and one MSAA resolve. Shared text
//! preparation and pipelines preserve the normal workspace overlay contract.

use super::*;

impl PreparedScene {
    pub(crate) fn is_overlay_only(&self) -> bool {
        !self.menu_overlay_vertices.is_empty()
            && self.panel_vertices.is_empty()
            && self.viewport_underlay_vertices.is_empty()
            && self.viewport_overlay_vertices.is_empty()
            && self.board_interaction_vertices.is_empty()
            && self.console_overlay_vertices.is_empty()
            && self.visible_draw_commands.is_empty()
            && self.surface_passes.is_empty()
            && self.text_runs.is_empty()
            && self.terminal_graphics.is_empty()
            && self.schematic_scene_viewport.is_none()
            && self.schematic_underlay_vertices.is_empty()
            && self.schematic_overlay_vertices.is_empty()
    }
}

impl Renderer {
    pub(crate) fn prepare_overlay_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        prepared: &PreparedScene,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let (overlay_indices, _) = self.text_buffers.indices(
            &mut self.font_system,
            prepared.menu_overlay_text_runs(),
            width,
            height,
        );
        let overlay_prepare = self.menu_overlay_text_renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            build_text_areas(
                self.text_buffers.entries(),
                &overlay_indices,
                prepared.menu_overlay_text_runs(),
            ),
            &mut self.swash_cache,
        );
        if let Err(initial_error) = overlay_prepare {
            self.atlas.trim();
            self.menu_overlay_text_renderer
                        .prepare(
                            device,
                            queue,
                            &mut self.font_system,
                            &mut self.atlas,
                            &self.viewport,
                            build_text_areas(
                                self.text_buffers.entries(),
                                &overlay_indices,
                                prepared.menu_overlay_text_runs(),
                            ),
                            &mut self.swash_cache,
                        )
                        .map_err(|retry_error| {
                            anyhow::anyhow!(
                                "prepare menu overlay text after atlas trim: {retry_error}; initial: {initial_error}"
                            )
                        })?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_overlay_only(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
        prepared: &PreparedScene,
        width: u32,
        height: u32,
        on_submitted: &mut dyn FnMut(wgpu::SubmissionIndex),
    ) -> anyhow::Result<()> {
        let mut measurement = self.begin_gpu_measurement()?;
        let started = std::time::Instant::now();
        // Overlay-only frames advance/evict shared shaped-buffer entries. A
        // later workspace frame must rebuild its separate glyph instances.
        self.last_text_prepare_signature = None;
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&ScreenUniform {
                resolution: [width as f32, height as f32],
                _pad: [0.0, 0.0],
            }),
        );
        self.menu_overlay_gpu.sync(
            device,
            queue,
            "datum-menu-overlay-vertex-buffer",
            prepared.menu_overlay_vertices(),
        );
        self.viewport.update(queue, Resolution { width, height });
        self.text_buffers
            .begin_frame(text_buffer_cache::Profile::Overlay);
        let has_text = !prepared.menu_overlay_text_runs().is_empty();
        if has_text {
            self.prepare_overlay_text(device, queue, prepared, width, height)?;
        }
        let msaa_view = self.ensure_msaa(device, width, height).clone();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("datum-dialog-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("datum-dialog-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &msaa_view,
                    resolve_target: Some(target),
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: APP_BG[0] as f64,
                            g: APP_BG[1] as f64,
                            b: APP_BG[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: measurement.as_mut().map(|m| m.pass("dialog")).transpose()?,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            pass.set_vertex_buffer(
                0,
                self.menu_overlay_gpu
                    .buffer()
                    .expect("dialog buffer uploaded")
                    .slice(..),
            );
            pass.draw(0..prepared.menu_overlay_vertices().len() as u32, 0..1);
            if has_text {
                self.menu_overlay_text_renderer
                    .render(&self.atlas, &self.viewport, &mut pass)
                    .map_err(|error| anyhow::anyhow!("render dialog text: {error}"))?;
            }
        }
        self.resolve_gpu_measurement(&mut measurement, &mut encoder)?;
        let submission = queue.submit([encoder.finish()]);
        on_submitted(submission);
        self.submit_gpu_measurement(measurement)?;
        self.text_buffers.trim_overlay();
        trace_render_timing(format!(
            "dialog renderer={}us passes=1",
            started.elapsed().as_micros()
        ));
        Ok(())
    }
}
