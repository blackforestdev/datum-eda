//! Retain per-pane view intent across same-document scene publication.
use super::*;
use datum_gui_protocol::{ReviewWorkspaceState, resolved_camera_scene_for_pane};
use datum_gui_viewport::{CameraConfig, CameraEngine, CameraViewport};

impl Runtime {
    pub(super) fn preserve_cameras_for_refresh(&mut self, next: &ReviewWorkspaceState) {
        let updates = refresh_cameras(
            self.workspace(),
            next,
            &self.current_layout(),
            self.camera,
            &self.pane_cameras,
        );
        for (pane, content, active, camera) in updates {
            if active {
                self.camera = camera;
            }
            self.pane_cameras.inherit(pane, content, camera);
        }
    }
}

fn refresh_cameras(
    current: &ReviewWorkspaceState,
    next: &ReviewWorkspaceState,
    shell: &datum_gui_render::ShellLayout,
    active: CameraState,
    warm: &PaneCameras,
) -> Vec<(datum_gui_protocol::PaneId, PaneContent, bool, CameraState)> {
    let visible = shell.viewport_panes(&current.ui.layout);
    let mut layout = current.ui.layout.clone();
    layout.zoomed = None;
    let all = shell.viewport_panes(&layout);
    let active_board = visible.scene_leaf_id().or_else(|| all.scene_leaf_id());
    all.panes
        .iter()
        .filter_map(|pane| {
            // Hidden siblings retain their unmaximized field; the visible leaf
            // uses its current field. Restoring a split must not reveal a refit.
            let pane = visible
                .panes
                .iter()
                .find(|p| p.id == pane.id)
                .unwrap_or(pane);
            let old = resolved_camera_scene_for_pane(current, pane.id)?;
            let new = resolved_camera_scene_for_pane(next, pane.id)?;
            if current.ui.layout.content_for(pane.id) != next.ui.layout.content_for(pane.id)
                || old.project_uuid != new.project_uuid
                || old.board_uuid != new.board_uuid
                || old.schematic_document != new.schematic_document
            {
                return None;
            }
            let is_active = pane.content == PaneContent::Board && active_board == Some(pane.id);
            let mut camera = if is_active {
                active
            } else {
                // Capture the old displayed fallback before replacing bounds. A
                // pane need not have received a navigation gesture to own its view.
                warm.camera(pane.id, pane.content)
                    .unwrap_or_else(|| CameraState::fit_to_bounds(&old.bounds))
            };
            let rect = pane.rect.scene;
            // The rendered world/hit field is inset 10px on all four sides.
            let viewport = CameraViewport {
                x: rect.x + 10.0,
                y: rect.y + 10.0,
                width: (rect.width - 20.0).max(1.0),
                height: (rect.height - 20.0).max(1.0),
            };
            CameraEngine::preserve_view_on_refresh(
                &mut camera,
                CameraConfig::default(),
                viewport,
                &old.bounds,
                &new.bounds,
            );
            Some((pane.id, pane.content, is_active, camera))
        })
        .collect()
}

#[cfg(test)]
#[path = "runtime_camera_refresh_tests.rs"]
mod tests;
