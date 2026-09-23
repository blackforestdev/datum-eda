//! A dialog-only frame needs one render pass and one MSAA resolve. Shared text
//! preparation and pipelines preserve the normal workspace overlay contract.

use super::*;

#[path = "gpu_text.rs"]
pub(crate) mod gpu_text;

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
    ) -> anyhow::Result<bool> {
        let started = std::env::var_os("DATUM_TRACE_TIMING")
            .is_some()
            .then(std::time::Instant::now);
        // Dialog-only frames have no world-camera consumers. Drop both owners
        // because a cached bundle also retains its camera bind group.
        self.surface_world_bundles.clear();
        self.surface_scene_uniforms.clear();
        self.uniform_buffer.sync(
            queue,
            ScreenUniform {
                resolution: [width as f32, height as f32],
                _pad: [0.0, 0.0],
            },
        );
        self.menu_overlay_gpu.sync(
            device,
            queue,
            "datum-menu-overlay-vertex-buffer",
            prepared.menu_overlay_vertices(),
        )?;
        let has_text = prepared.has_overlay_text();
        let Some((text_stats, _)) =
            self.prepare_text_uploads(device, queue, prepared, width, height, true, on_submitted)?
        else {
            return Ok(false);
        };
        let mut measurement = self.begin_gpu_measurement()?;
        let msaa_view = self.ensure_msaa(device, width, height)?.clone();
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
                    .render(&self.atlas, &mut pass)
                    .map_err(|error| anyhow::anyhow!("render dialog text: {error}"))?;
            }
        }
        self.resolve_gpu_measurement(&mut measurement, &mut encoder)?;
        let mut uploads = self.flush_frame_uploads(device, queue)?;
        let submission = queue.submit(
            uploads
                .as_mut()
                .map(|batch| batch.command())
                .into_iter()
                .chain([encoder.finish()]),
        );
        self.hold_frame_submission(queue);
        if let Some(batch) = uploads {
            batch.hold(queue);
        }
        on_submitted(submission);
        self.text_buffers.finish_frame();
        self.submit_gpu_measurement(queue, measurement)?;
        if let Some(started) = started {
            trace_render_timing(format!(
                "dialog renderer={}us passes=1 text_cache={}/{}",
                started.elapsed().as_micros(),
                text_stats.hits,
                text_stats.misses,
            ));
        }
        Ok(true)
    }
}
