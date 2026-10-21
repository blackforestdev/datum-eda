use super::*;

#[test]
fn prepared_geometry_counts_follow_production_ranges_and_reject_invalid_ranges() {
    let state = crate::gpu_surface_pass::board_fixture_state();
    let retained = RetainedScene::from_workspace_for_surface(&state, 960, 720, 1.0);
    let mut prepared = PreparedScene::from_workspace_with_terminal_renderer(
        &state,
        960,
        720,
        1.0,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
        &[],
        None,
        true,
    )
    .unwrap();
    let board = prepared
        .geometry_admission(&retained, None)
        .find(|p| p.surface == SceneSurface::Board)
        .unwrap();
    let counts = board.counts().unwrap();
    assert_eq!(counts.retained_vertices, retained.world_vertices.len());
    assert_eq!(
        counts.retained_stroke_instances,
        retained.world_strokes.len()
    );
    assert!(counts.prepared_commands > 0);
    assert!(counts.prepared_triangles > 0);
    assert_eq!(board.ranges().count(), counts.prepared_commands);
    let passes = std::mem::take(&mut prepared.surface_passes);
    assert!(!prepared.visible_draw_commands.is_empty());
    assert_eq!(
        prepared.geometry_admission(&retained, None).count(),
        0,
        "empty normalized descriptors cannot resurrect legacy board commands"
    );
    assert!(!prepared.requires_board_world());
    prepared.surface_passes = passes;
    prepared.visible_draw_commands = vec![RetainedDrawCommand::Quads {
        layer_id: None,
        range: 0..u32::MAX,
    }];
    assert!(
        prepared
            .geometry_admission(&retained, None)
            .next()
            .unwrap()
            .counts()
            .is_err()
    );
    prepared.visible_draw_commands.clear();
    prepared.schematic_scene_viewport = None;
    prepared.surface_passes.clear();
    assert_eq!(prepared.geometry_admission(&retained, None).count(), 0);
}

#[test]
fn geometry_count_triangle_lists_do_not_join_incomplete_draws() {
    let state = crate::gpu_surface_pass::board_fixture_state();
    let retained = RetainedScene::from_workspace_for_surface(&state, 960, 720, 1.0);
    assert!(retained.world_vertices.len() >= 6);
    let commands = [
        RetainedDrawCommand::Quads {
            layer_id: None,
            range: 0..2,
        },
        RetainedDrawCommand::Quads {
            layer_id: None,
            range: 2..6,
        },
    ];
    let observation = PreparedGeometryAdmission {
        pane_id: datum_gui_protocol::PaneId(1),
        surface: SceneSurface::Board,
        viewport: RectPx {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        },
        source: None,
        retained: Some(&retained),
        commands: VisibleDrawCommands {
            commands: &commands,
            filters: None,
        },
    };
    let counts = observation.counts().unwrap();
    assert_eq!(counts.prepared_vertices, 6);
    assert_eq!(counts.prepared_triangles, 1);
    let missing = PreparedGeometryAdmission {
        retained: None,
        ..observation
    };
    assert!(missing.counts().is_err());
}

#[test]
fn prepared_source_identity_is_a_snapshot_not_a_later_model_reference() {
    let mut state = crate::gpu_surface_pass::board_fixture_state();
    let observed = PreparedSourceIdentities::capture(&state, &[]);
    assert_eq!(observed.board.scene_id, state.scene.scene_id);
    assert_eq!(observed.board.source_revision, state.scene.source_revision);
    let before = state.scene.source_revision.clone();
    state.scene.source_revision.push_str("-later-model");
    assert_eq!(observed.board.source_revision, before);
    assert_ne!(observed.board.source_revision, state.scene.source_revision);
    assert_eq!(
        observed.schematic.as_ref().map(|s| &s.scene_id),
        state.schematic_scene.as_ref().map(|s| &s.scene_id)
    );
}

#[test]
fn schematic_layer_visibility_filters_draws_masks_and_counts_without_retiring_geometry() {
    let (fixture, mut state) = crate::native_selection_overlay::tests::selected_points_state();
    let source = fixture.source_bytes();
    let selection = state.selection.clone();
    let board = RetainedScene::from_workspace(&state, 1280, 800);
    let schematic =
        RetainedScene::from_workspace_schematic_for_surface(&state, 1280, 800, 1.0).unwrap();
    let retained_commands = schematic.all_draw_commands().to_vec();
    assert!(retained_commands.iter().any(|c| matches!(
        c, RetainedDrawCommand::Quads { layer_id: Some(layer), .. } if layer == "L202"
    )));
    assert!(retained_commands.iter().any(|c| matches!(
        c, RetainedDrawCommand::Strokes { layer_id: Some(layer), .. } if layer == "L202"
    )));
    let resident_vertices = schematic.world_vertices().as_ptr();
    let resident_strokes = schematic.world_strokes.as_ptr();
    let prepare = |state: &ReviewWorkspaceState| {
        let mut prepared = PreparedScene::from_workspace(
            state,
            1280,
            800,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &board,
        )
        .unwrap();
        prepared.append_native_selection(state).unwrap();
        prepared
    };
    let counts = |prepared: &PreparedScene, surface| {
        prepared
            .geometry_admission(&board, Some(&schematic))
            .find(|p| p.surface == surface)
            .unwrap()
            .counts()
            .unwrap()
    };
    let commands = |prepared: &PreparedScene| {
        prepared
            .surface_draw_commands(SceneSurface::Schematic, Some(&schematic))
            .iter()
            .cloned()
            .collect::<Vec<_>>()
    };
    let painted = |prepared: &PreparedScene| {
        prepared
            .selected_paint_ranges(&board, Some(&schematic))
            .filter(|range| range.surface == SceneSurface::Schematic)
            .count()
    };
    let visible = prepare(&state);
    let visible_counts = counts(&visible, SceneSurface::Schematic);
    let board_counts = counts(&visible, SceneSurface::Board);
    assert_eq!(commands(&visible), retained_commands);
    assert!(visible_counts.prepared_triangles > 0 && painted(&visible) > 0);

    state
        .ui
        .filters
        .layer_visibility
        .insert("L202".into(), false);
    let hidden = prepare(&state);
    assert!(commands(&hidden).is_empty());
    assert_eq!(painted(&hidden), 0);
    let hidden_counts = counts(&hidden, SceneSurface::Schematic);
    assert_eq!(hidden_counts.prepared_commands, 0);
    assert_eq!(hidden_counts.prepared_triangles, 0);
    assert_eq!(
        hidden_counts.retained_vertices,
        visible_counts.retained_vertices
    );
    assert_eq!(
        hidden_counts.retained_stroke_instances,
        visible_counts.retained_stroke_instances
    );
    assert_eq!(counts(&hidden, SceneSurface::Board), board_counts);
    assert_eq!(
        commands(&visible),
        retained_commands,
        "visibility is a frame snapshot"
    );

    state
        .ui
        .filters
        .layer_visibility
        .insert("L202".into(), true);
    let restored = prepare(&state);
    assert_eq!(commands(&restored), retained_commands);
    assert_eq!(counts(&restored, SceneSurface::Schematic), visible_counts);
    assert_eq!(painted(&restored), painted(&visible));
    assert!(
        commands(&hidden).is_empty(),
        "later UI changes cannot mutate a prepared frame"
    );
    assert_eq!(schematic.world_vertices().as_ptr(), resident_vertices);
    assert_eq!(schematic.world_strokes.as_ptr(), resident_strokes);
    assert_eq!(schematic.all_draw_commands(), retained_commands);
    assert_eq!(fixture.source_bytes(), source);
    assert_eq!(state.selection, selection);
}
