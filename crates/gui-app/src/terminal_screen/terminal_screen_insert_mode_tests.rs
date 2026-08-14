use datum_gui_protocol::TerminalLaneState;

use super::TerminalScreen;

fn terminal_state() -> TerminalLaneState {
    TerminalLaneState::default()
}

#[test]
fn insert_mode_shifts_existing_cells_until_reset() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\r\x1b[4hXY\x1b[4lZ");
    assert_eq!(state.grid_lines(), vec!["XYZbcdef"]);
}

#[test]
fn split_insert_mode_sequence_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\r\x1b[");
    assert_eq!(state.grid_lines(), vec!["abcdef"]);
    screen.apply_bytes(&mut state, b"4hZ");
    assert_eq!(state.grid_lines(), vec!["Zabcdef"]);
}

#[test]
fn reset_clears_insert_mode() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\r\x1b[4hX\x1bcYZ");
    assert_eq!(state.grid_lines(), vec!["YZ"]);
}

#[test]
fn repeat_preceding_character_honors_insert_mode() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\r\x1b[4hX\x1b[2b");
    assert_eq!(state.grid_lines(), vec!["XXXabcdef"]);
}
