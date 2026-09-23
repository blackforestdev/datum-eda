//! Bounded derived-scene history; document/session authority stays elsewhere.
use super::{RetainedScene, Runtime, retained_selection_cache_key};
use datum_gui_render::RetainedGeometryObserver;
use datum_gui_render::cpu_alloc::heap::capacity_bytes;

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
    geometry: RetainedGeometryObserver,
    _document_metadata: datum_gui_render::DocumentCpuCharge,
}

pub(super) struct RetainedSceneHistory {
    entries: Vec<Entry>,
    entry_storage: Option<datum_gui_render::DocumentCpuCharge>,
    heap_bytes: usize,
    active_bytes: usize,
    active_geometry: Option<RetainedGeometryObserver>,
    retired_geometry: Vec<RetainedGeometryObserver>,
    retired_storage: Option<datum_gui_render::DocumentCpuCharge>,
    budget: usize,
    construction_error: Option<String>,
}

impl Default for RetainedSceneHistory {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            entry_storage: None,
            heap_bytes: 0,
            active_bytes: 0,
            active_geometry: None,
            retired_geometry: Vec::new(),
            retired_storage: None,
            budget: MAX_PAYLOAD_BYTES,
            construction_error: None,
        }
    }
}

impl RetainedSceneCacheKey {
    fn heap_bytes(&self) -> Option<usize> {
        let mut bytes = capacity_bytes::<u8>(self.scene_id.capacity())
            .checked_add(capacity_bytes::<u8>(self.source_revision.capacity()))?
            .checked_add(capacity_bytes::<u8>(self.selection.capacity()))?
            .checked_add(capacity_bytes::<(String, bool)>(
                self.layer_visibility.len(),
            ))?;
        for (layer, _) in &self.layer_visibility {
            bytes = bytes.checked_add(capacity_bytes::<u8>(layer.capacity()))?;
        }
        Some(bytes)
    }
}

impl RetainedSceneHistory {
    /// Required derived content is never silently discarded to meet a tier cap.
    /// Report failure through the host render boundary while preserving model
    /// authority and the charged active/pinned owners for inspection or retry.
    pub(super) fn check_render_budget(&mut self) -> anyhow::Result<()> {
        if let Some(error) = &self.construction_error {
            anyhow::bail!("{error}");
        }
        self.trim_for_active_document(MAX_PAYLOAD_BYTES);
        let bytes = self.accounted_bytes();
        anyhow::ensure!(
            bytes <= self.budget,
            "retained world CPU budget exceeded: {bytes} accounted bytes; limit {} (active {}, history {}, pinned/shared ownership included)",
            self.budget,
            self.active_bytes,
            self.owned_history_bytes()
        );
        if let Some(active) = &self.active_geometry {
            active.check_document_cpu_budget()?;
        }
        Ok(())
    }

    pub(super) fn retry_construction(&mut self) {
        self.construction_error = None;
    }

    pub(super) fn clear(&mut self) {
        self.retry_construction();
        if let Some(observer) = self.active_geometry.take() {
            self.observe_retired(observer);
        }
        self.active_bytes = 0;
        self.clear_history_preserving_active();
    }

    fn clear_history_preserving_active(&mut self) {
        while !self.entries.is_empty() {
            self.evict(0);
        }
        self.entries = Vec::new();
        self.entry_storage = None;
        self.prune_retired();
    }

    /// Runtime keeps the exact registered scene when its construction is size
    /// independent. Preserve its complete cached charge without rewalking hits
    /// or draw commands on every resize; obsolete history still retires.
    pub(super) fn invalidate_surface_size(&mut self, active: &mut Option<RetainedScene>) {
        if active
            .as_ref()
            .is_some_and(RetainedScene::can_reuse_for_surface_resize)
        {
            self.clear_history_preserving_active();
        } else {
            *active = None;
            self.clear();
        }
    }

    fn owned_history_bytes(&self) -> usize {
        self.heap_bytes
            .saturating_add(capacity_bytes::<Entry>(self.entries.capacity()))
    }

    fn prune_retired(&mut self) {
        let entries = &self.entries;
        let active = &self.active_geometry;
        self.retired_geometry.retain(|observer| {
            observer.is_live()
                && observer.heap_bytes_excluding(
                    entries
                        .iter()
                        .map(|entry| &entry.geometry)
                        .chain(active.iter()),
                ) != 0
        });
        if self.retired_geometry.capacity() > self.retired_geometry.len().saturating_mul(4) {
            self.retired_geometry = std::mem::take(&mut self.retired_geometry)
                .into_boxed_slice()
                .into_vec();
            if self.retired_geometry.is_empty() {
                self.retired_storage = None;
            } else if let Some(charge) = &mut self.retired_storage {
                charge.resize(capacity_bytes::<RetainedGeometryObserver>(
                    self.retired_geometry.capacity(),
                ));
            }
        }
    }

    fn covered_geometry(&self) -> impl Iterator<Item = &RetainedGeometryObserver> {
        self.entries
            .iter()
            .map(|entry| &entry.geometry)
            .chain(self.active_geometry.iter())
    }

    fn observe_retired(&mut self, observer: RetainedGeometryObserver) {
        self.prune_retired();
        if observer.is_live()
            && observer.heap_bytes_excluding(self.covered_geometry().chain(&self.retired_geometry))
                != 0
        {
            let previous_capacity = self.retired_geometry.capacity();
            self.retired_geometry.push(observer);
            if self.retired_geometry.capacity() != previous_capacity {
                let bytes =
                    capacity_bytes::<RetainedGeometryObserver>(self.retired_geometry.capacity());
                self.retired_storage = Some(
                    self.retired_geometry
                        .last()
                        .expect("retired observer")
                        .charge_document_metadata(bytes),
                );
            }
        }
    }

    fn accounted_bytes(&self) -> usize {
        let mut duplicates = 0usize;
        for (index, entry) in self.entries.iter().enumerate() {
            let full = entry.geometry.heap_bytes_excluding([]);
            let unique = entry.geometry.heap_bytes_excluding(
                self.entries[..index]
                    .iter()
                    .map(|previous| &previous.geometry),
            );
            duplicates = duplicates.saturating_add(full - unique);
        }
        if self.active_bytes != 0
            && let Some(active) = &self.active_geometry
        {
            duplicates = duplicates.saturating_add(
                active.heap_bytes_excluding([])
                    - active.heap_bytes_excluding(self.entries.iter().map(|entry| &entry.geometry)),
            );
        }
        let retired_bytes = self
            .retired_geometry
            .iter()
            .enumerate()
            .map(|(index, observer)| {
                observer.heap_bytes_excluding(
                    self.covered_geometry()
                        .chain(&self.retired_geometry[..index]),
                )
            })
            .fold(0usize, usize::saturating_add);
        self.owned_history_bytes()
            .saturating_add(self.active_bytes)
            .saturating_sub(duplicates)
            .saturating_add(retired_bytes)
            .saturating_add(capacity_bytes::<RetainedGeometryObserver>(
                self.retired_geometry.capacity(),
            ))
    }

    fn bytes_with_candidate(&self, bytes: usize, geometry: &RetainedGeometryObserver) -> usize {
        let duplicates = geometry.heap_bytes_excluding([])
            - geometry.heap_bytes_excluding(self.entries.iter().map(|entry| &entry.geometry));
        self.accounted_bytes().saturating_add(bytes - duplicates)
    }

    fn evict(&mut self, index: usize) {
        let entry = self.remove(index);
        let observer = entry.geometry.clone();
        drop(entry);
        self.observe_retired(observer);
    }

    fn remove(&mut self, index: usize) -> Entry {
        let entry = self.entries.remove(index);
        self.heap_bytes -= entry.heap_bytes;
        entry
    }

    fn limit_for_active(&mut self, scene: &RetainedScene) {
        self.limit_for_active_document(scene, MAX_PAYLOAD_BYTES);
    }

    fn limit_for_active_document(&mut self, scene: &RetainedScene, document_limit: usize) {
        self.retry_construction();
        self.active_geometry = Some(scene.geometry_observer());
        self.prune_retired();
        self.active_bytes = scene
            .heap_payload_bytes()
            .unwrap_or(self.budget.saturating_add(1));
        self.trim_for_active_document(document_limit);
    }

    fn trim_for_active_document(&mut self, document_limit: usize) {
        while !self.entries.is_empty() {
            let victim = if self.accounted_bytes() > self.budget {
                Some(0)
            } else {
                let Some(active) = self.active_geometry.as_ref() else {
                    break;
                };
                if active.document_cpu_payload_bytes() <= document_limit {
                    break;
                }
                self.entries
                    .iter()
                    .position(|entry| entry.geometry.shares_document_with(active))
            };
            let Some(index) = victim else { break };
            self.evict(index);
        }
        if self.entries.is_empty() {
            self.entries = Vec::new();
            self.entry_storage = None;
        }
    }

    pub(super) fn insert(&mut self, key: RetainedSceneCacheKey, scene: RetainedScene) {
        self.insert_document(key, scene, MAX_PAYLOAD_BYTES);
    }

    fn insert_document(
        &mut self,
        key: RetainedSceneCacheKey,
        scene: RetainedScene,
        document_limit: usize,
    ) {
        // The caller moves the old active scene here before rebuilding/restoring.
        self.active_bytes = 0;
        let geometry = scene.geometry_observer();
        self.active_geometry = Some(geometry.clone());
        self.prune_retired();
        if let Some(index) = self.entries.iter().position(|entry| entry.key == key) {
            self.evict(index);
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
        // Keys belong to their document. The vector allocation has a separate
        // lifetime charge, including spare slots and its allocator header.
        let metadata_bytes = key.heap_bytes().unwrap_or(usize::MAX);
        while geometry
            .document_cpu_payload_bytes()
            .saturating_add(metadata_bytes)
            .saturating_add(self.entry_growth_bytes())
            > document_limit
            || geometry.document_history_entries() >= MAX_ENTRIES
        {
            let Some(index) = self
                .entries
                .iter()
                .position(|entry| entry.geometry.shares_document_with(&geometry))
            else {
                break;
            };
            self.evict(index);
        }
        if geometry
            .document_cpu_payload_bytes()
            .saturating_add(metadata_bytes)
            .saturating_add(self.entry_growth_bytes())
            > document_limit
            || geometry.document_history_entries() >= MAX_ENTRIES
        {
            self.active_geometry = None;
            drop(scene);
            self.observe_retired(geometry);
            return;
        }
        while self.entries.len() >= MAX_ENTRIES {
            self.evict(0);
        }
        // A growing vector may briefly own both allocations. Retire history
        // before requesting storage, and reconsider growth after each eviction.
        while !self.entries.is_empty()
            && self
                .bytes_with_candidate(bytes, &geometry)
                .saturating_add(self.entry_growth_bytes())
                > self.budget
        {
            self.evict(0);
        }
        if self
            .bytes_with_candidate(bytes, &geometry)
            .saturating_add(self.entry_growth_bytes())
            > self.budget
        {
            self.clear();
            return;
        }
        // Reserve the shared entry slot before allocating/publishing history.
        // A competing owner can consume the last slot after the preflight query.
        let Some(document_metadata) = geometry.try_charge_document_history(metadata_bytes) else {
            self.active_geometry = None;
            drop(scene);
            self.observe_retired(geometry);
            return;
        };
        if self.entries.len() == self.entries.capacity() {
            let capacity = self.entry_growth_capacity();
            self.entries.reserve_exact(capacity - self.entries.len());
            // Admission above includes the old allocation plus this replacement.
            // Keep the allocating document identity until the buffer is released.
            self.entry_storage = Some(
                geometry.charge_document_metadata(capacity_bytes::<Entry>(self.entries.capacity())),
            );
        }
        self.heap_bytes += bytes;
        self.active_geometry = None;
        self.entries.push(Entry {
            key,
            scene,
            heap_bytes: bytes,
            geometry,
            _document_metadata: document_metadata,
        });
    }

    fn entry_growth_capacity(&self) -> usize {
        self.entries
            .capacity()
            .saturating_mul(2)
            .clamp(1, MAX_ENTRIES)
    }

    fn entry_growth_bytes(&self) -> usize {
        if self.entries.len() < self.entries.capacity() {
            0
        } else {
            capacity_bytes::<Entry>(self.entry_growth_capacity())
        }
    }

    fn take(&mut self, key: &RetainedSceneCacheKey) -> Option<RetainedScene> {
        let index = self.entries.iter().position(|entry| &entry.key == key)?;
        let entry = self.remove(index);
        drop(entry._document_metadata);
        self.limit_for_active(&entry.scene);
        Some(entry.scene)
    }
}

#[path = "retained_scene_lifecycle.rs"]
mod lifecycle;

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn key(index: usize) -> RetainedSceneCacheKey {
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

    pub(super) fn scene() -> RetainedScene {
        RetainedScene::from_workspace(
            &datum_gui_protocol::load_fixture_workspace_state(),
            960,
            720,
        )
    }

    #[test]
    fn history_growth_admits_old_and_new_metadata_before_allocating() {
        for fits_overlap in [false, true] {
            let scene = scene();
            let mut history = RetainedSceneHistory::default();
            history.insert(key(0), scene.clone());
            history.insert(key(1), scene.clone());
            assert_eq!(history.entries.capacity(), 2);
            let candidate_key = key(2);
            let candidate = scene.clone();
            let bytes =
                candidate.heap_payload_bytes().unwrap() + candidate_key.heap_bytes().unwrap();
            let peak = history.bytes_with_candidate(bytes, &candidate.geometry_observer())
                + capacity_bytes::<Entry>(4);
            history.budget = peak - usize::from(!fits_overlap);
            history.insert(candidate_key, candidate);
            assert!(history.entries.iter().any(|entry| entry.key == key(2)));
            assert!(history.accounted_bytes() <= history.budget);
            if fits_overlap {
                assert_eq!(history.entries.len(), 3);
                assert_eq!(history.entries.capacity(), 4);
            } else {
                assert_eq!(history.entries.len(), 2);
                assert_eq!(history.entries.capacity(), 2);
                assert!(history.entries.iter().all(|entry| entry.key != key(0)));
            }
        }
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
        drop(history); // The following case isolates local byte pressure.
        let bytes = scene.clone().heap_payload_bytes().unwrap() + key(0).heap_bytes().unwrap();
        let shared_bytes = scene.geometry_observer().heap_bytes_excluding([]);
        let mut history = RetainedSceneHistory {
            budget: 2 * bytes - shared_bytes + capacity_bytes::<Entry>(4),
            ..Default::default()
        };
        for index in 0..3 {
            history.insert(key(index), scene.clone());
            assert!(history.accounted_bytes() <= history.budget);
        }
        assert_eq!(history.entries.len(), 2);
        assert!(history.take(&key(0)).is_none());
        let recovered = history.take(&key(2)).expect("recent geometry retained");
        assert_eq!(recovered, scene);
        assert!(history.accounted_bytes() <= history.budget);
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
        history.budget = history.accounted_bytes();
        history.limit_for_active(&scene);
        assert_eq!(
            history.entries.len(),
            3,
            "active clone shares complete scene payload"
        );
        history.budget -= 1;
        history.limit_for_active(&scene);
        assert!(history.entries.len() < 3);
        assert!(history.accounted_bytes() <= history.budget);
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
        let error = history.check_render_budget().unwrap_err().to_string();
        assert!(error.contains("retained world CPU budget exceeded"));
        assert!(error.contains("limit 1"));
        assert_eq!(
            scene, active_before,
            "budget failure preserves required scene"
        );
        history.budget = MAX_PAYLOAD_BYTES;
        history.limit_for_active(&scene);
        history.check_render_budget().unwrap();
    }
    #[test]
    fn externally_pinned_geometry_remains_charged_after_history_clear() {
        let old = scene();
        let pinned = old.clone();
        let observed = old.geometry_observer();
        let pinned_bytes = observed.heap_bytes_excluding([]);
        let mut history = RetainedSceneHistory::default();
        history.insert(key(0), old);
        history.clear();
        assert_eq!(history.owned_history_bytes(), 0);
        assert_eq!(history.retired_geometry.len(), 1);
        assert!(history.accounted_bytes() >= pinned_bytes);
        let next = scene();
        let next_bytes = next.heap_payload_bytes().unwrap() + key(1).heap_bytes().unwrap();
        history.budget = next_bytes
            + capacity_bytes::<Entry>(1)
            + capacity_bytes::<RetainedGeometryObserver>(history.retired_geometry.capacity())
            + pinned_bytes
            - 1;
        history.insert(key(1), next);
        assert!(
            history.entries.is_empty(),
            "pinned payload consumes admission headroom"
        );
        drop(pinned);
        assert!(!observed.is_live());
        history.insert(key(1), scene());
        assert_eq!(
            history.entries.len(),
            1,
            "released payload restores headroom"
        );
        assert!(history.retired_geometry.is_empty());
        assert_eq!(history.retired_geometry.capacity(), 0);
        assert!(history.accounted_bytes() <= history.budget);
    }
    #[test]
    fn active_and_history_charge_shared_geometry_once() {
        let scene = scene();
        let geometry_bytes = scene.geometry_observer().heap_bytes_excluding([]);
        let mut history = RetainedSceneHistory::default();
        for index in 0..3 {
            history.insert(key(index), scene.clone());
        }
        let history_bytes = history.owned_history_bytes() - 2 * geometry_bytes;
        assert_eq!(history.accounted_bytes(), history_bytes);
        history.limit_for_active(&scene);
        assert_eq!(
            history.accounted_bytes(),
            history_bytes + scene.heap_payload_bytes().unwrap() - geometry_bytes
        );
    }
}

#[cfg(test)]
#[path = "retained_scene_resize_tests.rs"]
mod resize_tests;

#[cfg(test)]
#[path = "retained_history_heap_tests.rs"]
mod heap_tests;
