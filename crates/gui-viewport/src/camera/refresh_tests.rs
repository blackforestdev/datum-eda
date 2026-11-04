use super::*;

#[test]
fn rebased_zoom_never_reverses_the_next_gesture_and_can_return_gradually() {
    let bounds = SceneBounds {
        min_x: 0,
        min_y: 0,
        max_x: 10_000_000,
        max_y: 10_000_000,
    };
    let config = CameraConfig::default();
    for (zoom, outward, inward) in [(16.0, 1.1, 0.9), (0.1, 0.9, 1.1)] {
        let mut camera = CameraState {
            zoom,
            ..CameraState::fit_to_bounds(&bounds)
        };
        let before = camera;
        CameraEngine::zoom_about_screen_point(
            &mut camera,
            config,
            viewport(),
            &bounds,
            171.0,
            203.0,
            outward,
        );
        assert_eq!(
            camera, before,
            "outward zoom at a rebased limit must be a no-op"
        );
        let anchor = screen_to_world(viewport(), &bounds, camera, config, 171.0, 203.0);
        CameraEngine::zoom_about_screen_point(
            &mut camera,
            config,
            viewport(),
            &bounds,
            171.0,
            203.0,
            inward,
        );
        assert_eq!(camera.zoom, zoom * inward);
        let after = screen_to_world(viewport(), &bounds, camera, config, 171.0, 203.0);
        assert!((anchor.0 - after.0).abs() < 16.0 && (anchor.1 - after.1).abs() < 16.0);
    }
}

fn viewport() -> CameraViewport {
    CameraViewport {
        x: 42.0,
        y: 75.0,
        width: 900.0,
        height: 500.0,
    }
}

#[test]
fn refresh_preserves_scale_and_center_through_changed_limiting_axis_and_history() {
    let before = SceneBounds {
        min_x: -10_000_000,
        min_y: -8_000_000,
        max_x: 20_000_000,
        max_y: 9_000_000,
    };
    let after = SceneBounds {
        min_x: -10_000_000,
        min_y: -80_000_000,
        max_x: 20_000_000,
        max_y: 9_000_000,
    };
    let config = CameraConfig::default();
    let mut camera = CameraState {
        center_x_nm: 2_000_000.0,
        center_y_nm: -1_000_000.0,
        zoom: 2.0,
    };
    let original = camera;
    let scale = projection_scale(viewport(), &before, camera, config);
    assert_ne!(
        scale,
        projection_scale(viewport(), &after, camera, config),
        "unchanged state alone reproduces the scale bug"
    );
    for _ in 0..100 {
        for (old, new) in [(&before, &after), (&after, &before)] {
            CameraEngine::preserve_view_on_refresh(&mut camera, config, viewport(), old, new);
            assert_eq!(camera.center_x_nm, original.center_x_nm);
            assert_eq!(camera.center_y_nm, original.center_y_nm);
            assert!(
                (projection_scale(viewport(), new, camera, config) / scale - 1.0).abs() < 0.000_001
            );
            let unchanged = camera;
            CameraEngine::preserve_view_on_refresh(&mut camera, config, viewport(), new, new);
            assert_eq!(camera, unchanged);
        }
    }
    let fit = CameraEngine::fit_to_bounds(&after);
    assert_eq!(fit.zoom, 1.0);
    assert_ne!(fit.center_y_nm, camera.center_y_nm);
}

#[test]
fn refresh_handles_degenerate_bounds_floor_and_invalid_input() {
    let bounds = SceneBounds {
        min_x: 0,
        min_y: 0,
        max_x: 0,
        max_y: 0,
    };
    let huge = SceneBounds {
        min_x: i64::MIN,
        min_y: i64::MIN,
        max_x: i64::MAX,
        max_y: i64::MAX,
    };
    let config = CameraConfig::default();
    let mut camera = CameraState::fit_to_bounds(&bounds);
    let original = camera;
    CameraEngine::preserve_view_on_refresh(&mut camera, config, viewport(), &bounds, &huge);
    assert!((projection_scale(viewport(), &huge, camera, config) - 500.0).abs() < 0.001);
    CameraEngine::preserve_view_on_refresh(&mut camera, config, viewport(), &huge, &bounds);
    assert_eq!(camera, original);
    CameraEngine::preserve_view_on_refresh(
        &mut camera,
        config,
        CameraViewport {
            width: 0.0,
            ..viewport()
        },
        &bounds,
        &huge,
    );
    assert_eq!(camera, original);
}
