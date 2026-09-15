fn inspector_height_for_state(state: &ReviewWorkspaceState) -> f32 {
    let board_text_selected = matches!(
        &state.selection,
        SelectionTarget::AuthoredObject(object_id)
            if state.scene.board_texts.iter().any(|text| &text.object_id == object_id)
    );
    let mut height: f32 = if board_text_selected { 330.0 } else { 150.0 };
    let detail_row_count = inspector_detail_row_count(state);
    if detail_row_count > 0 {
        let detail_bottom = 96.0 + detail_row_count as f32 * key_value_row_height() + 2.0;
        height = height.max(detail_bottom);
    }
    height
}

fn inspector_detail_row_count(state: &ReviewWorkspaceState) -> usize {
    let mut count = 0;
    if state.selected_review_action().is_some() {
        count += 3;
    }
    if state.selected_segment_evidence().is_some() {
        count += 1;
    }
    if state.last_command_status.is_some() {
        count += 1;
    }
    count
}
