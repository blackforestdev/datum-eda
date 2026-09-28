//! Retained-scene and transient interaction refresh policy.
//!
//! Keeping these paths together makes the performance boundary explicit:
//! pointer-only changes refresh interaction chrome, while authored/session
//! changes may invalidate the considerably more expensive retained geometry.

use super::Runtime;

impl Runtime {
    pub(super) fn invalidate_scene(&mut self) {
        self.render_sources.replaced();
        self.renderer.render_session_mut().clear_content();
    }

    pub(super) fn invalidate_surface_size(&mut self) {
        self.presented_console_layout = None;
        self.presented_hits.clear();
        // Historical entries carry old surface keys. Preserve only live geometry
        // whose construction has no dependency on the reference projection size.
        self.renderer.render_session_mut().resize_content();
    }

    pub(super) fn invalidate_frame(&mut self) {
        self.renderer.render_session_mut().composition_changed();
        // Camera/layout/chrome changes rebuild the prepared projection only.
        // Schematic authored geometry is camera-independent retained world data,
        // just like the board retained scene, and must stay warm here.
        self.refresh_global_preferences_accessibility();
        self.refresh_menu_accessibility();
        self.trace_action_state();
    }

    /// Local native-dialog state has no effect on Main's prepared projection.
    /// The owned-window adapter separately invalidates its own surface.
    pub(super) fn refresh_dialog_state(&mut self) {
        self.refresh_global_preferences_accessibility();
        self.trace_action_state();
    }

    /// Use the same paint dependencies as native redraw routing. Future-project
    /// defaults can commit without evicting the current Main projection.
    pub(super) fn refresh_global_preference_change(
        &mut self,
        before: crate::global_preferences_window::GlobalPreferenceRenderState,
    ) {
        use crate::global_preferences_window::DialogInputOutcome;
        if before.finish(DialogInputOutcome::Dependents, &self.workspace().ui)
            == DialogInputOutcome::Dialog
        {
            self.refresh_dialog_state();
        } else {
            self.invalidate_frame();
        }
    }

    /// Submit pointer intent. Shared dependencies decide whether the change is
    /// suffix-only or alters retained material emphasis and prepared hover text.
    pub(super) fn refresh_interaction_overlay(&mut self) {
        let generation = self.renderer.render_session().pointer_generation();
        let ui = &self.session.workspace().ui;
        self.renderer.render_session_mut().update_pointer(
            datum_gui_render::render_input::PointerUpdate {
                generation,
                cursor: ui.cursor_pos,
                hover: ui.hovered_object.as_ref(),
                style: ui.crosshair_style,
            },
        );
    }
}

#[cfg(test)]
#[path = "native_dialog_refresh_tests.rs"]
mod tests;
