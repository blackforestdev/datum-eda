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
    let pane = board.pane_id;
    prepared.surface_passes.clear();
    let fallback = prepared.geometry_admission(&retained, None).next().unwrap();
    assert_eq!(fallback.pane_id, pane);
    assert_eq!(fallback.counts().unwrap(), counts);
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
        commands: &commands,
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
