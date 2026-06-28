use datum_gui_protocol::{TerminalLaneState, TerminalStyledLine, TerminalTextStyle};

use super::terminal_cursor::{addressed_row, clamp_column, clamp_row};
use super::terminal_grid::{
    apply_delete_chars, apply_erase_chars, apply_erase_display, apply_erase_line,
    apply_insert_chars, apply_line_operation,
};
use super::terminal_intermediate::apply_escape_intermediate;
use super::terminal_osc::apply_osc_payload;
use super::terminal_sgr::apply_sgr;
use super::terminal_status::{cursor_position_report, window_report_response};
use super::{
    TerminalTabStops, ensure_row, ensure_row_at, linefeed_with_cursor, previous_tab_stop,
    put_char_with_cursor, reverse_index,
};

#[derive(Clone)]
pub(super) struct SavedTerminalScreen {
    pub(super) lines: Vec<String>,
    pub(super) styled_lines: Vec<TerminalStyledLine>,
    pub(super) cursor_row: usize,
    pub(super) cursor_col: usize,
}

#[derive(Default)]
pub(super) struct EscapeState {
    active: bool,
    csi: bool,
    osc: bool,
    osc_esc: bool,
    st_control_string: bool,
    st_control_string_esc: bool,
    charset_designation: bool,
    escape_intermediate: bool,
    params: Vec<u8>,
    osc_payload: Vec<u8>,
}

impl EscapeState {
    pub(super) fn start(&mut self) {
        self.active = true;
        self.csi = false;
        self.osc = false;
        self.osc_esc = false;
        self.st_control_string = false;
        self.st_control_string_esc = false;
        self.charset_designation = false;
        self.escape_intermediate = false;
        self.params.clear();
        self.osc_payload.clear();
    }

    pub(super) fn start_csi(&mut self) {
        self.start();
        self.csi = true;
    }

    pub(super) fn start_osc(&mut self) {
        self.start();
        self.osc = true;
    }

    pub(super) fn start_st_control_string(&mut self) {
        self.start();
        self.st_control_string = true;
    }

    pub(super) fn consume(
        &mut self,
        byte: u8,
        state: &mut TerminalLaneState,
        cursor_row: &mut usize,
        cursor_col: &mut usize,
        saved_cursor: &mut Option<(usize, usize)>,
        alternate_screen: &mut Option<SavedTerminalScreen>,
        scroll_region: &mut Option<(usize, usize)>,
        pending_wrap: &mut bool,
        columns: Option<usize>,
        rows: Option<usize>,
        last_printable: &mut Option<char>,
        bracketed_paste: &mut bool,
        autowrap: &mut bool,
        origin_mode: &mut bool,
        insert_mode: &mut bool,
        cursor_visible: &mut bool,
        tab_stops: &mut TerminalTabStops,
        current_style: &mut TerminalTextStyle,
        title_stack: &mut Vec<Option<String>>,
        responses: &mut Vec<Vec<u8>>,
    ) -> bool {
        if !self.active {
            return false;
        }
        if self.osc {
            self.consume_osc(byte, state);
            return true;
        }
        if self.st_control_string {
            self.consume_st_control_string(byte);
            return true;
        }
        if self.charset_designation {
            self.finish_control_string();
            return true;
        }
        if self.escape_intermediate {
            apply_escape_intermediate(byte, &self.params, state);
            self.finish_control_string();
            return true;
        }
        if !self.csi && byte == b'[' {
            self.csi = true;
            return true;
        }
        if !self.csi && byte == b']' {
            self.osc = true;
            self.osc_esc = false;
            return true;
        }
        if !self.csi && matches!(byte, b'P' | b'X' | b'^' | b'_') {
            self.st_control_string = true;
            self.st_control_string_esc = false;
            return true;
        }
        if !self.csi && matches!(byte, b'(' | b')' | b'*' | b'+' | b'-' | b'.' | b'/') {
            self.charset_designation = true;
            return true;
        }
        if !self.csi && byte == b'#' {
            self.escape_intermediate = true;
            self.params.push(byte);
            return true;
        }
        if self.csi {
            if (0x40..=0x7e).contains(&byte) {
                self.apply_csi(
                    byte,
                    state,
                    cursor_row,
                    cursor_col,
                    saved_cursor,
                    alternate_screen,
                    scroll_region,
                    pending_wrap,
                    columns,
                    rows,
                    last_printable,
                    bracketed_paste,
                    autowrap,
                    origin_mode,
                    insert_mode,
                    cursor_visible,
                    tab_stops,
                    current_style,
                    title_stack,
                    responses,
                );
                self.active = false;
                self.csi = false;
                self.params.clear();
            } else {
                self.params.push(byte);
            }
            return true;
        }
        match byte {
            b'D' => {
                *pending_wrap = false;
                linefeed_with_cursor(state, cursor_row, scroll_region);
            }
            b'E' => {
                *pending_wrap = false;
                linefeed_with_cursor(state, cursor_row, scroll_region);
                *cursor_col = 0;
            }
            b'7' => *saved_cursor = Some((*cursor_row, *cursor_col)),
            b'8' => {
                *pending_wrap = false;
                if let Some((row, col)) = *saved_cursor {
                    *cursor_row = row;
                    *cursor_col = col;
                    ensure_row_at(state, *cursor_row);
                }
            }
            b'=' => state.application_keypad = true,
            b'>' => state.application_keypad = false,
            b'M' => {
                *pending_wrap = false;
                reverse_index(state, cursor_row, scroll_region);
            }
            b'Z' => responses.push(b"\x1b[?1;2c".to_vec()),
            b'c' => {
                state.lines.clear();
                state.lines.push(String::new());
                state.styled_lines.clear();
                state.styled_lines.push(TerminalStyledLine::default());
                *cursor_row = 0;
                *cursor_col = 0;
                *saved_cursor = None;
                *alternate_screen = None;
                *scroll_region = None;
                *pending_wrap = false;
                *last_printable = None;
                *bracketed_paste = false;
                *autowrap = true;
                *origin_mode = false;
                *insert_mode = false;
                *cursor_visible = true;
                state.screen_cursor_style = None;
                state.application_cursor_keys = false;
                state.application_keypad = false;
                state.focus_event_reporting = false;
                state.mouse_reporting_mode = None;
                state.mouse_coordinate_encoding = None;
                *current_style = TerminalTextStyle::default();
                tab_stops.reset();
            }
            b'H' => {
                *pending_wrap = false;
                tab_stops.set(*cursor_col);
            }
            _ => {}
        }
        self.active = false;
        true
    }

    fn consume_osc(&mut self, byte: u8, state: &mut TerminalLaneState) {
        if self.osc_esc {
            if byte == b'\\' {
                self.apply_osc(state);
                self.finish_control_string();
            } else {
                self.osc_esc = byte == b'\x1b';
            }
            return;
        }
        match byte {
            0x07 | 0x9c => {
                self.apply_osc(state);
                self.finish_control_string();
            }
            b'\x1b' => self.osc_esc = true,
            _ if self.osc_payload.len() < 4096 => self.osc_payload.push(byte),
            _ => {}
        }
    }

    fn apply_osc(&self, state: &mut TerminalLaneState) {
        apply_osc_payload(&self.osc_payload, state);
    }

    fn consume_st_control_string(&mut self, byte: u8) {
        if self.st_control_string_esc {
            if byte == b'\\' {
                self.finish_control_string();
            } else {
                self.st_control_string_esc = byte == b'\x1b';
            }
            return;
        }
        match byte {
            0x9c => self.finish_control_string(),
            b'\x1b' => self.st_control_string_esc = true,
            _ => {}
        }
    }

    fn finish_control_string(&mut self) {
        self.active = false;
        self.csi = false;
        self.osc = false;
        self.osc_esc = false;
        self.st_control_string = false;
        self.st_control_string_esc = false;
        self.charset_designation = false;
        self.escape_intermediate = false;
        self.params.clear();
        self.osc_payload.clear();
    }

    fn apply_csi(
        &self,
        final_byte: u8,
        state: &mut TerminalLaneState,
        cursor_row: &mut usize,
        cursor_col: &mut usize,
        saved_cursor: &mut Option<(usize, usize)>,
        alternate_screen: &mut Option<SavedTerminalScreen>,
        scroll_region: &mut Option<(usize, usize)>,
        pending_wrap: &mut bool,
        columns: Option<usize>,
        rows: Option<usize>,
        last_printable: &mut Option<char>,
        bracketed_paste: &mut bool,
        autowrap: &mut bool,
        origin_mode: &mut bool,
        insert_mode: &mut bool,
        cursor_visible: &mut bool,
        tab_stops: &mut TerminalTabStops,
        current_style: &mut TerminalTextStyle,
        title_stack: &mut Vec<Option<String>>,
        responses: &mut Vec<Vec<u8>>,
    ) {
        *pending_wrap = false;
        match final_byte {
            b'@' => apply_insert_chars(
                state,
                *cursor_row,
                *cursor_col,
                self.first_param_or(1),
                columns,
            ),
            b'A' => *cursor_row = (*cursor_row).saturating_sub(self.first_param_or(1)),
            b'B' => {
                *cursor_row = clamp_row(*cursor_row + self.first_param_or(1), rows);
                ensure_row_at(state, *cursor_row);
            }
            b'C' => *cursor_col = clamp_column(*cursor_col + self.first_param_or(1), columns),
            b'D' => *cursor_col = (*cursor_col).saturating_sub(self.first_param_or(1)),
            b'E' => {
                *cursor_row = clamp_row(*cursor_row + self.first_param_or(1), rows);
                *cursor_col = 0;
                ensure_row_at(state, *cursor_row);
            }
            b'F' => {
                *cursor_row = (*cursor_row).saturating_sub(self.first_param_or(1));
                *cursor_col = 0;
            }
            b'd' => {
                *cursor_row =
                    addressed_row(self.first_param_or(1), scroll_region, *origin_mode, rows);
                ensure_row_at(state, *cursor_row);
            }
            b'e' => {
                *cursor_row = clamp_row(*cursor_row + self.first_param_or(1), rows);
                ensure_row_at(state, *cursor_row);
            }
            b'k' => *cursor_row = (*cursor_row).saturating_sub(self.first_param_or(1)),
            b'P' => apply_delete_chars(
                state,
                *cursor_row,
                *cursor_col,
                self.first_param_or(1),
                columns,
            ),
            b'X' => apply_erase_chars(
                state,
                *cursor_row,
                *cursor_col,
                self.first_param_or(1),
                columns,
            ),
            b'K' => apply_erase_line(
                state,
                *cursor_row,
                *cursor_col,
                self.first_param_or(0),
                columns,
            ),
            b'J' => {
                apply_erase_display(
                    state,
                    *cursor_row,
                    *cursor_col,
                    self.first_param_or(0),
                    columns,
                    rows,
                );
            }
            b'L' | b'M' | b'S' | b'T' => apply_line_operation(
                final_byte,
                state,
                *cursor_row,
                self.first_param_or(1),
                scroll_region,
                rows,
            ),
            b'G' => *cursor_col = clamp_column(self.first_param_or(1).saturating_sub(1), columns),
            b'`' => *cursor_col = clamp_column(self.first_param_or(1).saturating_sub(1), columns),
            b'a' => *cursor_col = clamp_column(*cursor_col + self.first_param_or(1), columns),
            b'I' => {
                for _ in 0..self.first_param_or(1).max(1) {
                    *cursor_col = tab_stops.next_after(*cursor_col, columns);
                }
            }
            b'Z' => *cursor_col = previous_tab_stop(tab_stops, *cursor_col, self.first_param_or(1)),
            b'b' => {
                if let Some(ch) = *last_printable {
                    for _ in 0..self.first_param_or(1) {
                        put_char_with_cursor(
                            state,
                            cursor_row,
                            cursor_col,
                            pending_wrap,
                            columns,
                            scroll_region,
                            *autowrap,
                            *insert_mode,
                            current_style,
                            ch,
                        );
                    }
                }
            }
            b'c' => match self.params.as_slice() {
                b"" | b"0" => responses.push(b"\x1b[?1;2c".to_vec()),
                params if params.starts_with(b">") => responses.push(b"\x1b[>0;0;0c".to_vec()),
                _ => {}
            },
            b'm' => apply_sgr(&self.params, current_style),
            b'n' => match self.params.as_slice() {
                b"5" => responses.push(b"\x1b[0n".to_vec()),
                b"6" => responses.push(cursor_position_report(*cursor_row, *cursor_col, false)),
                b"?6" => responses.push(cursor_position_report(*cursor_row, *cursor_col, true)),
                _ => {}
            },
            b'q' => {
                if let Some(style) = self.cursor_style_param() {
                    state.screen_cursor_style = Some(style.to_string());
                }
            }
            b't' => match self.first_param_or(0) {
                22 => title_stack.push(state.title.clone()),
                23 => {
                    if let Some(title) = title_stack.pop() {
                        state.title = title;
                    }
                }
                param => {
                    if let Some(response) = window_report_response(param, state) {
                        responses.push(response);
                    }
                }
            },
            b'H' | b'f' => {
                let (row, col) = self.row_col_params();
                *cursor_row = addressed_row(row, scroll_region, *origin_mode, rows);
                *cursor_col = clamp_column(col.saturating_sub(1), columns);
                ensure_row_at(state, *cursor_row);
            }
            b's' => *saved_cursor = Some((*cursor_row, *cursor_col)),
            b'u' => {
                if let Some((row, col)) = *saved_cursor {
                    *cursor_row = row;
                    *cursor_col = col;
                    ensure_row_at(state, *cursor_row);
                }
            }
            b'h' | b'l' if self.has_private_modes() => {
                self.apply_private_modes(
                    final_byte == b'h',
                    state,
                    cursor_row,
                    cursor_col,
                    saved_cursor,
                    alternate_screen,
                    scroll_region,
                    pending_wrap,
                    columns,
                    bracketed_paste,
                    autowrap,
                    origin_mode,
                    cursor_visible,
                );
            }
            b'h' if self.first_param_or(0) == 4 => *insert_mode = true,
            b'l' if self.first_param_or(0) == 4 => *insert_mode = false,
            b'g' => match self.first_param_or(0) {
                0 => tab_stops.clear_current(*cursor_col),
                3 => tab_stops.clear_all(),
                _ => {}
            },
            b'r' => {
                if let Some((top, bottom)) = self.scroll_region_params(rows) {
                    *scroll_region = Some((top, bottom));
                    ensure_row_at(state, bottom);
                } else {
                    *scroll_region = None;
                }
                *cursor_row = if *origin_mode {
                    scroll_region.map(|(top, _bottom)| top).unwrap_or(0)
                } else {
                    0
                };
                *cursor_col = 0;
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

    fn cursor_style_param(&self) -> Option<&'static str> {
        let params = std::str::from_utf8(&self.params).ok()?;
        let numeric = params.strip_suffix(' ')?;
        let value = if numeric.is_empty() {
            0
        } else {
            numeric.parse::<usize>().ok()?
        };
        match value {
            0 | 1 => Some("blinking_block"),
            2 => Some("steady_block"),
            3 => Some("blinking_underline"),
            4 => Some("steady_underline"),
            5 => Some("blinking_bar"),
            6 => Some("steady_bar"),
            _ => None,
        }
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

    fn has_private_modes(&self) -> bool {
        !self.private_modes().is_empty()
    }

    fn private_modes(&self) -> Vec<usize> {
        let Ok(params) = std::str::from_utf8(&self.params) else {
            return Vec::new();
        };
        let Some(rest) = params.strip_prefix('?') else {
            return Vec::new();
        };
        rest.split(';')
            .filter_map(|param| param.parse::<usize>().ok())
            .collect()
    }

    fn apply_private_modes(
        &self,
        enabled: bool,
        state: &mut TerminalLaneState,
        cursor_row: &mut usize,
        cursor_col: &mut usize,
        saved_cursor: &mut Option<(usize, usize)>,
        alternate_screen: &mut Option<SavedTerminalScreen>,
        scroll_region: &mut Option<(usize, usize)>,
        pending_wrap: &mut bool,
        columns: Option<usize>,
        bracketed_paste: &mut bool,
        autowrap: &mut bool,
        origin_mode: &mut bool,
        cursor_visible: &mut bool,
    ) {
        for mode in self.private_modes() {
            match (enabled, mode) {
                (true, 1048) => *saved_cursor = Some((*cursor_row, *cursor_col)),
                (true, 47 | 1047 | 1049) => {
                    enter_alternate_screen(state, cursor_row, cursor_col, alternate_screen);
                }
                (false, 1048) => restore_saved_cursor(state, cursor_row, cursor_col, saved_cursor),
                (false, 47 | 1047 | 1049) => {
                    restore_alternate_screen(state, cursor_row, cursor_col, alternate_screen);
                }
                (true, 2004) => *bracketed_paste = true,
                (false, 2004) => *bracketed_paste = false,
                (true, 6) => {
                    *origin_mode = true;
                    *pending_wrap = false;
                    *cursor_row = scroll_region.map(|(top, _bottom)| top).unwrap_or(0);
                    *cursor_col = 0;
                    ensure_row_at(state, *cursor_row);
                }
                (false, 6) => {
                    *origin_mode = false;
                    *pending_wrap = false;
                    *cursor_row = 0;
                    *cursor_col = 0;
                    ensure_row_at(state, *cursor_row);
                }
                (true, 7) => *autowrap = true,
                (false, 7) => {
                    *autowrap = false;
                    *pending_wrap = false;
                    if let Some(columns) = columns {
                        *cursor_col = (*cursor_col).min(columns.saturating_sub(1));
                    }
                }
                (true, 25) => *cursor_visible = true,
                (false, 25) => *cursor_visible = false,
                (true, 1) => state.application_cursor_keys = true,
                (false, 1) => state.application_cursor_keys = false,
                (true, 1004) => state.focus_event_reporting = true,
                (false, 1004) => state.focus_event_reporting = false,
                (true, 1000) => state.mouse_reporting_mode = Some("normal".to_string()),
                (true, 1002) => state.mouse_reporting_mode = Some("button_event".to_string()),
                (true, 1003) => state.mouse_reporting_mode = Some("any_event".to_string()),
                (false, 1000 | 1002 | 1003) => state.mouse_reporting_mode = None,
                (true, 1005) => {
                    state.mouse_coordinate_encoding = Some("utf8".to_string());
                }
                (true, 1006) => {
                    state.mouse_coordinate_encoding = Some("sgr".to_string());
                }
                (true, 1015) => {
                    state.mouse_coordinate_encoding = Some("urxvt".to_string());
                }
                (false, 1005 | 1006 | 1015) => state.mouse_coordinate_encoding = None,
                _ => {}
            }
        }
    }

    fn scroll_region_params(&self, rows: Option<usize>) -> Option<(usize, usize)> {
        let (top, bottom) = self.row_col_params();
        let top = top.saturating_sub(1);
        let mut bottom = bottom.saturating_sub(1);
        if let Some(rows) = rows {
            bottom = bottom.min(rows.saturating_sub(1));
        }
        (bottom > top).then_some((top, bottom))
    }
}

fn enter_alternate_screen(
    state: &mut TerminalLaneState,
    cursor_row: &mut usize,
    cursor_col: &mut usize,
    alternate_screen: &mut Option<SavedTerminalScreen>,
) {
    if alternate_screen.is_none() {
        *alternate_screen = Some(SavedTerminalScreen {
            lines: state.lines.clone(),
            styled_lines: state.styled_lines.clone(),
            cursor_row: *cursor_row,
            cursor_col: *cursor_col,
        });
    }
    state.lines.clear();
    state.lines.push(String::new());
    state.styled_lines.clear();
    state.styled_lines.push(TerminalStyledLine::default());
    *cursor_row = 0;
    *cursor_col = 0;
}

fn restore_alternate_screen(
    state: &mut TerminalLaneState,
    cursor_row: &mut usize,
    cursor_col: &mut usize,
    alternate_screen: &mut Option<SavedTerminalScreen>,
) {
    if let Some(saved) = alternate_screen.take() {
        state.lines = saved.lines;
        state.styled_lines = saved.styled_lines;
        *cursor_row = saved.cursor_row.min(state.lines.len().saturating_sub(1));
        *cursor_col = saved.cursor_col;
        ensure_row(state);
    }
}

fn restore_saved_cursor(
    state: &mut TerminalLaneState,
    cursor_row: &mut usize,
    cursor_col: &mut usize,
    saved_cursor: &mut Option<(usize, usize)>,
) {
    if let Some((row, col)) = *saved_cursor {
        *cursor_row = row;
        *cursor_col = col;
        ensure_row_at(state, *cursor_row);
    }
}
