//! Native source/view input adapter; shared rendering owns validity and history.
use super::*;

impl Runtime {
    pub(crate) fn ensure_retained_scene(&mut self) -> bool {
        if self.renderer.render_session().prepared().is_some()
            && self.renderer.render_session().board().is_some()
        {
            return true;
        }

        self.renderer
            .render_session_mut()
            .ensure_sources(
                self.session.workspace(),
                Some(self.render_sources.revision()),
                self.config.width,
                self.config.height,
                self.scale_factor,
            )
            .is_ok()
    }
}
