//! Native inputs to the shared derived-content owner.
use super::*;

impl Runtime {
    pub(crate) fn ensure_retained_scene(&mut self) -> bool {
        self.renderer.render_session_mut().ensure_board(
            self.session.workspace(),
            self.config.width,
            self.config.height,
            self.scale_factor,
        )
    }
    pub(crate) fn ensure_schematic_retained_scene(&mut self) -> bool {
        self.renderer.render_session_mut().ensure_schematic(
            self.session.workspace(),
            self.config.width,
            self.config.height,
            self.scale_factor,
        )
    }
    pub(crate) fn retained_scene_cache_key(&self) -> RetainedSceneCacheKey {
        RetainedSceneCacheKey::for_workspace(
            self.workspace(),
            self.config.width,
            self.config.height,
            self.scale_factor,
        )
    }
}
