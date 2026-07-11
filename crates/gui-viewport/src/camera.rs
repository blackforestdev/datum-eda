//! Shared camera policy and world/screen navigation math (UVT S2).

use datum_gui_protocol::SceneBounds;

/// Screen-space rectangle used by camera math without depending on a renderer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraViewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Per-surface camera limits and numerical floor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraConfig {
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub min_scale: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            min_zoom: 0.35,
            max_zoom: 8.0,
            min_scale: 0.000_001,
        }
    }
}

/// Camera state shared by every editor surface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraState {
    pub center_x_nm: f32,
    pub center_y_nm: f32,
    pub zoom: f32,
}

/// Stateless shared camera mechanism. Consumers supply surface configuration.
pub struct CameraEngine;

impl CameraEngine {
    pub fn fit_to_bounds(bounds: &SceneBounds) -> CameraState {
        CameraState {
            center_x_nm: midpoint(bounds.min_x, bounds.max_x),
            center_y_nm: midpoint(bounds.min_y, bounds.max_y),
            zoom: 1.0,
        }
    }

    pub fn pan_pixels(
        state: &mut CameraState,
        config: CameraConfig,
        viewport: CameraViewport,
        bounds: &SceneBounds,
        delta_x_px: f32,
        delta_y_px: f32,
    ) {
        if !valid_viewport(viewport)
            || !valid_config(config)
            || !delta_x_px.is_finite()
            || !delta_y_px.is_finite()
            || !valid_state(*state)
        {
            return;
        }
        let scale = projection_scale(viewport, bounds, *state, config);
        state.center_x_nm -= delta_x_px / scale;
        state.center_y_nm -= delta_y_px / scale;
    }

    pub fn zoom_about_screen_point(
        state: &mut CameraState,
        config: CameraConfig,
        viewport: CameraViewport,
        bounds: &SceneBounds,
        screen_x: f32,
        screen_y: f32,
        zoom_delta: f32,
    ) {
        if !valid_viewport(viewport)
            || !valid_config(config)
            || !valid_state(*state)
            || !zoom_delta.is_finite()
            || zoom_delta <= 0.0
        {
            return;
        }
        let before = screen_to_world(viewport, bounds, *state, config, screen_x, screen_y);
        state.zoom = (state.zoom * zoom_delta).clamp(config.min_zoom, config.max_zoom);
        let after = screen_to_world(viewport, bounds, *state, config, screen_x, screen_y);
        state.center_x_nm += before.0 - after.0;
        state.center_y_nm += before.1 - after.1;
    }
}

impl CameraState {
    pub fn fit_to_bounds(bounds: &SceneBounds) -> Self {
        CameraEngine::fit_to_bounds(bounds)
    }

    pub fn pan_pixels<V: Into<CameraViewport>>(
        &mut self,
        viewport: V,
        bounds: &SceneBounds,
        delta_x_px: f32,
        delta_y_px: f32,
    ) {
        CameraEngine::pan_pixels(
            self,
            CameraConfig::default(),
            viewport.into(),
            bounds,
            delta_x_px,
            delta_y_px,
        );
    }

    pub fn zoom_about_screen_point<V: Into<CameraViewport>>(
        &mut self,
        viewport: V,
        bounds: &SceneBounds,
        screen_x: f32,
        screen_y: f32,
        zoom_delta: f32,
    ) {
        CameraEngine::zoom_about_screen_point(
            self,
            CameraConfig::default(),
            viewport.into(),
            bounds,
            screen_x,
            screen_y,
            zoom_delta,
        );
    }
}

fn midpoint(min: i64, max: i64) -> f32 {
    ((min as f64 + max as f64) * 0.5) as f32
}

fn projection_scale(
    viewport: CameraViewport,
    bounds: &SceneBounds,
    state: CameraState,
    config: CameraConfig,
) -> f32 {
    let width = ((bounds.max_x as i128 - bounds.min_x as i128).max(1) as f64) as f32;
    let height = ((bounds.max_y as i128 - bounds.min_y as i128).max(1) as f64) as f32;
    let fit = (viewport.width / width)
        .min(viewport.height / height)
        .max(config.min_scale);
    (fit * state.zoom).max(config.min_scale)
}

fn valid_viewport(viewport: CameraViewport) -> bool {
    viewport.x.is_finite()
        && viewport.y.is_finite()
        && viewport.width.is_finite()
        && viewport.height.is_finite()
        && viewport.width > 0.0
        && viewport.height > 0.0
}

fn valid_state(state: CameraState) -> bool {
    state.center_x_nm.is_finite()
        && state.center_y_nm.is_finite()
        && state.zoom.is_finite()
        && state.zoom > 0.0
}

fn valid_config(config: CameraConfig) -> bool {
    config.min_zoom.is_finite()
        && config.max_zoom.is_finite()
        && config.min_scale.is_finite()
        && config.min_zoom > 0.0
        && config.max_zoom >= config.min_zoom
        && config.min_scale > 0.0
}

fn screen_to_world(
    viewport: CameraViewport,
    bounds: &SceneBounds,
    state: CameraState,
    config: CameraConfig,
    x: f32,
    y: f32,
) -> (f32, f32) {
    let scale = projection_scale(viewport, bounds, state, config);
    let center_x = viewport.x + viewport.width * 0.5;
    let center_y = viewport.y + viewport.height * 0.5;
    (
        state.center_x_nm + (x - center_x) / scale,
        state.center_y_nm + (y - center_y) / scale,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> SceneBounds {
        SceneBounds { min_x: -2_000_000, min_y: 1_000_000, max_x: 6_000_000, max_y: 5_000_000 }
    }

    fn viewport() -> CameraViewport {
        CameraViewport { x: 100.0, y: 50.0, width: 800.0, height: 400.0 }
    }

    #[test]
    fn fit_centers_each_axis_without_overflow_prone_integer_addition() {
        let camera = CameraEngine::fit_to_bounds(&bounds());
        assert_eq!((camera.center_x_nm, camera.center_y_nm, camera.zoom), (2_000_000.0, 3_000_000.0, 1.0));
    }

    #[test]
    fn pan_uses_visible_projection_scale() {
        let mut camera = CameraEngine::fit_to_bounds(&bounds());
        CameraEngine::pan_pixels(&mut camera, CameraConfig::default(), viewport(), &bounds(), 80.0, -40.0);
        assert_eq!((camera.center_x_nm, camera.center_y_nm), (1_200_000.0, 3_400_000.0));
    }

    #[test]
    fn zoom_clamps_and_keeps_anchor_world_position_stationary() {
        let mut camera = CameraEngine::fit_to_bounds(&bounds());
        CameraEngine::zoom_about_screen_point(&mut camera, CameraConfig::default(), viewport(), &bounds(), 700.0, 250.0, 100.0);
        assert_eq!(camera.zoom, 8.0);
        assert_eq!((camera.center_x_nm, camera.center_y_nm), (3_750_000.0, 3_000_000.0));
        CameraEngine::zoom_about_screen_point(&mut camera, CameraConfig::default(), viewport(), &bounds(), 700.0, 250.0, 0.0001);
        assert_eq!(camera.zoom, 0.35);
    }

    #[test]
    fn invalid_zoom_delta_is_a_noop() {
        let mut camera = CameraEngine::fit_to_bounds(&bounds());
        let before = camera;
        CameraEngine::zoom_about_screen_point(&mut camera, CameraConfig::default(), viewport(), &bounds(), 500.0, 250.0, f32::NAN);
        assert_eq!(camera, before);
    }

    #[test]
    fn extreme_bounds_do_not_overflow_fit_or_projection_math() {
        let extreme = SceneBounds {
            min_x: i64::MIN,
            min_y: i64::MIN,
            max_x: i64::MAX,
            max_y: i64::MAX,
        };
        let mut camera = CameraEngine::fit_to_bounds(&extreme);
        assert!(camera.center_x_nm.is_finite());
        assert!(camera.center_y_nm.is_finite());
        CameraEngine::pan_pixels(
            &mut camera,
            CameraConfig::default(),
            viewport(),
            &extreme,
            1.0,
            1.0,
        );
        assert!(camera.center_x_nm.is_finite());
        assert!(camera.center_y_nm.is_finite());
    }

    #[test]
    fn invalid_viewport_and_pan_delta_are_noops() {
        let mut camera = CameraEngine::fit_to_bounds(&bounds());
        let before = camera;
        CameraEngine::pan_pixels(
            &mut camera,
            CameraConfig::default(),
            CameraViewport { width: 0.0, ..viewport() },
            &bounds(),
            10.0,
            10.0,
        );
        CameraEngine::pan_pixels(
            &mut camera,
            CameraConfig::default(),
            viewport(),
            &bounds(),
            f32::NAN,
            10.0,
        );
        assert_eq!(camera, before);
    }

    #[test]
    fn invalid_camera_config_is_a_noop() {
        let mut camera = CameraEngine::fit_to_bounds(&bounds());
        let before = camera;
        let invalid = CameraConfig {
            min_zoom: 2.0,
            max_zoom: 1.0,
            min_scale: f32::NAN,
        };
        CameraEngine::zoom_about_screen_point(
            &mut camera,
            invalid,
            viewport(),
            &bounds(),
            500.0,
            250.0,
            2.0,
        );
        assert_eq!(camera, before);
    }
}
