//! Shared authored/proposed/layer visibility policy for scene construction.
use super::*;

/// Borrowed executed command incidence; retained allocations stay resident.
#[derive(Clone, Copy)]
pub(super) struct VisibleDrawCommands<'a> {
    pub(super) commands: &'a [RetainedDrawCommand],
    pub(super) filters: Option<&'a datum_gui_protocol::WorkspaceFilterState>,
}

impl<'a> VisibleDrawCommands<'a> {
    pub(super) fn iter(self) -> impl Iterator<Item = &'a RetainedDrawCommand> + Clone {
        let commands = if self.filters.is_some_and(|filters| !filters.show_authored) {
            &[][..]
        } else {
            self.commands
        };
        commands.iter().filter(move |command| {
            self.filters.is_none_or(|filters| {
                let layer = match command {
                    RetainedDrawCommand::Quads { layer_id, .. }
                    | RetainedDrawCommand::Glyphs { layer_id, .. }
                    | RetainedDrawCommand::Strokes { layer_id, .. } => layer_id.as_deref(),
                };
                layer.is_none_or(|id| filters.layer_visibility.get(id).copied().unwrap_or(true))
            })
        })
    }
}

pub(super) fn authored_visible(state: &ReviewWorkspaceState) -> bool {
    state.ui.filters.show_authored
}

pub(super) fn proposed_visible(state: &ReviewWorkspaceState) -> bool {
    state.ui.filters.show_proposed
}

pub(super) fn unrouted_visible(state: &ReviewWorkspaceState) -> bool {
    state.ui.filters.show_unrouted
}

pub(super) fn layer_visible(state: &ReviewWorkspaceState, layer_id: &str) -> bool {
    state
        .ui
        .filters
        .layer_visibility
        .get(layer_id)
        .copied()
        .unwrap_or(true)
}

pub(super) fn via_visible(
    state: &ReviewWorkspaceState,
    start_layer_id: &str,
    end_layer_id: &str,
) -> bool {
    layer_visible(state, start_layer_id) || layer_visible(state, end_layer_id)
}

pub(super) fn pad_copper_layer_ids(pad: &datum_gui_protocol::PadPrimitive) -> Vec<&str> {
    if pad.copper_layer_ids.is_empty() {
        vec![pad.layer_id.as_str()]
    } else {
        pad.copper_layer_ids.iter().map(String::as_str).collect()
    }
}

pub(super) fn pad_visible_on_any_copper_layer(
    state: &ReviewWorkspaceState,
    pad: &datum_gui_protocol::PadPrimitive,
) -> bool {
    if pad.copper_layer_ids.is_empty() {
        layer_visible(state, &pad.layer_id)
    } else {
        pad.copper_layer_ids
            .iter()
            .any(|layer_id| layer_visible(state, layer_id))
    }
}

pub(super) fn immediate_selection_workspace(state: &ReviewWorkspaceState) -> bool {
    matches!(state.selection, SelectionTarget::Native(_))
        || state
            .backing
            .as_ref()
            .is_some_and(|b| b.request.board_file.is_none())
}

pub(super) fn dim_unrelated_active(state: &ReviewWorkspaceState) -> bool {
    // Native relationship presentation belongs after the authored world pass.
    if immediate_selection_workspace(state) || !state.ui.filters.dim_unrelated {
        return false;
    }
    has_review_focus(state) || !matches!(state.selection, SelectionTarget::None)
}

pub(super) fn is_hovered(state: &ReviewWorkspaceState, object_id: &str) -> bool {
    if !matches!(state.selection, SelectionTarget::None) {
        return false;
    }
    state.ui.hovered_object.as_ref().is_some_and(|hover| {
        hover.surface == datum_gui_protocol::PaneContent::Board && hover.object_id == object_id
    })
}
