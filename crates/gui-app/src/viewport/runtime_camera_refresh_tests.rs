use super::*;
use datum_gui_render::{PreparedScene, RetainedScene, ShellLayout};

#[test]
fn maximized_edit_preserves_hidden_duplicate_and_keeps_input_hidden() {
    let mut before = state();
    before
        .ui
        .layout
        .split_focused(datum_gui_protocol::SplitOrientation::Horizontal);
    let focused = before.ui.layout.focused;
    let hidden = before
        .ui
        .layout
        .leaves()
        .into_iter()
        .find(|id| {
            *id != focused && before.ui.layout.content_for(*id) == Some(PaneContent::Schematic)
        })
        .unwrap();
    let shell = ShellLayout::for_surface(1344, 840, 1.0, None);
    let active = CameraState::fit_to_bounds(&before.scene.bounds);
    let mut warm = PaneCameras::new(focused, PaneContent::Schematic, active);
    let hidden_camera = CameraState {
        zoom: 3.0,
        center_x_nm: 1_000_000.0,
        center_y_nm: 2_000_000.0,
    };
    warm.inherit(hidden, PaneContent::Schematic, hidden_camera);
    let initial = refresh_cameras(&before, &before, &shell, active, &warm);
    let image = prepared(&before, &initial);
    let original = image
        .world_point_for_pane_unclipped(hidden, 430.0, 400.0)
        .unwrap();
    before.ui.layout.zoomed = Some(focused);
    let mut after = before.clone();
    // The narrow split starts at the renderer's minimum scale. Shrinking both
    // bounds exercises a changed fit instead of remaining on that same floor.
    let bounds = &mut after.schematic_scene.as_mut().unwrap().bounds;
    bounds.max_x /= 4;
    bounds.max_y /= 4;
    assert!(datum_gui_protocol::camera_scene_for_pane(&after, hidden).is_none());
    let updates = refresh_cameras(&before, &after, &shell, active, &warm);
    let (_, _, _, updated) = updates.iter().find(|(id, ..)| *id == hidden).unwrap();
    assert_eq!(updated.center_x_nm, hidden_camera.center_x_nm);
    assert_eq!(updated.center_y_nm, hidden_camera.center_y_nm);
    assert_ne!(updated.zoom, hidden_camera.zoom);
    after.ui.layout.zoomed = None;
    let image = prepared(&after, &updates);
    let actual = image
        .world_point_for_pane_unclipped(hidden, 430.0, 400.0)
        .unwrap();
    assert_same_projection(actual, original, &before.scene.bounds);
}

fn assert_same_projection(
    actual: PointNm,
    expected: PointNm,
    bounds: &datum_gui_protocol::SceneBounds,
) {
    // Allow accumulated f32 roundoff across six refreshes and projection at
    // scene magnitude: camera subtraction can cancel digits in the result.
    // Eight relative epsilons is a fixture-scale allowance (<0.001px here),
    // not a universal error bound for arbitrary coordinates or history length.
    let magnitude = [
        bounds.min_x,
        bounds.min_y,
        bounds.max_x,
        bounds.max_y,
        expected.x,
        expected.y,
    ]
    .into_iter()
    .map(|n| (n as f64).abs())
    .fold(1.0, f64::max);
    let rounding = 1.0 + 8.0 * f64::from(f32::EPSILON) * magnitude;
    for (actual, expected) in [(actual.x, expected.x), (actual.y, expected.y)] {
        assert!(
            ((actual - expected) as f64).abs() <= rounding,
            "{actual} != {expected}"
        );
    }
}

fn state() -> ReviewWorkspaceState {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.schematic_scene = Some(state.scene.clone());
    state.ui.layout.set_focused_content(PaneContent::Schematic);
    state
}

fn prepared(
    state: &ReviewWorkspaceState,
    cameras: &[(datum_gui_protocol::PaneId, PaneContent, bool, CameraState)],
) -> PreparedScene {
    let retained = RetainedScene::from_workspace(state, 1344, 840);
    let mut result = PreparedScene::from_workspace(
        state,
        1344,
        840,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    )
    .unwrap();
    for &(pane, _, _, camera) in cameras {
        result.set_surface_camera(pane, camera);
    }
    result
}

#[test]
fn publication_preserves_cold_and_warm_pane_projection_and_history() {
    let before = state();
    let pane = before.ui.layout.focused;
    let shell = ShellLayout::for_surface(1344, 840, 1.0, None);
    let mut after = before.clone();
    let bounds = &mut after.schematic_scene.as_mut().unwrap().bounds;
    bounds.min_y -= 100_000_000;
    bounds.max_x += 50_000_000;
    let active = CameraState::fit_to_bounds(&before.scene.bounds);
    for warm_camera in [
        None,
        Some(CameraState {
            center_x_nm: 8_000_000.0,
            center_y_nm: -4_000_000.0,
            zoom: 2.2,
        }),
    ] {
        let mut warm = PaneCameras::new(
            datum_gui_protocol::PaneId(u32::MAX),
            PaneContent::Board,
            active,
        );
        if let Some(camera) = warm_camera {
            warm.inherit(pane, PaneContent::Schematic, camera);
        }
        let initial = refresh_cameras(&before, &before, &shell, active, &warm);
        let image = prepared(&before, &initial);
        let point =
            |image: &PreparedScene, x, y| image.world_point_for_pane_unclipped(pane, x, y).unwrap();
        let expected = [point(&image, 450.0, 250.0), point(&image, 600.0, 600.0)];
        let unfixed = prepared(&after, &initial);
        assert_ne!(expected[0], point(&unfixed, 450.0, 250.0));
        for _ in 0..3 {
            for (old, new) in [(&before, &after), (&after, &before)] {
                let updates = refresh_cameras(old, new, &shell, active, &warm);
                let image = prepared(new, &updates);
                for (actual, expected) in [point(&image, 450.0, 250.0), point(&image, 600.0, 600.0)]
                    .into_iter()
                    .zip(expected)
                {
                    assert_same_projection(
                        actual,
                        expected,
                        &after.schematic_scene.as_ref().unwrap().bounds,
                    );
                }
                for (id, content, _, camera) in updates {
                    warm.inherit(id, content, camera);
                }
            }
        }
    }
}

#[test]
fn refresh_does_not_transfer_camera_between_documents_or_content() {
    let before = state();
    let pane = before.ui.layout.focused;
    let shell = ShellLayout::for_surface(1344, 840, 1.0, None);
    let active = CameraState::fit_to_bounds(&before.scene.bounds);
    let warm = PaneCameras::new(pane, PaneContent::Schematic, active);
    let mut after = before.clone();
    after.schematic_scene.as_mut().unwrap().board_uuid = uuid::Uuid::new_v4().to_string();
    assert!(
        refresh_cameras(&before, &after, &shell, active, &warm)
            .iter()
            .all(|(id, ..)| *id != pane)
    );
    after = before.clone();
    after.ui.layout.set_focused_content(PaneContent::Board);
    assert!(
        refresh_cameras(&before, &after, &shell, active, &warm)
            .iter()
            .all(|(id, ..)| *id != pane)
    );
}
