use datum_gui_protocol::TerminalLaneState;

use super::TerminalScreen;

fn terminal_state() -> TerminalLaneState {
    TerminalLaneState::default()
}

#[test]
fn charset_designation_sequences_do_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"a\x1b(Bb\x1b)0c");
    assert_eq!(state.grid_lines(), vec!["abc"]);
}

#[test]
fn split_charset_designation_sequence_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"a\x1b(");
    assert_eq!(state.grid_lines(), vec!["a"]);
    screen.apply_bytes(&mut state, b"Bb");
    assert_eq!(state.grid_lines(), vec!["ab"]);
}

#[test]
fn dec_screen_alignment_test_fills_visible_grid() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    state.columns = 6;
    state.rows = 3;

    screen.apply_bytes(&mut state, b"prompt\x1b#8");

    assert_eq!(state.grid_lines(), vec!["EEEEEE", "EEEEEE", "EEEEEE"]);
    assert_eq!(state.grid_styled_lines().len(), 3);
    assert!(
        state
            .grid_styled_lines()
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

    assert_eq!(state.grid_lines(), vec!["ab"]);
}
