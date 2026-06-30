pub(super) fn retained_selection_cache_key(
    workspace: &datum_gui_protocol::ReviewWorkspaceState,
    selection: &datum_gui_protocol::SelectionTarget,
) -> String {
    match selection {
        datum_gui_protocol::SelectionTarget::None => "none".to_string(),
        datum_gui_protocol::SelectionTarget::ReviewAction(id) => format!("review:{id}"),
        datum_gui_protocol::SelectionTarget::CheckFinding(id) => format!("finding:{id}"),
        datum_gui_protocol::SelectionTarget::AuthoredObject(id) => {
            let lightweight = workspace
                .scene
                .board_texts
                .iter()
                .any(|text| &text.object_id == id)
                || workspace
                    .scene
                    .outline
                    .iter()
                    .any(|outline| &outline.object_id == id)
                || workspace
                    .scene
                    .board_graphics
                    .iter()
                    .any(|graphic| &graphic.object_id == id);
            if lightweight && !workspace.ui.filters.dim_unrelated {
                "none".to_string()
            } else if lightweight {
                "lightweight-authored".to_string()
            } else {
                format!("object:{id}")
            }
        }
    }
}
