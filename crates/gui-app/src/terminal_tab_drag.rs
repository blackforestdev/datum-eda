//! Pointer gesture state for reordering terminal tabs.
//!
//! The gesture stays consumer-local: it identifies a session and translates a
//! deliberate horizontal drag into registry ordering. PTY ownership, terminal
//! projections, and process lifecycle remain with the moved session slot.

const TAB_DRAG_THRESHOLD_PX: f32 = 4.0;

pub(crate) struct TerminalTabDrag {
    session_id: String,
    press: (f32, f32),
    moved: bool,
    target_session_id: Option<String>,
}

impl TerminalTabDrag {
    pub(crate) fn new(session_id: String, press: (f32, f32)) -> Self {
        Self {
            session_id,
            press,
            moved: false,
            target_session_id: None,
        }
    }

    pub(crate) fn session_id(&self) -> &str {
        &self.session_id
    }

    pub(crate) fn advance(&mut self, pointer: (f32, f32), target_session_id: Option<&str>) -> bool {
        self.moved |= (pointer.0 - self.press.0).abs() >= TAB_DRAG_THRESHOLD_PX;
        if !self.moved {
            return false;
        }
        let next = target_session_id
            .filter(|target| *target != self.session_id)
            .map(str::to_string);
        if self.target_session_id == next {
            return false;
        }
        self.target_session_id = next;
        true
    }

    pub(crate) fn target_session_id(&self) -> Option<&str> {
        self.target_session_id.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_drag_requires_deliberate_horizontal_motion() {
        let mut drag = TerminalTabDrag::new("terminal-2".to_string(), (100.0, 20.0));
        assert!(!drag.advance((103.9, 200.0), Some("terminal-1")));
        assert!(drag.advance((104.0, 20.0), Some("terminal-1")));
        assert_eq!(drag.target_session_id(), Some("terminal-1"));
        assert!(!drag.advance((101.0, 20.0), Some("terminal-1")));
        assert_eq!(drag.session_id(), "terminal-2");
    }
}
