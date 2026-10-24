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
        let layout =
            ShellLayout::for_surface(width, height, scale_factor, dock_height_for_state(state));
        let Some(scene_viewport) = layout.schematic_scene_viewport(&state.ui.layout) else {
            return Ok(None);
        };
        Self::schematic_source_bounded(
            state,
            schematic_scene,
            scene_viewport,
            &schematic_scene.scene_id,
            limit,
        )
        .map(Some)
    }

    /// Build one immutable qualified source using the shared schematic allowance.
    pub(crate) fn schematic_source_bounded(
        state: &ReviewWorkspaceState,
        schematic_scene: &BoardReviewSceneV1,
        scene_viewport: RectPx,
        document_budget_identity: &str,
        limit: usize,
    ) -> anyhow::Result<Self> {
        let budget = retained_scene_owner::document_cpu::for_scene(document_budget_identity);
        let scope = crate::cpu_alloc::Scope::new("retained-schematic-construction");
        retained_scene_owner::document_cpu::with_constructor(&scope, || {
            let board_field = inset_rect(scene_viewport, 10.0, 10.0, 10.0, 10.0);
            let reference_projection = Projection::new(
                board_field,
                &schematic_scene.bounds,
                CameraState::fit_to_bounds(&schematic_scene.bounds),
            );
            let mut world_quads = crate::geometry_output::Admitted::new(|bytes| {
                retained_scene_owner::document_cpu::admit_constructor_allocation(
                    &budget,
                    &scope,
                    bytes,
                    limit,
                    "geometry emission",
                )
            })
            .track_sources();
            let mut world_strokes = crate::geometry_output::Admitted::new(|bytes| {
                retained_scene_owner::document_cpu::admit_constructor_allocation(
                    &budget,
                    &scope,
                    bytes,
                    limit,
                    "geometry emission",
                )
            })
            .track_sources();
            let retained_glyph::Builders {
                points: mut glyph_points,
                materials: mut glyph_materials,
            } = retained_glyph::builders(&budget, &scope, limit);
            let mut draw_commands = crate::geometry_output::Admitted::new(|bytes| {
                retained_scene_owner::document_cpu::admit_constructor_allocation(
                    &budget,
                    &scope,
                    bytes,
                    limit,
                    "geometry emission",
                )
            });
            // The grid is immediate screen-space geometry; retain only scene geometry.
            push_retained_scene_geometry(
                &mut world_quads,
                &mut world_strokes,
                &mut draw_commands,
                schematic_scene,
                &reference_projection,
                state,
            );
            retained_glyph::push(
                &mut world_quads,
                &mut glyph_points,
                &mut glyph_materials,
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
            let (mut world_quads, quad_sources) = world_quads.finish_with_sources()?;
            let (mut world_strokes, stroke_sources) = world_strokes.finish_with_sources()?;
            let mut draw_commands = draw_commands.finish()?;
            let (glyph_points, glyph_sources) = glyph_points.finish_with_sources()?;
            let glyph_materials = glyph_materials.finish()?;
            scene_retained_access::locked_material::apply(
                schematic_scene,
                &mut world_quads,
                &quad_sources,
                &mut world_strokes,
                &stroke_sources,
                |bytes| {
                    retained_scene_owner::document_cpu::admit_constructor_allocation(
                        &budget,
                        &scope,
                        bytes,
                        limit,
                        "locked material source index",
                    )
                },
            )?;

            scene_retained_access::sort_retained_draw_commands(
                &mut draw_commands,
                &schematic_scene.layers,
                |bytes| {
                    retained_scene_owner::document_cpu::admit_constructor_allocation(
                        &budget,
                        &scope,
                        bytes,
                        limit,
                        "draw command sorting",
                    )
                },
            )?;
            retained_scene_owner::document_cpu::admit_vertex_expansion(
                &budget,
                &scope,
                gpu_data::retained_vertex_count(&world_quads, &draw_commands)?,
                limit,
            )?;
            let quad_sources = crate::presentation_ranges::remap(
                &world_quads,
                &draw_commands,
                &quad_sources,
                |bytes| {
                    retained_scene_owner::document_cpu::admit_constructor_allocation(
                        &budget,
                        &scope,
                        bytes,
                        limit,
                        "presentation remap",
                    )
                },
            )?;
            retained_scene_owner::document_cpu::admit_constructor_allocation(
                &budget,
                &scope,
                crate::presentation_ranges::Presentation::container_bytes(),
                limit,
                "presentation owner",
            )?;
            let source = crate::presentation_ranges::Source::capture(schematic_scene, |bytes| {
                retained_scene_owner::document_cpu::admit_constructor_allocation(
                    &budget,
                    &scope,
                    bytes,
                    limit,
                    "presentation source",
                )
            })?;
            let presentation = std::sync::Arc::new(crate::presentation_ranges::Presentation {
                source: Some(source),
                glyphs: glyph_sources,
                quads: quad_sources,
                strokes: stroke_sources,
            });
            let world_vertices = gpu_data::try_retained_vertices(&world_quads, &mut draw_commands)?;
            drop(world_quads);
            // S3 / UVT-004: build typed schematic hit shapes independently from the
            // current tool's selection eligibility.
            let world_hit_regions =
                coordinate_hit::schematic_hit_regions(schematic_scene, |bytes| {
                    retained_scene_owner::document_cpu::admit_constructor_allocation(
                        &budget,
                        &scope,
                        bytes,
                        limit,
                        "schematic hit regions",
                    )
                })?;
            let world_hit_index =
                Self::admitted_hit_index(world_hit_regions, &budget, &scope, limit)?;
            retained_glyph::Geometry::admit(&glyph_points, &glyph_materials, |bytes| {
                retained_scene_owner::document_cpu::admit_constructor_allocation(
                    &budget,
                    &scope,
                    bytes,
                    limit,
                    "glyph owners",
                )
            })?;
            Self::admit_shared_owners(&world_vertices, &world_strokes, &budget, &scope, limit)?;
            Ok(Self {
                surface_size_independent: Self::scene_is_surface_size_independent(schematic_scene),
                hover_membership: Default::default(),
                presentation,
                glyph: retained_glyph::Geometry::from_document(
                    glyph_points,
                    glyph_materials,
                    document_budget_identity,
                ),
                world_vertices: gpu_data::shared_geometry::SharedGeometry::for_document(
                    world_vertices,
                    document_budget_identity,
                ),
                world_strokes: gpu_data::shared_geometry::SharedGeometry::for_document(
                    world_strokes,
                    document_budget_identity,
                ),
                draw_commands: draw_commands.into(),
                world_hit_index: world_hit_index.into(),
            }
            .registered_cpu(&scope, limit)?)
        })
    }
}
