use super::*;

fn duplicate_surface_state(content: datum_gui_protocol::PaneContent) -> ReviewWorkspaceState {
    let schematic = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
    let projected = datum_gui_protocol::load_kicad_schematic_workspace_state(&schematic)
        .expect("simple schematic fixture should load");
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.schematic_scene = Some(projected.scene);
    state.ui.layout.set_focused_content(content);
    state.ui.layout.focus_next();
    state.ui.layout.set_focused_content(content);
    state
}

fn prepared_for(state: &ReviewWorkspaceState, retained: &RetainedScene) -> PreparedScene {
    PreparedScene::from_workspace_for_surface(
        state,
        1600,
        1000,
        1.0,
        CameraState::fit_to_bounds(&state.scene.bounds),
        retained,
    )
}

#[test]
fn duplicate_board_leaves_build_independent_surface_passes() {
    let mut state = duplicate_surface_state(datum_gui_protocol::PaneContent::Board);
    let retained = RetainedScene::from_workspace_for_surface(&state, 1600, 1000, 1.0);
    let mut prepared = prepared_for(&state, &retained);
    let passes: Vec<_> = prepared
        .surface_passes()
        .iter()
        .filter(|pass| pass.surface == SceneSurface::Board)
        .cloned()
        .collect();
    assert_eq!(passes.len(), 2);
    assert_ne!(passes[0].pane_id, passes[1].pane_id);
    assert_ne!(passes[0].scene_viewport, passes[1].scene_viewport);
    let (grid_vertices, grid_batches) = surface_grid_pass::build_surface_grids(&prepared);
    assert!(!grid_vertices.is_empty());
    assert_eq!(grid_batches.len(), 2);
    assert_eq!(grid_batches[0].viewport, passes[0].scene_viewport);
    assert_eq!(grid_batches[1].viewport, passes[1].scene_viewport);

    let mut second_camera = passes[1].camera;
    second_camera.center_x_nm += 5_000_000.0;
    prepared.set_surface_camera(passes[1].pane_id, second_camera);
    let first = prepared
        .surface_passes()
        .iter()
        .find(|pass| pass.pane_id == passes[0].pane_id)
        .unwrap();
    let second = prepared
        .surface_passes()
        .iter()
        .find(|pass| pass.pane_id == passes[1].pane_id)
        .unwrap();
    assert_eq!(first.camera, passes[0].camera);
    assert_eq!(second.camera, second_camera);
    state.ui.cursor_pos = Some(datum_gui_protocol::ScreenPointPx {
        x: passes[1].scene_viewport.x + passes[1].scene_viewport.width * 0.5,
        y: passes[1].scene_viewport.y + passes[1].scene_viewport.height * 0.5,
    });
    prepared.refresh_interaction(&state, &retained);
    assert_eq!(
        prepared.interaction_viewport(SceneSurface::Board),
        Some(passes[1].scene_viewport)
    );
    assert!(!prepared.board_interaction_vertices().is_empty());
}

#[test]
fn duplicate_schematic_leaves_project_through_their_own_cameras() {
    let mut state = duplicate_surface_state(datum_gui_protocol::PaneContent::Schematic);
    let retained = RetainedScene::from_workspace_for_surface(&state, 1600, 1000, 1.0);
    let mut prepared = prepared_for(&state, &retained);
    let passes: Vec<_> = prepared
        .surface_passes()
        .iter()
        .filter(|pass| pass.surface == SceneSurface::Schematic)
        .cloned()
        .collect();
    assert_eq!(passes.len(), 2);
    let (grid_vertices, grid_batches) = surface_grid_pass::build_surface_grids(&prepared);
    assert!(!grid_vertices.is_empty());
    assert_eq!(grid_batches.len(), 2);
    let mut second_camera = passes[1].camera;
    second_camera.center_y_nm += 7_000_000.0;
    prepared.set_surface_camera(passes[1].pane_id, second_camera);

    let resolve_center = |pass: &PreparedSurfacePass| {
        prepared
            .world_point_at_screen(
                pass.scene_viewport.x + pass.scene_viewport.width * 0.5,
                pass.scene_viewport.y + pass.scene_viewport.height * 0.5,
            )
            .expect("each duplicate pane centre must resolve")
            .0
    };
    let first_world = resolve_center(&passes[0]);
    let second_pass = prepared
        .surface_passes()
        .iter()
        .find(|pass| pass.pane_id == passes[1].pane_id)
        .unwrap();
    let second_world = resolve_center(second_pass);
    assert_ne!(first_world.y, second_world.y);
    assert_eq!(second_world.y, second_camera.center_y_nm.round() as i64);
    let second_viewport = second_pass.scene_viewport;
    state.ui.cursor_pos = Some(datum_gui_protocol::ScreenPointPx {
        x: second_viewport.x + second_viewport.width * 0.5,
        y: second_viewport.y + second_viewport.height * 0.5,
    });
    prepared.refresh_interaction(&state, &retained);
    assert_eq!(
        prepared.interaction_viewport(SceneSurface::Schematic),
        Some(second_viewport)
    );
    assert!(!prepared.schematic_overlay_vertices().is_empty());
}
