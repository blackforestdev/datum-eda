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
fn c1_csi_and_osc_sequences_do_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\x9b3DZ");
    assert_eq!(state.lines, vec!["abcZef"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"a\x9d0;datum gui\x9cb");
    assert_eq!(state.lines, vec!["ab"]);
}

#[test]
fn c1_index_next_line_and_reverse_index_match_escape_aliases() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"ab\x84Z");
    assert_eq!(state.lines, vec!["ab", "  Z"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"ab\x85Z");
    assert_eq!(state.lines, vec!["ab", "Z"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"one\ntwo\x9b2;2H\x8dZ");
    assert_eq!(state.lines, vec!["oZe", "two"]);
}
