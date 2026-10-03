#[path = "board_hit_construction.rs"]
mod board_hit_construction;
#[path = "hit_construction.rs"]
mod hit_construction;
#[path = "prepared_scene_access.rs"]
mod prepared_scene_access;
pub use prepared_scene_access::{PreparedScene, SelectedPaintRange};
#[path = "scene_console.rs"]
mod scene_console;
#[path = "schematic_retained.rs"]
mod schematic_retained;
#[path = "shell_identity.rs"]
mod shell_identity;

// Per-pane coordinate + hit resolution stays a child module with private scene access.
#[path = "coordinate_hit.rs"]
mod coordinate_hit;
pub use coordinate_hit::resolve_pane_hover;

// S4 interaction overlays and status stay child modules with private geometry access.
mod hit_clipping;
#[path = "interaction_overlay.rs"]
mod interaction_overlay;
#[path = "status_bar.rs"]
mod status_bar;

#[path = "frame_preparation.rs"]
mod frame_preparation;

fn dock_height_for_state(state: &ReviewWorkspaceState) -> Option<u32> {
    if state.ui.active_dock_tab.is_some() {
        Some(state.ui.effective_dock_height_px())
    } else {
        None
    }
}

#[path = "shell_chrome.rs"]
mod shell_chrome;
use shell_chrome::render_phase1_shell_chrome;

// Workspace pane-chrome rendering (viewport panes, per-pane headers, and
// non-live placeholders) lives in the `pane_chrome` submodule; entry point
// `render_viewport_panes` is imported at the crate root.

#[path = "scene_projection.rs"]
mod scene_projection;
use scene_projection::render_scene;

#[path = "scene_visibility.rs"]
mod scene_visibility;
use scene_visibility::*;

#[path = "scene_relationships.rs"]
mod scene_relationships;
use scene_relationships::*;

#[path = "selection_relation.rs"]
mod selection_relation;
use selection_relation::{component_is_selection_active, component_is_selection_related};

fn component_object_id_for_uuid<'a>(
    scene: &'a BoardReviewSceneV1,
    component_uuid: &str,
) -> Option<&'a str> {
    scene.components.iter().find_map(|component| {
        (component.component_uuid == component_uuid).then_some(component.object_id.as_str())
    })
}
