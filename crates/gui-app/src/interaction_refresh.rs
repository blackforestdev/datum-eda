//! Retained-scene and transient interaction refresh policy.
//!
//! Keeping these paths together makes the performance boundary explicit:
//! pointer-only changes refresh interaction chrome, while authored/session
//! changes may invalidate the considerably more expensive retained geometry.

use super::{RetainedScene, RetainedSceneCacheKey, Runtime};

impl Runtime {
    pub(super) fn cache_retained_scene(
        &mut self,
        key: RetainedSceneCacheKey,
        retained: RetainedScene,
    ) {
        self.retained_scene_cache.insert(key, retained);
    }

    pub(super) fn invalidate_scene_for_session_change(
        &mut self,
        previous_key: RetainedSceneCacheKey,
    ) {
        if let Some(retained) = self.retained_scene.take() {
            self.cache_retained_scene(previous_key, retained);
        }
        self.prepared_scene = None;
        self.clear_schematic_retained_scene();
        self.scene_dirty = true;
        self.restore_cached_retained_scene();
    }

    pub(super) fn invalidate_scene(&mut self) {
        self.retained_scene = None;
        self.retained_scene_cache.clear();
        self.prepared_scene = None;
        self.clear_schematic_retained_scene();
        self.scene_dirty = true;
    }

    pub(super) fn invalidate_surface_size(&mut self) {
        self.presented_console_layout = None;
        self.presented_hits.clear();
        // Historical entries carry old surface keys. Preserve only live geometry
        // whose construction has no dependency on the reference projection size.
        self.retained_scene_cache
            .invalidate_surface_size(&mut self.retained_scene);
        self.schematic_scene_accounting
            .invalidate_surface_size(&mut self.schematic_retained_scene);
        self.prepared_scene = None;
        self.scene_dirty = true;
    }

    pub(super) fn invalidate_frame(&mut self) {
        self.prepared_scene = None;
        // Camera/layout/chrome changes rebuild the prepared projection only.
        // Schematic authored geometry is camera-independent retained world data,
        // just like the board retained scene, and must stay warm here.
        self.scene_dirty = true;
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

    /// Refresh only screen-space interaction chrome. Cursor and hover motion
    /// must never evict the prepared shell or authored board/schematic geometry:
    /// all three are expensive and independent of transient pointer state.
    pub(super) fn refresh_interaction_overlay(&mut self) {
        if let (Some(prepared), Some(retained)) =
            (self.prepared_scene.as_mut(), self.retained_scene.as_ref())
        {
            prepared.refresh_interaction(self.session.workspace(), retained);
        }
        self.scene_dirty = true;
    }
}

#[cfg(test)]
#[path = "native_dialog_refresh_tests.rs"]
mod tests;
