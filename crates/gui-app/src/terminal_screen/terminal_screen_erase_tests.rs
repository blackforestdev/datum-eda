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
fn erase_in_line_modes_clear_whole_line_or_prefix() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\x1b[3D\x1b[1KZ");
    assert_eq!(state.lines, vec!["   Zef"]);
    screen.apply_bytes(&mut state, b"\r\x1b[2KXY");
    assert_eq!(state.lines, vec!["XY"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\x1b[3D\x1b[2KZ");
    assert_eq!(state.lines, vec!["   Z"]);
}

#[test]
fn known_width_erase_in_line_materializes_visible_blank_cells() {
    let mut screen = TerminalScreen::default();
    screen.resize(6);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abc\r\x1b[5G\x1b[KZ");
    assert_eq!(state.lines, vec!["abc Z "]);

    let mut screen = TerminalScreen::default();
    screen.resize(6);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\x1b[5G\x1b[1KZ");
    assert_eq!(state.lines, vec!["    Zf"]);

    let mut screen = TerminalScreen::default();
    screen.resize(6);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\r\x1b[3G\x1b[2KZ");
    assert_eq!(state.lines, vec!["  Z   "]);
}

#[test]
fn known_geometry_erase_display_materializes_visible_blank_cells() {
    let mut screen = TerminalScreen::default();
    screen.resize_grid(6, 3);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abc\nDEF\nxyz\x1b[2;3H\x1b[JZ");
    assert_eq!(state.lines, vec!["abc", "DEZ   ", "      "]);

    let mut screen = TerminalScreen::default();
    screen.resize_grid(6, 3);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abc\nDEF\nxyz\x1b[2;3H\x1b[1JZ");
    assert_eq!(state.lines, vec!["      ", "  Z", "xyz"]);

    let mut screen = TerminalScreen::default();
    screen.resize_grid(6, 3);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abc\nDEF\nxyz\x1b[2;3H\x1b[2JZ");
    assert_eq!(state.lines, vec!["      ", "  Z   ", "      "]);
}

#[test]
fn erase_display_full_modes_preserve_cursor_position() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\nbottom\x1b[1A\x1b[4G\x1b[2JZ");
    assert_eq!(state.lines, vec!["   Z"]);
    assert_eq!(state.screen_cursor_row, 0);
    assert_eq!(state.screen_cursor_col, 4);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\nbottom\x1b[5G\x1b[3JZ");
    assert_eq!(state.lines, vec!["", "    Z"]);
    assert_eq!(state.screen_cursor_row, 1);
    assert_eq!(state.screen_cursor_col, 5);
}

#[test]
fn known_width_erase_character_blanks_visible_cells_beyond_row_text() {
    let mut screen = TerminalScreen::default();
    screen.resize(6);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abc\r\x1b[4G\x1b[2XZ");
    assert_eq!(state.lines, vec!["abcZ "]);
}
