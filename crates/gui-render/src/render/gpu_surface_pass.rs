use super::*;

#[path = "gpu_surface_pass/world_bundles.rs"]
mod world_bundles;
pub(super) use world_bundles::CachedSurfaceBundle;

pub(super) fn prepare_schematic_pass<'a>(
    prepared: &PreparedScene,
    schematic_retained: Option<&'a RetainedScene>,
) -> Option<(RectPx, RectPx, Projection, &'a RetainedScene)> {
    match (prepared.schematic_scene_viewport, schematic_retained) {
        (Some(scene_viewport), Some(scene)) if !scene.world_vertices().is_empty() => {
            let field = inset_rect(scene_viewport, 10.0, 10.0, 10.0, 10.0);
            let projection =
                Projection::new(field, &prepared.schematic_bounds, prepared.schematic_camera);
            Some((scene_viewport, field, projection, scene))
        }
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
impl Renderer {
    pub(crate) fn draw_surface_grids<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        batches: &[surface_grid_pass::SurfaceGridBatch],
    ) {
        // Empty geometry releases its upload owner and has nothing to bind.
        if batches.is_empty() {
            return;
        }
        let Some(buffer) = self.surface_grid_gpu.buffer() else {
            return;
        };
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, buffer.slice(..));
        for batch in batches {
            set_scissor(pass, batch.viewport);
            pass.draw(batch.vertices.clone(), 0..1);
        }
    }

    pub(crate) fn prepare_surface_uniforms(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        prepared: &PreparedScene,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        self.surface_scene_uniforms
            .truncate(prepared.surface_passes().len());
        while self.surface_scene_uniforms.len() < prepared.surface_passes().len() {
            self.surface_scene_uniforms.push(
                gpu_data::uniform_buffer::UniformBinding::from_buffer(
                    device,
                    &self.scene_bind_group_layout,
                    "datum-surface-scene-bind-group",
                    gpu_data::uniform_buffer::UniformBuffer::empty_in_generation(
                        device,
                        "datum-surface-scene-uniform",
                        &self.screen_budget,
                        self.pane_uniform_generations
                            .for_slot(self.surface_scene_uniforms.len()),
                    )?,
                )?,
            );
        }
        for (surface, binding) in prepared
            .surface_passes()
            .iter()
            .zip(&mut self.surface_scene_uniforms)
        {
            let field = inset_rect(surface.scene_viewport, 10.0, 10.0, 10.0, 10.0);
            let projection = Projection::new(field, &surface.bounds, surface.camera);
            binding.buffer.sync(
                queue,
                SceneUniform {
                    resolution: [width as f32, height as f32, 0.0, 0.0],
                    viewport_origin: [field.x, field.y, 0.0, 0.0],
                    viewport_size: [field.width, field.height, 0.0, 0.0],
                    camera_center_scale: [
                        surface.camera.center_x_nm,
                        surface.camera.center_y_nm,
                        projection.scale,
                        0.0,
                    ],
                },
            );
        }
        Ok(())
    }

    pub(crate) fn draw_surface_world_passes<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        prepared: &PreparedScene,
    ) {
        for (surface, cached) in prepared
            .surface_passes()
            .iter()
            .zip(&self.surface_world_bundles)
        {
            set_scissor(pass, surface.scene_viewport);
            pass.execute_bundles(std::iter::once(&cached.bundle));
        }
    }
}

fn set_scissor(pass: &mut wgpu::RenderPass<'_>, viewport: RectPx) {
    pass.set_scissor_rect(
        viewport.x.max(0.0).floor() as u32,
        viewport.y.max(0.0).floor() as u32,
        viewport.width.max(1.0).ceil() as u32,
        viewport.height.max(1.0).ceil() as u32,
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DrawKind {
    Quads,
    Strokes,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DrawBatch {
    kind: DrawKind,
    range: std::ops::Range<u32>,
}

fn command_batch(command: &RetainedDrawCommand) -> DrawBatch {
    match command {
        RetainedDrawCommand::Quads { range, .. } => DrawBatch {
            kind: DrawKind::Quads,
            range: range.clone(),
        },
        RetainedDrawCommand::Strokes { range, .. } => DrawBatch {
            kind: DrawKind::Strokes,
            range: range.clone(),
        },
    }
}

/// Adjacent draws with identical GPU state and contiguous buffer ranges can be
/// submitted together. Metadata has already selected visibility/order upstream;
/// it does not change pipeline state. Gaps, overlaps and backwards ranges remain
/// separate draws, and a primitive switch is always a batching boundary.
fn draw_batches(commands: &[RetainedDrawCommand]) -> impl Iterator<Item = DrawBatch> + '_ {
    let mut remaining = commands;
    std::iter::from_fn(move || {
        let (first, rest) = remaining.split_first()?;
        remaining = rest;
        let mut batch = command_batch(first);
        while let Some(next) = remaining.first() {
            let next = command_batch(next);
            if batch.range.is_empty()
                || next.range.is_empty()
                || next.kind != batch.kind
                || next.range.start != batch.range.end
            {
                break;
            }
            // Each draw starts a fresh triangle list. Joining incomplete lists
            // could create a triangle that neither original draw emitted.
            if batch.kind == DrawKind::Quads
                && (!(batch.range.end - batch.range.start).is_multiple_of(3)
                    || !(next.range.end - next.range.start).is_multiple_of(3))
            {
                break;
            }
            batch.range.end = next.range.end;
            remaining = &remaining[1..];
        }
        Some(batch)
    })
}

#[cfg(test)]
pub(super) fn board_fixture_state() -> ReviewWorkspaceState {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad");
    datum_gui_protocol::load_board_editor_workspace_state(&datum_gui_protocol::LiveReviewRequest {
        board_file: Some(root.join("simple-demo.kicad_pcb")),
        project_root: root,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    })
    .expect("checked-in KiCad board fixture loads")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quads(layer: &str, range: std::ops::Range<u32>) -> RetainedDrawCommand {
        RetainedDrawCommand::Quads {
            layer_id: Some(layer.into()),
            range,
        }
    }

    fn expand(batches: impl Iterator<Item = DrawBatch>) -> Vec<(DrawKind, u32)> {
        batches
            .flat_map(|batch| batch.range.map(move |index| (batch.kind, index)))
            .collect()
    }

    #[test]
    fn contiguous_draws_share_submission_without_reordering_layers() {
        let commands = [
            quads("B.Cu", 0..6),
            quads("F.Cu", 6..12),
            quads("F.SilkS", 12..18),
        ];
        let batches: Vec<_> = draw_batches(&commands).collect();
        assert_eq!(
            batches,
            [DrawBatch {
                kind: DrawKind::Quads,
                range: 0..18
            }]
        );
        assert_eq!(
            expand(draw_batches(&commands)),
            expand(commands.iter().map(command_batch))
        );
    }

    #[test]
    fn gaps_overlaps_backwards_ranges_and_primitive_switches_stay_ordered() {
        let commands = [
            quads("F.Cu", 6..12),
            quads("F.Cu", 18..24),
            quads("F.Cu", 21..27),
            quads("B.Cu", 0..6),
            RetainedDrawCommand::Strokes {
                layer_id: None,
                range: 6..9,
            },
            RetainedDrawCommand::Strokes {
                layer_id: None,
                range: 9..12,
            },
            quads("F.Cu", 12..18),
        ];
        assert_eq!(draw_batches(&commands).count(), 6);
        assert_eq!(
            expand(draw_batches(&commands)),
            expand(commands.iter().map(command_batch))
        );
        assert_eq!(draw_batches(&[]).count(), 0);
    }

    #[test]
    fn incomplete_triangle_lists_and_empty_draws_are_not_joined() {
        for commands in [
            vec![quads("F.Cu", 0..2), quads("F.Cu", 2..4)],
            vec![
                quads("F.Cu", 0..6),
                quads("F.Cu", 6..6),
                quads("F.Cu", 6..12),
            ],
        ] {
            assert_eq!(draw_batches(&commands).count(), commands.len());
        }
    }

    #[test]
    fn real_board_fixture_preserves_exact_draw_sequence_with_layer_filtering() {
        let mut state = board_fixture_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        assert!(!retained.visible_draw_commands(&state).is_empty());
        for hidden in [
            None,
            state
                .scene
                .layers
                .first()
                .map(|layer| layer.layer_id.clone()),
        ] {
            if let Some(layer) = hidden {
                state.ui.filters.layer_visibility.insert(layer, false);
            }
            let commands = retained.visible_draw_commands(&state);
            assert_eq!(
                expand(draw_batches(&commands)),
                expand(commands.iter().map(command_batch))
            );
            assert!(draw_batches(&commands).count() <= commands.len());
        }
    }
}
