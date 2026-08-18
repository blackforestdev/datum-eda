use datum_gui_protocol::TerminalLaneState;

pub(crate) fn terminal_scrollback_copy_text(state: &TerminalLaneState) -> Option<String> {
    let mut lines = state.grid_lines();
    while matches!(lines.last(), Some(line) if line.is_empty()) {
        lines = &lines[..lines.len().saturating_sub(1)];
    }
    if lines.is_empty() {
        return None;
    }
    Some(lines.join("\n"))
}

pub(crate) fn terminal_clipboard_copy_text(state: &TerminalLaneState) -> Option<String> {
    let Some(((start_row, _), (end_row, _))) = state.text_selection_ordered() else {
        return terminal_scrollback_copy_text(state);
    };
    let lines = state.grid_lines();
    if start_row >= lines.len() {
        return terminal_scrollback_copy_text(state);
    }

    let mut selected = Vec::new();
    for (row, line) in lines
        .iter()
        .enumerate()
        .take(end_row.saturating_add(1))
        .skip(start_row)
    {
        let line_columns = line.chars().count();
        let text = state
            .text_selection_span(row, line_columns)
            .map(|(first, last)| {
                line.chars()
                    .skip(first)
                    .take(last - first)
                    .collect::<String>()
            })
            .unwrap_or_default();
        selected.push(text.trim_end_matches(' ').to_string());
    }
    let text = selected.join("\n");
    (!text.is_empty())
        .then_some(text)
        .or_else(|| terminal_scrollback_copy_text(state))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_terminal_cells_replace_copy_all_in_forward_and_reverse_order() {
        let mut state = TerminalLaneState::default();
        *state.pty_grid_mut().lines = vec![
            "zero".to_string(),
            "alpha beta".to_string(),
            "gamma delta".to_string(),
        ];
        for (anchor, focus) in [((1, 6), (2, 4)), ((2, 4), (1, 6))] {
            state.set_text_selection(anchor, focus);
            assert_eq!(
                terminal_clipboard_copy_text(&state).as_deref(),
                Some("beta\ngamma")
            );
        }
    }

    #[test]
    fn no_selection_preserves_the_existing_scrollback_copy_fallback() {
        let mut state = TerminalLaneState::default();
        *state.pty_grid_mut().lines = vec!["first".to_string(), "second".to_string()];
        assert_eq!(
            terminal_clipboard_copy_text(&state).as_deref(),
            Some("first\nsecond")
        );
    }
}
