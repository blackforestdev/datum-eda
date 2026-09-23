use super::*;

pub(crate) fn finish_retained_draw_commands(
    commands: &mut Vec<RetainedDrawCommand>,
    layer_id: Option<String>,
    quad_start: usize,
    quad_end: usize,
    stroke_start: usize,
    stroke_end: usize,
) {
    if quad_end > quad_start {
        append_retained_draw_command(
            commands,
            RetainedDrawCommand::Quads {
                layer_id: layer_id.clone(),
                range: (quad_start * 6) as u32..(quad_end * 6) as u32,
            },
        );
    }
    if stroke_end > stroke_start {
        append_retained_draw_command(
            commands,
            RetainedDrawCommand::Strokes {
                layer_id,
                range: stroke_start as u32..stroke_end as u32,
            },
        );
    }
}

fn append_retained_draw_command(
    commands: &mut Vec<RetainedDrawCommand>,
    command: RetainedDrawCommand,
) {
    let merged = match (commands.last_mut(), &command) {
        (
            Some(RetainedDrawCommand::Quads {
                layer_id: previous_layer,
                range: previous,
            }),
            RetainedDrawCommand::Quads { layer_id, range },
        )
        | (
            Some(RetainedDrawCommand::Strokes {
                layer_id: previous_layer,
                range: previous,
            }),
            RetainedDrawCommand::Strokes { layer_id, range },
        ) if previous_layer == layer_id && previous.end == range.start => {
            previous.end = range.end;
            true
        }
        _ => false,
    };
    if !merged {
        commands.push(command);
    }
}

pub(crate) fn sort_retained_draw_commands(
    commands: &mut [RetainedDrawCommand],
    layers: &[datum_gui_protocol::SceneLayer],
) {
    commands.sort_by_key(|command| {
        let layer_id = match command {
            RetainedDrawCommand::Quads { layer_id, .. }
            | RetainedDrawCommand::Strokes { layer_id, .. } => layer_id.as_deref(),
        };
        layer_id
            .map(|id| scene_layer_stack_priority(id, layers))
            .unwrap_or(u32::MAX)
    });
}

impl RetainedScene {
    pub fn world_vertices(&self) -> &[Vertex] {
        &self.world_vertices
    }
    pub(crate) fn world_strokes(
        &self,
    ) -> &gpu_data::shared_geometry::SharedGeometry<WorldStrokeInstance> {
        &self.world_strokes
    }

    pub(crate) fn visible_draw_commands(
        &self,
        state: &ReviewWorkspaceState,
    ) -> Vec<RetainedDrawCommand> {
        if !authored_visible(state) {
            return Vec::new();
        }
        self.draw_commands
            .iter()
            .filter(|command| match command {
                RetainedDrawCommand::Quads { layer_id, .. }
                | RetainedDrawCommand::Strokes { layer_id, .. } => layer_id
                    .as_deref()
                    .is_none_or(|id| layer_visible(state, id)),
            })
            .cloned()
            .collect()
    }

    pub(crate) fn all_draw_commands(&self) -> &[RetainedDrawCommand] {
        &self.draw_commands
    }
}

impl RetainedScene {
    pub fn from_workspace(state: &ReviewWorkspaceState, width: u32, height: u32) -> Self {
        Self::from_workspace_for_surface(state, width, height, 1.0)
    }

    pub fn from_workspace_for_surface(
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> Self {
        Self::try_from_workspace_for_surface(state, width, height, scale_factor)
            .expect("retained scene construction fits document budget")
    }

    pub fn try_from_workspace_for_surface(
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> anyhow::Result<Self> {
        Self::from_workspace_bounded(
            state,
            width,
            height,
            scale_factor,
            retained_scene_owner::document_cpu::DOCUMENT_LIMIT,
        )
    }

    pub(crate) fn from_workspace_bounded(
        state: &ReviewWorkspaceState,
        width: u32,
        height: u32,
        scale_factor: f32,
        limit: usize,
    ) -> anyhow::Result<Self> {
        let budget = retained_scene_owner::document_cpu::for_scene(&state.scene.scene_id);
        let scope = crate::cpu_alloc::Scope::new("retained-board-construction");
        scope.with(|| {
        // This is the single world-scene resolve entry point; count the miss.
        // (`reference_projection` below is derived here and nowhere else, so a pane
        // op that reuses the retained scene provably never recomputes it.)
        RETAINED_RESOLVE_COUNT.with(|count| count.set(count.get() + 1));
        let started = std::time::Instant::now();
        let layout =
            ShellLayout::for_surface(width, height, scale_factor, dock_height_for_state(state));
        let scene_viewport = layout.scene_viewport(&state.ui.layout);
        let board_field = inset_rect(scene_viewport, 10.0, 10.0, 10.0, 10.0);
        let reference_projection = Projection::new(
            board_field,
            &state.scene.bounds,
            CameraState::fit_to_bounds(&state.scene.bounds),
        );
        let mut world_quads = Vec::new();
        let mut world_strokes = Vec::new();
        let mut draw_commands = Vec::new();
        let mut world_hit_regions = Vec::new();
        let geometry_started = std::time::Instant::now();
        push_retained_scene_geometry(
            &mut world_quads,
            &mut world_strokes,
            &mut draw_commands,
            &state.scene,
            &reference_projection,
            state,
        );
        let board_graphics_started = std::time::Instant::now();
        let board_graphics_before = world_quads.len();
        push_retained_board_text_geometry_batches(
            &mut world_quads,
            &mut draw_commands,
            &state.scene,
            &reference_projection,
            state,
        );
        push_retained_board_graphic_batches(
            &mut world_quads,
            &mut world_strokes,
            &mut draw_commands,
            &state.scene,
            &reference_projection,
            state,
        );
        scene_retained_access::sort_retained_draw_commands(&mut draw_commands, &state.scene.layers);
        trace_render_timing(format!(
            "retained text+board_graphics batches={}ms/{}q",
            board_graphics_started.elapsed().as_millis(),
            world_quads.len().saturating_sub(board_graphics_before)
        ));
        let geometry_elapsed = geometry_started.elapsed();
        let hits_started = std::time::Instant::now();
        push_retained_world_hit_regions(&mut world_hit_regions, &state.scene, state);
        let hits_elapsed = hits_started.elapsed();
        let vertex_started = std::time::Instant::now();
        retained_scene_owner::document_cpu::admit_vertex_expansion(&budget, &scope, world_quads.len(), limit)?;
        let world_vertices = quads_to_vertices(&world_quads);
        let quad_count = world_quads.len();
        drop(world_quads);
        let vertex_elapsed = vertex_started.elapsed();
        trace_render_timing(format!(
            "retained total={}ms geometry={}ms hits={}ms vertices={}ms quads={} vertices={} hit_regions={}",
            started.elapsed().as_millis(),
            geometry_elapsed.as_millis(),
            hits_elapsed.as_millis(),
            vertex_elapsed.as_millis(),
            quad_count,
            world_vertices.len(),
            world_hit_regions.len()
        ));
        Ok(Self {
            surface_size_independent: Self::scene_is_surface_size_independent(&state.scene),
            world_vertices: gpu_data::shared_geometry::SharedGeometry::for_document(
                world_vertices,
                &state.scene.scene_id,
            ),
            world_strokes: gpu_data::shared_geometry::SharedGeometry::for_document(
                world_strokes,
                &state.scene.scene_id,
            ),
            draw_commands: draw_commands.into(),
            world_hit_index: datum_gui_viewport::SpatialHitIndex::new(world_hit_regions).into(),
        }
        .registered_cpu())
        })
    }

    // `hit_test_authored_world` (board) and `hit_test_world` (schematic,
    // unfiltered) live in the `coordinate_hit` include-module, sharing one scan
    // core so the board path stays byte-identical while the schematic surface
    // gets a filter-free twin.
}

impl RetainedScene {
    /// Physical-size-only changes can retain geometry with no reference-scale
    /// dependency. Content/style/DPI changes still require their usual rebuild.
    pub fn can_reuse_for_surface_resize(&self) -> bool {
        self.surface_size_independent
    }

    pub(super) fn scene_is_surface_size_independent(scene: &BoardReviewSceneV1) -> bool {
        // These production paths call world_stroke_nm while constructing world
        // vertices: unrouted endpoints/widths and closed mechanical dash/gaps.
        // Be conservative even when those primitives are currently hidden.
        scene.unrouted_primitives.is_empty()
            && !scene
                .component_graphics
                .iter()
                .any(|graphic| graphic.closed && graphic.render_role == "component_mechanical")
    }
}

#[cfg(test)]
mod retained_storage_tests {
    #[test]
    fn retained_revisions_share_one_document_gpu_budget_for_vertices_and_strokes() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let first = RetainedScene::from_workspace(&state, 960, 720);
        let second = RetainedScene::from_workspace(&state, 1280, 800);
        let budget = first.world_vertices.document_budget().unwrap();
        assert!(std::sync::Arc::ptr_eq(
            budget,
            first.world_strokes.document_budget().unwrap()
        ));
        assert!(std::sync::Arc::ptr_eq(
            budget,
            second.world_vertices.document_budget().unwrap()
        ));
        assert!(std::sync::Arc::ptr_eq(
            budget,
            second.world_strokes.document_budget().unwrap()
        ));
    }

    #[test]
    fn retained_payload_accounts_geometry_and_refuses_unknown_targets() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let mut scene = RetainedScene::from_workspace(&state, 960, 720);
        assert!(
            scene.heap_payload_bytes().unwrap()
                > std::mem::size_of_val(scene.world_vertices.as_ref())
        );
        scene.world_hit_index = datum_gui_viewport::SpatialHitIndex::new(vec![WorldHitRegion {
            target: HitTarget::ReviewAction("unsupported retained target".into()),
            layer_id: None,
            shape: WorldHitShape::Circle {
                center: PointNm { x: 0, y: 0 },
                radius_nm: 1.0,
            },
        }])
        .into();
        assert_eq!(scene.heap_payload_bytes(), None);
    }

    use super::*;

    #[test]
    fn geometry_observers_deduplicate_individual_allocations_and_follow_release() {
        let first = RetainedScene::from_workspace(
            &datum_gui_protocol::load_fixture_workspace_state(),
            960,
            720,
        );
        let first_observer = first.geometry_observer();
        let mut second = first.clone();
        second.world_vertices = first.world_vertices.to_vec().into();
        let second_observer = second.geometry_observer();
        assert_eq!(
            second_observer.heap_bytes_excluding([&first_observer]),
            second.world_vertices.heap_bytes()
        );
        assert_eq!(first_observer.heap_bytes_excluding([&first_observer]), 0);
        drop(first);
        assert!(
            first_observer.is_live(),
            "shared strokes still have a strong owner"
        );
        drop(second);
        assert!(!first_observer.is_live());
        assert!(!second_observer.is_live());
        assert!(
            first_observer.heap_bytes_excluding([]) > 0,
            "dead payload leaves observable Arc container ownership"
        );
    }

    #[test]
    fn cloned_retained_scene_shares_immutable_stroke_storage() {
        let state = crate::gpu_surface_pass::board_fixture_state();
        let retained = RetainedScene::from_workspace(&state, 1280, 800);
        assert!(!retained.world_strokes.is_empty());
        let cloned = retained.clone();
        assert_eq!(
            retained.world_vertices.as_ptr(),
            cloned.world_vertices.as_ptr()
        );
        assert_eq!(
            retained.world_strokes.as_ptr(),
            cloned.world_strokes.as_ptr()
        );
    }
}

#[cfg(test)]
#[path = "retained_heap_tests.rs"]
mod heap_tests;
