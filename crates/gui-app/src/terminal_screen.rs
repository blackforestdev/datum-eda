use datum_gui_protocol::TerminalLaneState;

const MAX_TERMINAL_ROWS: usize = 240;

#[derive(Default)]
pub(super) struct TerminalScreen {
    escape: EscapeState,
    utf8_pending: Vec<u8>,
    cursor_row: usize,
    cursor_col: usize,
}

impl TerminalScreen {
    pub(super) fn apply_bytes(&mut self, state: &mut TerminalLaneState, bytes: &[u8]) {
        ensure_row(state);
        self.cursor_row = self.cursor_row.min(state.lines.len().saturating_sub(1));
        for byte in bytes {
            if self
                .escape
                .consume(*byte, state, &mut self.cursor_row, &mut self.cursor_col)
            {
                continue;
            }
            match *byte {
                b'\x1b' => {
                    self.flush_utf8(state);
                    self.escape.start();
                }
                b'\r' => {
                    self.flush_utf8(state);
                    self.cursor_col = 0;
                }
                b'\n' => {
                    self.flush_utf8(state);
                    self.cursor_row += 1;
                    if self.cursor_row >= state.lines.len() {
                        state.lines.push(String::new());
                    }
                    self.cursor_col = 0;
                }
                0x08 => {
                    self.flush_utf8(state);
                    self.cursor_col = self.cursor_col.saturating_sub(1);
                }
                0x7f => {
                    self.flush_utf8(state);
                    remove_char_at(row_mut_at(state, self.cursor_row), self.cursor_col);
                }
                b'\t' => {
                    self.flush_utf8(state);
                    for _ in 0..4 {
                        self.put_char(state, ' ');
                    }
                }
                byte if byte >= 0x20 => {
                    self.utf8_pending.push(byte);
                    self.flush_utf8(state);
                }
                _ => {
                    self.flush_utf8(state);
                }
            }
            trim_rows(state);
            self.cursor_row = self.cursor_row.min(state.lines.len().saturating_sub(1));
        }
        state.scroll_offset = 0;
    }

    fn flush_utf8(&mut self, state: &mut TerminalLaneState) {
        loop {
            if self.utf8_pending.is_empty() {
                return;
            }
            let pending = std::mem::take(&mut self.utf8_pending);
            match std::str::from_utf8(&pending) {
                Ok(text) => {
                    for ch in text.chars() {
                        self.put_char(state, ch);
                    }
                    return;
                }
                Err(error) => {
                    let valid_up_to = error.valid_up_to();
                    if valid_up_to > 0 {
                        let valid = std::str::from_utf8(&pending[..valid_up_to])
                            .expect("valid prefix should decode");
                        for ch in valid.chars() {
                            self.put_char(state, ch);
                        }
                    }
                    match error.error_len() {
                        Some(len) => {
                            self.put_char(state, '\u{fffd}');
                            self.utf8_pending = pending[valid_up_to + len..].to_vec();
                        }
                        None => {
                            self.utf8_pending = pending[valid_up_to..].to_vec();
                            return;
                        }
                    }
                }
            }
        }
    }

    fn put_char(&mut self, state: &mut TerminalLaneState, ch: char) {
        put_char_at(row_mut_at(state, self.cursor_row), self.cursor_col, ch);
        self.cursor_col += 1;
    }
}

pub(super) fn terminal_scrollback_copy_text(state: &TerminalLaneState) -> Option<String> {
    let mut lines = state.lines.as_slice();
    while matches!(lines.last(), Some(line) if line.is_empty()) {
        lines = &lines[..lines.len().saturating_sub(1)];
    }
    if lines.is_empty() {
        return None;
    }
    Some(lines.join("\n"))
}

#[derive(Default)]
struct EscapeState {
    active: bool,
    csi: bool,
    params: Vec<u8>,
}

impl EscapeState {
    fn start(&mut self) {
        self.active = true;
        self.csi = false;
        self.params.clear();
    }

    fn consume(
        &mut self,
        byte: u8,
        state: &mut TerminalLaneState,
        cursor_row: &mut usize,
        cursor_col: &mut usize,
    ) -> bool {
        if !self.active {
            return false;
        }
        if !self.csi && byte == b'[' {
            self.csi = true;
            return true;
        }
        if self.csi {
            if (0x40..=0x7e).contains(&byte) {
                self.apply_csi(byte, state, cursor_row, cursor_col);
                self.active = false;
                self.csi = false;
                self.params.clear();
            } else {
                self.params.push(byte);
            }
            return true;
        }
        self.active = false;
        true
    }

    fn apply_csi(
        &self,
        final_byte: u8,
        state: &mut TerminalLaneState,
        cursor_row: &mut usize,
        cursor_col: &mut usize,
    ) {
        match final_byte {
            b'A' => {
                *cursor_row = (*cursor_row).saturating_sub(self.first_param_or(1));
            }
            b'B' => {
                *cursor_row += self.first_param_or(1);
                ensure_row_at(state, *cursor_row);
            }
            b'C' => {
                *cursor_col += self.first_param_or(1);
            }
            b'D' => {
                *cursor_col = (*cursor_col).saturating_sub(self.first_param_or(1));
            }
            b'K' => match self.first_param_or(0) {
                0 => truncate_row_at(row_mut_at(state, *cursor_row), *cursor_col),
                1 => clear_row_to_cursor(row_mut_at(state, *cursor_row), *cursor_col),
                2 => {
                    row_mut_at(state, *cursor_row).clear();
                    *cursor_col = 0;
                }
                _ => {}
            },
            b'J' => match self.first_param_or(0) {
                0 => clear_from_cursor_to_end(state, *cursor_row, *cursor_col),
                1 => clear_start_to_cursor(state, *cursor_row, *cursor_col),
                2 | 3 => {
                    state.lines.clear();
                    state.lines.push(String::new());
                    *cursor_row = 0;
                    *cursor_col = 0;
                }
                _ => {}
            },
            b'G' => {
                *cursor_col = self.first_param_or(1).saturating_sub(1);
            }
            b'H' | b'f' => {
                let (row, col) = self.row_col_params();
                *cursor_row = row.saturating_sub(1);
                *cursor_col = col.saturating_sub(1);
                ensure_row_at(state, *cursor_row);
            }
            _ => {}
        }
    }

    fn first_param_or(&self, default: usize) -> usize {
        std::str::from_utf8(&self.params)
            .ok()
            .and_then(|params| params.split(';').next())
            .and_then(|param| {
                if param.is_empty() {
                    None
                } else {
                    param.parse::<usize>().ok()
                }
            })
            .unwrap_or(default)
    }

    fn row_col_params(&self) -> (usize, usize) {
        let mut parts = std::str::from_utf8(&self.params)
            .unwrap_or("")
            .split(';')
            .map(|param| {
                if param.is_empty() {
                    None
                } else {
                    param.parse::<usize>().ok()
                }
                .unwrap_or(1)
            });
        (parts.next().unwrap_or(1), parts.next().unwrap_or(1))
    }
}

fn ensure_row(state: &mut TerminalLaneState) {
    if state.lines.is_empty() {
        state.lines.push(String::new());
    }
}

fn ensure_row_at(state: &mut TerminalLaneState, row: usize) {
    ensure_row(state);
    while state.lines.len() <= row {
        state.lines.push(String::new());
    }
}

fn row_mut_at(state: &mut TerminalLaneState, row: usize) -> &mut String {
    ensure_row_at(state, row);
    state.lines.get_mut(row).expect("terminal row should exist")
}

fn put_char_at(row: &mut String, column: usize, ch: char) {
    let len = row.chars().count();
    if column > len {
        row.push_str(&" ".repeat(column - len));
    }
    if column == row.chars().count() {
        row.push(ch);
        return;
    }
    let start = char_to_byte_pos(row, column);
    let end = char_to_byte_pos(row, column + 1);
    row.replace_range(start..end, &ch.to_string());
}

fn remove_char_at(row: &mut String, column: usize) {
    if column >= row.chars().count() {
        return;
    }
    let start = char_to_byte_pos(row, column);
    let end = char_to_byte_pos(row, column + 1);
    row.replace_range(start..end, "");
}

fn truncate_row_at(row: &mut String, column: usize) {
    let end = char_to_byte_pos(row, column.min(row.chars().count()));
    row.truncate(end);
}

fn clear_row_to_cursor(row: &mut String, column: usize) {
    let end_col = column.saturating_add(1).min(row.chars().count());
    let end = char_to_byte_pos(row, end_col);
    row.replace_range(0..end, &" ".repeat(end_col));
}

fn clear_from_cursor_to_end(state: &mut TerminalLaneState, row: usize, column: usize) {
    truncate_row_at(row_mut_at(state, row), column);
    state.lines.truncate(row.saturating_add(1));
}

fn clear_start_to_cursor(state: &mut TerminalLaneState, row: usize, column: usize) {
    let end_row = row.min(state.lines.len().saturating_sub(1));
    for line in state.lines.iter_mut().take(end_row) {
        line.clear();
    }
    clear_row_to_cursor(row_mut_at(state, end_row), column);
}

fn char_to_byte_pos(s: &str, char_index: usize) -> usize {
    s.char_indices()
        .nth(char_index)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

fn trim_rows(state: &mut TerminalLaneState) {
    if state.lines.len() > MAX_TERMINAL_ROWS {
        let overflow = state.lines.len() - MAX_TERMINAL_ROWS;
        state.lines.drain(0..overflow);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terminal_state() -> TerminalLaneState {
        TerminalLaneState {
            lines: Vec::new(),
            activity_summary: Vec::new(),
            input: String::new(),
            cursor: 0,
            scroll_offset: 0,
            status: "running".to_string(),
        }
    }

    #[test]
    fn applies_partial_prompt_without_newline() {
        let mut screen = TerminalScreen::default();
        let mut state = terminal_state();
        screen.apply_bytes(&mut state, b"datum$ ");
        assert_eq!(state.lines, vec!["datum$ "]);
    }

    #[test]
    fn applies_carriage_return_as_column_rewrite() {
        let mut screen = TerminalScreen::default();
        let mut state = terminal_state();
        screen.apply_bytes(&mut state, b"abcdef\rXY");
        assert_eq!(state.lines, vec!["XYcdef"]);
    }

    #[test]
    fn erase_in_line_clears_from_cursor() {
        let mut screen = TerminalScreen::default();
        let mut state = terminal_state();
        screen.apply_bytes(&mut state, b"building 10%\rbuilding 20%\x1b[K");
        assert_eq!(state.lines, vec!["building 20%"]);
    }

    #[test]
    fn erase_in_line_modes_clear_whole_line_or_prefix() {
        let mut screen = TerminalScreen::default();
        let mut state = terminal_state();
        screen.apply_bytes(&mut state, b"abcdef\x1b[3D\x1b[1KZ");
        assert_eq!(state.lines, vec!["   Zef"]);
        screen.apply_bytes(&mut state, b"\r\x1b[2KXY");
        assert_eq!(state.lines, vec!["XY"]);
    }

    #[test]
    fn cursor_left_and_right_support_progress_rewrites() {
        let mut screen = TerminalScreen::default();
        let mut state = terminal_state();
        screen.apply_bytes(&mut state, b"abcdef\x1b[3D\x1b[KXY");
        assert_eq!(state.lines, vec!["abcXY"]);

        let mut screen = TerminalScreen::default();
        let mut state = terminal_state();
        screen.apply_bytes(&mut state, b"ab\r\x1b[4Cz");
        assert_eq!(state.lines, vec!["ab  z"]);
    }

    #[test]
    fn cursor_up_and_down_rewrite_addressed_rows() {
        let mut screen = TerminalScreen::default();
        let mut state = terminal_state();
        screen.apply_bytes(&mut state, b"one\ntwo\x1b[1A\rONE\x1b[1B\rTWO");
        assert_eq!(state.lines, vec!["ONE", "TWO"]);
    }

    #[test]
    fn cursor_position_addresses_screen_rows_and_columns() {
        let mut screen = TerminalScreen::default();
        let mut state = terminal_state();
        screen.apply_bytes(&mut state, b"alpha\nbeta\x1b[1;3HZ");
        assert_eq!(state.lines, vec!["alZha", "beta"]);
    }

    #[test]
    fn erase_display_modes_clear_terminal_rows() {
        let mut screen = TerminalScreen::default();
        let mut state = terminal_state();
        screen.apply_bytes(&mut state, b"stale\nrows\x1b[2Jfresh");
        assert_eq!(state.lines, vec!["fresh"]);

        let mut screen = TerminalScreen::default();
        let mut state = terminal_state();
        screen.apply_bytes(&mut state, b"top\nbottom\x1b[1A\x1b[2G\x1b[J");
        assert_eq!(state.lines, vec!["t"]);
    }

    #[test]
    fn sgr_sequences_do_not_leak_into_terminal_rows() {
        let mut screen = TerminalScreen::default();
        let mut state = terminal_state();
        screen.apply_bytes(&mut state, b"\x1b[1;31mred\x1b[0m plain");
        assert_eq!(state.lines, vec!["red plain"]);
    }

    #[test]
    fn split_csi_sequence_does_not_leak_bytes() {
        let mut screen = TerminalScreen::default();
        let mut state = terminal_state();
        screen.apply_bytes(&mut state, b"\x1b[");
        screen.apply_bytes(&mut state, b"31mred");
        assert_eq!(state.lines, vec!["red"]);
    }

    #[test]
    fn split_utf8_sequence_decodes_once_complete() {
        let mut screen = TerminalScreen::default();
        let mut state = terminal_state();
        screen.apply_bytes(&mut state, b"ok \xe2");
        assert_eq!(state.lines, vec!["ok "]);
        screen.apply_bytes(&mut state, b"\x9c\x93");
        assert_eq!(state.lines, vec!["ok \u{2713}"]);
    }

    #[test]
    fn terminal_scrollback_copy_text_joins_rows_and_trims_blank_tail() {
        let mut state = terminal_state();
        state.lines = vec!["first".to_string(), "second".to_string(), String::new()];
        assert_eq!(
            terminal_scrollback_copy_text(&state).as_deref(),
            Some("first\nsecond")
        );

        state.lines = vec![String::new()];
        assert_eq!(terminal_scrollback_copy_text(&state), None);
    }
}
