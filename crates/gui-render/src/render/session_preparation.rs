//! Shared workspace preparation, derived terminal storage and pane projection.
use super::*;
use crate::{CameraState, PreparedScene, Renderer};
use datum_gui_protocol::{PaneContent, PaneId};

/// Coherent editor camera intent and physical target inputs. Editors supply
/// values before preparation; only the session mutates derived projections.
pub struct WorkspaceView<'a> {
    pub source_revision: Option<render_input::SourceRevision>,
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    pub camera: CameraState,
    pub schematic_camera: Option<CameraState>,
    pub include_preferences_overlay: bool,
    /// Compatibility capture supplies one root snapshot, not an active split-tab input set.
    pub single_terminal_snapshot: bool,
    pub pane_cameras: &'a [(PaneId, CameraState)],
}

impl RenderSession {
    pub(super) fn begin_workspace_preparation(
        &mut self,
        state: &ReviewWorkspaceState,
        view: &WorkspaceView<'_>,
        panes: &[TerminalPaneRenderState<'_>],
    ) -> anyhow::Result<RetainedScene> {
        self.retain_terminal_damage(panes);
        self.composition_changed();
        self.ensure_sources(
            state,
            view.source_revision,
            view.width,
            view.height,
            view.scale,
        )?;
        Ok(self.board.clone().expect("validated board"))
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
        // Lease consumed producer damage before source validation/admission can fail.
        let retained = self
            .render_session
            .begin_workspace_preparation(state, &view, panes)?;
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
                    (!view.single_terminal_snapshot)
                        .then_some(&mut self.render_session.terminal_cache),
                    view.include_preferences_overlay,
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
        self.render_session.install_prepared(
            prepared,
            Preparation::workspace(state, &retained, [view.width, view.height]),
        );
        Ok(())
    }
}
