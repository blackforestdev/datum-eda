//! Existing lighter hover treatment over a captured local conflict candidate.
use super::*;

pub(crate) fn capture(
    state: &ReviewWorkspaceState,
    retained: &RetainedScene,
) -> Option<(
    Option<datum_gui_protocol::RectNm>,
    Option<datum_gui_protocol::RectNm>,
)> {
    let menu = state.ui.marking_menu.as_ref()?.selection.as_ref()?;
    if !menu.open {
        return None;
    }
    let empty = Some((None, None));
    let scene = if menu.schematic {
        state.schematic_scene.as_ref()?
    } else {
        &state.scene
    };
    if menu.project.to_string() != scene.project_uuid
        || menu.revision != scene.source_revision
        || menu.source_scene != scene.scene_id
        || menu.filters != state.ui.filters
    {
        return empty;
    }
    let Some((id, keys)) = menu.preview_candidate() else {
        return empty;
    };
    // The committed visual remains authoritative for an already selected member.
    if matches!(&state.selection, SelectionTarget::Native(v) if !v.stale && v.members.contains(id))
    {
        return empty;
    }
    let bounds = keys
        .iter()
        .filter_map(|key| {
            if menu.schematic {
                schematic_hover_bounds(scene, key)
            } else {
                board_hover_bounds(retained, key)
            }
        })
        .reduce(|a, b| datum_gui_protocol::RectNm {
            min_x: a.min_x.min(b.min_x),
            min_y: a.min_y.min(b.min_y),
            max_x: a.max_x.max(b.max_x),
            max_y: a.max_y.max(b.max_y),
        });
    if menu.schematic {
        Some((None, bounds))
    } else {
        Some((bounds, None))
    }
}
