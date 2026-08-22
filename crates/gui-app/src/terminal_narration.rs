//! Datum-owned narration routing at the native-terminal boundary.
//!
//! Terminal cells are foreign-shell state. Application narration has one
//! legacy route into `ConsoleLaneState`; keeping it outside the runtime makes
//! the no-grid-write invariant directly testable. Decision 033 requires the
//! Console implementation successor to classify these producers and replace
//! this untyped sink before anything becomes visible.

use datum_gui_protocol::ConsoleLaneState;

pub(super) fn route_gui_narration(console: &mut ConsoleLaneState, message: impl Into<String>) {
    console.push_line(message.into());
}

#[cfg(test)]
mod tests {
    use super::route_gui_narration;
    use datum_gui_protocol::ConsoleLaneState;

    #[test]
    fn production_narration_route_cannot_mutate_shell_cells() {
        let mut console = ConsoleLaneState::default();
        route_gui_narration(&mut console, "terminal write failed: broken pipe");
        assert_eq!(
            console.lines.last().map(String::as_str),
            Some("terminal write failed: broken pipe")
        );
    }
}
