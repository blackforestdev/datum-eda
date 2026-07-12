//! Pure state reducer for the Space + primary-button pan gesture.
//!
//! Window-system events stay in `main.rs`; this module only records ordering and
//! ownership so camera movement and primary-click dispatch cannot disagree.

use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PanGestureState {
    space_held: bool,
    active: bool,
    ever_activated: bool,
}

impl Runtime {
    pub(super) fn handle_pan_key(&mut self, event: &KeyEvent) -> bool {
        match (&event.logical_key, event.state) {
            (Key::Named(NamedKey::Space), state) => {
                self.pan_gesture.set_space(state == ElementState::Pressed);
                true
            }
            _ => false,
        }
    }

    pub(super) fn cancel_active_pan(&mut self) -> bool {
        if !self.pan_gesture.is_active() {
            return false;
        }
        self.pan_gesture.cancel();
        true
    }

    pub(super) fn handle_context_menu_button(&mut self, state: ElementState) -> bool {
        match state {
            ElementState::Pressed => {
                // Context-menu ownership is exclusive: a simultaneous secondary
                // press terminates 2D pan and must never let it resume afterward.
                self.pan_gesture.cancel();
                self.open_marking_menu_at_cursor()
            }
            ElementState::Released => self.dismiss_marking_menu(),
        }
    }
}

impl PanGestureState {
    /// Record the Space key state. Releasing Space ends camera movement
    /// immediately, while retaining click suppression until primary release.
    pub(crate) fn set_space(&mut self, held: bool) {
        self.space_held = held;
        if !held {
            self.active = false;
        }
    }

    /// Begin a primary-button gesture. Pan activates only when Space was already
    /// held and the press belongs to an authoring scene.
    pub(crate) fn primary_pressed(&mut self, pointer_in_scene: bool) -> bool {
        self.active = self.space_held && pointer_in_scene;
        self.ever_activated = self.active;
        self.active
    }

    /// Finish a primary-button gesture and report whether its normal click must
    /// be suppressed because this press was ever owned by pan.
    pub(crate) fn primary_released(&mut self) -> bool {
        let suppress_click = self.ever_activated;
        self.active = false;
        self.ever_activated = false;
        suppress_click
    }

    /// Stop movement after focus or cursor capture is lost. If pan owned a
    /// still-held primary press, preserve suppression for its eventual release.
    pub(crate) fn cancel(&mut self) {
        self.space_held = false;
        self.active = false;
    }

    pub(crate) fn is_active(&self) -> bool {
        self.active
    }
}

#[cfg(test)]
mod tests {
    use super::PanGestureState;

    #[test]
    fn space_before_primary_activates_and_suppresses_click() {
        let mut state = PanGestureState::default();
        state.set_space(true);

        assert!(state.primary_pressed(true));
        assert!(state.is_active());
        assert!(state.primary_released());
        assert!(!state.is_active());
    }

    #[test]
    fn primary_before_space_requires_a_fresh_primary_press() {
        let mut state = PanGestureState::default();
        assert!(!state.primary_pressed(true));
        state.set_space(true);

        assert!(!state.is_active());
        assert!(!state.primary_released());
        assert!(state.primary_pressed(true));
        assert!(state.primary_released());
    }

    #[test]
    fn a_press_outside_the_scene_does_not_activate_or_suppress_click() {
        let mut state = PanGestureState::default();
        state.set_space(true);

        assert!(!state.primary_pressed(false));
        assert!(!state.primary_released());
    }

    #[test]
    fn releasing_space_stops_pan_but_preserves_release_suppression() {
        let mut state = PanGestureState::default();
        state.set_space(true);
        assert!(state.primary_pressed(true));

        state.set_space(false);
        assert!(!state.is_active());
        assert!(state.primary_released());
    }

    #[test]
    fn releasing_primary_first_resets_ownership() {
        let mut state = PanGestureState::default();
        state.set_space(true);
        assert!(state.primary_pressed(true));
        assert!(state.primary_released());

        state.set_space(false);
        assert!(!state.is_active());
        assert!(!state.primary_released());
    }

    #[test]
    fn focus_or_cursor_loss_stops_pan_but_suppresses_the_late_release() {
        let mut state = PanGestureState::default();
        state.set_space(true);
        assert!(state.primary_pressed(true));

        state.cancel();
        assert!(!state.is_active());
        assert!(state.primary_released());
        assert!(!state.primary_released());
    }

    #[test]
    fn unrelated_mouse_buttons_cannot_activate_the_reducer() {
        let mut state = PanGestureState::default();
        state.set_space(true);

        // Right and middle events deliberately have no reducer entry point.
        assert!(!state.is_active());
    }
}
