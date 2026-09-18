//! Changed-state boundary shared by pointer and focused-pane zoom handlers.

use datum_gui_protocol::SceneBounds;
use datum_gui_viewport::{CameraConfig, CameraEngine, CameraState, CameraViewport};

pub(super) fn apply(
    camera: &mut CameraState,
    config: CameraConfig,
    viewport: CameraViewport,
    bounds: &SceneBounds,
    x: f32,
    y: f32,
    delta: f32,
) -> bool {
    let before = *camera;
    CameraEngine::zoom_about_screen_point(camera, config, viewport, bounds, x, y, delta);
    // Compare the resulting state, not the input magnitude: clamping and
    // rejected input can be no-ops, while tiny representable changes matter.
    *camera != before
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> SceneBounds {
        SceneBounds {
            min_x: -2_000_000,
            min_y: 1_000_000,
            max_x: 6_000_000,
            max_y: 5_000_000,
        }
    }

    fn viewport() -> CameraViewport {
        CameraViewport {
            x: 100.0,
            y: 50.0,
            width: 800.0,
            height: 400.0,
        }
    }

    #[test]
    fn repeated_clamped_zoom_preserves_off_center_camera_exactly() {
        let config = CameraConfig::default();
        for (zoom, delta) in [(config.max_zoom, 1.12), (config.min_zoom, 0.89)] {
            let mut camera = CameraState::fit_to_bounds(&bounds());
            camera.zoom = zoom;
            let before = camera;
            for _ in 0..1000 {
                assert!(!apply(
                    &mut camera,
                    config,
                    viewport(),
                    &bounds(),
                    217.5,
                    183.25,
                    delta
                ));
            }
            assert_eq!(camera, before);
        }
    }

    #[test]
    fn unity_and_rejected_deltas_do_not_report_damage() {
        for delta in [1.0, 0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut camera = CameraState::fit_to_bounds(&bounds());
            let before = camera;
            assert!(!apply(
                &mut camera,
                CameraConfig::default(),
                viewport(),
                &bounds(),
                217.5,
                183.25,
                delta
            ));
            assert_eq!(camera, before);
        }
    }

    #[test]
    fn effective_zoom_preserves_existing_anchor_math_without_a_dead_zone() {
        for delta in [1.12, 0.89, 1.000_001] {
            let mut camera = CameraState::fit_to_bounds(&bounds());
            let mut expected = camera;
            CameraEngine::zoom_about_screen_point(
                &mut expected,
                CameraConfig::default(),
                viewport(),
                &bounds(),
                217.5,
                183.25,
                delta,
            );
            assert!(apply(
                &mut camera,
                CameraConfig::default(),
                viewport(),
                &bounds(),
                217.5,
                183.25,
                delta
            ));
            assert_eq!(camera, expected);
        }
    }
}
