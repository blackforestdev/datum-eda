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
fn device_status_report_replies_ready_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    let responses = screen.apply_bytes_with_responses(&mut state, b"ok\x1b[5n");
    assert_eq!(state.lines, vec!["ok"]);
    assert_eq!(responses, vec![b"\x1b[0n".to_vec()]);
}

#[test]
fn cursor_position_report_uses_one_based_cursor_position() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    let responses = screen.apply_bytes_with_responses(&mut state, b"one\ntwo\x1b[2G\x1b[6n");
    assert_eq!(state.lines, vec!["one", "two"]);
    assert_eq!(responses, vec![b"\x1b[2;2R".to_vec()]);
}

#[test]
fn private_cursor_position_report_uses_dec_private_prefix() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    let responses = screen.apply_bytes_with_responses(&mut state, b"one\ntwo\x1b[3G\x1b[?6n");
    assert_eq!(state.lines, vec!["one", "two"]);
    assert_eq!(responses, vec![b"\x1b[?2;3R".to_vec()]);
}

#[test]
fn split_cursor_position_report_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    let responses = screen.apply_bytes_with_responses(&mut state, b"abc\x1b[");
    assert!(responses.is_empty());
    assert_eq!(state.lines, vec!["abc"]);
    let responses = screen.apply_bytes_with_responses(&mut state, b"6nZ");
    assert_eq!(state.lines, vec!["abcZ"]);
    assert_eq!(responses, vec![b"\x1b[1;4R".to_vec()]);
}

#[test]
fn split_private_cursor_position_report_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    let responses = screen.apply_bytes_with_responses(&mut state, b"abc\x1b[?");
    assert!(responses.is_empty());
    assert_eq!(state.lines, vec!["abc"]);
    let responses = screen.apply_bytes_with_responses(&mut state, b"6nZ");
    assert_eq!(state.lines, vec!["abcZ"]);
    assert_eq!(responses, vec![b"\x1b[?1;4R".to_vec()]);
}

#[test]
fn primary_device_attributes_reply_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    let responses = screen.apply_bytes_with_responses(&mut state, b"pre\x1b[cpost");
    assert_eq!(state.lines, vec!["prepost"]);
    assert_eq!(responses, vec![b"\x1b[?1;2c".to_vec()]);
}

#[test]
fn decid_replies_like_primary_device_attributes_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    let responses = screen.apply_bytes_with_responses(&mut state, b"pre\x1bZpost");
    assert_eq!(state.lines, vec!["prepost"]);
    assert_eq!(responses, vec![b"\x1b[?1;2c".to_vec()]);
}

#[test]
fn secondary_device_attributes_reply_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    let responses = screen.apply_bytes_with_responses(&mut state, b"\x1b[>c");
    assert_eq!(state.lines, vec![""]);
    assert_eq!(responses, vec![b"\x1b[>0;0;0c".to_vec()]);
}

#[test]
fn split_device_attributes_query_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    let responses = screen.apply_bytes_with_responses(&mut state, b"\x1b[>");
    assert!(responses.is_empty());
    assert_eq!(state.lines, vec![""]);
    let responses = screen.apply_bytes_with_responses(&mut state, b"cX");
    assert_eq!(state.lines, vec!["X"]);
    assert_eq!(responses, vec![b"\x1b[>0;0;0c".to_vec()]);
}

#[test]
fn split_decid_query_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    let responses = screen.apply_bytes_with_responses(&mut state, b"\x1b");
    assert!(responses.is_empty());
    assert_eq!(state.lines, vec![""]);
    let responses = screen.apply_bytes_with_responses(&mut state, b"ZX");
    assert_eq!(state.lines, vec!["X"]);
    assert_eq!(responses, vec![b"\x1b[?1;2c".to_vec()]);
}

#[test]
fn terminal_size_report_uses_protocol_rows_and_columns() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    state.columns = 132;
    state.rows = 43;
    let responses = screen.apply_bytes_with_responses(&mut state, b"\x1b[18t\x1b[19t");
    assert_eq!(state.lines, vec![""]);
    assert_eq!(
        responses,
        vec![b"\x1b[8;43;132t".to_vec(), b"\x1b[9;43;132t".to_vec()]
    );
}

#[test]
fn terminal_pixel_reports_use_protocol_grid_and_stable_cell_size() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    state.columns = 100;
    state.rows = 30;
    let responses = screen.apply_bytes_with_responses(&mut state, b"\x1b[14t\x1b[15t\x1b[16t");
    assert_eq!(state.lines, vec![""]);
    assert_eq!(
        responses,
        vec![
            b"\x1b[4;480;800t".to_vec(),
            b"\x1b[5;480;800t".to_vec(),
            b"\x1b[6;16;8t".to_vec()
        ]
    );
}

#[test]
fn window_state_report_replies_normal_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    let responses = screen.apply_bytes_with_responses(&mut state, b"pre\x1b[11t\x1b[13tpost");
    assert_eq!(state.lines, vec!["prepost"]);
    assert_eq!(
        responses,
        vec![b"\x1b[1t".to_vec(), b"\x1b[3;0;0t".to_vec()]
    );
}

#[test]
fn window_title_reports_use_osc_title_state_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    state.title = Some("datum layout".to_string());
    let responses = screen.apply_bytes_with_responses(&mut state, b"pre\x1b[20t\x1b[21tpost");
    assert_eq!(state.lines, vec!["prepost"]);
    assert_eq!(
        responses,
        vec![
            b"\x1b]Ldatum layout\x1b\\".to_vec(),
            b"\x1b]ldatum layout\x1b\\".to_vec()
        ]
    );
}

#[test]
fn window_title_reports_strip_control_bytes_from_response_payload() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    state.title = Some("safe\x1b]2;bad\x07title".to_string());
    let responses = screen.apply_bytes_with_responses(&mut state, b"\x1b[21t");
    assert_eq!(state.lines, vec![""]);
    assert_eq!(responses, vec![b"\x1b]lsafe]2;badtitle\x1b\\".to_vec()]);
}

#[test]
fn window_title_save_and_restore_update_protocol_title_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    state.title = Some("datum shell".to_string());
    let responses = screen.apply_bytes_with_responses(&mut state, b"pre\x1b[22tpost");
    assert_eq!(state.lines, vec!["prepost"]);
    assert!(responses.is_empty());

    state.title = Some("full screen app".to_string());
    let responses = screen.apply_bytes_with_responses(&mut state, b"\x1b[23t");
    assert_eq!(state.lines, vec!["prepost"]);
    assert!(responses.is_empty());
    assert_eq!(state.title.as_deref(), Some("datum shell"));
}

#[test]
fn split_window_title_save_restore_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    state.title = Some("outer".to_string());
    let responses = screen.apply_bytes_with_responses(&mut state, b"pre\x1b[");
    assert!(responses.is_empty());
    assert_eq!(state.lines, vec!["pre"]);
    let responses = screen.apply_bytes_with_responses(&mut state, b"22tpost");
    assert!(responses.is_empty());
    assert_eq!(state.lines, vec!["prepost"]);

    state.title = Some("inner".to_string());
    let responses = screen.apply_bytes_with_responses(&mut state, b"\x1b[");
    assert!(responses.is_empty());
    assert_eq!(state.lines, vec!["prepost"]);
    let responses = screen.apply_bytes_with_responses(&mut state, b"23tdone");
    assert!(responses.is_empty());
    assert_eq!(state.lines, vec!["prepostdone"]);
    assert_eq!(state.title.as_deref(), Some("outer"));
}

#[test]
fn split_terminal_size_report_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    state.columns = 100;
    state.rows = 30;
    let responses = screen.apply_bytes_with_responses(&mut state, b"pre\x1b[");
    assert!(responses.is_empty());
    assert_eq!(state.lines, vec!["pre"]);
    let responses = screen.apply_bytes_with_responses(&mut state, b"18tpost");
    assert_eq!(state.lines, vec!["prepost"]);
    assert_eq!(responses, vec![b"\x1b[8;30;100t".to_vec()]);
}

#[test]
fn split_terminal_pixel_report_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    state.columns = 120;
    state.rows = 40;
    let responses = screen.apply_bytes_with_responses(&mut state, b"pre\x1b[");
    assert!(responses.is_empty());
    assert_eq!(state.lines, vec!["pre"]);
    let responses = screen.apply_bytes_with_responses(&mut state, b"14tpost");
    assert_eq!(state.lines, vec!["prepost"]);
    assert_eq!(responses, vec![b"\x1b[4;640;960t".to_vec()]);
}

#[test]
fn split_window_state_report_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    let responses = screen.apply_bytes_with_responses(&mut state, b"pre\x1b[");
    assert!(responses.is_empty());
    assert_eq!(state.lines, vec!["pre"]);
    let responses = screen.apply_bytes_with_responses(&mut state, b"11tpost");
    assert_eq!(state.lines, vec!["prepost"]);
    assert_eq!(responses, vec![b"\x1b[1t".to_vec()]);
}
