use super::{TerminalScreen, terminal_screen_basic_tests::terminal_state};

#[test]
fn vertical_position_backward_preserves_column() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abc\n123\nXYZ\r\x1b[2C\x1b[2kQ");
    assert_eq!(state.lines, vec!["abQ", "123", "XYZ"]);
}

#[test]
fn vertical_position_backward_defaults_to_one_and_saturates_at_top() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\nmid\nbot\r\x1b[1C\x1b[kQ\x1b[9kR");
    assert_eq!(state.lines, vec!["toR", "mQd", "bot"]);
}

#[test]
fn known_height_vertical_cursor_positions_clamp_to_bottom_margin() {
    let mut screen = TerminalScreen::default();
    screen.resize_grid(10, 3);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\nmid\nbot\x1b[99BZ");
    assert_eq!(state.lines, vec!["top", "mid", "botZ"]);

    let mut screen = TerminalScreen::default();
    screen.resize_grid(10, 3);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\nmid\nbot\x1b[99dZ");
    assert_eq!(state.lines, vec!["top", "mid", "botZ"]);

    let mut screen = TerminalScreen::default();
    screen.resize_grid(10, 3);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\nmid\nbot\x1b[99;1HZ");
    assert_eq!(state.lines, vec!["top", "mid", "Zot"]);
}
