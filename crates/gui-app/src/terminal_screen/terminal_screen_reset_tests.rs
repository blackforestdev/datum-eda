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
fn reset_clears_screen_cursor_saved_state_and_repeat_character() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abc\x1b7\nstatus\x1bc\x1b[3bZ\x1b8Y");
    assert_eq!(state.lines, vec!["ZY"]);
}

#[test]
fn reset_clears_scroll_region_and_alternate_screen_state() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"main\x1b[?1049halt\x1bcX");
    assert_eq!(state.lines, vec!["X"]);

    screen.apply_bytes(&mut state, b"\nY\x1b[1;1H\x1b[1;1r\x1bcA\nB");
    assert_eq!(state.lines, vec!["A", "B"]);
}

#[test]
fn reset_restores_autowrap_mode() {
    let mut screen = TerminalScreen::default();
    screen.resize(4);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[?7labcdZ\x1bcabcdZ");
    assert_eq!(state.lines, vec!["abcd", "Z"]);
}
