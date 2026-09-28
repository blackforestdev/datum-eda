//! Shared workspace preparation, derived terminal storage and pane projection.
use super::*;
use crate::{CameraState, PreparedScene, Renderer};
use datum_gui_protocol::{PaneContent, PaneId};

/// Coherent editor camera intent and physical target inputs. Editors supply
/// values before preparation; only the session mutates derived projections.
pub struct WorkspaceView<'a> {
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    pub camera: CameraState,
    pub schematic_camera: Option<CameraState>,
    pub pane_cameras: &'a [(PaneId, CameraState)],
}

impl RenderSession {
    fn begin_workspace_preparation(
        &mut self,
        panes: &[TerminalPaneRenderState<'_>],
    ) -> anyhow::Result<RetainedScene> {
        self.retain_terminal_damage(panes);
        self.board
            .clone()
            .ok_or_else(|| anyhow::anyhow!("workspace preparation requires retained content"))
    }

    pub fn retire_pane_projection(&mut self, pane: PaneId, content: PaneContent) {
        self.grid_lod.retarget(pane, content);
    }
    pub fn retain_pane_projections(&mut self, live: &[PaneId]) {
        self.grid_lod.retain_live(live);
    }
    pub fn reset_pane_projections(&mut self) {
        self.grid_lod.reset();
    }
}

impl Renderer {
    /// One preparation authority for native drawing, input projection and capture.
    /// Failed construction keeps the producer damage lease for retry/rollback.
    pub fn prepare_session_workspace(
        &mut self,
        state: &ReviewWorkspaceState,
        view: WorkspaceView<'_>,
        panes: &[TerminalPaneRenderState<'_>],
    ) -> anyhow::Result<()> {
        // Lease consumed producer damage before any fallible prerequisite.
        let retained = self.render_session.begin_workspace_preparation(panes)?;
        let _host = self.resource_host.enter();
        let prepared = crate::text_metrics::measurement_owner::with_owner(
            &mut self.measurement_fonts,
            || {
                PreparedScene::from_workspace_with_controls(
                    state,
                    view.width,
                    view.height,
                    view.scale,
                    view.camera,
                    &retained,
                    panes,
                    Some(&mut self.render_session.terminal_cache),
                    false,
                    &mut self.control_meshes,
                )
            },
        )?;
        let mut prepared = prepared;
        if let Some(camera) = view.schematic_camera {
            prepared.set_schematic_camera(camera);
        }
        for &(pane, camera) in view.pane_cameras {
            prepared.set_surface_camera(pane, camera);
        }
        self.render_session
            .grid_lod
            .apply_to_prepared(&mut prepared);
        self.render_session.install_prepared(prepared);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_content_keeps_consumed_terminal_damage_available_for_rollback() {
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let panes = [TerminalPaneRenderState {
            session_id: "producer".into(),
            focused: true,
            lane: &state.ui.terminal,
            snapshot: crate::terminal_core_render::test_snapshot(b"damage before failure"),
            damage: vec![datum_terminal_core::Damage::Full],
        }];
        let mut session = RenderSession::default();
        assert!(session.begin_workspace_preparation(&panes).is_err());
        assert_eq!(
            session.restore_terminal_damage(),
            vec![(
                "producer".to_string(),
                vec![datum_terminal_core::Damage::Full]
            )]
        );
        assert!(session.has_pending_frame());
    }
}
