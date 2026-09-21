//! Bounded derived-scene history; document/session authority stays elsewhere.
use super::{RetainedScene, Runtime, retained_selection_cache_key};

const MAX_ENTRIES: usize = 6;
const MAX_PAYLOAD_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RetainedSceneCacheKey {
    scene_id: String,
    source_revision: String,
    width: u32,
    height: u32,
    scale_bits: u32,
    dock_height_px: u32,
    show_authored: bool,
    show_proposed: bool,
    show_unrouted: bool,
    dim_unrelated: bool,
    layer_visibility: Box<[(String, bool)]>,
    pub(super) selection: String,
}

struct Entry {
    key: RetainedSceneCacheKey,
    scene: RetainedScene,
    heap_bytes: usize,
}

pub(super) struct RetainedSceneHistory {
    entries: Vec<Entry>,
    heap_bytes: usize,
    active_bytes: usize,
    budget: usize,
}

impl Default for RetainedSceneHistory {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            heap_bytes: 0,
            active_bytes: 0,
            budget: MAX_PAYLOAD_BYTES,
        }
    }
}

impl RetainedSceneCacheKey {
    fn heap_bytes(&self) -> Option<usize> {
        let mut bytes = self
            .scene_id
            .capacity()
            .checked_add(self.source_revision.capacity())?
            .checked_add(self.selection.capacity())?
            .checked_add(std::mem::size_of_val(self.layer_visibility.as_ref()))?;
        for (layer, _) in &self.layer_visibility {
            bytes = bytes.checked_add(layer.capacity())?;
        }
        Some(bytes)
    }
}

impl RetainedSceneHistory {
    pub(super) fn clear(&mut self) {
        self.entries = Vec::new();
        self.heap_bytes = 0;
        self.active_bytes = 0;
    }

    fn owned_history_bytes(&self) -> usize {
        self.heap_bytes.saturating_add(
            self.entries
                .capacity()
                .saturating_mul(std::mem::size_of::<Entry>()),
        )
    }

    fn remove(&mut self, index: usize) -> Entry {
        let entry = self.entries.remove(index);
        self.heap_bytes -= entry.heap_bytes;
        entry
    }

    fn limit_for_active(&mut self, scene: &RetainedScene) {
        self.active_bytes = scene
            .heap_payload_bytes()
            .unwrap_or(self.budget.saturating_add(1));
        while !self.entries.is_empty()
            && self.owned_history_bytes().saturating_add(self.active_bytes) > self.budget
        {
            self.remove(0);
        }
        if self.entries.is_empty() {
            self.entries = Vec::new();
        }
    }

    pub(super) fn insert(&mut self, key: RetainedSceneCacheKey, scene: RetainedScene) {
        // The caller moves the old active scene here before rebuilding/restoring.
        self.active_bytes = 0;
        if let Some(index) = self.entries.iter().position(|entry| entry.key == key) {
            self.remove(index);
        }
        let Some(bytes) = scene
            .heap_payload_bytes()
            .and_then(|bytes| bytes.checked_add(key.heap_bytes()?))
        else {
            self.clear();
            return;
        };
        if bytes > self.budget {
            self.clear();
            return;
        }
        while self.entries.len() >= MAX_ENTRIES {
            self.remove(0);
        }
        self.entries.reserve(1);
        while !self.entries.is_empty()
            && self.owned_history_bytes().saturating_add(bytes) > self.budget
        {
            self.remove(0);
        }
        if self.owned_history_bytes().saturating_add(bytes) > self.budget {
            self.clear();
            return;
        }
        self.heap_bytes += bytes;
        self.entries.push(Entry {
            key,
            scene,
            heap_bytes: bytes,
        });
    }

    fn take(&mut self, key: &RetainedSceneCacheKey) -> Option<RetainedScene> {
        let index = self.entries.iter().position(|entry| &entry.key == key)?;
        let entry = self.remove(index);
        self.limit_for_active(&entry.scene);
        Some(entry.scene)
    }
}

impl Runtime {
    pub(super) fn retained_scene_cache_key(&self) -> RetainedSceneCacheKey {
        let workspace = self.workspace();
        RetainedSceneCacheKey {
            scene_id: workspace.scene.scene_id.clone(),
            source_revision: workspace.scene.source_revision.clone(),
            width: self.config.width,
            height: self.config.height,
            scale_bits: self.scale_factor.to_bits(),
            dock_height_px: workspace.ui.effective_dock_height_px(),
            show_authored: workspace.ui.filters.show_authored,
            show_proposed: workspace.ui.filters.show_proposed,
            show_unrouted: workspace.ui.filters.show_unrouted,
            dim_unrelated: workspace.ui.filters.dim_unrelated,
            layer_visibility: workspace
                .ui
                .filters
                .layer_visibility
                .iter()
                .map(|(key, value)| (key.clone(), *value))
                .collect(),
            selection: retained_selection_cache_key(workspace, &workspace.selection),
        }
    }

    pub(super) fn restore_cached_retained_scene(&mut self) -> bool {
        let key = self.retained_scene_cache_key();
        if let Some(retained) = self.retained_scene_cache.take(&key) {
            self.retained_scene = Some(retained);
            return true;
        }
        false
    }

    pub(super) fn ensure_retained_scene(&mut self) {
        if self.retained_scene.is_none() {
            let retained = RetainedScene::from_workspace_for_surface(
                self.session.workspace(),
                self.config.width,
                self.config.height,
                self.scale_factor,
            );
            self.retained_scene_cache.limit_for_active(&retained);
            self.retained_scene = Some(retained);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(index: usize) -> RetainedSceneCacheKey {
        RetainedSceneCacheKey {
            scene_id: format!("scene-{index}"),
            source_revision: "revision".into(),
            width: 960,
            height: 720,
            scale_bits: 1.0_f32.to_bits(),
            dock_height_px: 100,
            show_authored: true,
            show_proposed: true,
            show_unrouted: true,
            dim_unrelated: false,
            layer_visibility: vec![("F.Cu".into(), true)].into_boxed_slice(),
            selection: "none".into(),
        }
    }

    fn scene() -> RetainedScene {
        RetainedScene::from_workspace(
            &datum_gui_protocol::load_fixture_workspace_state(),
            960,
            720,
        )
    }

    #[test]
    fn history_preserves_six_entries_and_retires_oldest_under_byte_pressure() {
        let scene = scene();
        let mut history = RetainedSceneHistory::default();
        for index in 0..8 {
            history.insert(key(index), scene.clone());
        }
        assert_eq!(history.entries.len(), 6);
        assert!(history.take(&key(0)).is_none());
        assert!(history.take(&key(7)).is_some());
        let bytes = scene.heap_payload_bytes().unwrap() + key(0).heap_bytes().unwrap();
        let mut history = RetainedSceneHistory {
            budget: 2 * bytes + 4 * std::mem::size_of::<Entry>(),
            ..Default::default()
        };
        for index in 0..3 {
            history.insert(key(index), scene.clone());
            assert!(history.owned_history_bytes() <= history.budget);
        }
        assert_eq!(history.entries.len(), 2);
        assert!(history.take(&key(0)).is_none());
        let recovered = history.take(&key(2)).expect("recent geometry retained");
        assert_eq!(recovered, scene);
        assert!(history.owned_history_bytes() + history.active_bytes <= history.budget);
        history.clear();
        assert_eq!(history.owned_history_bytes(), 0);
    }

    #[test]
    fn active_payload_prunes_history_and_oversized_scene_bypasses_retention() {
        let scene = scene();
        let mut history = RetainedSceneHistory::default();
        for index in 0..3 {
            history.insert(key(index), scene.clone());
        }
        history.budget = history.owned_history_bytes();
        history.limit_for_active(&scene);
        assert!(history.entries.len() < 3);
        assert!(history.owned_history_bytes() + history.active_bytes <= history.budget);
        let active_before = scene.clone();
        history.budget = 1;
        history.insert(key(8), scene.clone());
        assert!(history.entries.is_empty());
        assert_eq!(history.owned_history_bytes(), 0);
        history.limit_for_active(&scene);
        assert_eq!(
            scene, active_before,
            "cache refusal never changes active geometry"
        );
        assert_eq!(history.owned_history_bytes(), 0);
    }
}
