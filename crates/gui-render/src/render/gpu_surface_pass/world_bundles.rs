//! Reusable world draws. Camera uniforms and pane scissors remain live per frame;
//! buffer/binding replacement or a changed visible command sequence rebuilds the bundle.

use super::*;

pub(crate) struct CachedSurfaceBundle {
    pub(super) bundle: wgpu::RenderBundle,
    vertex_buffer: Option<wgpu::Buffer>,
    stroke_buffer: Option<wgpu::Buffer>,
    bind_group: wgpu::BindGroup,
    commands: Vec<RetainedDrawCommand>,
}

impl CachedSurfaceBundle {
    fn matches(
        &self,
        vertex: Option<&wgpu::Buffer>,
        stroke: Option<&wgpu::Buffer>,
        binding: &wgpu::BindGroup,
        commands: &[RetainedDrawCommand],
    ) -> bool {
        self.vertex_buffer.as_ref() == vertex
            && self.stroke_buffer.as_ref() == stroke
            && &self.bind_group == binding
            && self.commands == commands
    }
}

impl Renderer {
    pub(crate) fn prepare_surface_world_bundles(
        &mut self,
        device: &wgpu::Device,
        prepared: &PreparedScene,
        schematic: Option<&RetainedScene>,
    ) {
        for (index, (surface, (_, bind_group))) in prepared
            .surface_passes()
            .iter()
            .zip(&self.surface_scene_uniforms)
            .enumerate()
        {
            let (vertex, stroke, commands) = match surface.surface {
                SceneSurface::Board => (
                    self.world_vertex_buffer.as_ref(),
                    self.world_stroke_buffer.as_ref(),
                    prepared.visible_draw_commands(),
                ),
                SceneSurface::Schematic => (
                    self.schematic_world_vertex_buffer.as_ref(),
                    self.schematic_world_stroke_buffer.as_ref(),
                    schematic.map_or(&[][..], RetainedScene::all_draw_commands),
                ),
            };
            if self
                .surface_world_bundles
                .get(index)
                .is_some_and(|cached| cached.matches(vertex, stroke, bind_group, commands))
            {
                continue;
            }
            let mut encoder =
                device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                    label: Some("datum-world-bundle"),
                    color_formats: &[Some(self.msaa_format)],
                    depth_stencil: None,
                    sample_count: self.msaa_samples,
                    multiview: None,
                });
            encoder.set_bind_group(0, bind_group, &[]);
            let mut bound_kind = None;
            let mut draws = 0;
            for batch in draw_batches(commands) {
                let (pipeline, buffer) = match batch.kind {
                    DrawKind::Quads => (&self.world_pipeline, vertex),
                    DrawKind::Strokes => (&self.world_stroke_pipeline, stroke),
                };
                let Some(buffer) = buffer else { continue };
                if bound_kind != Some(batch.kind) {
                    encoder.set_pipeline(pipeline);
                    encoder.set_vertex_buffer(0, buffer.slice(..));
                    bound_kind = Some(batch.kind);
                }
                match batch.kind {
                    DrawKind::Quads => encoder.draw(batch.range, 0..1),
                    DrawKind::Strokes => encoder.draw(0..6, batch.range),
                }
                draws += 1;
            }
            let cached = CachedSurfaceBundle {
                bundle: encoder.finish(&wgpu::RenderBundleDescriptor {
                    label: Some("datum-world-bundle"),
                }),
                vertex_buffer: vertex.cloned(),
                stroke_buffer: stroke.cloned(),
                bind_group: bind_group.clone(),
                commands: commands.to_vec(),
            };
            if index == self.surface_world_bundles.len() {
                self.surface_world_bundles.push(cached);
            } else {
                self.surface_world_bundles[index] = cached;
            }
            trace_render_timing(format!(
                "world bundle rebuilt pane={} source_commands={} draws={draws}",
                index,
                commands.len()
            ));
        }
        // Bound retained resources to currently visible panes, including layout shrink.
        self.surface_world_bundles
            .truncate(prepared.surface_passes().len());
    }
}

#[cfg(all(test, feature = "visual"))]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires local GPU; run explicitly with the visual feature"]
    fn world_bundle_reuse_and_invalidation_on_real_gpu() {
        let instance = wgpu::Instance::default();
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("bundle-regression"),
            size: wgpu::Extent3d {
                width: 1280,
                height: 800,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut renderer = Renderer::new(&device, &queue, format, 4);
        let mut state = board_fixture_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        let mut camera = CameraState::fit_to_bounds(&state.scene.bounds);
        let make_prepared = |state: &ReviewWorkspaceState, camera| {
            PreparedScene::from_workspace_for_surface(state, 1280, 800, 1.0, camera, &retained)
        };
        let mut prepared = make_prepared(&state, camera);
        renderer
            .render(
                &device, &queue, &view, &prepared, &retained, None, 1280, 800,
            )
            .unwrap();
        let initial = renderer.surface_world_bundles[0].bundle.clone();
        camera.zoom *= 1.5;
        prepared = make_prepared(&state, camera);
        renderer
            .render(
                &device, &queue, &view, &prepared, &retained, None, 1280, 800,
            )
            .unwrap();
        assert_eq!(
            initial, renderer.surface_world_bundles[0].bundle,
            "camera contents do not change bound GPU resources"
        );
        let layer = prepared
            .visible_draw_commands
            .iter()
            .find_map(|command| match command {
                RetainedDrawCommand::Quads { layer_id, .. }
                | RetainedDrawCommand::Strokes { layer_id, .. } => layer_id.clone(),
            })
            .expect("fixture has a layered draw");
        state.ui.filters.layer_visibility.insert(layer, false);
        prepared = make_prepared(&state, camera);
        renderer
            .render(
                &device, &queue, &view, &prepared, &retained, None, 1280, 800,
            )
            .unwrap();
        let filtered = renderer.surface_world_bundles[0].bundle.clone();
        assert_ne!(
            initial, filtered,
            "visibility must change the recorded draw sequence"
        );
        renderer.world_vertex_buffer = None;
        renderer
            .render(
                &device, &queue, &view, &prepared, &retained, None, 1280, 800,
            )
            .unwrap();
        let replaced_buffer = renderer.surface_world_bundles[0].bundle.clone();
        assert_ne!(
            filtered, replaced_buffer,
            "a replacement GPU allocation must be rebound"
        );
        renderer.surface_scene_uniforms.clear();
        renderer
            .render(
                &device, &queue, &view, &prepared, &retained, None, 1280, 800,
            )
            .unwrap();
        assert_ne!(
            replaced_buffer, renderer.surface_world_bundles[0].bundle,
            "a new pane camera binding must be rebound"
        );
        prepared.surface_passes.clear();
        renderer
            .render(
                &device, &queue, &view, &prepared, &retained, None, 1280, 800,
            )
            .unwrap();
        assert!(
            renderer.surface_world_bundles.is_empty(),
            "closed panes release their cached bundles"
        );
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    }
}
