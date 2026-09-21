//! Actual auxiliary-host redraw dispatch through the application's frame owner.
use super::*;

pub(super) enum OwnedHost {
    Global,
    Project,
    New,
}

impl App {
    pub(super) fn request_restored_native_frames(&mut self) {
        for window in [
            self.window,
            self.global_preferences_window.as_deref(),
            self.project_preferences_window.as_deref(),
            self.new_project_window.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            self.frames.request_restored(window);
        }
    }

    pub(super) fn redraw_owned_window(&mut self, event_loop: &ActiveEventLoop, host: OwnedHost) {
        let (window, surface, project, new, label) = match host {
            OwnedHost::Global => (
                &self.global_preferences_window,
                &mut self.global_preferences_surface,
                false,
                false,
                "Global Preferences",
            ),
            OwnedHost::Project => (
                &self.project_preferences_window,
                &mut self.project_preferences_surface,
                true,
                false,
                "Project Preferences",
            ),
            OwnedHost::New => (
                &self.new_project_window,
                &mut self.new_project_surface,
                false,
                true,
                "New Project",
            ),
        };
        let (Some(window), Some(surface), Some(runtime)) = (window, surface, &self.runtime) else {
            return;
        };
        let Some(receipt) = self.frames.redraw_received(window.id()) else {
            return;
        };
        let presented = match surface.render(runtime, project, new) {
            Ok(presented) => presented,
            Err(_) if runtime.device_health.failed() => false,
            Err(error) => fatal_gui_error(event_loop, &format!("render {label} window"), error),
        };
        self.frames.frame_finished(window, receipt, presented);
    }
}
