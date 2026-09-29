//! Full-frame GPU encoding and submission; resource lifetime lives on Renderer.
use super::*;
#[path = "gpu_frame_painter.rs"]
mod painter;
#[path = "gpu_frame_target.rs"]
pub(crate) mod target;

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
        prepared: &PreparedScene,
        retained: &RetainedScene,
        schematic_retained: Option<&RetainedScene>,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        // Synchronous convenience for offscreen/capture clients. Native hosts use
        // render_with_acquisition and yield to the coordinator between chunks.
        loop {
            if self.render_with_submission(
                device,
                queue,
                target,
                prepared,
                retained,
                schematic_retained,
                width,
                height,
                &mut |_| {},
            )? {
                return Ok(());
            }
            device.poll(wgpu::PollType::wait_indefinitely())?;
        }
    }

    /// Offscreen callers already own their target and observe every submission.
    /// Native hosts must use render_with_acquisition to avoid acquiring swapchain
    /// images for upload-only turns. False retains pending damage in either path.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_with_submission(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
        prepared: &PreparedScene,
        retained: &RetainedScene,
        schematic_retained: Option<&RetainedScene>,
        width: u32,
        height: u32,
        on_submitted: &mut dyn FnMut(wgpu::SubmissionIndex),
    ) -> anyhow::Result<bool> {
        self.render_with_acquisition(
            device,
            queue,
            prepared,
            retained,
            schematic_retained,
            width,
            height,
            &mut (),
            &mut |_| Ok(Some(target.clone())),
            &mut |_, submission| on_submitted(submission),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_submission_inner(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &mut dyn target::Target,
        prepared: &PreparedScene,
        retained: &RetainedScene,
        schematic_retained: Option<&RetainedScene>,
        width: u32,
        height: u32,
    ) -> anyhow::Result<bool> {
        if self.resume_glyph_upload(device, queue, &mut |submission| {
            target.submitted(submission)
        })? {
            return Ok(false);
        }
        if self.cold_world.active
            && self.world_upload_sources_match(prepared, retained, schematic_retained)
        {
            self.submit_world_upload_chunk(device, queue, &mut |submission| {
                target.submitted(submission)
            })?;
            return Ok(false);
        }
        self.cold_world.active = false;
        // Encodings of replaced CPU geometry are no longer reusable. Retire
        // their CPU holds before admission; actual submissions keep their own.
        if !self.world_upload_sources_match(prepared, retained, schematic_retained) {
            self.surface_world_bundles.clear();
        }
        self.cancel_vertex_uploads();
        self.cancel_uniform_uploads();
        if prepared.is_overlay_only() {
            self.surface_attachments.release_prefix();
            return self.render_overlay_only(device, queue, target, prepared, width, height);
        }
        self.prepare_world_pipelines(device);
        let render_started = std::time::Instant::now();
        let panel_vertices = prepared.panel_vertices();
        let viewport_underlay_vertices = prepared.viewport_underlay_vertices();
        let viewport_overlay_vertices = prepared.viewport_overlay_vertices();
        let board_interaction_vertices = prepared.board_interaction_vertices();
        let console_overlay_vertices = prepared.console_overlay_vertices();
        let menu_overlay_vertices = prepared.menu_overlay_vertices();
        let world_vertices = prepared
            .requires_board_world()
            .then_some(&retained.world_vertices);
        let world_strokes = retained.world_strokes();
        let schematic_pass = gpu_surface_pass::prepare_schematic_pass(prepared, schematic_retained);
        // S4 grid and interaction overlays remain immediate screen-space geometry;
        // offscreen captures supply neither cursor nor hover quads.
        let schematic_overlay_vertices = prepared.schematic_overlay_vertices();
        self.surface_grids
            .prepare(prepared.surface_passes(), &self.atlas.staging_budget)?;
        self.observe_surface_grids()?;
        self.prepare_surface_uniforms(device, queue, prepared, width, height)?;
        self.uniform_buffer.sync(
            queue,
            ScreenUniform {
                resolution: [width as f32, height as f32],
                _pad: [0.0, 0.0],
            },
        );
        let upload_started = std::time::Instant::now();
        self.upload_frame_vertices(
            device,
            queue,
            panel_vertices,
            viewport_underlay_vertices,
            viewport_overlay_vertices,
            board_interaction_vertices,
            console_overlay_vertices,
            menu_overlay_vertices,
            world_vertices,
            schematic_pass,
            schematic_overlay_vertices,
        )?;
        self.surface_grid_gpu.sync(
            device,
            queue,
            "datum-surface-grid-vertex-buffer",
            &self.surface_grids.vertices,
        )?;
        if world_vertices.is_some() {
            self.world_strokes_gpu
                .sync(device, queue, "datum-world-strokes", world_strokes)?;
        }
        if let Some(scene) = schematic_pass {
            self.schematic_world_strokes_gpu.sync(
                device,
                queue,
                "datum-schematic-world-strokes",
                scene.world_strokes(),
            )?;
        }
        if self.pending_world_upload_bytes() != 0 {
            self.submit_world_upload_chunk(device, queue, &mut |submission| {
                target.submitted(submission)
            })?;
            return Ok(false);
        }
        self.sync_terminal_graphics(device, queue, prepared, width, height)?;
        if self.submit_terminal_upload_chunk(device, queue, &mut |submission| {
            target.submitted(submission)
        })? {
            return Ok(false);
        }
        let upload_elapsed = upload_started.elapsed();
        let text_prepare_started = std::time::Instant::now();
        let on_submitted = &mut |submission| target.submitted(submission);
        let Some((text_cache_stats, skipped_text_prepare)) =
            self.prepare_text_uploads(device, queue, prepared, width, height, false, on_submitted)?
        else {
            return Ok(false);
        };
        let text_prepare_elapsed = text_prepare_started.elapsed();
        let Some(view) = target.acquire()? else {
            return Ok(false);
        };
        let mut measurement = self.begin_gpu_measurement()?;
        let encode_started = std::time::Instant::now();
        let msaa_view = self.ensure_msaa(device, width, height)?.clone();
        self.publish_resource_consumers();
        self.prepare_surface_world_bundles(device, prepared, schematic_retained);
        let images = self
            .render_session
            .prefix
            .requested()
            .then(|| {
                self.surface_attachments
                    .prefix_images(device, measurement.is_some())
            })
            .flatten();
        self.publish_resource_consumers();
        let reuse = images
            .as_ref()
            .is_some_and(|images| self.render_session.prefix.reusable(images.identity));
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("datum-gui-render-encoder"),
        });
        let clear = wgpu::LoadOp::Clear(wgpu::Color {
            r: APP_BG[0] as f64,
            g: APP_BG[1] as f64,
            b: APP_BG[2] as f64,
            a: 1.0,
        });
        if let Some(images) = &images {
            if reuse {
                // Timestamp the queue interval before copy, not just suffix draw.
                // This 1x1 attachment is charged and held through submission.
                if let Some(marker) = images.marker_view()
                    && let Some(m) = &mut measurement
                {
                    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("datum-prefix-copy-start"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: marker,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: clear,
                                store: wgpu::StoreOp::Discard,
                            },
                        })],
                        timestamp_writes: Some(m.pass("copy-start")?),
                        ..Default::default()
                    });
                }
            } else {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("datum-retained-prefix"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: images.prefix_view(),
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: clear,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    timestamp_writes: measurement.as_mut().map(|m| m.pass("frame")).transpose()?,
                    ..Default::default()
                });
                self.draw_frame_prefix(&mut pass, prepared, width, height, &mut measurement)?;
            }
            images.copy(&mut encoder);
        }
        let text_encode_elapsed;
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("datum-gui-final-resolve"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &msaa_view,
                    resolve_target: Some(&view),
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: if images.is_some() {
                            wgpu::LoadOp::Load
                        } else {
                            clear
                        },
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                timestamp_writes: measurement
                    .as_mut()
                    .map(|m| m.pass(if images.is_some() { "suffix" } else { "frame" }))
                    .transpose()?,
                ..Default::default()
            });
            if images.is_none() {
                self.draw_frame_prefix(&mut pass, prepared, width, height, &mut measurement)?;
            }
            text_encode_elapsed = self.draw_frame_suffix(&mut pass, prepared, width, height)?;
        }
        let encode_elapsed = encode_started.elapsed().saturating_sub(text_encode_elapsed);

        let trace_enabled = std::env::var_os("DATUM_TRACE_TIMING").is_some();
        let finish_started = trace_enabled.then(std::time::Instant::now);
        self.resolve_gpu_measurement(&mut measurement, &mut encoder)?;
        let command_buffer = encoder.finish();
        let finish_elapsed = finish_started.map(|started| started.elapsed());
        let submit_started = std::time::Instant::now();
        let mut uploads = self.flush_frame_uploads(device, queue)?;
        let submission = queue.submit(
            uploads
                .as_mut()
                .map(|batch| batch.command())
                .into_iter()
                .chain([command_buffer]),
        );
        self.hold_frame_submission(queue, images.as_ref());
        if let Some(batch) = uploads {
            batch.hold(queue);
        }
        target.submitted(submission);
        self.text_buffers.finish_frame();
        self.submit_gpu_measurement(queue, measurement)?;
        if let Some(images) = images {
            self.render_session
                .prefix
                .encoded(images.identity, reuse, images.logical_copy_bytes());
        }
        let submit_elapsed = submit_started.elapsed();
        if let Some(finish_elapsed) = finish_elapsed {
            trace_render_timing(|| {
                format!(
                    "renderer total={}ms upload={}ms encode={}ms text_prepare={}ms text_encode={}ms submit={}ms finish_us={} submit_us={} vertices panel={} underlay={} world={} overlay={} text_runs={} text_cache={}/{} text_prepare_skipped={}",
                    render_started.elapsed().as_millis(),
                    upload_elapsed.as_millis(),
                    encode_elapsed.as_millis(),
                    text_prepare_elapsed.as_millis(),
                    text_encode_elapsed.as_millis(),
                    submit_elapsed.as_millis(),
                    finish_elapsed.as_micros(),
                    submit_elapsed.as_micros(),
                    panel_vertices.len(),
                    viewport_underlay_vertices.len(),
                    world_vertices.map_or(0, |vertices| vertices.len()),
                    viewport_overlay_vertices.len(),
                    prepared.text_runs.len(),
                    text_cache_stats.hits,
                    text_cache_stats.misses,
                    skipped_text_prepare,
                )
            });
        }
        Ok(true)
    }
}
