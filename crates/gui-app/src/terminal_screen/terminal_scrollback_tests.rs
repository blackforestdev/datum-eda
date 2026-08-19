use datum_gui_protocol::TerminalLaneState;

use super::terminal_scrollback_copy_text;

#[test]
fn terminal_scrollback_copy_text_joins_rows_and_trims_blank_tail() {
    let mut state = TerminalLaneState::default();
    // Simulated PTY-derived rows (screen-authority gate; T0-C01).
    *state.pty_grid_mut().lines = vec!["first".to_string(), "second".to_string(), String::new()];
    assert_eq!(
        terminal_scrollback_copy_text(&state).as_deref(),
        Some("first\nsecond")
    );

    *state.pty_grid_mut().lines = vec![String::new()];
    assert_eq!(terminal_scrollback_copy_text(&state), None);
}
