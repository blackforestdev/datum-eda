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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSearchPoint {
    pub line: u64,
    pub cluster: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSearchMatch {
    pub start: TerminalSearchPoint,
    pub end: TerminalSearchPoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TerminalSearchState {
    pub active: bool,
    /// Escape closes search on key press, but its matching release remains
    /// search-owned so it cannot also eject keyboard focus from the terminal.
    pub escape_release_pending: bool,
    pub query: String,
    pub case_sensitive: bool,
    pub regex: bool,
    pub matches: Vec<TerminalSearchMatch>,
    /// Sorted, disjoint match ranges for bounded renderer lookup. Navigation
    /// retains the exact possibly-overlapping matches above.
    pub highlights: Vec<TerminalSearchMatch>,
    pub active_match: Option<usize>,
    pub matched: Option<TerminalSearchMatch>,
    pub status: String,
}

/// A bounded target derived from terminal content. Terminal output can create
/// this inert presentation value, but it cannot launch a desktop application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalLinkTarget {
    pub kind: TerminalLinkKind,
    /// Exact target retained for clipboard copy or an explicitly confirmed
    /// desktop handoff. Renderers may truncate the visual projection only.
    pub target: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalLinkKind {
    HttpUri,
    Path,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalClipboardSelection {
    Clipboard,
    Primary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalClipboardConfirmation {
    pub selection: TerminalClipboardSelection,
    pub byte_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TerminalProgressState {
    #[default]
    Clear,
    Set {
        percent: u8,
    },
    Error {
        percent: u8,
    },
    Paused {
        percent: u8,
    },
    Indeterminate,
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
    pub search: TerminalSearchState,
    /// Explicit user confirmation for an inert HTTP(S) target. This is
    /// session-local chrome and never enters the terminal cell stream.
    pub link_confirmation: Option<TerminalLinkTarget>,
    /// Matching Escape release remains link-chrome-owned after cancellation so
    /// it cannot also eject keyboard focus from the terminal.
    pub link_escape_release_pending: bool,
    /// Presentation-only summary for a focused-session OSC 52 write. The
    /// decoded payload remains private to gui-app until explicit confirmation.
    pub clipboard_confirmation: Option<TerminalClipboardConfirmation>,
    pub clipboard_escape_release_pending: bool,
    pub latest_notification: Option<String>,
    pub progress: TerminalProgressState,
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
            search,
            link_confirmation,
            link_escape_release_pending,
            clipboard_confirmation,
            clipboard_escape_release_pending,
            latest_notification,
            progress,
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
            search: TerminalSearchState::default(),
            link_confirmation: None,
            link_escape_release_pending: false,
            clipboard_confirmation: None,
            clipboard_escape_release_pending: false,
            latest_notification: None,
            progress: TerminalProgressState::Clear,
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
            search: TerminalSearchState {
                active: true,
                query: "active query".to_string(),
                ..Default::default()
            },
            link_confirmation: Some(TerminalLinkTarget {
                kind: TerminalLinkKind::HttpUri,
                target: "https://active.example".to_string(),
            }),
            clipboard_confirmation: Some(TerminalClipboardConfirmation {
                selection: TerminalClipboardSelection::Clipboard,
                byte_count: 5,
            }),
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
            search: TerminalSearchState {
                active: true,
                query: "parked query".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        active.swap_session_projection(&mut parked);

        assert_eq!(active.title.as_deref(), Some("parked title"));
        assert_eq!(parked.title.as_deref(), Some("active title"));
        assert_eq!(active.search.query, "parked query");
        assert_eq!(parked.search.query, "active query");
        assert_eq!(
            parked
                .link_confirmation
                .as_ref()
                .map(|link| link.target.as_str()),
            Some("https://active.example")
        );
        assert_eq!(
            parked
                .clipboard_confirmation
                .as_ref()
                .map(|request| request.byte_count),
            Some(5)
        );
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
