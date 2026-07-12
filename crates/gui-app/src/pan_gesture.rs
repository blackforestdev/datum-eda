//! Pure state reducer for the Space + primary-button pan gesture.
//!
//! Window-system events stay in `main.rs`; this module only records ordering and
//! ownership so camera movement and primary-click dispatch cannot disagree.

use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PanGestureState {
    space_held: bool,
    space_key_owned: bool,
    active: bool,
    ever_activated: bool,
}

impl Runtime {
    pub(super) fn handle_pan_key(&mut self, event: &KeyEvent) -> bool {
        let pointer_in_scene = self.cursor_in_editor_scene();
        let consumed = self.pan_gesture.handle_space_key(
            event.physical_key,
            &event.logical_key,
            event.state,
            event.repeat,
            pointer_in_scene,
        );
        append_gui_verbose_diagnostic_line(format!(
            "pan key physical={:?} logical={:?} state={:?} repeat={} pointer_in_scene={} held={} consumed={}",
            event.physical_key, event.logical_key, event.state, event.repeat,
            pointer_in_scene, self.pan_gesture.space_is_held(), consumed
        ));
        consumed
    }

    pub(super) fn begin_primary_pan(&mut self) -> bool {
        let pointer_in_scene = self.cursor_in_editor_scene();
        let active = self.pan_gesture.primary_pressed(pointer_in_scene);
        append_gui_verbose_diagnostic_line(format!(
            "pan primary pressed cursor={:?} pointer_in_scene={} held={} active={}",
            self.last_cursor_pos, pointer_in_scene, self.pan_gesture.space_is_held(), active
        ));
        active
    }

    pub(super) fn advance_primary_pan(
        &mut self,
        previous: (f32, f32),
        next: (f32, f32),
    ) -> bool {
        let changed = self.handle_pan_drag(previous, next);
        append_gui_verbose_diagnostic_line(format!(
            "pan move previous={previous:?} next={next:?} changed={changed}"
        ));
        changed
    }

    pub(super) fn finish_primary_pan(&mut self) -> bool {
        let suppress_click = self.pan_gesture.primary_released();
        append_gui_verbose_diagnostic_line(format!(
            "pan primary released suppress_click={suppress_click}"
        ));
        suppress_click
    }

    pub(super) fn cancel_active_pan(&mut self) -> bool {
        if !self.pan_gesture.space_is_held() {
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
    /// Route a window-system Space event to the pan gesture.
    ///
    /// Space-down state is recorded independently of event ownership. Ownership
    /// only decides whether this keyboard event is consumed (so a dock/text owner
    /// can still receive Space); primary press later decides whether a viewport
    /// pan actually begins from an editor scene.
    pub(crate) fn handle_space_key(
        &mut self,
        physical_key: PhysicalKey,
        logical_key: &Key,
        state: ElementState,
        repeat: bool,
        pointer_in_scene: bool,
    ) -> bool {
        let is_space = matches!(physical_key, PhysicalKey::Code(KeyCode::Space))
            || matches!(logical_key, Key::Named(NamedKey::Space))
            || matches!(logical_key, Key::Character(text) if text.as_str() == " ");
        if !is_space {
            return false;
        }

        match state {
            ElementState::Pressed if repeat => {
                self.set_space(true);
                self.space_key_owned
            }
            ElementState::Pressed => {
                self.space_key_owned = pointer_in_scene;
                self.set_space(true);
                self.space_key_owned
            }
            ElementState::Released => {
                let owned = self.space_key_owned;
                self.set_space(false);
                self.space_key_owned = false;
                owned
            }
        }
    }

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
        self.space_key_owned = false;
        self.active = false;
    }

    pub(crate) fn is_active(&self) -> bool {
        self.active
    }

    pub(crate) fn space_is_held(&self) -> bool {
        self.space_held
    }
}

#[cfg(test)]
mod tests {
    use super::PanGestureState;
    use winit::{
        event::ElementState,
        keyboard::{Key, KeyCode, NamedKey, NativeKey, NativeKeyCode, PhysicalKey},
    };

    fn route_space(
        state: &mut PanGestureState,
        key_state: ElementState,
        repeat: bool,
        pointer_in_scene: bool,
    ) -> bool {
        state.handle_space_key(
            PhysicalKey::Code(KeyCode::Space),
            &Key::Named(NamedKey::Space),
            key_state,
            repeat,
            pointer_in_scene,
        )
    }

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

    #[test]
    fn open_dock_does_not_prevent_scene_pointer_from_claiming_space() {
        let mut state = PanGestureState::default();

        assert!(route_space(&mut state, ElementState::Pressed, false, true));
        assert!(state.primary_pressed(true));
    }

    #[test]
    fn pointer_in_dock_leaves_space_event_for_the_dock_and_dock_press_inactive() {
        let mut state = PanGestureState::default();

        assert!(!route_space(
            &mut state,
            ElementState::Pressed,
            false,
            false
        ));
        assert!(!state.primary_pressed(false));
    }

    #[test]
    fn missing_cursor_at_space_down_can_activate_on_later_scene_press() {
        let mut state = PanGestureState::default();

        // No cached cursor means the key event is not consumed, but the physical
        // chord remains known. The primary event owns its actual hit location.
        assert!(!route_space(
            &mut state,
            ElementState::Pressed,
            false,
            false
        ));
        assert!(state.primary_pressed(true));
        assert!(state.is_active());
        assert!(state.primary_released());
    }

    #[test]
    fn missing_scene_geometry_keeps_primary_press_inactive() {
        let mut state = PanGestureState::default();
        assert!(!route_space(
            &mut state,
            ElementState::Pressed,
            false,
            false
        ));

        // Runtime supplies false when a Schematic leaf has no resolved bounds.
        assert!(!state.primary_pressed(false));
        assert!(!state.is_active());
        assert!(!state.primary_released());
    }

    #[test]
    fn claimed_release_clears_space_after_pointer_moves_into_dock() {
        let mut state = PanGestureState::default();
        assert!(route_space(&mut state, ElementState::Pressed, false, true));

        assert!(route_space(
            &mut state,
            ElementState::Released,
            false,
            false
        ));
        assert!(!state.primary_pressed(true));
    }

    #[test]
    fn cancel_while_space_is_held_prevents_a_later_primary_press() {
        let mut state = PanGestureState::default();
        assert!(route_space(&mut state, ElementState::Pressed, false, true));

        state.cancel();
        assert!(!state.primary_pressed(true));
    }

    #[test]
    fn key_repeat_preserves_event_ownership_without_losing_the_held_chord() {
        let mut scene_owned = PanGestureState::default();
        assert!(route_space(
            &mut scene_owned,
            ElementState::Pressed,
            false,
            true
        ));
        assert!(route_space(
            &mut scene_owned,
            ElementState::Pressed,
            true,
            false
        ));
        assert!(scene_owned.primary_pressed(true));

        let mut dock_owned = PanGestureState::default();
        assert!(!route_space(
            &mut dock_owned,
            ElementState::Pressed,
            false,
            false
        ));
        assert!(!route_space(
            &mut dock_owned,
            ElementState::Pressed,
            true,
            true
        ));
        assert!(dock_owned.primary_pressed(true));
    }

    #[test]
    fn physical_space_and_logical_fallback_are_both_recognized() {
        let mut physical = PanGestureState::default();
        assert!(physical.handle_space_key(
            PhysicalKey::Code(KeyCode::Space),
            &Key::Unidentified(NativeKey::Unidentified),
            ElementState::Pressed,
            false,
            true,
        ));

        let mut logical = PanGestureState::default();
        assert!(logical.handle_space_key(
            PhysicalKey::Unidentified(NativeKeyCode::Unidentified),
            &Key::Named(NamedKey::Space),
            ElementState::Pressed,
            false,
            true,
        ));

        let mut logical_character = PanGestureState::default();
        assert!(logical_character.handle_space_key(
            PhysicalKey::Unidentified(NativeKeyCode::Unidentified),
            &Key::Character(" ".into()),
            ElementState::Pressed,
            false,
            true,
        ));
    }
}
