//! Production immutable TerminalCore snapshot-to-scene composition.

use super::*;

impl Runtime {
    pub(super) fn build_terminal_prepared_scene(&mut self) -> Result<PreparedScene> {
        let schematic_camera = self.schematic_camera_for_render();
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
        let retained = self
            .retained_scene
            .as_ref()
            .context("retained scene should exist before prepared scene rebuild")?;
        let mut prepared = self.renderer.prepare_workspace_with_terminal_renderer(
            self.session.workspace(),
            self.config.width,
            self.config.height,
            self.scale_factor,
            self.camera,
            retained,
            &terminal_panes,
            Some(&mut self.terminal_render_cache),
            false,
        );
        if let Some(camera) = schematic_camera {
            prepared.set_schematic_camera(camera);
        }
        self.apply_prepared_grid_lod(&mut prepared);
        append_gui_verbose_diagnostic_line(|| {
            format!(
                "native control_meshes window={:?} builds={}",
                self.window.id(),
                self.renderer.control_mesh_build_count()
            )
        });
        self.presented_hits.mark_pending();
        Ok(prepared)
    }
}
