use datum_gui_protocol::TerminalLaneState;

use super::TerminalScreen;

fn terminal_state() -> TerminalLaneState {
    TerminalLaneState::default()
}

#[test]
fn hts_sets_custom_tab_stop_without_visible_output() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[6G\x1bH\rx\ty");
    assert_eq!(state.grid_lines(), vec!["x    y"]);
}

#[test]
fn split_hts_sequence_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[6G\x1b");
    assert_eq!(state.grid_lines(), vec![""]);
    screen.apply_bytes(&mut state, b"H\rx\ty");
    assert_eq!(state.grid_lines(), vec!["x    y"]);
}

#[test]
fn tab_clear_current_removes_custom_stop() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[6G\x1bH\x1b[g\rx\ty");
    assert_eq!(state.grid_lines(), vec!["x       y"]);
}

#[test]
fn tab_clear_current_can_remove_default_stop() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[9G\x1b[g\rx\ty");
    assert_eq!(state.grid_lines(), vec!["x               y"]);
}

#[test]
fn tab_clear_all_makes_tabs_noop_until_custom_stop() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[3gA\tB\x1b[6G\x1bH\rx\ty");
    assert_eq!(state.grid_lines(), vec!["x    y"]);
}

#[test]
fn split_tab_clear_sequences_do_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[");
    assert_eq!(state.grid_lines(), vec![""]);
    screen.apply_bytes(&mut state, b"3gA\tB");
    assert_eq!(state.grid_lines(), vec!["AB"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[9G\x1b[");
    assert_eq!(state.grid_lines(), vec![""]);
    screen.apply_bytes(&mut state, b"g\rx\ty");
    assert_eq!(state.grid_lines(), vec!["x               y"]);
}

#[test]
fn terminal_reset_restores_default_tab_stops() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[3gA\tB\x1bcx\ty");
    assert_eq!(state.grid_lines(), vec!["x       y"]);
}
