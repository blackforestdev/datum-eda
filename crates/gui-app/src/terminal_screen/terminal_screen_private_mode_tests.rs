use super::*;

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
fn private_1047_alternate_screen_restores_main_buffer() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(
        &mut state,
        b"prompt> keep\x1b[?1047hmenu\nrow\x1b[?1047l cmd",
    );
    assert_eq!(state.lines, vec!["prompt> keep cmd"]);
}

#[test]
fn private_47_alternate_screen_restores_main_buffer() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(
        &mut state,
        b"prompt> keep\x1b[?47hlegacy menu\nrow\x1b[?47l cmd",
    );
    assert_eq!(state.lines, vec!["prompt> keep cmd"]);
}

#[test]
fn split_private_47_sequence_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"main\x1b[?");
    assert_eq!(state.lines, vec!["main"]);
    screen.apply_bytes(&mut state, b"47halt\x1b[?");
    assert_eq!(state.lines, vec!["alt"]);
    screen.apply_bytes(&mut state, b"47l done");
    assert_eq!(state.lines, vec!["main done"]);
}

#[test]
fn split_private_1049_sequence_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"main\x1b[?");
    assert_eq!(state.lines, vec!["main"]);
    screen.apply_bytes(&mut state, b"1049halt\x1b[?");
    assert_eq!(state.lines, vec!["alt"]);
    screen.apply_bytes(&mut state, b"1049l done");
    assert_eq!(state.lines, vec!["main done"]);
}

#[test]
fn private_1048_saves_and_restores_cursor_without_switching_buffers() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"prompt> \x1b[?1048hstatus\nok\x1b[?1048lcmd");
    assert_eq!(state.lines, vec!["prompt> cmdtus", "ok"]);
}

#[test]
fn private_2004_tracks_bracketed_paste_mode_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"prompt \x1b[?2004hready");
    assert_eq!(state.lines, vec!["prompt ready"]);
    assert!(screen.bracketed_paste_enabled());

    screen.apply_bytes(&mut state, b"\x1b[?2004l done");
    assert_eq!(state.lines, vec!["prompt ready done"]);
    assert!(!screen.bracketed_paste_enabled());
}

#[test]
fn private_7_disables_and_reenables_autowrap() {
    let mut screen = TerminalScreen::default();
    screen.resize(5);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[?7labcdefg");
    assert_eq!(state.lines, vec!["abcdg"]);

    screen.apply_bytes(&mut state, b"\x1b[?7hXY");
    assert_eq!(state.lines, vec!["abcdX", "Y"]);
}

#[test]
fn private_7_split_sequence_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    screen.resize(4);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[?");
    assert_eq!(state.lines, vec![""]);
    screen.apply_bytes(&mut state, b"7labcdZ");
    assert_eq!(state.lines, vec!["abcZ"]);
}

#[test]
fn combined_private_modes_enable_origin_and_autowrap() {
    let mut screen = TerminalScreen::default();
    screen.resize(5);
    let mut state = terminal_state();
    screen.apply_bytes(
        &mut state,
        b"top\none\ntwo\nbot\x1b[2;3r\x1b[?7l\x1b[?6;7hABCDEZ",
    );
    assert_eq!(state.lines, vec!["top", "ABCDE", "Zwo", "bot"]);
}

#[test]
fn combined_private_modes_disable_origin_and_autowrap() {
    let mut screen = TerminalScreen::default();
    screen.resize(4);
    let mut state = terminal_state();
    screen.apply_bytes(
        &mut state,
        b"top\none\ntwo\x1b[2;3r\x1b[?6h\x1b[?6;7l\x1b[1;1HABCDE",
    );
    assert_eq!(state.lines, vec!["ABCE", "one", "two"]);
}

#[test]
fn repeat_preceding_character_inherits_autowrap_mode() {
    let mut screen = TerminalScreen::default();
    screen.resize(4);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[?7lAB\x1b[5b");
    assert_eq!(state.lines, vec!["ABBB"]);
}

#[test]
fn private_6_origin_mode_addresses_scroll_region() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\none\ntwo\nbot\x1b[2;3r\x1b[?6h\x1b[1;1HZ");
    assert_eq!(state.lines, vec!["top", "Zne", "two", "bot"]);

    screen.apply_bytes(&mut state, b"\x1b[2;1HY");
    assert_eq!(state.lines, vec!["top", "Zne", "Ywo", "bot"]);
}

#[test]
fn private_6_origin_mode_offsets_vertical_absolute_position() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(
        &mut state,
        b"top\none\ntwo\nbot\x1b[2;3r\x1b[?6h\x1b[1;2H\x1b[2dZ",
    );
    assert_eq!(state.lines, vec!["top", "one", "tZo", "bot"]);
}

#[test]
fn private_6_origin_mode_clamps_to_scroll_region_bottom() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\none\ntwo\nbot\x1b[2;3r\x1b[?6h\x1b[9;1HZ");
    assert_eq!(state.lines, vec!["top", "one", "Zwo", "bot"]);
}

#[test]
fn private_6_reset_restores_absolute_origin() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(
        &mut state,
        b"top\none\ntwo\nbot\x1b[2;3r\x1b[?6h\x1b[?6l\x1b[1;1HZ",
    );
    assert_eq!(state.lines, vec!["Zop", "one", "two", "bot"]);
}

#[test]
fn private_6_split_sequence_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\none\ntwo\x1b[2;3r\x1b[?");
    assert_eq!(state.lines, vec!["top", "one", "two"]);
    screen.apply_bytes(&mut state, b"6h\x1b[1;1HZ");
    assert_eq!(state.lines, vec!["top", "Zne", "two"]);
}

#[test]
fn split_private_2004_sequence_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"main\x1b[?200");
    assert_eq!(state.lines, vec!["main"]);
    assert!(!screen.bracketed_paste_enabled());

    screen.apply_bytes(&mut state, b"4h paste");
    assert_eq!(state.lines, vec!["main paste"]);
    assert!(screen.bracketed_paste_enabled());
}

#[test]
fn terminal_reset_clears_bracketed_paste_mode() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[?2004harmed\x1bcreset");
    assert_eq!(state.lines, vec!["reset"]);
    assert!(!screen.bracketed_paste_enabled());
}

#[test]
fn private_1004_tracks_focus_event_reporting_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"prompt\x1b[?1004h focus");

    assert_eq!(state.lines, vec!["prompt focus"]);
    assert!(state.focus_event_reporting);

    screen.apply_bytes(&mut state, b"\x1b[?1004l done");
    assert_eq!(state.lines, vec!["prompt focus done"]);
    assert!(!state.focus_event_reporting);
}

#[test]
fn split_private_1004_sequence_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"main\x1b[?100");
    assert_eq!(state.lines, vec!["main"]);
    assert!(!state.focus_event_reporting);

    screen.apply_bytes(&mut state, b"4h focus");
    assert_eq!(state.lines, vec!["main focus"]);
    assert!(state.focus_event_reporting);
}

#[test]
fn terminal_reset_clears_focus_event_reporting() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[?1004harmed\x1bcreset");

    assert_eq!(state.lines, vec!["reset"]);
    assert!(!state.focus_event_reporting);
}

#[test]
fn private_1_tracks_application_cursor_keys_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"prompt\x1b[?1h app");

    assert_eq!(state.lines, vec!["prompt app"]);
    assert!(state.application_cursor_keys);

    screen.apply_bytes(&mut state, b"\x1b[?1l normal");
    assert_eq!(state.lines, vec!["prompt app normal"]);
    assert!(!state.application_cursor_keys);
}

#[test]
fn split_private_1_sequence_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"main\x1b[?");
    assert_eq!(state.lines, vec!["main"]);
    assert!(!state.application_cursor_keys);

    screen.apply_bytes(&mut state, b"1h app");
    assert_eq!(state.lines, vec!["main app"]);
    assert!(state.application_cursor_keys);
}

#[test]
fn terminal_reset_clears_application_cursor_keys() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[?1harmed\x1bcreset");

    assert_eq!(state.lines, vec!["reset"]);
    assert!(!state.application_cursor_keys);
}

#[test]
fn dec_keypad_application_mode_tracks_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"prompt\x1b= keypad");

    assert_eq!(state.lines, vec!["prompt keypad"]);
    assert!(state.application_keypad);

    screen.apply_bytes(&mut state, b"\x1b> numeric");
    assert_eq!(state.lines, vec!["prompt keypad numeric"]);
    assert!(!state.application_keypad);
}

#[test]
fn split_dec_keypad_application_mode_sequence_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"main\x1b");
    assert_eq!(state.lines, vec!["main"]);
    assert!(!state.application_keypad);

    screen.apply_bytes(&mut state, b"= keypad");
    assert_eq!(state.lines, vec!["main keypad"]);
    assert!(state.application_keypad);
}

#[test]
fn terminal_reset_clears_application_keypad() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b=armed\x1bcreset");

    assert_eq!(state.lines, vec!["reset"]);
    assert!(!state.application_keypad);
}

#[test]
fn private_mouse_reporting_modes_are_tracked_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"prompt\x1b[?1000h normal");

    assert_eq!(state.lines, vec!["prompt normal"]);
    assert_eq!(state.mouse_reporting_mode.as_deref(), Some("normal"));

    screen.apply_bytes(&mut state, b"\x1b[?1002h drag\x1b[?1003h any");
    assert_eq!(state.lines, vec!["prompt normal drag any"]);
    assert_eq!(state.mouse_reporting_mode.as_deref(), Some("any_event"));

    screen.apply_bytes(&mut state, b"\x1b[?1003l done");
    assert_eq!(state.lines, vec!["prompt normal drag any done"]);
    assert_eq!(state.mouse_reporting_mode, None);
}

#[test]
fn private_mouse_coordinate_encoding_is_tracked_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[?1006h sgr\x1b[?1015h urxvt");

    assert_eq!(state.lines, vec![" sgr urxvt"]);
    assert_eq!(state.mouse_coordinate_encoding.as_deref(), Some("urxvt"));

    screen.apply_bytes(&mut state, b"\x1b[?1015l plain");
    assert_eq!(state.lines, vec![" sgr urxvt plain"]);
    assert_eq!(state.mouse_coordinate_encoding, None);
}

#[test]
fn split_private_mouse_mode_sequence_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"main\x1b[?100");
    assert_eq!(state.lines, vec!["main"]);
    assert_eq!(state.mouse_reporting_mode, None);

    screen.apply_bytes(&mut state, b"2h drag");
    assert_eq!(state.lines, vec!["main drag"]);
    assert_eq!(state.mouse_reporting_mode.as_deref(), Some("button_event"));
}

#[test]
fn terminal_reset_clears_mouse_reporting_modes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[?1002h\x1b[?1006harmed\x1bcreset");

    assert_eq!(state.lines, vec!["reset"]);
    assert_eq!(state.mouse_reporting_mode, None);
    assert_eq!(state.mouse_coordinate_encoding, None);
}

#[test]
fn private_25_tracks_cursor_visibility_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"prompt\x1b[?25l hidden");
    assert_eq!(state.lines, vec!["prompt hidden"]);
    assert_eq!(state.screen_cursor_row, 0);
    assert_eq!(state.screen_cursor_col, "prompt hidden".len());
    assert!(!state.screen_cursor_visible);

    screen.apply_bytes(&mut state, b"\x1b[?25h visible");
    assert_eq!(state.lines, vec!["prompt hidden visible"]);
    assert_eq!(state.screen_cursor_col, "prompt hidden visible".len());
    assert!(state.screen_cursor_visible);
}

#[test]
fn decscusr_cursor_style_updates_protocol_state() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"prompt\x1b[5 q");

    assert_eq!(state.lines, vec!["prompt"]);
    assert_eq!(state.screen_cursor_style.as_deref(), Some("blinking_bar"));

    screen.apply_bytes(&mut state, b"\x1b[4 q");
    assert_eq!(
        state.screen_cursor_style.as_deref(),
        Some("steady_underline")
    );
}

#[test]
fn split_decscusr_cursor_style_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"main\x1b[6");
    assert_eq!(state.lines, vec!["main"]);
    assert_eq!(state.screen_cursor_style, None);

    screen.apply_bytes(&mut state, b" q done");
    assert_eq!(state.lines, vec!["main done"]);
    assert_eq!(state.screen_cursor_style.as_deref(), Some("steady_bar"));
}

#[test]
fn terminal_reset_clears_cursor_style() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[3 qstyled\x1bcreset");

    assert_eq!(state.lines, vec!["reset"]);
    assert_eq!(state.screen_cursor_style, None);
}

#[test]
fn split_private_1047_sequence_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"main\x1b[?1047");
    assert_eq!(state.lines, vec!["main"]);
    screen.apply_bytes(&mut state, b"halt\x1b[?1047l done");
    assert_eq!(state.lines, vec!["main done"]);
}
