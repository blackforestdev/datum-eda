use datum_gui_protocol::TerminalLaneState;

use super::TerminalScreen;

pub(super) fn terminal_state() -> TerminalLaneState {
    TerminalLaneState::default()
}

#[test]
fn applies_basic_prompt_and_row_rewrites() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"datum$ ");
    assert_eq!(state.grid_lines(), vec!["datum$ "]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\rXY");
    assert_eq!(state.grid_lines(), vec!["XYcdef"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"building 10%\rbuilding 20%\x1b[K");
    assert_eq!(state.grid_lines(), vec!["building 20%"]);
}

#[test]
fn colored_bash_prompt_places_cursor_after_dollar_and_trailing_space() {
    let mut screen = TerminalScreen::default();
    screen.resize_grid(164, 6);
    let mut state = terminal_state();
    let bytes = b"\x1b[01;32mbfadmin@debian3520\x1b[00m:\x1b[01;34m/tmp/datum-eda/gui-imports/DOA2526-5825f90fe7490128\x1b[01;33m$\x1b[00m ";

    screen.apply_bytes(&mut state, bytes);

    let prompt = "bfadmin@debian3520:/tmp/datum-eda/gui-imports/DOA2526-5825f90fe7490128$ ";
    assert_eq!(state.grid_lines(), vec![prompt]);
    assert_eq!(state.screen_cursor_row, 0);
    assert_eq!(state.screen_cursor_col, prompt.chars().count());
}

#[test]
fn cursor_left_and_right_support_progress_rewrites() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\x1b[3D\x1b[KXY");
    assert_eq!(state.grid_lines(), vec!["abcXY"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"ab\r\x1b[4Cz");
    assert_eq!(state.grid_lines(), vec!["ab  z"]);
}

#[test]
fn tabs_use_default_stops_and_backtab_saturates() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"a\tb");
    assert_eq!(state.grid_lines(), vec!["a       b"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"ab\x1b[2Iz");
    assert_eq!(state.grid_lines(), vec!["ab              z"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"0123456789\x1b[Zx");
    assert_eq!(state.grid_lines(), vec!["01234567x9"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\x1b[2Zz");
    assert_eq!(state.grid_lines(), vec!["zbcdef"]);
}

#[test]
fn delete_character_shifts_row_left_without_moving_cursor() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\x1b[3D\x1b[2PZ");
    assert_eq!(state.grid_lines(), vec!["abcZ"]);
}

#[test]
fn erase_character_blanks_cells_without_moving_cursor() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\x1b[4D\x1b[3XZ");
    assert_eq!(state.grid_lines(), vec!["abZ  f"]);
}

#[test]
fn insert_character_opens_blank_cells_without_moving_cursor() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\x1b[3D\x1b[2@XY");
    assert_eq!(state.grid_lines(), vec!["abcXYdef"]);
}

#[test]
fn insert_and_delete_character_are_bounded_by_terminal_width() {
    let mut screen = TerminalScreen::default();
    screen.resize(6);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\x1b[3D\x1b[2@Z");
    assert_eq!(state.grid_lines(), vec!["abcZ d"]);

    let mut screen = TerminalScreen::default();
    screen.resize(6);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcdef\x1b[3D\x1b[2PZ");
    assert_eq!(state.grid_lines(), vec!["abcZ  "]);
}

#[test]
fn insert_and_delete_line_shift_rows_within_screen() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\nmid\nbot\x1b[2;1H\x1b[Lnew");
    assert_eq!(state.grid_lines(), vec!["top", "new", "mid"]);
    screen.apply_bytes(&mut state, b"\x1b[2;1H\x1b[M");
    assert_eq!(state.grid_lines(), vec!["top", "mid", ""]);
}

#[test]
fn delete_line_count_larger_than_visible_region_clears_without_underflow() {
    let mut screen = TerminalScreen::default();
    screen.resize_grid(8, 3);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\nmid\nbot\x1b[1;1H\x1b[80M");
    assert_eq!(state.grid_lines(), vec!["", "", ""]);
}

#[test]
fn cursor_up_and_down_rewrite_addressed_rows() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"one\ntwo\x1b[1A\rONE\x1b[1B\rTWO");
    assert_eq!(state.grid_lines(), vec!["ONE", "TWO"]);
}

#[test]
fn cursor_next_and_previous_line_reset_to_column_zero() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"one\ntwo\x1b[1FZERO\x1b[1Eend");
    assert_eq!(state.grid_lines(), vec!["ZERO", "end"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abc\x1b[Ez");
    assert_eq!(state.grid_lines(), vec!["abc", "z"]);
}

#[test]
fn cursor_vertical_position_preserves_column() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"one\ntwo\x1b[1;2H\x1b[2dZ");
    assert_eq!(state.grid_lines(), vec!["one", "tZo"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"ab\x1b[2G\x1b[2ez");
    assert_eq!(state.grid_lines(), vec!["ab", "", " z"]);
}

#[test]
fn cursor_position_addresses_screen_rows_and_columns() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"alpha\nbeta\x1b[1;3HZ");
    assert_eq!(state.grid_lines(), vec!["alZha", "beta"]);
}

#[test]
fn cursor_save_and_restore_supports_status_rewrites() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"prompt> \x1b7status\nok\x1b8cmd");
    assert_eq!(state.grid_lines(), vec!["prompt> cmdtus", "ok"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"ab\x1b[s\nzz\x1b[ucd");
    assert_eq!(state.grid_lines(), vec!["abcd", "zz"]);
}

#[test]
fn claude_keyboard_controls_cannot_restore_a_stale_cursor() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.resize_grid(40, 4);

    screen.apply_bytes(&mut state, b"prompt> \x1b[s\x1b[1;16H@");
    assert_eq!((state.screen_cursor_row, state.screen_cursor_col), (0, 16));

    // Claude Code 2.1.233 emits these kitty-keyboard push, query, and pop
    // controls. Datum does not advertise that later protocol yet, so these are
    // ignored. Most importantly, their final `u` is not the parameter-free
    // CSI restore-cursor command.
    let responses = screen.apply_bytes_with_responses(&mut state, b"\x1b[>1u\x1b[?u\x1b[<uclaude");
    assert!(responses.is_empty());
    assert_eq!((state.screen_cursor_row, state.screen_cursor_col), (0, 22));
    assert_eq!(state.grid_lines(), vec!["prompt>        @claude"]);

    screen.apply_bytes(&mut state, b"\x1b[u!");
    assert_eq!(state.grid_lines(), vec!["prompt> !      @claude"]);
}

#[test]
fn split_claude_keyboard_controls_are_cursor_state_invariant() {
    fn render(chunks: &[&[u8]]) -> (Vec<String>, usize, usize) {
        let mut screen = TerminalScreen::default();
        let mut state = terminal_state();
        screen.resize_grid(40, 4);
        screen.apply_bytes(&mut state, b"candidate\x1b[s\x1b[2;8H");
        for chunk in chunks {
            screen.apply_bytes(&mut state, chunk);
        }
        (
            state.grid_lines().to_vec(),
            state.screen_cursor_row,
            state.screen_cursor_col,
        )
    }

    let whole = render(&[b"\x1b[>1u\x1b[?u\x1b[<uaccepted"]);
    let split = render(&[b"\x1b[>", b"1u\x1b[", b"?u\x1b", b"[<", b"uaccepted"]);
    assert_eq!(split, whole);
    assert_eq!(whole.1, 1);
    assert_eq!(whole.2, 15);
}

#[test]
fn reverse_index_moves_cursor_up_without_scrolling() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"one\ntwo\x1b[2;2H\x1bMZ");
    assert_eq!(state.grid_lines(), vec!["oZe", "two"]);
}

#[test]
fn alternate_screen_restores_main_scrollback() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(
        &mut state,
        b"prompt> keep\x1b[?1049hmenu\nrow\x1b[?1049l cmd",
    );
    assert_eq!(state.grid_lines(), vec!["prompt> keep cmd"]);
}

#[test]
fn scroll_region_linefeed_only_scrolls_margin_rows() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\nmid\nbot\x1b[2;3r\x1b[3;1H\nX");
    assert_eq!(state.grid_lines(), vec!["top", "bot", "X"]);
    screen.apply_bytes(&mut state, b"\x1b[r\x1b[3;1H\nend");
    assert_eq!(state.grid_lines(), vec!["top", "bot", "X", "end"]);
}

#[test]
fn known_height_scroll_region_clamps_to_bottom_margin() {
    let mut screen = TerminalScreen::default();
    screen.resize_grid(10, 3);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\nmid\nbot\x1b[2;999r\x1b[3;1H\nX");
    assert_eq!(state.grid_lines(), vec!["top", "bot", "X"]);
}

#[test]
fn insert_and_delete_line_respect_scroll_region() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\none\ntwo\nbot\x1b[2;3r\x1b[2;1H\x1b[Lnew");
    assert_eq!(state.grid_lines(), vec!["top", "new", "one", "bot"]);
    screen.apply_bytes(&mut state, b"\x1b[2;1H\x1b[M");
    assert_eq!(state.grid_lines(), vec!["top", "one", "", "bot"]);
}

#[test]
fn known_height_line_operations_use_visible_screen_region() {
    let mut screen = TerminalScreen::default();
    screen.resize_grid(5, 3);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\x1b[1;1H\x1b[LZ");
    assert_eq!(state.grid_lines(), vec!["Z", "top", ""]);

    let mut screen = TerminalScreen::default();
    screen.resize_grid(5, 3);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\nmid\x1b[1;1H\x1b[MZ");
    assert_eq!(state.grid_lines(), vec!["Zid", "", ""]);

    let mut screen = TerminalScreen::default();
    screen.resize_grid(5, 3);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\x1b[1S");
    assert_eq!(state.grid_lines(), vec!["", "", ""]);

    let mut screen = TerminalScreen::default();
    screen.resize_grid(5, 3);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\nmid\x1b[1Tz");
    assert_eq!(state.grid_lines(), vec!["", "topz", "mid"]);
}

#[test]
fn scroll_up_and_down_shift_active_screen_rows() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"one\ntwo\nthree\x1b[1S");
    assert_eq!(state.grid_lines(), vec!["two", "three", ""]);
    screen.apply_bytes(&mut state, b"\x1b[1T\x1b[1;1Hone");
    assert_eq!(state.grid_lines(), vec!["one", "two", "three"]);
}

#[test]
fn scroll_up_and_down_respect_scroll_region() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\none\ntwo\nbot\x1b[2;3r\x1b[1S");
    assert_eq!(state.grid_lines(), vec!["top", "two", "", "bot"]);
    screen.apply_bytes(&mut state, b"\x1b[1T\x1b[2;1Hone");
    assert_eq!(state.grid_lines(), vec!["top", "one", "two", "bot"]);
}

#[test]
fn reverse_index_scrolls_only_region_at_top_margin() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\nmid\nbot\x1b[2;3r\x1b[2;1H\x1bMZ");
    assert_eq!(state.grid_lines(), vec!["top", "Z", "mid"]);
}

#[test]
fn printable_text_wraps_at_terminal_columns() {
    let mut screen = TerminalScreen::default();
    screen.resize(5);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcde");
    assert_eq!(state.grid_lines(), vec!["abcde"]);
    screen.apply_bytes(&mut state, b"fghi");
    assert_eq!(state.grid_lines(), vec!["abcde", "fghi"]);
}

#[test]
fn carriage_return_clears_pending_wrap_at_last_column() {
    let mut screen = TerminalScreen::default();
    screen.resize(5);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"abcde\rXY");
    assert_eq!(state.grid_lines(), vec!["XYcde"]);
}

#[test]
fn autowrap_at_scroll_region_bottom_scrolls_only_region() {
    let mut screen = TerminalScreen::default();
    screen.resize(3);
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\nmid\nbot\x1b[2;3r\x1b[3;1HABCd");
    assert_eq!(state.grid_lines(), vec!["top", "ABC", "d"]);
}

#[test]
fn erase_display_modes_clear_terminal_rows() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"stale\nrows\x1b[2Jfresh");
    assert_eq!(state.grid_lines(), vec!["", "    fresh"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"top\nbottom\x1b[1A\x1b[2G\x1b[J");
    assert_eq!(state.grid_lines(), vec!["t"]);
}

#[test]
fn split_csi_sequence_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"\x1b[");
    screen.apply_bytes(&mut state, b"31mred");
    assert_eq!(state.grid_lines(), vec!["red"]);
}

#[test]
fn osc_sequences_do_not_leak_into_terminal_rows() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"prompt \x1b]0;datum gui\x07ready");
    assert_eq!(state.grid_lines(), vec!["prompt ready"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"a\x1b]2;ignored\x1b\\b");
    assert_eq!(state.grid_lines(), vec!["ab"]);
}

#[test]
fn split_osc_sequence_does_not_leak_bytes() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"pre\x1b]0;dat");
    assert_eq!(state.grid_lines(), vec!["pre"]);
    screen.apply_bytes(&mut state, b"um\x07post");
    assert_eq!(state.grid_lines(), vec!["prepost"]);

    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"a\x1b]2;ignored\x1b");
    assert_eq!(state.grid_lines(), vec!["a"]);
    screen.apply_bytes(&mut state, b"\\b");
    assert_eq!(state.grid_lines(), vec!["ab"]);
}

#[test]
fn split_utf8_sequence_decodes_once_complete() {
    let mut screen = TerminalScreen::default();
    let mut state = terminal_state();
    screen.apply_bytes(&mut state, b"ok \xe2");
    assert_eq!(state.grid_lines(), vec!["ok "]);
    screen.apply_bytes(&mut state, b"\x9c\x93");
    assert_eq!(state.grid_lines(), vec!["ok \u{2713}"]);
}
