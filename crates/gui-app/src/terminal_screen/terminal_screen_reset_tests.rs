use datum_gui_protocol::TerminalLaneState;

use super::TerminalScreen;

fn terminal_state() -> TerminalLaneState {
    TerminalLaneState::default()
}

#[test]
fn reset_clears_screen_cursor_saved_state_and_repeat_character() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abc\x1b7\nstatus\x1bc\x1b[3bZ\x1b8Y");
    assert_eq!(state.grid_lines(), vec!["ZY"]);
}

#[test]
fn reset_clears_scroll_region_and_alternate_screen_state() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"main\x1b[?1049halt\x1bcX");
    assert_eq!(state.grid_lines(), vec!["X"]);

    screen.apply_bytes(&mut state, b"\nY\x1b[1;1H\x1b[1;1r\x1bcA\nB");
    assert_eq!(state.grid_lines(), vec!["A", "B"]);
}

#[test]
fn reset_restores_autowrap_mode() {
    let mut screen = TerminalScreen::default();
    screen.resize(4);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[?7labcdZ\x1bcabcdZ");
    assert_eq!(state.grid_lines(), vec!["abcd", "Z"]);
}
