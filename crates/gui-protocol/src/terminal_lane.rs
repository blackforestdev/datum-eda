#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalTabState {
    pub session_id: String,
    pub previous_session_id: Option<String>,
    pub label: String,
    pub event_log_path: String,
    pub activity_event_count: usize,
    pub activity_summary: Vec<String>,
    pub active: bool,
    pub attached: bool,
    pub status: String,
    pub restart_count: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TerminalTextStyle {
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub overline: bool,
    pub blink: bool,
    pub strikethrough: bool,
    pub conceal: bool,
    pub inverse: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalStyleSpan {
    pub start: usize,
    pub end: usize,
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub overline: bool,
    pub blink: bool,
    pub strikethrough: bool,
    pub conceal: bool,
    pub inverse: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TerminalStyledLine {
    pub text: String,
    pub spans: Vec<TerminalStyleSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalLaneState {
    pub lines: Vec<String>,
    pub styled_lines: Vec<TerminalStyledLine>,
    pub activity_summary: Vec<String>,
    pub tabs: Vec<TerminalTabState>,
    pub active_session_id: Option<String>,
    pub rename_session_id: Option<String>,
    pub title: Option<String>,
    pub current_working_directory: Option<String>,
    pub bell_count: usize,
    pub input: String,
    pub cursor: usize,
    pub columns: u16,
    pub rows: u16,
    pub screen_cursor_row: usize,
    pub screen_cursor_col: usize,
    pub screen_cursor_visible: bool,
    pub screen_cursor_style: Option<String>,
    pub application_cursor_keys: bool,
    pub application_keypad: bool,
    pub focus_event_reporting: bool,
    pub mouse_reporting_mode: Option<String>,
    pub mouse_coordinate_encoding: Option<String>,
    pub scroll_offset: usize,
    pub status: String,
}

impl Default for TerminalLaneState {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            styled_lines: Vec::new(),
            activity_summary: Vec::new(),
            tabs: Vec::new(),
            active_session_id: None,
            rename_session_id: None,
            title: None,
            current_working_directory: None,
            bell_count: 0,
            input: String::new(),
            cursor: 0,
            columns: 80,
            rows: 24,
            screen_cursor_row: 0,
            screen_cursor_col: 0,
            screen_cursor_visible: true,
            screen_cursor_style: None,
            application_cursor_keys: false,
            application_keypad: false,
            focus_event_reporting: false,
            mouse_reporting_mode: None,
            mouse_coordinate_encoding: None,
            scroll_offset: 0,
            status: "running".to_string(),
        }
    }
}
