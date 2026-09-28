//! Production immutable TerminalCore snapshot-to-scene composition.

use super::*;

impl Runtime {
    pub(super) fn prepared_scene(&mut self) -> Option<&PreparedScene> {
        if self.renderer.render_session().prepared().is_none() {
            if !self.ensure_retained_scene() {
                return None;
            }
            match self.build_terminal_prepared_scene() {
                Ok(()) => {}
                Err(error) => {
                    append_gui_verbose_diagnostic_line(|| {
                        format!("scene preparation refused: {error:#}")
                    });
                    return None;
                }
            }
        }
        self.renderer.render_session().prepared()
    }

    pub(super) fn build_terminal_prepared_scene(&mut self) -> Result<()> {
        let schematic_camera = self.schematic_camera_for_render();
        let pane_cameras = self.render_camera_inputs();
        let retry = self.renderer.render_session_mut().restore_terminal_damage();
        self.terminal_sessions.restore_render_damage(retry);
        // Closed docks consume no terminal render input. Leave dirty rows in
        // TerminalCore until the dock opens instead of copying its full screen
        // for each board camera frame.
        let terminal_panes = if self.workspace().ui.active_dock_tab.is_some() {
            self.terminal_sessions
                .take_active_tab_render_states(&self.session.workspace().ui.terminal)
                .context("snapshot active terminal tab panes for rendering")?
        } else {
            Vec::new()
        };
        let preparation = self.renderer.prepare_session_workspace(
            self.session.workspace(),
            datum_gui_render::WorkspaceView {
                source_revision: Some(self.render_sources.revision()),
                width: self.config.width,
                height: self.config.height,
                scale: self.scale_factor,
                camera: self.camera,
                schematic_camera,
                pane_cameras: &pane_cameras,
                include_preferences_overlay: false,
                single_terminal_snapshot: false,
            },
            &terminal_panes,
        );
        if let Err(error) = preparation {
            drop(terminal_panes);
            let damage = self.renderer.render_session_mut().restore_terminal_damage();
            self.terminal_sessions.restore_render_damage(damage);
            return Err(error);
        }
        append_gui_verbose_diagnostic_line(|| {
            format!(
                "native control_meshes window={:?} builds={}",
                self.window.id(),
                self.renderer.control_mesh_build_count()
            )
        });
        Ok(())
    }
}
