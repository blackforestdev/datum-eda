//! Workspace-global terminal font zoom.
//!
//! The scalar lives above individual PTY/session projections. Applying it
//! immediately recomputes the shared cell geometry, resizes every pane in the
//! active tab once, and invalidates the scene so renderer, hit testing, cursor,
//! graphics, selection, and IME placement all observe the same metrics.

use super::*;
use datum_gui_protocol::{
    TERMINAL_FONT_SCALE_DEFAULT_MILLIS, TERMINAL_FONT_SCALE_MAX_MILLIS,
    TERMINAL_FONT_SCALE_MIN_MILLIS, TERMINAL_FONT_SCALE_STEP_MILLIS,
};

pub(super) fn adjusted_terminal_font_scale(current: u16, steps: i16) -> u16 {
    let delta = i32::from(steps) * i32::from(TERMINAL_FONT_SCALE_STEP_MILLIS);
    (i32::from(current) + delta).clamp(
        i32::from(TERMINAL_FONT_SCALE_MIN_MILLIS),
        i32::from(TERMINAL_FONT_SCALE_MAX_MILLIS),
    ) as u16
}

impl Runtime {
    pub(super) fn adjust_terminal_font_zoom(&mut self, steps: i16) -> bool {
        let current = self.workspace().ui.terminal.font_scale_millis;
        self.apply_terminal_font_scale(adjusted_terminal_font_scale(current, steps))
    }

    pub(super) fn reset_terminal_font_zoom(&mut self) -> bool {
        self.apply_terminal_font_scale(TERMINAL_FONT_SCALE_DEFAULT_MILLIS)
    }

    fn apply_terminal_font_scale(&mut self, scale_millis: u16) -> bool {
        let terminal = &mut self.session.workspace_mut().ui.terminal;
        if terminal.font_scale_millis == scale_millis {
            return true;
        }
        terminal.font_scale_millis = scale_millis;
        self.resize_terminal_to_dock();
        self.invalidate_scene();
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_font_zoom_steps_and_clamps_without_session_local_state() {
        assert_eq!(adjusted_terminal_font_scale(1_000, 1), 1_100);
        assert_eq!(adjusted_terminal_font_scale(1_000, -1), 900);
        assert_eq!(adjusted_terminal_font_scale(1_950, 1), 2_000);
        assert_eq!(adjusted_terminal_font_scale(650, -1), 600);
        assert_eq!(adjusted_terminal_font_scale(1_000, 20), 2_000);
        assert_eq!(adjusted_terminal_font_scale(1_000, -20), 600);
    }
}
