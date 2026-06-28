use datum_gui_protocol::TerminalLaneState;

use super::terminal_scrollback_copy_text;

#[test]
fn terminal_scrollback_copy_text_joins_rows_and_trims_blank_tail() {
    let mut state = TerminalLaneState {
        lines: vec!["first".to_string(), "second".to_string(), String::new()],
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
    };
    assert_eq!(
        terminal_scrollback_copy_text(&state).as_deref(),
        Some("first\nsecond")
    );

    state.lines = vec![String::new()];
    assert_eq!(terminal_scrollback_copy_text(&state), None);
}
