use super::{TerminalScreen, terminal_screen_basic_tests::terminal_state};

#[test]
fn cursor_horizontal_position_and_relative_move_are_supported() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\r\x1b[4`Z");
    assert_eq!(state.lines, vec!["abcZef"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"ab\r\x1b[3aZ");
    assert_eq!(state.lines, vec!["ab Z"]);
}

#[test]
fn known_width_horizontal_cursor_positions_clamp_to_right_margin() {
    let mut screen = TerminalScreen::default();
    screen.resize(5);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcde\r\x1b[99CZ");
    assert_eq!(state.lines, vec!["abcdZ"]);

    let mut screen = TerminalScreen::default();
    screen.resize(5);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcde\r\x1b[99`Z");
    assert_eq!(state.lines, vec!["abcdZ"]);

    let mut screen = TerminalScreen::default();
    screen.resize(5);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcde\x1b[1;99HZ");
    assert_eq!(state.lines, vec!["abcdZ"]);
}
