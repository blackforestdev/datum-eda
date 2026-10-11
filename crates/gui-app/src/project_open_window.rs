//! Semantic Open controls hosted by the shared owned Project form window.
use super::*;
use datum_gui_protocol::ProjectOpenFocus;

impl Runtime {
    pub(super) fn activate_project_open_hit(&mut self, target: &HitTarget) -> bool {
        if !self.workspace().ui.project_open.open {
            return false;
        }
        match target {
            HitTarget::ProjectOpenModal => return true,
            HitTarget::ProjectOpenDirectory => {
                self.session.workspace_mut().ui.project_open.focus = ProjectOpenFocus::Directory;
            }
            HitTarget::ProjectOpenCancel => {
                self.close_project_open();
            }
            HitTarget::ProjectOpenConfirm => {
                self.begin_project_open();
            }
            _ => return false,
        }
        self.invalidate_frame();
        true
    }
}
