//! Camera-independent retained geometry for the companion schematic surface.

use super::*;

impl RetainedScene {
    /// Build the companion schematic world buffer without changing the board
    /// resolve counter. Its camera, viewport, and hit index remain independent.
    pub fn from_workspace_schematic_for_surface(
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> Option<Self> {
        Self::try_from_workspace_schematic_for_surface(state, width, height, scale_factor)
            .expect("schematic construction fits document budget")
    }

    pub fn try_from_workspace_schematic_for_surface(
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> anyhow::Result<Option<Self>> {
        Self::schematic_workspace_bounded(
            state,
            width,
            height,
            scale_factor,
            retained_scene_owner::document_cpu::DOCUMENT_LIMIT,
        )
    }

    pub(crate) fn schematic_workspace_bounded(
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        scale_factor: f32,
        limit: usize,
    ) -> anyhow::Result<Option<Self>> {
        let Some(schematic_scene) = state.schematic_scene.as_ref() else {
            return Ok(None);
        };
        let budget = retained_scene_owner::document_cpu::for_scene(&schematic_scene.scene_id);
        let scope = crate::cpu_alloc::Scope::new("retained-schematic-construction");
        scope.with(|| {
            let layout =
                ShellLayout::for_surface(width, height, scale_factor, dock_height_for_state(state));
            let Some(scene_viewport) = layout.schematic_scene_viewport(&state.ui.layout) else {
                return Ok(None);
            };
            let board_field = inset_rect(scene_viewport, 10.0, 10.0, 10.0, 10.0);
            let reference_projection = Projection::new(
                board_field,
                &schematic_scene.bounds,
                CameraState::fit_to_bounds(&schematic_scene.bounds),
            );
            let mut world_quads = Vec::new();
            let mut world_strokes = Vec::new();
            let mut draw_commands = Vec::new();
            // The grid is immediate screen-space geometry; retain only scene geometry.
            push_retained_scene_geometry(
                &mut world_quads,
                &mut world_strokes,
                &mut draw_commands,
                schematic_scene,
                &reference_projection,
                state,
            );
            push_retained_board_text_geometry_batches(
                &mut world_quads,
                &mut draw_commands,
                schematic_scene,
                &reference_projection,
                state,
            );
            push_retained_board_graphic_batches(
                &mut world_quads,
                &mut world_strokes,
                &mut draw_commands,
                schematic_scene,
                &reference_projection,
                state,
            );
            scene_retained_access::sort_retained_draw_commands(
                &mut draw_commands,
                &schematic_scene.layers,
            );
            retained_scene_owner::document_cpu::admit_vertex_expansion(
                &budget,
                &scope,
                world_quads.len(),
                limit,
            )?;
            let world_vertices = gpu_data::try_quads_to_vertices(&world_quads)?;
            drop(world_quads);
            // S3 / UVT-004: build typed schematic hit shapes independently from the
            // current tool's selection eligibility.
            let mut world_hit_regions = Vec::new();
            coordinate_hit::push_schematic_hit_regions(&mut world_hit_regions, schematic_scene);
            let world_hit_index =
                Self::admitted_hit_index(world_hit_regions, &budget, &scope, limit)?;
            Ok(Some(
                Self {
                    surface_size_independent: Self::scene_is_surface_size_independent(
                        schematic_scene,
                    ),
                    world_vertices: gpu_data::shared_geometry::SharedGeometry::for_document(
                        world_vertices,
                        &schematic_scene.scene_id,
                    ),
                    world_strokes: gpu_data::shared_geometry::SharedGeometry::for_document(
                        world_strokes,
                        &schematic_scene.scene_id,
                    ),
                    draw_commands: draw_commands.into(),
                    world_hit_index: world_hit_index.into(),
                }
                .registered_cpu(),
            ))
        })
    }
}
