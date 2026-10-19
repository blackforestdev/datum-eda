//! Selected terminal accessibility projection over its existing core snapshot.
use super::*;

fn title(lane: &datum_gui_protocol::TerminalLaneState, core_title: Option<&str>) -> String {
    let mut title = core_title.unwrap_or("Terminal unavailable").to_owned();
    if core_title.is_some()
        && let Some(context) = &lane.session_context
    {
        title.push_str(&format!("; {}", context.description()));
    }
    title.push_str(&format!("; Status: {}", lane.status));
    for reason in [
        lane.application_shutdown_blocked.as_deref(),
        lane.project_open_cleanup_failure.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        title.push_str(&format!("; {reason}"));
    }
    title
}

impl LinuxTerminalAccessibilityBridge {
    /// Metadata-only comparison avoids copying the core text on routine tab
    /// refreshes. Text/caret/bounds updates retain their existing publishers.
    pub(crate) fn session_projection_matches(
        &self,
        lane: &datum_gui_protocol::TerminalLaneState,
        attached: bool,
    ) -> bool {
        self.current.as_ref().is_some_and(|snapshot| {
            snapshot.session_id == lane.active_session_id.as_deref().unwrap_or("")
                && snapshot.title
                    == title(
                        lane,
                        attached.then(|| lane.title.as_deref().unwrap_or("Terminal")),
                    )
        })
    }
}

impl Runtime {
    pub(crate) fn refresh_terminal_accessibility(&mut self) {
        if !self.terminal_sessions.active_attached() {
            let lane = &self.workspace().ui.terminal;
            // Pending/absent consumers have no core snapshot. Publish that
            // absence instead of retaining a previous session's authority.
            self.terminal_accessibility
                .update(TerminalAccessibilitySnapshot {
                    session_id: lane.active_session_id.clone().unwrap_or_default(),
                    title: title(lane, None),
                    text: String::new(),
                    caret: 0,
                    selection: None,
                    links: Vec::new(),
                    focused: false,
                    bell_count: 0,
                    bounds: TerminalAccessibilityBounds::default(),
                });
            return;
        }
        let geometry = self.terminal_screen_geometry();
        let scroll_offset = self.workspace().ui.terminal.scroll_offset;
        let focused = self.application_focus() == ApplicationFocus::Terminal;
        match self.terminal_sessions.active_accessibility_snapshot(
            usize::from(geometry.rows),
            scroll_offset,
            focused,
        ) {
            Ok(mut snapshot) => {
                let lane = &self.workspace().ui.terminal;
                snapshot.title = title(lane, Some(&snapshot.title));
                snapshot.bounds = TerminalAccessibilityBounds {
                    x: geometry.screen.x.round() as i32,
                    y: geometry.screen.y.round() as i32,
                    width: geometry.screen.width.round() as i32,
                    height: geometry.screen.height.round() as i32,
                };
                self.terminal_accessibility.update(snapshot);
            }
            Err(error) => {
                self.log_terminal_event(format!("terminal accessibility refresh failed: {error}"));
            }
        }
    }
}
