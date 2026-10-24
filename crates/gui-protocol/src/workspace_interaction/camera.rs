//! Resolve scene authority for shared viewport camera operations and their UI
//! consumers. Menus inspect this result; they do not own a second camera model.

use crate::{BoardReviewSceneV1, PaneContent, PaneId, ReviewWorkspaceState};

/// No fallback from an unresolved, hidden or stale pane to another pane's Board.
pub fn camera_scene_for_pane(
    state: &ReviewWorkspaceState,
    pane: PaneId,
) -> Option<&BoardReviewSceneV1> {
    if state.project_unopened {
        return None;
    }
    if state.ui.layout.zoomed.is_some_and(|zoomed| zoomed != pane) {
        return None;
    }
    match state.ui.layout.content_for(pane)? {
        PaneContent::Board => Some(&state.scene),
        PaneContent::Schematic => {
            if state.schematic_documents.project.is_some() {
                state
                    .schematic_documents
                    .scene_for_pane_with_primary(pane, state.schematic_scene.as_ref())
            } else {
                state.schematic_scene.as_ref()
            }
        }
        PaneContent::Library | PaneContent::Revision(_) => None,
    }
}
