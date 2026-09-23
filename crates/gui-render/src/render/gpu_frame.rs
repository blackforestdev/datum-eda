//! Full-frame GPU encoding and submission; resource lifetime lives on Renderer.
use super::*;

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub fn render(
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
        self.render_with_submission(
            device,
            queue,
            target,
            prepared,
            retained,
            schematic_retained,
            width,
            height,
            &mut |_| {},
        )
    }

    /// Native hosts observe the actual frame submission before any fallible
    /// post-submit measurement collection or presentation can discard the frame.
    #[allow(clippy::too_many_arguments)]
    pub fn render_with_submission(
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
    ) -> anyhow::Result<()> {
        self.cancel_vertex_uploads();
        if prepared.is_overlay_only() {
            return self.render_overlay_only(
                device,
                queue,
                target,
                prepared,
                width,
                height,
                on_submitted,
            );
        }
        let mut measurement = self.begin_gpu_measurement()?;
        let render_started = std::time::Instant::now();
        let panel_vertices = prepared.panel_vertices();
        let viewport_underlay_vertices = prepared.viewport_underlay_vertices();
        let viewport_overlay_vertices = prepared.viewport_overlay_vertices();
        let board_interaction_vertices = prepared.board_interaction_vertices();
        let console_overlay_vertices = prepared.console_overlay_vertices();
        let menu_overlay_vertices = prepared.menu_overlay_vertices();
        let world_vertices = &retained.world_vertices;
        let world_strokes = retained.world_strokes();
        let schematic_pass = gpu_surface_pass::prepare_schematic_pass(prepared, schematic_retained);
        // S4 grid and interaction overlays remain immediate screen-space geometry;
        // offscreen captures supply neither cursor nor hover quads.
        let schematic_underlay_vertices = prepared.schematic_underlay_vertices();
        let schematic_overlay_vertices = prepared.schematic_overlay_vertices();
        let (surface_grid_vertices, surface_grid_batches) =
            surface_grid_pass::build_surface_grids(prepared);
        self.prepare_surface_uniforms(device, queue, prepared, width, height);
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
            schematic_pass.as_ref().map(|(_, _, _, scene)| *scene),
            schematic_underlay_vertices,
            schematic_overlay_vertices,
        );
        self.surface_grid_gpu.sync(
            device,
            queue,
            "datum-surface-grid-vertex-buffer",
            &surface_grid_vertices,
        );
        self.world_strokes_gpu
            .sync(device, queue, "datum-world-strokes", world_strokes);
        if let Some((_, _, _, scene)) = schematic_pass.as_ref() {
            self.schematic_world_strokes_gpu.sync(
                device,
                queue,
                "datum-schematic-world-strokes",
                scene.world_strokes(),
            );
        }
        self.sync_terminal_graphics(device, queue, prepared, width, height);
        let upload_elapsed = upload_started.elapsed();
        let encode_started = std::time::Instant::now();
        let msaa_view = self.ensure_msaa(device, width, height).clone();
        self.prepare_surface_world_bundles(device, prepared, schematic_retained);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("datum-gui-render-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("datum-gui-render-pass"),
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
                timestamp_writes: measurement.as_mut().map(|m| m.pass("scene")).transpose()?,
                multiview_mask: None,
            });
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
            }
            self.draw_surface_grids(&mut pass, &surface_grid_batches);
            self.draw_surface_world_passes(&mut pass, prepared);
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            if prepared.surface_passes().is_empty() {
                for command in prepared.visible_draw_commands() {
                    match command {
                        RetainedDrawCommand::Quads { range, .. } => {
                            let Some(buffer) = self.world_vertices_gpu.buffer() else {
                                continue;
                            };
                            pass.set_pipeline(&self.world_pipeline);
                            pass.set_bind_group(0, &self.scene_bind_group, &[]);
                            pass.set_scissor_rect(
                                prepared.scene_viewport.x.max(0.0).floor() as u32,
                                prepared.scene_viewport.y.max(0.0).floor() as u32,
                                prepared.scene_viewport.width.max(1.0).ceil() as u32,
                                prepared.scene_viewport.height.max(1.0).ceil() as u32,
                            );
                            pass.set_vertex_buffer(0, buffer.slice(..));
                            pass.draw(range.clone(), 0..1);
                        }
                        RetainedDrawCommand::Strokes { range, .. } => {
                            let Some(buffer) = self.world_strokes_gpu.buffer() else {
                                continue;
                            };
                            draw_world_strokes(
                                &mut pass,
                                &self.world_stroke_pipeline,
                                &self.scene_bind_group,
                                buffer,
                                prepared.scene_viewport,
                                std::slice::from_ref(range),
                            );
                        }
                    }
                }
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            }
            // Keep the schematic grid screen-space so zoom cannot thicken it.
            if prepared.surface_passes().is_empty()
                && !schematic_underlay_vertices.is_empty()
                && let Some((scene_viewport, _, _, _)) = schematic_pass.as_ref()
                && let Some(buffer) = self.schematic_underlay_gpu.buffer()
            {
                pass.set_scissor_rect(
                    scene_viewport.x.max(0.0).floor() as u32,
                    scene_viewport.y.max(0.0).floor() as u32,
                    scene_viewport.width.max(1.0).ceil() as u32,
                    scene_viewport.height.max(1.0).ceil() as u32,
                );
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..schematic_underlay_vertices.len() as u32, 0..1);
            }
            // The companion pass uses its own camera uniforms and pane scissor.
            if prepared.surface_passes().is_empty()
                && let Some((scene_viewport, _, _, sr)) = schematic_pass.as_ref()
            {
                for command in sr.all_draw_commands() {
                    match command {
                        RetainedDrawCommand::Quads { range, .. } => {
                            let Some(buffer) = self.schematic_world_vertices_gpu.buffer() else {
                                continue;
                            };
                            pass.set_pipeline(&self.world_pipeline);
                            pass.set_bind_group(0, &self.schematic_scene_bind_group, &[]);
                            pass.set_scissor_rect(
                                scene_viewport.x.max(0.0).floor() as u32,
                                scene_viewport.y.max(0.0).floor() as u32,
                                scene_viewport.width.max(1.0).ceil() as u32,
                                scene_viewport.height.max(1.0).ceil() as u32,
                            );
                            pass.set_vertex_buffer(0, buffer.slice(..));
                            pass.draw(range.clone(), 0..1);
                        }
                        RetainedDrawCommand::Strokes { range, .. } => {
                            let Some(buffer) = self.schematic_world_strokes_gpu.buffer() else {
                                continue;
                            };
                            draw_world_strokes(
                                &mut pass,
                                &self.world_stroke_pipeline,
                                &self.schematic_scene_bind_group,
                                buffer,
                                *scene_viewport,
                                std::slice::from_ref(range),
                            );
                        }
                    }
                }
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            }
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
            }
            self.draw_console(&mut pass, console_overlay_vertices, prepared);
            // NOTE: the menu dropdown card is intentionally NOT drawn here. It is
            // composited AFTER the main text pass (below) so it occludes not only
            // the work-pane quads but every underlying text_run too; its own text
            // then draws in a final pass on top of the card.
        }
        self.encode_terminal_graphics(
            &mut encoder,
            &msaa_view,
            target,
            false,
            measurement.as_mut(),
        )?;
        let encode_elapsed = encode_started.elapsed();
        let text_prepare_started = std::time::Instant::now();
        let (text_cache_stats, skipped_text_prepare) =
            self.prepare_frame_text(device, queue, prepared, width, height, false)?;
        let text_prepare_elapsed = text_prepare_started.elapsed();
        let text_encode_started = std::time::Instant::now();
        // Geometry already clears/resolves the target. An empty text stage has
        // no load/store dependency and must not add another pass/resolve.
        if prepared.has_workspace_text() {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("datum-gui-text-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &msaa_view,
                    resolve_target: Some(target),
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: measurement.as_mut().map(|m| m.pass("text")).transpose()?,
                multiview_mask: None,
            });
            self.text_renderer
                .render(&self.atlas, &mut pass)
                .map_err(|error| anyhow::anyhow!("render GUI text: {error}"))?;
        }
        let text_encode_elapsed = text_encode_started.elapsed();

        self.encode_terminal_graphics(
            &mut encoder,
            &msaa_view,
            target,
            true,
            measurement.as_mut(),
        )?;

        // Composite the menu card and its text after the main text pass.
        if !menu_overlay_vertices.is_empty() {
            // Pass C: the dropdown card background/rows. Base pipeline + screen
            // uniform; full-window scissor so the drop below the menu bar is not
            // re-clipped to the scene viewport.
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("datum-gui-menu-overlay-pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &msaa_view,
                        resolve_target: Some(target),
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: measurement
                        .as_mut()
                        .map(|m| m.pass("menu-background"))
                        .transpose()?,
                    multiview_mask: None,
                });
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
            }

            // Pass D: the dropdown's own text, on top of the card. Uses the
            // dedicated overlay text renderer so the main renderer's prepared
            // state/caching is untouched. The content-keyed text_buffer_cache is
            // shared, so overlay glyph buffers reuse the same atlas.
            if prepared.has_overlay_text() {
                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("datum-gui-menu-overlay-text-pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &msaa_view,
                            resolve_target: Some(target),
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: measurement
                            .as_mut()
                            .map(|m| m.pass("menu-text"))
                            .transpose()?,
                        multiview_mask: None,
                    });
                    self.menu_overlay_text_renderer
                        .render(&self.atlas, &mut pass)
                        .map_err(|error| anyhow::anyhow!("render menu overlay text: {error}"))?;
                }
            }
        }

        let trace_enabled = std::env::var_os("DATUM_TRACE_TIMING").is_some();
        let finish_started = trace_enabled.then(std::time::Instant::now);
        self.resolve_gpu_measurement(&mut measurement, &mut encoder)?;
        let command_buffer = encoder.finish();
        let finish_elapsed = finish_started.map(|started| started.elapsed());
        let submit_started = std::time::Instant::now();
        self.flush_frame_uploads(queue);
        let submission = queue.submit([command_buffer]);
        self.hold_frame_submission(queue);
        on_submitted(submission);
        self.text_buffers.finish_frame();
        self.submit_gpu_measurement(measurement)?;
        let submit_elapsed = submit_started.elapsed();
        if let Some(finish_elapsed) = finish_elapsed {
            trace_render_timing(format!(
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
                world_vertices.len(),
                viewport_overlay_vertices.len(),
                prepared.text_runs.len(),
                text_cache_stats.hits,
                text_cache_stats.misses,
                skipped_text_prepare,
            ));
        }
        Ok(())
    }
}
