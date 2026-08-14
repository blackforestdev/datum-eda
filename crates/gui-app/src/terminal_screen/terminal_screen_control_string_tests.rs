use datum_gui_protocol::TerminalLaneState;

use super::TerminalScreen;

fn terminal_state() -> TerminalLaneState {
    TerminalLaneState::default()
}

#[test]
fn st_control_strings_do_not_leak_bytes() {
    for bytes in [
        b"a\x1bPpayload\x1b\\b".as_slice(),
        b"a\x1b^private\x1b\\b".as_slice(),
        b"a\x1b_private\x1b\\b".as_slice(),
        b"a\x1bXprivate\x1b\\b".as_slice(),
    ] {
        let mut screen = TerminalScreen::default();
        let mut state = terminal_state();
        screen.apply_bytes(&mut state, bytes);
        assert_eq!(state.grid_lines(), vec!["ab"]);
    }
}

#[test]
fn split_st_control_string_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"a\x1bPpay");
    assert_eq!(state.grid_lines(), vec!["a"]);
    screen.apply_bytes(&mut state, b"load\x1b\\b");
    assert_eq!(state.grid_lines(), vec!["ab"]);
}

#[test]
fn c1_st_control_strings_do_not_leak_bytes() {
    for bytes in [
        b"a\x90payload\x9cb".as_slice(),
        b"a\x98private\x9cb".as_slice(),
        b"a\x9eprivate\x9cb".as_slice(),
        b"a\x9fprivate\x9cb".as_slice(),
    ] {
        let mut screen = TerminalScreen::default();
        let mut state = terminal_state();
        screen.apply_bytes(&mut state, bytes);
        assert_eq!(state.grid_lines(), vec!["ab"]);
    }
}

#[test]
fn split_c1_st_control_string_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"a\x90pay");
    assert_eq!(state.grid_lines(), vec!["a"]);
    screen.apply_bytes(&mut state, b"load\x9cb");
    assert_eq!(state.grid_lines(), vec!["ab"]);
}

#[test]
fn split_st_terminator_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"a\x1bPignored\x1b");
    assert_eq!(state.grid_lines(), vec!["a"]);
    screen.apply_bytes(&mut state, b"\\b");
    assert_eq!(state.grid_lines(), vec!["ab"]);
}

#[test]
fn osc_title_updates_terminal_state_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"prompt\x1b]2;datum shell\x07 ready");

    assert_eq!(state.grid_lines(), vec!["prompt ready"]);
    assert_eq!(state.title.as_deref(), Some("datum shell"));
}

#[test]
fn osc_icon_title_updates_terminal_session_label_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"prompt\x1b]1;codex agent\x07 ready");

    assert_eq!(state.grid_lines(), vec!["prompt ready"]);
    assert_eq!(state.title.as_deref(), Some("codex agent"));
}

#[test]
fn osc_current_directory_updates_terminal_state_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(
        &mut state,
        b"prompt\x1b]7;file://datum-host/home/user/Datum%20Project\x07 ready",
    );

    assert_eq!(state.grid_lines(), vec!["prompt ready"]);
    assert_eq!(
        state.current_working_directory.as_deref(),
        Some("/home/user/Datum Project")
    );
}

#[test]
fn unsupported_osc_current_directory_uri_is_swallowed_without_changing_cwd() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(
        &mut state,
        b"\x1b]7;file://datum-host/home/user/project\x07a\x1b]7;ssh://host/tmp\x07b",
    );

    assert_eq!(state.grid_lines(), vec!["ab"]);
    assert_eq!(
        state.current_working_directory.as_deref(),
        Some("/home/user/project")
    );
}

#[test]
fn osc_zero_title_accepts_st_terminator_and_split_input() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"a\x1b]0;layout");
    assert_eq!(state.grid_lines(), vec!["a"]);
    assert_eq!(state.title, None);

    screen.apply_bytes(&mut state, b" shell\x1b\\b");
    assert_eq!(state.grid_lines(), vec!["ab"]);
    assert_eq!(state.title.as_deref(), Some("layout shell"));
}

#[test]
fn unsupported_osc_is_swallowed_without_changing_title() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b]2;first\x07a\x1b]9;ignored\x07b");

    assert_eq!(state.grid_lines(), vec!["ab"]);
    assert_eq!(state.title.as_deref(), Some("first"));
}

#[test]
fn bare_bell_increments_protocol_counter_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"a\x07b\x07");

    assert_eq!(state.grid_lines(), vec!["ab"]);
    assert_eq!(state.bell_count, 2);

    screen.apply_bytes(&mut state, b"\x1b]2;title\x07c");
    assert_eq!(state.grid_lines(), vec!["abc"]);
    assert_eq!(state.title.as_deref(), Some("title"));
    assert_eq!(
        state.bell_count, 2,
        "OSC BEL terminator should not count as a terminal alert"
    );
}

#[test]
fn esc_intermediate_sequences_do_not_leak_final_byte() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"a\x1b#7b");
    assert_eq!(state.grid_lines(), vec!["ab"]);
}

#[test]
fn split_esc_intermediate_sequence_does_not_leak_final_byte() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"a\x1b#");
    assert_eq!(state.grid_lines(), vec!["a"]);
    screen.apply_bytes(&mut state, b"7b");
    assert_eq!(state.grid_lines(), vec!["ab"]);
}
