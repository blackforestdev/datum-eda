//! Ordinary empty launch uses existing Main and the shared New/Open controllers.
use super::*;

impl GuiArgs {
    pub(super) fn has_no_project_source(&self) -> bool {
        !self.demo_known_good
            && self.project_root.is_none()
            && self.board_file.is_none()
            && self.schematic_file.is_none()
            && self.artifact_path.is_none()
            && self.net_uuid.is_none()
            && self.from_anchor_pad_uuid.is_none()
            && self.to_anchor_pad_uuid.is_none()
            && self.profile.is_none()
    }

    pub(super) fn unopened_launch_state(
        &self,
        proxy: Option<winit::event_loop::EventLoopProxy<()>>,
    ) -> Result<LaunchState> {
        let mut state = datum_gui_protocol::ReviewWorkspaceState::unopened();
        self.apply_initial_layout(&mut state.ui.layout);
        self.apply_focus_pane(&mut state.ui.layout);
        state.ui.active_menu = self.open_menu.clone();
        let profiles = terminal_profile::TerminalProfileCatalog::from_args(&self.terminal_profile)?;
        state.ui.terminal.theme = profiles.selected().theme();
        state.ui.terminal.font_scale_millis = profiles.selected().font_scale_millis();
        state.ui.terminal.launch_profile_name = profiles.selected().name().to_string();
        let context = TerminalLaunchContext::unbound(
            crate::terminal_process::unbound::launch_directory()?,
            profiles.selected().clone(),
        );
        let terminal_sessions = TerminalSessionRegistry::spawn_with_proxy(&context, proxy)?;
        state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
        state.ui.terminal.status = terminal_sessions.active().running_status();
        let camera = CameraState::fit_to_bounds(&state.scene.bounds);
        Ok(LaunchState {
            reuse_resize_buffers: app_bootstrap::ordinary_resize_reuse(
                self.reuse_resize_buffers,
                std::env::var_os("DATUM_DIAGNOSTIC_RESIZE_ALLOCATION").is_some(),
            ),
            request: None,
            state,
            camera,
            terminal_launch_context: Some(context),
            terminal_profiles: profiles,
            terminal_sessions,
            workspace_include_review: false,
            open_global_preferences: self.open_global_preferences,
            open_new_project: self.open_new_project,
            open_project_preferences: self.open_project_preferences,
        })
    }
}

#[cfg(test)]
#[path = "native_project_startup_tests.rs"]
mod tests;
