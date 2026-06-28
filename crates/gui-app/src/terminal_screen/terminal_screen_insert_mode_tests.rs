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
fn insert_mode_shifts_existing_cells_until_reset() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\r\x1b[4hXY\x1b[4lZ");
    assert_eq!(state.lines, vec!["XYZbcdef"]);
}

#[test]
fn split_insert_mode_sequence_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\r\x1b[");
    assert_eq!(state.lines, vec!["abcdef"]);
    screen.apply_bytes(&mut state, b"4hZ");
    assert_eq!(state.lines, vec!["Zabcdef"]);
}

#[test]
fn reset_clears_insert_mode() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\r\x1b[4hX\x1bcYZ");
    assert_eq!(state.lines, vec!["YZ"]);
}

#[test]
fn repeat_preceding_character_honors_insert_mode() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\r\x1b[4hX\x1b[2b");
    assert_eq!(state.lines, vec!["XXXabcdef"]);
}
