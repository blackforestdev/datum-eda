//! Retained-scene and transient interaction refresh policy.
//!
//! Keeping these paths together makes the performance boundary explicit:
//! pointer-only changes refresh interaction chrome, while authored/session
//! changes may invalidate the considerably more expensive retained geometry.

use super::{RETAINED_SCENE_CACHE_LIMIT, RetainedScene, RetainedSceneCacheKey, Runtime};

impl Runtime {
    pub(super) fn cache_retained_scene(
        &mut self,
        key: RetainedSceneCacheKey,
        retained: RetainedScene,
    ) {
        if let Some(index) = self
            .retained_scene_cache
            .iter()
            .position(|(cached_key, _)| cached_key == &key)
        {
            self.retained_scene_cache.remove(index);
        }
        self.retained_scene_cache.push((key, retained));
        if self.retained_scene_cache.len() > RETAINED_SCENE_CACHE_LIMIT {
            self.retained_scene_cache.remove(0);
        }
    }

    pub(super) fn invalidate_scene_for_session_change(
        &mut self,
        previous_key: RetainedSceneCacheKey,
    ) {
        if let Some(retained) = self.retained_scene.take() {
            self.cache_retained_scene(previous_key, retained);
        }
        self.prepared_scene = None;
        self.schematic_retained_scene = None;
        self.scene_dirty = true;
        self.restore_cached_retained_scene();
    }

    pub(super) fn invalidate_scene(&mut self) {
        self.retained_scene = None;
        self.retained_scene_cache.clear();
        self.prepared_scene = None;
        self.schematic_retained_scene = None;
        self.scene_dirty = true;
    }

    pub(super) fn invalidate_frame(&mut self) {
        self.prepared_scene = None;
        // Camera/layout/chrome changes rebuild the prepared projection only.
        // Schematic authored geometry is camera-independent retained world data,
        // just like the board retained scene, and must stay warm here.
        self.scene_dirty = true;
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
