//! Workspace selection of the launch template used by new terminal sessions.

use super::*;

impl Runtime {
    pub(super) fn cycle_terminal_profile(&mut self) -> bool {
        let profile = self.terminal_profiles.select_next().clone();
        self.terminal_launch_context.terminal_profile = profile.clone();
        let terminal = &mut self.session.workspace_mut().ui.terminal;
        terminal.launch_profile_name = profile.name().to_string();
        terminal.theme = profile.theme();
        terminal.font_scale_millis = profile.font_scale_millis();
        terminal.status = format!("new terminals use profile {}", profile.name());
        self.resize_terminal_to_dock();
        self.invalidate_scene();
        true
    }
}
