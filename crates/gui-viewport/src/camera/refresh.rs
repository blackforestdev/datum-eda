//! Preserve the displayed scale when a document refresh changes its fit bounds.
use super::*;

impl CameraEngine {
    /// Re-express a camera against new scene bounds without navigating. Unlike
    /// a user zoom, this must not clamp to the fit-relative navigation limits.
    pub fn preserve_view_on_refresh(
        state: &mut CameraState,
        config: CameraConfig,
        viewport: CameraViewport,
        before: &SceneBounds,
        after: &SceneBounds,
    ) {
        if before == after
            || !valid_viewport(viewport)
            || !valid_config(config)
            || !valid_state(*state)
        {
            return;
        }
        let scale = projection_scale(viewport, before, *state, config);
        let fit = projection_scale(
            viewport,
            after,
            CameraState {
                zoom: 1.0,
                ..*state
            },
            config,
        );
        let zoom = scale / fit;
        if zoom.is_finite() && zoom > 0.0 {
            state.zoom = zoom;
        }
    }
}

#[cfg(test)]
#[path = "refresh_tests.rs"]
mod tests;
