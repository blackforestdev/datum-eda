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
fn sgr_foreground_color_is_retained_as_terminal_spans() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[31mred\x1b[0m plain");
    assert_eq!(state.lines, vec!["red plain"]);
    assert_eq!(state.styled_lines[0].text, "red plain");
    assert_eq!(state.styled_lines[0].spans.len(), 1);
    assert_eq!(state.styled_lines[0].spans[0].start, 0);
    assert_eq!(state.styled_lines[0].spans[0].end, 3);
    assert_eq!(state.styled_lines[0].spans[0].fg.as_deref(), Some("red"));
    assert!(!state.styled_lines[0].spans[0].bold);
}

#[test]
fn sgr_bold_and_foreground_share_one_span_until_reset() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[1;31mERR\x1b[0m ok");
    assert_eq!(state.lines, vec!["ERR ok"]);
    assert_eq!(state.styled_lines[0].spans.len(), 1);
    assert_eq!(state.styled_lines[0].spans[0].start, 0);
    assert_eq!(state.styled_lines[0].spans[0].end, 3);
    assert_eq!(state.styled_lines[0].spans[0].fg.as_deref(), Some("red"));
    assert!(state.styled_lines[0].spans[0].bold);
}

#[test]
fn sgr_reset_prevents_style_from_leaking_to_later_cells() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(
        &mut state,
        b"pre \x1b[32mok\x1b[39m neutral \x1b[1mbold\x1b[22m done",
    );
    assert_eq!(state.lines, vec!["pre ok neutral bold done"]);
    assert_eq!(state.styled_lines[0].spans.len(), 2);
    assert_eq!(state.styled_lines[0].spans[0].start, 4);
    assert_eq!(state.styled_lines[0].spans[0].end, 6);
    assert_eq!(state.styled_lines[0].spans[0].fg.as_deref(), Some("green"));
    assert!(!state.styled_lines[0].spans[0].bold);
    assert_eq!(state.styled_lines[0].spans[1].start, 15);
    assert_eq!(state.styled_lines[0].spans[1].end, 19);
    assert_eq!(state.styled_lines[0].spans[1].fg, None);
    assert!(state.styled_lines[0].spans[1].bold);
}

#[test]
fn sgr_background_color_is_retained_as_terminal_span_metadata() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[44mblue-bg\x1b[49m plain");
    assert_eq!(state.lines, vec!["blue-bg plain"]);
    assert_eq!(state.styled_lines[0].spans.len(), 1);
    assert_eq!(state.styled_lines[0].spans[0].start, 0);
    assert_eq!(state.styled_lines[0].spans[0].end, 7);
    assert_eq!(state.styled_lines[0].spans[0].bg.as_deref(), Some("blue"));
    assert_eq!(state.styled_lines[0].spans[0].fg, None);
}

#[test]
fn sgr_extended_ansi256_colors_are_retained_as_terminal_span_metadata() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[38;5;196mfg\x1b[48;5;22mbg\x1b[0m");
    assert_eq!(state.lines, vec!["fgbg"]);
    assert_eq!(state.styled_lines[0].spans.len(), 2);
    assert_eq!(state.styled_lines[0].spans[0].start, 0);
    assert_eq!(state.styled_lines[0].spans[0].end, 2);
    assert_eq!(
        state.styled_lines[0].spans[0].fg.as_deref(),
        Some("ansi256:196")
    );
    assert_eq!(state.styled_lines[0].spans[1].start, 2);
    assert_eq!(state.styled_lines[0].spans[1].end, 4);
    assert_eq!(
        state.styled_lines[0].spans[1].fg.as_deref(),
        Some("ansi256:196")
    );
    assert_eq!(
        state.styled_lines[0].spans[1].bg.as_deref(),
        Some("ansi256:22")
    );
}

#[test]
fn sgr_extended_truecolor_values_are_retained_as_terminal_span_metadata() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[38;2;12;34;56;48;2;1;2;3mrgb");
    assert_eq!(state.lines, vec!["rgb"]);
    assert_eq!(state.styled_lines[0].spans.len(), 1);
    assert_eq!(
        state.styled_lines[0].spans[0].fg.as_deref(),
        Some("rgb:12:34:56")
    );
    assert_eq!(
        state.styled_lines[0].spans[0].bg.as_deref(),
        Some("rgb:1:2:3")
    );
}

#[test]
fn malformed_extended_sgr_color_does_not_clear_existing_style() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[31mred\x1b[38;5;999m still-red");
    assert_eq!(state.lines, vec!["red still-red"]);
    assert_eq!(state.styled_lines[0].spans.len(), 1);
    assert_eq!(state.styled_lines[0].spans[0].start, 0);
    assert_eq!(state.styled_lines[0].spans[0].end, 13);
    assert_eq!(state.styled_lines[0].spans[0].fg.as_deref(), Some("red"));
}

#[test]
fn sgr_inverse_state_is_retained_until_reset() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"pre \x1b[7;31;42minv\x1b[27m fg");
    assert_eq!(state.lines, vec!["pre inv fg"]);
    assert_eq!(state.styled_lines[0].spans.len(), 2);
    assert_eq!(state.styled_lines[0].spans[0].start, 4);
    assert_eq!(state.styled_lines[0].spans[0].end, 7);
    assert_eq!(state.styled_lines[0].spans[0].fg.as_deref(), Some("red"));
    assert_eq!(state.styled_lines[0].spans[0].bg.as_deref(), Some("green"));
    assert!(state.styled_lines[0].spans[0].inverse);
    assert_eq!(state.styled_lines[0].spans[1].start, 7);
    assert_eq!(state.styled_lines[0].spans[1].end, 10);
    assert_eq!(state.styled_lines[0].spans[1].fg.as_deref(), Some("red"));
    assert_eq!(state.styled_lines[0].spans[1].bg.as_deref(), Some("green"));
    assert!(!state.styled_lines[0].spans[1].inverse);
}

#[test]
fn sgr_underline_italic_and_strikethrough_are_retained_until_reset() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(
        &mut state,
        b"\x1b[3;4;9mdecor\x1b[23m no-italic\x1b[24m no-under\x1b[29m plain",
    );

    assert_eq!(state.lines, vec!["decor no-italic no-under plain"]);
    assert_eq!(state.styled_lines[0].spans.len(), 3);

    let decorated = &state.styled_lines[0].spans[0];
    assert_eq!(decorated.start, 0);
    assert_eq!(decorated.end, 5);
    assert!(decorated.italic);
    assert!(decorated.underline);
    assert!(decorated.strikethrough);

    let no_italic = &state.styled_lines[0].spans[1];
    assert_eq!(no_italic.start, 5);
    assert_eq!(no_italic.end, 15);
    assert!(!no_italic.italic);
    assert!(no_italic.underline);
    assert!(no_italic.strikethrough);

    let no_under = &state.styled_lines[0].spans[2];
    assert_eq!(no_under.start, 15);
    assert_eq!(no_under.end, 24);
    assert!(!no_under.italic);
    assert!(!no_under.underline);
    assert!(no_under.strikethrough);
}

#[test]
fn sgr_dim_and_conceal_are_retained_until_matching_reset() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(
        &mut state,
        b"\x1b[1;2;8msecret\x1b[22m plain-dim-reset\x1b[28m visible",
    );

    assert_eq!(state.lines, vec!["secret plain-dim-reset visible"]);
    assert_eq!(state.styled_lines[0].spans.len(), 2);

    let secret = &state.styled_lines[0].spans[0];
    assert_eq!(secret.start, 0);
    assert_eq!(secret.end, 6);
    assert!(secret.bold);
    assert!(secret.dim);
    assert!(secret.conceal);

    let concealed = &state.styled_lines[0].spans[1];
    assert_eq!(concealed.start, 6);
    assert_eq!(concealed.end, 22);
    assert!(!concealed.bold);
    assert!(!concealed.dim);
    assert!(concealed.conceal);
}

#[test]
fn sgr_blink_is_retained_until_reset() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[5mblink\x1b[25m plain \x1b[6mrapid");

    assert_eq!(state.lines, vec!["blink plain rapid"]);
    assert_eq!(state.styled_lines[0].spans.len(), 2);

    let blink = &state.styled_lines[0].spans[0];
    assert_eq!(blink.start, 0);
    assert_eq!(blink.end, 5);
    assert!(blink.blink);

    let rapid = &state.styled_lines[0].spans[1];
    assert_eq!(rapid.start, 12);
    assert_eq!(rapid.end, 17);
    assert!(rapid.blink);
    assert!(!rapid.bold);
    assert!(!rapid.dim);
}

#[test]
fn sgr_overline_is_retained_until_reset() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[53mover\x1b[55m plain");

    assert_eq!(state.lines, vec!["over plain"]);
    assert_eq!(state.styled_lines[0].spans.len(), 1);

    let over = &state.styled_lines[0].spans[0];
    assert_eq!(over.start, 0);
    assert_eq!(over.end, 4);
    assert!(over.overline);
    assert!(!over.underline);
    assert!(!over.strikethrough);
}
