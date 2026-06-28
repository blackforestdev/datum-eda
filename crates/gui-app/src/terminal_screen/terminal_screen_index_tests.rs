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
fn index_moves_down_without_resetting_column() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abc\x1bDZ");
    assert_eq!(state.lines, vec!["abc", "   Z"]);
}

#[test]
fn next_line_moves_down_and_resets_column() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abc\x1bEZ");
    assert_eq!(state.lines, vec!["abc", "Z"]);
}

#[test]
fn vertical_tab_and_form_feed_move_down_without_resetting_column() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abc\x0bZ\x0cY");
    assert_eq!(state.lines, vec!["abc", "   Z", "    Y"]);
}

#[test]
fn index_scrolls_only_scroll_region_at_bottom_margin() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\nmid\nbot\x1b[2;3r\x1b[3;2H\x1bDZ");
    assert_eq!(state.lines, vec!["top", "bot", " Z"]);
}
