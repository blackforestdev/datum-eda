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

/// Split mutable borrow of the terminal screen grid, obtainable only through
/// [`TerminalLaneState::pty_grid_mut`]. See that method for the screen-authority
/// invariant that governs every use of this type.
pub struct TerminalPtyGrid<'a> {
    pub lines: &'a mut Vec<String>,
    pub styled_lines: &'a mut Vec<TerminalStyledLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalLaneState {
    /// Terminal screen grid rows. PRIVATE by design: terminal cell content may
    /// be mutated only by bytes emitted by the active PTY and interpreted by
    /// the terminal core (T0-C01 in DATUM_NATIVE_TERMINAL_SPEC.md; FT-001 in
    /// decision 027). All mutation goes through [`Self::pty_grid_mut`].
    lines: Vec<String>,
    /// Styled projection of `lines`; same screen-authority invariant applies.
    styled_lines: Vec<TerminalStyledLine>,
    pub activity_summary: Vec<String>,
    pub tabs: Vec<TerminalTabState>,
    pub active_session_id: Option<String>,
    pub title: Option<String>,
    pub current_working_directory: Option<String>,
    pub bell_count: usize,
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
    /// Application-close authority, distinct from the active session's
    /// teardown status so per-session refreshes cannot erase Retry/Cancel.
    pub application_shutdown_blocked: Option<String>,
}

impl TerminalLaneState {
    /// Swap only the PTY-owned projection for one session. Global dock chrome,
    /// tab identity, activity summary, and keyboard focus stay
    /// with the workspace while inactive sessions retain their own screen.
    pub fn swap_session_projection(&mut self, other: &mut Self) {
        macro_rules! swap_fields {
            ($($field:ident),+ $(,)?) => { $(std::mem::swap(&mut self.$field, &mut other.$field);)+ };
        }
        swap_fields!(
            lines,
            styled_lines,
            title,
            current_working_directory,
            bell_count,
            columns,
            rows,
            screen_cursor_row,
            screen_cursor_col,
            screen_cursor_visible,
            screen_cursor_style,
            application_cursor_keys,
            application_keypad,
            focus_event_reporting,
            mouse_reporting_mode,
            mouse_coordinate_encoding,
            scroll_offset,
            status,
        );
    }

    /// Read-only view of the terminal screen grid rows.
    pub fn grid_lines(&self) -> &[String] {
        &self.lines
    }

    /// Read-only view of the styled terminal screen grid rows.
    pub fn grid_styled_lines(&self) -> &[TerminalStyledLine] {
        &self.styled_lines
    }

    /// Sole mutation gateway into the terminal screen grid.
    ///
    /// Screen-authority invariant (T0-C01, DATUM_NATIVE_TERMINAL_SPEC.md;
    /// FT-001, docs/decisions/PRODUCT_MECHANICS_027_FULL_NATIVE_TERMINAL.md):
    /// the only legal caller is the terminal core's PTY-byte interpretation
    /// path (`TerminalScreen::apply_bytes_with_responses` and its helpers).
    /// Datum notices, diagnostics, activity summaries, lifecycle messages, and
    /// GUI command echoes must never enter the grid — they belong to terminal
    /// chrome, notifications, logs, or the console sink
    /// (`WorkspaceUiState::push_console_line`). Test code may call this only to
    /// simulate PTY-derived screen content.
    pub fn pty_grid_mut(&mut self) -> TerminalPtyGrid<'_> {
        TerminalPtyGrid {
            lines: &mut self.lines,
            styled_lines: &mut self.styled_lines,
        }
    }
}

impl Default for TerminalLaneState {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            styled_lines: Vec::new(),
            activity_summary: Vec::new(),
            tabs: Vec::new(),
            active_session_id: None,
            title: None,
            current_working_directory: None,
            bell_count: 0,
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
            application_shutdown_blocked: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_projection_swap_preserves_workspace_chrome_and_focus() {
        let mut active = TerminalLaneState {
            lines: vec!["active".to_string()],
            title: Some("active title".to_string()),
            active_session_id: Some("active-id".to_string()),
            activity_summary: vec!["activity".to_string()],
            application_shutdown_blocked: Some("shutdown blocked".to_string()),
            ..Default::default()
        };
        let chrome = (
            active.active_session_id.clone(),
            active.activity_summary.clone(),
            active.application_shutdown_blocked.clone(),
        );
        let mut parked = TerminalLaneState {
            lines: vec!["parked".to_string()],
            title: Some("parked title".to_string()),
            ..Default::default()
        };

        active.swap_session_projection(&mut parked);

        assert_eq!(active.grid_lines(), &["parked"]);
        assert_eq!(active.title.as_deref(), Some("parked title"));
        assert_eq!(parked.grid_lines(), &["active"]);
        assert_eq!(parked.title.as_deref(), Some("active title"));
        assert_eq!(
            (
                active.active_session_id,
                active.activity_summary,
                active.application_shutdown_blocked,
            ),
            chrome,
        );
    }
}
