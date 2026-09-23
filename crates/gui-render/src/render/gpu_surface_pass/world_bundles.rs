//! Reusable world draws. Camera uniforms and pane scissors remain live per frame;
//! buffer/binding replacement or a changed visible command sequence rebuilds the bundle.

use super::*;

pub(crate) struct CachedSurfaceBundle {
    pub(super) bundle: wgpu::RenderBundle,
    pane_id: datum_gui_protocol::PaneId,
    surface: SceneSurface,
    used_kinds: u8,
    vertex_buffer: Option<wgpu::Buffer>,
    stroke_buffer: Option<wgpu::Buffer>,
    bind_group: wgpu::BindGroup,
    // Only actual encoded state belongs here. Visibility/layer metadata has
    // already selected the command stream and owns no additional GPU state.
    batches: Box<[DrawBatch]>,
    // Dropped after the bundle and raw handles, preserving the same allocation records.
    _allocations: Vec<crate::text_gpu::lifetime::SubmissionRef>,
}

impl CachedSurfaceBundle {
    fn matches(
        &self,
        vertex: Option<&wgpu::Buffer>,
        stroke: Option<&wgpu::Buffer>,
        binding: &wgpu::BindGroup,
        commands: &[RetainedDrawCommand],
    ) -> bool {
        self.vertex_buffer.as_ref() == vertex.filter(|_| self.used_kinds & 1 != 0)
            && self.stroke_buffer.as_ref() == stroke.filter(|_| self.used_kinds & 2 != 0)
            && &self.bind_group == binding
            && draw_batches(commands).eq(self.batches.iter().cloned())
    }
}

impl Renderer {
    /// Prepared retained-world associations: (allocation ID, pane ID, surface).
    /// Shared allocations occur once per referencing pane; sum allocation records
    /// by ID, never by incidence. This iterator allocates nothing and does not
    /// claim submission/presentation or include immediate shell/overlay resources.
    pub fn retained_surface_resource_consumers(
        &self,
    ) -> impl Iterator<Item = (u64, datum_gui_protocol::PaneId, SceneSurface)> + '_ {
        self.surface_world_bundles.iter().flat_map(|cached| {
            cached
                ._allocations
                .iter()
                .map(move |allocation| (allocation.allocation_id, cached.pane_id, cached.surface))
        })
    }

    pub(crate) fn prepare_surface_world_bundles(
        &mut self,
        device: &wgpu::Device,
        prepared: &PreparedScene,
        schematic: Option<&RetainedScene>,
    ) {
        for (index, (surface, binding)) in prepared
            .surface_passes()
            .iter()
            .zip(&self.surface_scene_uniforms)
            .enumerate()
        {
            let bind_group = &binding.bind_group;
            let (vertex, stroke, commands) = match surface.surface {
                SceneSurface::Board => (
                    self.world_vertices_gpu.buffer(),
                    self.world_strokes_gpu.buffer(),
                    prepared.visible_draw_commands(),
                ),
                SceneSurface::Schematic => (
                    self.schematic_world_vertices_gpu.buffer(),
                    self.schematic_world_strokes_gpu.buffer(),
                    schematic.map_or(&[][..], RetainedScene::all_draw_commands),
                ),
            };
            if let Some(cached) = self.surface_world_bundles.get_mut(index)
                && cached.matches(vertex, stroke, bind_group, commands)
            {
                // Pane identity is incidence metadata, not encoded GPU state.
                cached.pane_id = surface.pane_id;
                cached.surface = surface.surface;
                continue;
            }
            let batches: Box<[_]> = draw_batches(commands).collect();
            let used_kinds = batches.iter().fold(0, |bits, batch| {
                bits | match batch.kind {
                    DrawKind::Quads => 1,
                    DrawKind::Strokes => 2,
                }
            });
            let vertex = vertex.filter(|_| used_kinds & 1 != 0);
            let stroke = stroke.filter(|_| used_kinds & 2 != 0);
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
            for batch in &batches {
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
                    DrawKind::Quads => encoder.draw(batch.range.clone(), 0..1),
                    DrawKind::Strokes => encoder.draw(0..6, batch.range.clone()),
                }
                draws += 1;
            }
            let allocations = match surface.surface {
                SceneSurface::Board => [
                    self.world_vertices_gpu.prepared_ref(),
                    self.world_strokes_gpu.prepared_ref(),
                ],
                SceneSurface::Schematic => [
                    self.schematic_world_vertices_gpu.prepared_ref(),
                    self.schematic_world_strokes_gpu.prepared_ref(),
                ],
            }
            .into_iter()
            .enumerate()
            .filter(|(index, _)| used_kinds & (1 << index) != 0)
            .filter_map(|(_, allocation)| allocation)
            .chain(std::iter::once(binding.buffer.prepared_ref()))
            .collect();
            let cached = CachedSurfaceBundle {
                pane_id: surface.pane_id,
                surface: surface.surface,
                used_kinds,
                bundle: encoder.finish(&wgpu::RenderBundleDescriptor {
                    label: Some("datum-world-bundle"),
                }),
                vertex_buffer: vertex.cloned(),
                stroke_buffer: stroke.cloned(),
                bind_group: bind_group.clone(),
                batches,
                _allocations: allocations,
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
        let mut renderer = Renderer::new(&device, &queue, format, 4).unwrap();
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
        let mut equivalent = prepared.clone();
        for command in &mut equivalent.visible_draw_commands {
            match command {
                RetainedDrawCommand::Quads { layer_id, .. }
                | RetainedDrawCommand::Strokes { layer_id, .. } => {
                    *layer_id = Some("metadata already applied upstream".into());
                }
            }
        }
        let (split_index, split_range) = equivalent
            .visible_draw_commands
            .iter()
            .enumerate()
            .find_map(|(index, command)| match command {
                RetainedDrawCommand::Quads { range, .. } if range.end - range.start >= 6 => {
                    Some((index, range.clone()))
                }
                _ => None,
            })
            .expect("fixture has a splittable triangle list");
        equivalent.visible_draw_commands.splice(
            split_index..=split_index,
            [
                RetainedDrawCommand::Quads {
                    layer_id: None,
                    range: split_range.start..split_range.start + 3,
                },
                RetainedDrawCommand::Quads {
                    layer_id: None,
                    range: split_range.start + 3..split_range.end,
                },
            ],
        );
        renderer
            .render(
                &device,
                &queue,
                &view,
                &equivalent,
                &retained,
                None,
                1280,
                800,
            )
            .unwrap();
        assert_eq!(
            initial, renderer.surface_world_bundles[0].bundle,
            "equivalent encoded batches ignore upstream labels and segmentation"
        );
        let key = &renderer.surface_world_bundles[0].batches;
        assert!(key.len() < equivalent.visible_draw_commands.len());
        eprintln!(
            "bundle key: source_commands={} batches={} payload_bytes={}",
            equivalent.visible_draw_commands.len(),
            key.len(),
            std::mem::size_of_val(key.as_ref())
        );
        equivalent.visible_draw_commands.reverse();
        renderer
            .render(
                &device,
                &queue,
                &view,
                &equivalent,
                &retained,
                None,
                1280,
                800,
            )
            .unwrap();
        assert_ne!(
            initial, renderer.surface_world_bundles[0].bundle,
            "painter order remains part of the encoding key"
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
        renderer.world_vertices_gpu.clear();
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
        // A real retained-scene replacement can shrink a previous large world
        // allocation. Its original prefix and draw commands preserve geometry.
        let mut oversized = retained.clone();
        oversized.world_vertices = crate::gpu_data::shared_geometry::SharedGeometry::for_document(
            retained.world_vertices.repeat(8),
            &state.scene.scene_id,
        );
        renderer
            .render(
                &device, &queue, &view, &prepared, &oversized, None, 1280, 800,
            )
            .unwrap();
        let large_bundle = renderer.surface_world_bundles[0].bundle.clone();
        let live_bytes = std::mem::size_of_val(retained.world_vertices.as_ref()) as u64;
        assert_eq!(
            renderer.world_vertices_gpu.buffer().unwrap().size(),
            live_bytes * 8
        );
        renderer
            .render(
                &device, &queue, &view, &prepared, &retained, None, 1280, 800,
            )
            .unwrap();
        assert_eq!(
            renderer.world_vertices_gpu.buffer().unwrap().size(),
            live_bytes
        );
        let shrunk_bundle = renderer.surface_world_bundles[0].bundle.clone();
        assert_ne!(
            large_bundle, shrunk_bundle,
            "capacity shrink must rebind world draws"
        );
        renderer
            .render(
                &device, &queue, &view, &prepared, &retained, None, 1280, 800,
            )
            .unwrap();
        assert_eq!(
            shrunk_bundle, renderer.surface_world_bundles[0].bundle,
            "unchanged replacement remains warm"
        );
        renderer.surface_scene_uniforms.clear();
        renderer
            .render(
                &device, &queue, &view, &prepared, &retained, None, 1280, 800,
            )
            .unwrap();
        assert_ne!(
            shrunk_bundle, renderer.surface_world_bundles[0].bundle,
            "a new pane camera binding must be rebound"
        );
        let mut quads_only = prepared.clone();
        // The earlier visibility case hides the fixture's quad layer.
        quads_only.visible_draw_commands = retained
            .all_draw_commands()
            .iter()
            .filter(|command| matches!(command, RetainedDrawCommand::Quads { .. }))
            .cloned()
            .collect();
        assert!(!quads_only.visible_draw_commands.is_empty());
        renderer.prepare_surface_world_bundles(&device, &quads_only, None);
        assert!(renderer.surface_world_bundles[0].stroke_buffer.is_none());
        let vertex_id = renderer
            .world_vertices_gpu
            .submission_ref()
            .unwrap()
            .allocation_id;
        let stroke_id = renderer
            .world_strokes_gpu
            .submission_ref()
            .map(|reference| reference.allocation_id);
        assert!(
            !renderer
                .retained_surface_resource_consumers()
                .any(|(id, _, _)| Some(id) == stroke_id)
        );
        let single = renderer.surface_world_bundles[0].bundle.clone();
        renderer.world_strokes_gpu.clear();
        renderer.prepare_surface_world_bundles(&device, &quads_only, None);
        assert_eq!(
            single, renderer.surface_world_bundles[0].bundle,
            "unused stroke allocation does not invalidate a quad-only bundle"
        );
        let mut second_pane = quads_only.surface_passes[0].clone();
        second_pane.pane_id = datum_gui_protocol::PaneId(u32::MAX - 1);
        quads_only.surface_passes.truncate(1);
        quads_only.surface_passes.push(second_pane);
        renderer
            .prepare_surface_uniforms(&device, &queue, &quads_only, 1280, 800)
            .unwrap();
        renderer.prepare_surface_world_bundles(&device, &quads_only, None);
        let consumers: Vec<_> = renderer
            .retained_surface_resource_consumers()
            .filter(|(id, _, _)| *id == vertex_id)
            .collect();
        assert_eq!(consumers.len(), 2);
        assert_ne!(consumers[0].1, consumers[1].1);
        assert_eq!(
            Renderer::gpu_process_allocations()
                .iter()
                .filter(|record| record.id == vertex_id)
                .count(),
            1
        );
        let second_bundle = renderer.surface_world_bundles[1].bundle.clone();
        quads_only.surface_passes[1].pane_id = datum_gui_protocol::PaneId(u32::MAX);
        renderer.prepare_surface_world_bundles(&device, &quads_only, None);
        assert_eq!(second_bundle, renderer.surface_world_bundles[1].bundle);
        assert!(
            !renderer
                .retained_surface_resource_consumers()
                .any(|(_, pane, _)| pane == datum_gui_protocol::PaneId(u32::MAX - 1))
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
