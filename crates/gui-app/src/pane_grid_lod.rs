//! Editor camera intent adapter. Derived projection and grid history live in RenderSession.
use super::{CameraState, PaneCameras, Runtime};
use datum_gui_protocol::{PaneContent, PaneId};

impl Runtime {
    pub(super) fn workspace(&self) -> &datum_gui_protocol::ReviewWorkspaceState {
        self.session.workspace()
    }

    pub(super) fn render_camera_inputs(&self) -> Vec<(PaneId, CameraState)> {
        let panes = self
            .current_layout()
            .viewport_panes(&self.workspace().ui.layout);
        panes
            .panes
            .iter()
            .filter_map(|pane| {
                camera_for_render(
                    panes.scene_leaf_id(),
                    self.camera,
                    &self.pane_cameras,
                    pane.id,
                    pane.content,
                )
                .map(|camera| (pane.id, camera))
            })
            .collect()
    }
}

/// Resolve the single authoritative camera for a rendered pane. The active
/// Board camera lives in `Runtime::camera`; only inactive Board panes and all
/// other surfaces read their independent camera from the warm pane store.
fn camera_for_render(
    active_board: Option<PaneId>,
    active_camera: CameraState,
    warm: &PaneCameras,
    pane: PaneId,
    content: PaneContent,
) -> Option<CameraState> {
    if content == PaneContent::Board && active_board == Some(pane) {
        Some(active_camera)
    } else {
        warm.camera(pane, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn active_board_render_uses_live_zoom_instead_of_stale_warm_camera() {
        let pane = PaneId(1);
        let stale = CameraState {
            center_x_nm: 10.0,
            center_y_nm: 20.0,
            zoom: 1.0,
        };
        let zoomed = CameraState { zoom: 2.4, ..stale };
        let warm = PaneCameras::new(pane, PaneContent::Board, stale);

        assert_eq!(
            camera_for_render(Some(pane), zoomed, &warm, pane, PaneContent::Board),
            Some(zoomed)
        );
    }

    #[test]
    fn inactive_duplicate_board_keeps_its_independent_warm_camera() {
        let active = PaneId(1);
        let inactive = PaneId(2);
        let active_camera = CameraState {
            center_x_nm: 10.0,
            center_y_nm: 20.0,
            zoom: 2.4,
        };
        let inactive_camera = CameraState {
            center_x_nm: 30.0,
            center_y_nm: 40.0,
            zoom: 0.75,
        };
        let mut warm = PaneCameras::new(
            active,
            PaneContent::Board,
            CameraState {
                zoom: 1.0,
                ..active_camera
            },
        );
        warm.inherit(inactive, PaneContent::Board, inactive_camera);

        assert_eq!(
            camera_for_render(
                Some(active),
                active_camera,
                &warm,
                inactive,
                PaneContent::Board,
            ),
            Some(inactive_camera)
        );
    }
}
