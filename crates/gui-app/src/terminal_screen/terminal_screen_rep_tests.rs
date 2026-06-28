use datum_gui_protocol::TerminalLaneState;

use super::TerminalScreen;

fn terminal_state() -> TerminalLaneState {
    TerminalLaneState {
        lines: Vec::new(),
        styled_lines: Vec::new(),
        activity_summary: Vec::new(),
        tabs: Vec::new(),
        active_session_id: None,
        rename_session_id: None,
        title: None,
        current_working_directory: None,
        bell_count: 0,
        input: String::new(),
        cursor: 0,
        columns: 80,
        rows: 24,
        screen_cursor_row: 0,
        screen_cursor_col: 0,
        screen_cursor_visible: true,
        screen_cursor_style: None,
        application_cursor_keys: false,
        application_keypad: false,
        focus_event_reporting: false,
        mouse_reporting_mode: None,
        mouse_coordinate_encoding: None,
        scroll_offset: 0,
        status: "running".to_string(),
    }
}

#[test]
fn repeat_preceding_character_uses_printable_cursor_semantics() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"ab\x1b[4bZ");
    assert_eq!(state.lines, vec!["abbbbbZ"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[3bZ");
    assert_eq!(state.lines, vec!["Z"]);
}

#[test]
fn repeat_preceding_character_wraps_at_terminal_columns() {
    let mut screen = TerminalScreen::default();
    screen.resize(4);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"ab\x1b[3bZ");
    assert_eq!(state.lines, vec!["abbb", "bZ"]);
}
