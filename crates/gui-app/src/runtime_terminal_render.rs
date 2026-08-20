//! Production immutable TerminalCore snapshot-to-scene composition.

use super::*;

impl Runtime {
    pub(super) fn build_terminal_prepared_scene(&mut self) -> Result<PreparedScene> {
        let schematic_camera = self.schematic_camera_for_render();
        let (terminal_snapshot, terminal_damage) = self
            .terminal_sessions
            .take_active_render_state()
            .context("snapshot active TerminalCore for rendering")?;
        let retained = self
            .retained_scene
            .as_ref()
            .context("retained scene should exist before prepared scene rebuild")?;
        let mut prepared = PreparedScene::from_workspace_with_terminal_renderer(
            self.session.workspace(),
            self.config.width,
            self.config.height,
            self.scale_factor,
            self.camera,
            retained,
            Some(&terminal_snapshot),
            &terminal_damage,
            Some(&mut self.terminal_render_cache),
        );
        if let Some(camera) = schematic_camera {
            prepared.set_schematic_camera(camera);
        }
        self.apply_prepared_grid_lod(&mut prepared);
        Ok(prepared)
    }
}
