//! Linux accessibility-provider state for the active native terminal.
//!
//! TerminalCore supplies the immutable text model. This bridge computes exact
//! AT-SPI event intent without retaining PTY bytes or inventing a second grid.

use crate::Runtime;
use crate::console_accessibility::AccessibilityAnnouncement;
use crate::terminal_accessibility::{TerminalAccessibilityBounds, TerminalAccessibilitySnapshot};
use crate::terminal_accessibility_platform::PlatformBridge;
use datum_gui_protocol::ApplicationFocus;
use std::collections::VecDeque;

const ANNOUNCEMENT_LOG_CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TerminalAccessibilityEvent {
    TextChanged,
    CaretMoved,
    SelectionChanged,
    FocusChanged,
    TitleChanged,
    BoundsChanged,
    Bell,
}

pub(crate) struct LinuxTerminalAccessibilityBridge {
    current: Option<TerminalAccessibilitySnapshot>,
    platform: Option<PlatformBridge>,
    publish_platform: bool,
    announcement_log: VecDeque<AccessibilityAnnouncement>,
}

impl Default for LinuxTerminalAccessibilityBridge {
    fn default() -> Self {
        Self {
            current: None,
            platform: None,
            publish_platform: true,
            announcement_log: VecDeque::new(),
        }
    }
}

impl LinuxTerminalAccessibilityBridge {
    pub(crate) fn announce_console(&mut self, announcement: AccessibilityAnnouncement) {
        if self.announcement_log.len() == ANNOUNCEMENT_LOG_CAPACITY {
            self.announcement_log.pop_front();
        }
        self.announcement_log.push_back(announcement.clone());
        if !self.publish_platform {
            return;
        }
        match &mut self.platform {
            Some(platform) => platform.publish_announcement(announcement),
            None => {
                self.platform = PlatformBridge::start_announcement(announcement).ok();
            }
        }
    }

    pub(crate) fn update(
        &mut self,
        next: TerminalAccessibilitySnapshot,
    ) -> Vec<TerminalAccessibilityEvent> {
        let mut events = Vec::new();
        if let Some(current) = &self.current {
            if current.text != next.text {
                events.push(TerminalAccessibilityEvent::TextChanged);
            }
            if current.caret != next.caret {
                events.push(TerminalAccessibilityEvent::CaretMoved);
            }
            if current.selection != next.selection {
                events.push(TerminalAccessibilityEvent::SelectionChanged);
            }
            if current.focused != next.focused {
                events.push(TerminalAccessibilityEvent::FocusChanged);
            }
            if current.title != next.title {
                events.push(TerminalAccessibilityEvent::TitleChanged);
            }
            if current.bounds != next.bounds {
                events.push(TerminalAccessibilityEvent::BoundsChanged);
            }
            if current.bell_count != next.bell_count {
                events.push(TerminalAccessibilityEvent::Bell);
            }
        } else {
            events.extend([
                TerminalAccessibilityEvent::TextChanged,
                TerminalAccessibilityEvent::CaretMoved,
                TerminalAccessibilityEvent::FocusChanged,
                TerminalAccessibilityEvent::BoundsChanged,
            ]);
        }
        self.current = Some(next.clone());
        if self.publish_platform {
            match &mut self.platform {
                Some(platform) => platform.publish(next, events.clone()),
                None => {
                    self.platform = PlatformBridge::start(next, events.clone()).ok();
                }
            }
        }
        events
    }

    #[cfg(test)]
    pub(crate) fn current(&self) -> Option<&TerminalAccessibilitySnapshot> {
        self.current.as_ref()
    }

    #[cfg(test)]
    fn without_platform() -> Self {
        Self {
            current: None,
            platform: None,
            publish_platform: false,
            announcement_log: VecDeque::new(),
        }
    }
}

impl Runtime {
    pub(super) fn refresh_terminal_accessibility(&mut self) {
        if !self.terminal_sessions.active_attached() {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(text: &str, caret: usize, focused: bool) -> TerminalAccessibilitySnapshot {
        TerminalAccessibilitySnapshot {
            session_id: "session-a".into(),
            title: "Terminal".into(),
            text: text.into(),
            caret,
            selection: None,
            links: Vec::new(),
            focused,
            bell_count: 0,
            bounds: Default::default(),
        }
    }

    #[test]
    fn bridge_emits_only_changed_terminal_semantics() {
        let mut bridge = LinuxTerminalAccessibilityBridge::without_platform();
        assert_eq!(bridge.update(snapshot("a", 1, true)).len(), 4);
        assert!(bridge.update(snapshot("a", 1, true)).is_empty());
        assert_eq!(
            bridge.update(snapshot("ab", 2, true)),
            vec![
                TerminalAccessibilityEvent::TextChanged,
                TerminalAccessibilityEvent::CaretMoved,
            ]
        );
        assert_eq!(bridge.current().unwrap().text.chars().count(), 2);
    }

    #[test]
    fn console_announces_without_a_terminal_snapshot() {
        let mut bridge = LinuxTerminalAccessibilityBridge::without_platform();
        assert!(bridge.current().is_none());
        bridge.announce_console(AccessibilityAnnouncement {
            text: "Action. Information: fit board".to_string(),
            priority: crate::console_accessibility::AnnouncementPriority::Medium,
        });
        assert_eq!(bridge.announcement_log.len(), 1);
        assert!(bridge.current().is_none());
    }
}
