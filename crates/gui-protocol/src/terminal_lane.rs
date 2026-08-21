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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalLaneState {
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
    /// Application-local IME composition. Preedit is rendered at the core
    /// cursor but is never inserted into the PTY byte stream.
    pub ime_preedit: Option<String>,
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
            title,
            current_working_directory,
            bell_count,
            columns,
            rows,
            screen_cursor_row,
            screen_cursor_col,
            screen_cursor_visible,
            screen_cursor_style,
            ime_preedit,
            application_cursor_keys,
            application_keypad,
            focus_event_reporting,
            mouse_reporting_mode,
            mouse_coordinate_encoding,
            scroll_offset,
            status,
        );
    }
}

impl Default for TerminalLaneState {
    fn default() -> Self {
        Self {
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
            ime_preedit: None,
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
            title: Some("parked title".to_string()),
            ..Default::default()
        };

        active.swap_session_projection(&mut parked);

        assert_eq!(active.title.as_deref(), Some("parked title"));
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
