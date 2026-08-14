use datum_gui_protocol::TerminalLaneState;

use super::TerminalScreen;

fn terminal_state() -> TerminalLaneState {
    TerminalLaneState::default()
}

#[test]
fn repeat_preceding_character_uses_printable_cursor_semantics() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"ab\x1b[4bZ");
    assert_eq!(state.grid_lines(), vec!["abbbbbZ"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[3bZ");
    assert_eq!(state.grid_lines(), vec!["Z"]);
}

#[test]
fn repeat_preceding_character_wraps_at_terminal_columns() {
    let mut screen = TerminalScreen::default();
    screen.resize(4);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"ab\x1b[3bZ");
    assert_eq!(state.grid_lines(), vec!["abbb", "bZ"]);
}
