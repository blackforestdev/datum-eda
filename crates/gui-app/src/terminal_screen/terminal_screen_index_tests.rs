use datum_gui_protocol::TerminalLaneState;

use super::TerminalScreen;

fn terminal_state() -> TerminalLaneState {
    TerminalLaneState::default()
}

#[test]
fn index_moves_down_without_resetting_column() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abc\x1bDZ");
    assert_eq!(state.grid_lines(), vec!["abc", "   Z"]);
}

#[test]
fn next_line_moves_down_and_resets_column() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abc\x1bEZ");
    assert_eq!(state.grid_lines(), vec!["abc", "Z"]);
}

#[test]
fn vertical_tab_and_form_feed_move_down_without_resetting_column() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abc\x0bZ\x0cY");
    assert_eq!(state.grid_lines(), vec!["abc", "   Z", "    Y"]);
}

#[test]
fn index_scrolls_only_scroll_region_at_bottom_margin() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\nmid\nbot\x1b[2;3r\x1b[3;2H\x1bDZ");
    assert_eq!(state.grid_lines(), vec!["top", "bot", " Z"]);
}
