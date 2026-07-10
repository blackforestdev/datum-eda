use datum_gui_protocol::{TerminalLaneState, TerminalTextStyle};
use std::collections::BTreeSet;
mod terminal_cursor;
mod terminal_escape;
mod terminal_grid;
mod terminal_intermediate;
mod terminal_osc;
mod terminal_sgr;
mod terminal_status;
mod terminal_style;

use terminal_escape::{EscapeState, SavedTerminalScreen};
use terminal_grid::{
    insert_blank_chars_at, scroll_region_down, scroll_region_up, shift_delete_style,
    shift_insert_style,
};
use terminal_style::{clear_style_range, set_style_at, sync_styled_lines};

const MAX_TERMINAL_ROWS: usize = 240;

pub(super) struct TerminalScreen {
    escape: EscapeState,
    utf8_pending: Vec<u8>,
    cursor_row: usize,
    cursor_col: usize,
    columns: Option<usize>,
    rows: Option<usize>,
    pending_wrap: bool,
    saved_cursor: Option<(usize, usize)>,
    alternate_screen: Option<SavedTerminalScreen>,
    scroll_region: Option<(usize, usize)>,
    last_printable: Option<char>,
    bracketed_paste: bool,
    autowrap: bool,
    origin_mode: bool,
    insert_mode: bool,
    cursor_visible: bool,
    tab_stops: TerminalTabStops,
    current_style: TerminalTextStyle,
    title_stack: Vec<Option<String>>,
}

impl Default for TerminalScreen {
    fn default() -> Self {
        Self {
            escape: EscapeState::default(),
            utf8_pending: Vec::new(),
            cursor_row: 0,
            cursor_col: 0,
            columns: None,
            rows: None,
            pending_wrap: false,
            saved_cursor: None,
            alternate_screen: None,
            scroll_region: None,
            last_printable: None,
            bracketed_paste: false,
            autowrap: true,
            origin_mode: false,
            insert_mode: false,
            cursor_visible: true,
            tab_stops: TerminalTabStops::default(),
            current_style: TerminalTextStyle::default(),
            title_stack: Vec::new(),
        }
    }
}

impl TerminalScreen {
    pub(super) fn bracketed_paste_enabled(&self) -> bool {
        self.bracketed_paste
    }

    pub(super) fn resize(&mut self, columns: u16) {
        self.columns = Some(columns.max(1) as usize);
        if let Some(columns) = self.columns
            && self.cursor_col > columns
        {
            self.cursor_col = columns;
        }
    }

    pub(super) fn resize_grid(&mut self, columns: u16, rows: u16) {
        self.resize(columns);
        self.rows = Some(rows.max(1) as usize);
        if let Some(rows) = self.rows {
            self.cursor_row = self.cursor_row.min(rows.saturating_sub(1));
        }
    }

    #[allow(dead_code)]
    pub(super) fn apply_bytes(&mut self, state: &mut TerminalLaneState, bytes: &[u8]) {
        let _ = self.apply_bytes_with_responses(state, bytes);
    }

    pub(super) fn apply_bytes_with_responses(
        &mut self,
        state: &mut TerminalLaneState,
        bytes: &[u8],
    ) -> Vec<Vec<u8>> {
        let mut responses = Vec::new();
        ensure_row(state);
        self.cursor_row = self.cursor_row.min(state.lines.len().saturating_sub(1));
        for byte in bytes {
            if self.escape.consume(
                *byte,
                state,
                &mut self.cursor_row,
                &mut self.cursor_col,
                &mut self.saved_cursor,
                &mut self.alternate_screen,
                &mut self.scroll_region,
                &mut self.pending_wrap,
                self.columns,
                self.rows,
                &mut self.last_printable,
                &mut self.bracketed_paste,
                &mut self.autowrap,
                &mut self.origin_mode,
                &mut self.insert_mode,
                &mut self.cursor_visible,
                &mut self.tab_stops,
                &mut self.current_style,
                &mut self.title_stack,
                &mut responses,
            ) {
                continue;
            }
            match *byte {
                b'\x1b' => {
                    self.flush_utf8(state);
                    self.escape.start();
                }
                0x84 if self.utf8_pending.is_empty() => {
                    self.flush_utf8(state);
                    self.pending_wrap = false;
                    self.linefeed(state);
                }
                0x85 if self.utf8_pending.is_empty() => {
                    self.flush_utf8(state);
                    self.pending_wrap = false;
                    self.linefeed(state);
                    self.cursor_col = 0;
                }
                0x8d if self.utf8_pending.is_empty() => {
                    self.flush_utf8(state);
                    self.pending_wrap = false;
                    reverse_index(state, &mut self.cursor_row, &self.scroll_region);
                }
                0x90 | 0x98 | 0x9e | 0x9f if self.utf8_pending.is_empty() => {
                    self.flush_utf8(state);
                    self.escape.start_st_control_string();
                }
                0x9b if self.utf8_pending.is_empty() => {
                    self.flush_utf8(state);
                    self.escape.start_csi();
                }
                0x9d if self.utf8_pending.is_empty() => {
                    self.flush_utf8(state);
                    self.escape.start_osc();
                }
                b'\r' => {
                    self.flush_utf8(state);
                    self.pending_wrap = false;
                    self.cursor_col = 0;
                }
                b'\n' => {
                    self.flush_utf8(state);
                    self.pending_wrap = false;
                    self.linefeed(state);
                    self.cursor_col = 0;
                }
                0x0b | 0x0c => {
                    self.flush_utf8(state);
                    self.pending_wrap = false;
                    self.linefeed(state);
                }
                0x08 => {
                    self.flush_utf8(state);
                    self.pending_wrap = false;
                    self.cursor_col = self.cursor_col.saturating_sub(1);
                }
                0x07 => {
                    self.flush_utf8(state);
                    state.bell_count = state.bell_count.saturating_add(1);
                }
                0x7f => {
                    self.flush_utf8(state);
                    self.pending_wrap = false;
                    remove_char_at_with_style(state, self.cursor_row, self.cursor_col);
                }
                b'\t' => {
                    self.flush_utf8(state);
                    let next = self.tab_stops.next_after(self.cursor_col, self.columns);
                    for _ in self.cursor_col..next {
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
        self.sync_cursor_state(state);
        state.scroll_offset = 0;
        responses
    }

    fn sync_cursor_state(&self, state: &mut TerminalLaneState) {
        state.screen_cursor_row = self.cursor_row;
        state.screen_cursor_col = self.cursor_col;
        state.screen_cursor_visible = self.cursor_visible;
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
        put_char_with_cursor(
            state,
            &mut self.cursor_row,
            &mut self.cursor_col,
            &mut self.pending_wrap,
            self.columns,
            &self.scroll_region,
            self.autowrap,
            self.insert_mode,
            &self.current_style,
            ch,
        );
        self.last_printable = Some(ch);
    }

    fn linefeed(&mut self, state: &mut TerminalLaneState) {
        self.pending_wrap = false;
        if let Some((top, bottom)) = self.scroll_region
            && self.cursor_row == bottom
        {
            scroll_region_up(state, top, bottom);
            return;
        }
        self.cursor_row += 1;
        ensure_row_at(state, self.cursor_row);
    }
}

#[derive(Clone)]
pub(super) struct TerminalTabStops {
    custom: BTreeSet<usize>,
    cleared_defaults: BTreeSet<usize>,
    defaults_enabled: bool,
}

impl Default for TerminalTabStops {
    fn default() -> Self {
        Self {
            custom: BTreeSet::new(),
            cleared_defaults: BTreeSet::new(),
            defaults_enabled: true,
        }
    }
}

impl TerminalTabStops {
    fn is_stop(&self, column: usize) -> bool {
        self.custom.contains(&column)
            || (self.defaults_enabled
                && column.is_multiple_of(8)
                && !self.cleared_defaults.contains(&column))
    }

    pub(super) fn set(&mut self, column: usize) {
        self.custom.insert(column);
        self.cleared_defaults.remove(&column);
    }

    pub(super) fn clear_current(&mut self, column: usize) {
        self.custom.remove(&column);
        if column.is_multiple_of(8) {
            self.cleared_defaults.insert(column);
        }
    }

    pub(super) fn clear_all(&mut self) {
        self.custom.clear();
        self.cleared_defaults.clear();
        self.defaults_enabled = false;
    }

    pub(super) fn reset(&mut self) {
        self.custom.clear();
        self.cleared_defaults.clear();
        self.defaults_enabled = true;
    }

    pub(super) fn next_after(&self, column: usize, columns: Option<usize>) -> usize {
        let limit = columns
            .map(|columns| columns.saturating_sub(1))
            .filter(|last_column| *last_column > column)
            .unwrap_or_else(|| column.saturating_add(256));
        ((column + 1)..=limit)
            .find(|candidate| self.is_stop(*candidate))
            .unwrap_or(column)
    }

    pub(super) fn previous_before(&self, column: usize) -> usize {
        (0..column)
            .rev()
            .find(|candidate| self.is_stop(*candidate))
            .unwrap_or(0)
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

fn ensure_row(state: &mut TerminalLaneState) {
    if state.lines.is_empty() {
        state.lines.push(String::new());
    }
    sync_styled_lines(state);
}

fn ensure_row_at(state: &mut TerminalLaneState, row: usize) {
    ensure_row(state);
    while state.lines.len() <= row {
        state.lines.push(String::new());
    }
    sync_styled_lines(state);
}

pub(super) fn row_mut_at(state: &mut TerminalLaneState, row: usize) -> &mut String {
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

// Terminal helper threads many escape/screen-state parameters.
#[allow(clippy::too_many_arguments)]
fn put_char_with_cursor(
    state: &mut TerminalLaneState,
    cursor_row: &mut usize,
    cursor_col: &mut usize,
    pending_wrap: &mut bool,
    columns: Option<usize>,
    scroll_region: &Option<(usize, usize)>,
    autowrap: bool,
    insert_mode: bool,
    style: &TerminalTextStyle,
    ch: char,
) {
    if *pending_wrap {
        linefeed_with_cursor(state, cursor_row, scroll_region);
        *cursor_col = 0;
        *pending_wrap = false;
    }
    if !autowrap && let Some(columns) = columns {
        *cursor_col = (*cursor_col).min(columns.saturating_sub(1));
    }
    if insert_mode {
        insert_blank_chars_at(row_mut_at(state, *cursor_row), *cursor_col, 1);
        shift_insert_style(state, *cursor_row, *cursor_col, 1);
    }
    put_char_at(row_mut_at(state, *cursor_row), *cursor_col, ch);
    set_style_at(state, *cursor_row, *cursor_col, style);
    *cursor_col += 1;
    if let Some(columns) = columns
        && *cursor_col >= columns
    {
        if autowrap {
            *cursor_col = columns;
            *pending_wrap = true;
        } else {
            *cursor_col = columns.saturating_sub(1);
            *pending_wrap = false;
        }
    }
}

pub(super) fn linefeed_with_cursor(
    state: &mut TerminalLaneState,
    cursor_row: &mut usize,
    scroll_region: &Option<(usize, usize)>,
) {
    if let Some((top, bottom)) = scroll_region
        && *cursor_row == *bottom
    {
        scroll_region_up(state, *top, *bottom);
        return;
    }
    *cursor_row += 1;
    ensure_row_at(state, *cursor_row);
}

fn remove_char_at(row: &mut String, column: usize) {
    if column >= row.chars().count() {
        return;
    }
    let start = char_to_byte_pos(row, column);
    let end = char_to_byte_pos(row, column + 1);
    row.replace_range(start..end, "");
}

pub(super) fn remove_char_at_with_style(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
) {
    remove_char_at(row_mut_at(state, row_index), column);
    shift_delete_style(state, row_index, column, 1);
}

fn erase_chars_at(row: &mut String, column: usize, count: usize) {
    let row_len = row.chars().count();
    if column >= row_len || count == 0 {
        return;
    }
    let end_col = column.saturating_add(count).min(row_len);
    let start = char_to_byte_pos(row, column);
    let end = char_to_byte_pos(row, end_col);
    row.replace_range(start..end, &" ".repeat(end_col - column));
}

pub(super) fn erase_chars_at_with_style(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
    count: usize,
) {
    erase_chars_at(row_mut_at(state, row_index), column, count);
    clear_style_range(state, row_index, column, column.saturating_add(count));
}

fn truncate_row_at(row: &mut String, column: usize) {
    let end = char_to_byte_pos(row, column.min(row.chars().count()));
    row.truncate(end);
}

pub(super) fn truncate_row_at_with_style(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
) {
    truncate_row_at(row_mut_at(state, row_index), column);
    if let Some(row) = state.styled_lines.get_mut(row_index) {
        row.spans.retain_mut(|span| {
            if span.start >= column {
                return false;
            }
            span.end = span.end.min(column);
            span.start < span.end
        });
    }
    sync_styled_lines(state);
}

fn clear_row_to_cursor(row: &mut String, column: usize) {
    let end_col = column.saturating_add(1).min(row.chars().count());
    let end = char_to_byte_pos(row, end_col);
    row.replace_range(0..end, &" ".repeat(end_col));
}

pub(super) fn clear_row_to_cursor_with_style(
    state: &mut TerminalLaneState,
    row_index: usize,
    column: usize,
) {
    clear_row_to_cursor(row_mut_at(state, row_index), column);
    clear_style_range(state, row_index, 0, column.saturating_add(1));
}

pub(super) fn clear_from_cursor_to_end(state: &mut TerminalLaneState, row: usize, column: usize) {
    truncate_row_at_with_style(state, row, column);
    state.lines.truncate(row.saturating_add(1));
    state.styled_lines.truncate(state.lines.len());
}

pub(super) fn clear_start_to_cursor(state: &mut TerminalLaneState, row: usize, column: usize) {
    let end_row = row.min(state.lines.len().saturating_sub(1));
    for index in 0..end_row {
        state.lines[index].clear();
        if let Some(styled) = state.styled_lines.get_mut(index) {
            styled.text.clear();
            styled.spans.clear();
        }
    }
    clear_row_to_cursor_with_style(state, end_row, column);
}

fn reverse_index(
    state: &mut TerminalLaneState,
    cursor_row: &mut usize,
    scroll_region: &Option<(usize, usize)>,
) {
    let (top, bottom) = scroll_region.unwrap_or((0, state.lines.len().saturating_sub(1)));
    ensure_row_at(state, bottom);
    if *cursor_row == top {
        scroll_region_down(state, top, bottom);
    } else {
        *cursor_row = (*cursor_row).saturating_sub(1);
    }
}

fn previous_tab_stop(tab_stops: &TerminalTabStops, mut column: usize, count: usize) -> usize {
    for _ in 0..count {
        column = tab_stops.previous_before(column);
    }
    column
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
        state.styled_lines.drain(0..overflow);
    }
    sync_styled_lines(state);
}

#[cfg(test)]
mod terminal_screen_basic_tests;

#[cfg(test)]
mod terminal_screen_c1_tests;
#[cfg(test)]
mod terminal_screen_charset_tests;
#[cfg(test)]
mod terminal_screen_control_string_tests;
#[cfg(test)]
mod terminal_screen_erase_tests;
#[cfg(test)]
mod terminal_screen_horizontal_position_tests;
#[cfg(test)]
mod terminal_screen_index_tests;
#[cfg(test)]
mod terminal_screen_insert_mode_tests;
#[cfg(test)]
mod terminal_screen_private_mode_tests;
#[cfg(test)]
mod terminal_screen_rep_tests;
#[cfg(test)]
mod terminal_screen_reset_tests;
#[cfg(test)]
mod terminal_screen_sgr_tests;
#[cfg(test)]
mod terminal_screen_status_tests;
#[cfg(test)]
mod terminal_screen_tab_stop_tests;
#[cfg(test)]
mod terminal_screen_vertical_position_tests;
#[cfg(test)]
mod terminal_scrollback_tests;
