use datum_gui_protocol::TerminalLaneState;

use super::TerminalScreen;

fn terminal_state() -> TerminalLaneState {
    TerminalLaneState::default()
}

#[test]
fn c1_csi_and_osc_sequences_do_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\x9b3DZ");
    assert_eq!(state.grid_lines(), vec!["abcZef"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"a\x9d0;datum gui\x9cb");
    assert_eq!(state.grid_lines(), vec!["ab"]);
}

#[test]
fn c1_index_next_line_and_reverse_index_match_escape_aliases() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"ab\x84Z");
    assert_eq!(state.grid_lines(), vec!["ab", "  Z"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"ab\x85Z");
    assert_eq!(state.grid_lines(), vec!["ab", "Z"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"one\ntwo\x9b2;2H\x8dZ");
    assert_eq!(state.grid_lines(), vec!["oZe", "two"]);
}
