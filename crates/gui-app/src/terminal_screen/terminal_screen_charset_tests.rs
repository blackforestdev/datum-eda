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
fn charset_designation_sequences_do_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"a\x1b(Bb\x1b)0c");
    assert_eq!(state.lines, vec!["abc"]);
}

#[test]
fn split_charset_designation_sequence_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"a\x1b(");
    assert_eq!(state.lines, vec!["a"]);
    screen.apply_bytes(&mut state, b"Bb");
    assert_eq!(state.lines, vec!["ab"]);
}

#[test]
fn dec_screen_alignment_test_fills_visible_grid() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    state.columns = 6;
    state.rows = 3;

    screen.apply_bytes(&mut state, b"prompt\x1b#8");

    assert_eq!(state.lines, vec!["EEEEEE", "EEEEEE", "EEEEEE"]);
    assert_eq!(state.styled_lines.len(), 3);
    assert!(
        state
            .styled_lines
            .iter()
            .all(|line| line.text == "EEEEEE" && line.spans.is_empty()),
        "DECALN should replace the visible screen with unstyled E cells"
    );
}

#[test]
fn unsupported_escape_intermediate_still_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();

    screen.apply_bytes(&mut state, b"a\x1b#7b");

    assert_eq!(state.lines, vec!["ab"]);
}
