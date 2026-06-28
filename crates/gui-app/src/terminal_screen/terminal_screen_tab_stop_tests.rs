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
fn hts_sets_custom_tab_stop_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[6G\x1bH\rx\ty");
    assert_eq!(state.lines, vec!["x    y"]);
}

#[test]
fn split_hts_sequence_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[6G\x1b");
    assert_eq!(state.lines, vec![""]);
    screen.apply_bytes(&mut state, b"H\rx\ty");
    assert_eq!(state.lines, vec!["x    y"]);
}

#[test]
fn tab_clear_current_removes_custom_stop() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[6G\x1bH\x1b[g\rx\ty");
    assert_eq!(state.lines, vec!["x       y"]);
}

#[test]
fn tab_clear_current_can_remove_default_stop() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[9G\x1b[g\rx\ty");
    assert_eq!(state.lines, vec!["x               y"]);
}

#[test]
fn tab_clear_all_makes_tabs_noop_until_custom_stop() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[3gA\tB\x1b[6G\x1bH\rx\ty");
    assert_eq!(state.lines, vec!["x    y"]);
}

#[test]
fn split_tab_clear_sequences_do_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[");
    assert_eq!(state.lines, vec![""]);
    screen.apply_bytes(&mut state, b"3gA\tB");
    assert_eq!(state.lines, vec!["AB"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[9G\x1b[");
    assert_eq!(state.lines, vec![""]);
    screen.apply_bytes(&mut state, b"g\rx\ty");
    assert_eq!(state.lines, vec!["x               y"]);
}

#[test]
fn terminal_reset_restores_default_tab_stops() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[3gA\tB\x1bcx\ty");
    assert_eq!(state.lines, vec!["x       y"]);
}
