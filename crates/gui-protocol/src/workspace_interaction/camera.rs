//! Resolve scene authority for shared viewport camera operations and their UI
//! consumers. Menus inspect this result; they do not own a second camera model.

use crate::{BoardReviewSceneV1, PaneContent, PaneId, ReviewWorkspaceState};

/// No fallback from an unresolved, hidden or stale pane to another pane's Board.
pub fn camera_scene_for_pane(
    state: &ReviewWorkspaceState,
    pane: PaneId,
) -> Option<&BoardReviewSceneV1> {
    if state.ui.layout.zoomed.is_some_and(|zoomed| zoomed != pane) {
        return None;
    }
    match state.ui.layout.content_for(pane)? {
        PaneContent::Board => Some(&state.scene),
        PaneContent::Schematic => state.schematic_scene.as_ref(),
        PaneContent::Revision(_) => None,
    }
}
