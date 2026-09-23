//! Shared retained payload accounting for revision-independent document owners.
//! Includes owned registry and registered history metadata. Construction peaks
//! and admission remain separate. Observation never retains scene payloads.
use super::{RetainedGeometryObserver, RetainedScene};
use crate::cpu_alloc::heap::{allocation_bytes, capacity_bytes};
use crate::text_gpu::budget::Budget;
use std::sync::{Arc, Mutex, Weak};

const DOCUMENT_LIMIT: usize = 64 * 1024 * 1024;
const DOCUMENT_HISTORY_LIMIT: usize = 6;
struct Document {
    scene_id: String,
    identity: Weak<Budget>,
    scenes: Vec<RetainedGeometryObserver>,
    metadata_bytes: usize,
    history_entries: usize,
    next: Option<Box<Document>>,
}
// One allocation per document gives each registry record an exact owner;
// no process vector spare capacity is hidden outside document accounting.
#[derive(Default)]
struct Registry {
    head: Option<Box<Document>>,
}
static DOCUMENTS: Mutex<Registry> = Mutex::new(Registry { head: None });

impl Registry {
    fn find(&mut self, identity: &Weak<Budget>) -> Option<&mut Document> {
        let mut cursor = self.head.as_deref_mut();
        while let Some(document) = cursor {
            if document.identity.ptr_eq(identity) {
                return Some(document);
            }
            cursor = document.next.as_deref_mut();
        }
        None
    }
    fn insert(
        &mut self,
        scene_id: String,
        identity: Weak<Budget>,
        scenes: Vec<RetainedGeometryObserver>,
    ) {
        self.head = Some(Box::new(Document {
            scene_id,
            identity,
            scenes,
            metadata_bytes: 0,
            history_entries: 0,
            next: self.head.take(),
        }));
    }
}

fn prune(documents: &mut Registry) {
    let mut cursor = &mut documents.head;
    while let Some(mut document) = cursor.take() {
        document.scenes.retain(RetainedGeometryObserver::is_live);
        if document.scenes.capacity() > document.scenes.len().saturating_mul(4) {
            document.scenes.shrink_to_fit();
        }
        if document.scenes.is_empty()
            && document.metadata_bytes == 0
            && document.identity.strong_count() == 0
        {
            *cursor = document.next.take();
        } else {
            *cursor = Some(document);
            cursor = &mut cursor.as_mut().expect("retained document").next;
        }
    }
}

pub(crate) fn for_scene(scene_id: &str) -> Arc<Budget> {
    let mut documents = DOCUMENTS.lock().unwrap_or_else(|e| e.into_inner());
    prune(&mut documents);
    let mut cursor = documents.head.as_deref();
    while let Some(document) = cursor {
        if document.scene_id == scene_id
            && let Some(budget) = document.identity.upgrade()
        {
            return budget;
        }
        cursor = document.next.as_deref();
    }
    let budget = Budget::new(DOCUMENT_LIMIT as u64);
    documents.insert(scene_id.to_owned(), Arc::downgrade(&budget), Vec::new());
    budget
}

pub(crate) fn gpu_usage() -> Vec<crate::gpu_data::DocumentGpuUsage> {
    let mut documents = DOCUMENTS.lock().unwrap_or_else(|e| e.into_inner());
    prune(&mut documents);
    let mut result = Vec::new();
    let mut cursor = documents.head.as_deref();
    while let Some(document) = cursor {
        if let Some(budget) = document.identity.upgrade() {
            result.push(crate::gpu_data::DocumentGpuUsage {
                scene_id: document.scene_id.clone(),
                reserved_bytes: budget.used(),
                limit_bytes: DOCUMENT_LIMIT as u64,
            });
        }
        cursor = document.next.as_deref();
    }
    result.sort_unstable_by(|a, b| a.scene_id.cmp(&b.scene_id));
    result
}

pub(super) fn register(scene: &RetainedScene) {
    let observer = scene.geometry_observer();
    let Some(identity) = observer.document.clone() else {
        return;
    };
    let mut documents = DOCUMENTS.lock().unwrap_or_else(|e| e.into_inner());
    prune(&mut documents);
    let document = documents
        .find(&identity)
        .expect("scene owns registered document identity");
    if observer.heap_bytes_excluding(&document.scenes) != 0 {
        document.scenes.push(observer);
    }
}

fn usage(observer: &RetainedGeometryObserver) -> usize {
    let Some(identity) = &observer.document else {
        return 0;
    };
    let mut documents = DOCUMENTS.lock().unwrap_or_else(|e| e.into_inner());
    prune(&mut documents);
    let Some(document) = documents.find(identity) else {
        return 0;
    };
    document.scenes.iter().enumerate().fold(
        capacity_bytes::<RetainedGeometryObserver>(document.scenes.capacity())
            .saturating_add(document.metadata_bytes)
            .saturating_add(capacity_bytes::<u8>(document.scene_id.capacity()))
            .saturating_add(Budget::cpu_allocation_bytes())
            .saturating_add(allocation_bytes(std::alloc::Layout::new::<Document>())),
        |bytes, (index, scene)| {
            bytes.saturating_add(scene.heap_bytes_excluding(&document.scenes[..index]))
        },
    )
}

/// Exclusive lifetime charge for a history owner's separately allocated metadata.
/// Moving a charge transfers ownership; dropping it releases the document charge.
pub struct DocumentCpuCharge {
    identity: Option<Arc<Budget>>,
    bytes: usize,
    history_entry: bool,
}
impl DocumentCpuCharge {
    /// Update the charge after the same owner's buffer changes capacity.
    pub fn resize(&mut self, bytes: usize) {
        if let Some(identity) = &self.identity {
            let mut documents = DOCUMENTS.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(document) = documents.find(&Arc::downgrade(identity)) {
                document.metadata_bytes = document
                    .metadata_bytes
                    .checked_sub(self.bytes)
                    .and_then(|value| value.checked_add(bytes))
                    .expect("document CPU metadata accounting");
            }
        }
        self.bytes = bytes;
    }
}
impl Drop for DocumentCpuCharge {
    fn drop(&mut self) {
        let Some(identity) = &self.identity else {
            return;
        };
        let mut documents = DOCUMENTS.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(document) = documents.find(&Arc::downgrade(identity)) {
            document.metadata_bytes -= self.bytes;
            document.history_entries -= usize::from(self.history_entry);
        }
        prune(&mut documents);
    }
}

impl RetainedGeometryObserver {
    /// Register metadata already owned by the caller. Admission must include the
    /// candidate bytes before publication; the charge keeps cross-owner checks exact.
    pub fn charge_document_metadata(&self, bytes: usize) -> DocumentCpuCharge {
        self.charge_metadata(bytes, false)
            .unwrap_or(DocumentCpuCharge {
                identity: None,
                bytes,
                history_entry: false,
            })
    }

    /// Atomically admit one historical entry across all owners of this document.
    /// Active scenes and externally pinned payloads are not historical entries.
    pub fn try_charge_document_history(&self, bytes: usize) -> Option<DocumentCpuCharge> {
        self.charge_metadata(bytes, true)
    }

    pub fn document_history_entries(&self) -> usize {
        let Some(identity) = &self.document else {
            return 0;
        };
        DOCUMENTS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .find(identity)
            .map_or(0, |document| document.history_entries)
    }

    fn charge_metadata(&self, bytes: usize, history_entry: bool) -> Option<DocumentCpuCharge> {
        let Some(identity) = &self.document else {
            return Some(DocumentCpuCharge {
                identity: None,
                bytes,
                history_entry: false,
            });
        };
        let live_identity = identity.upgrade()?;
        let mut documents = DOCUMENTS.lock().unwrap_or_else(|e| e.into_inner());
        let document = documents.find(identity)?;
        if history_entry && document.history_entries >= DOCUMENT_HISTORY_LIMIT {
            return None;
        }
        document.metadata_bytes = document
            .metadata_bytes
            .checked_add(bytes)
            .expect("document CPU metadata overflow");
        document.history_entries += usize::from(history_entry);
        Some(DocumentCpuCharge {
            identity: Some(live_identity),
            bytes,
            history_entry,
        })
    }

    /// Retained document payload, registered history metadata and observer storage.
    /// Includes its owned registry record; excludes construction scratch.
    pub fn document_cpu_payload_bytes(&self) -> usize {
        usage(self)
    }

    pub fn shares_document_with(&self, other: &Self) -> bool {
        match (&self.document, &other.document) {
            (Some(first), Some(second)) => first.ptr_eq(second),
            _ => false,
        }
    }

    /// Enforce retained payload across all registered owners of this document.
    /// Includes registered history metadata and per-document observer storage.
    /// Construction peaks and admission remain separate.
    pub fn check_document_cpu_budget(&self) -> anyhow::Result<()> {
        check_limit(self, DOCUMENT_LIMIT)
    }
}
fn check_limit(observer: &RetainedGeometryObserver, limit: usize) -> anyhow::Result<()> {
    let bytes = usage(observer);
    anyhow::ensure!(
        bytes <= limit,
        "document retained CPU payload budget exceeded: {bytes} bytes across live/retiring owners; limit {limit}"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn concurrent_history_admission_cannot_exceed_six_or_release_early() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.scene.scene_id = "cpu-document-history-concurrency".into();
        let scene = RetainedScene::from_workspace(&state, 960, 720);
        let observer = scene.geometry_observer();
        let barrier = Arc::new(std::sync::Barrier::new(9));
        let workers: Vec<_> = (0..8)
            .map(|_| {
                let observer = observer.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    let charge = observer.try_charge_document_history(0);
                    barrier.wait();
                    barrier.wait();
                    charge
                })
            })
            .collect();
        barrier.wait();
        assert_eq!(observer.document_history_entries(), 6);
        assert!(observer.try_charge_document_history(0).is_none());
        barrier.wait();
        let charges: Vec<_> = workers
            .into_iter()
            .filter_map(|worker| worker.join().unwrap())
            .collect();
        assert_eq!(charges.len(), 6);
        assert_eq!(observer.document_history_entries(), 6);
        drop(charges);
        assert_eq!(observer.document_history_entries(), 0);
        assert!(observer.try_charge_document_history(0).is_some());
    }

    #[test]
    fn production_document_payload_and_identity_match_live_allocator_bytes() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.scene.scene_id = "cpu-document-allocator-warmup".into();
        drop(RetainedScene::from_workspace(&state, 960, 720));
        gpu_usage();
        state.scene.scene_id = "cpu-document-allocator-owned".into();
        let scope = crate::cpu_alloc::Scope::new("complete-document-owner");
        let scene = scope.with(|| RetainedScene::from_workspace(&state, 960, 720));
        let observer = scene.geometry_observer();
        let live = || {
            let value = scope.usage();
            (value.payload_bytes + value.tracking_bytes) as usize
        };
        assert_eq!(usage(&observer), live());
        let clone = scene.clone();
        drop(scene);
        assert_eq!(usage(&observer), live());
        drop(clone);
        drop(observer);
        gpu_usage();
        assert_eq!(live(), 0);
    }

    #[test]
    fn registry_records_match_allocator_and_release_independently() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.scene.scene_id = "cpu-registry-first".into();
        let first = RetainedScene::from_workspace(&state, 960, 720);
        state.scene.scene_id = "cpu-registry-second".into();
        let second = RetainedScene::from_workspace(&state, 960, 720);
        let scope = crate::cpu_alloc::Scope::new("document-registry-storage");
        let mut registry = Registry::default();
        for scene in [&first, &second] {
            let observer = scene.geometry_observer();
            scope.with(|| {
                registry.insert(
                    "registry-owned-key".into(),
                    observer.document.clone().unwrap(),
                    vec![observer],
                )
            });
        }
        let record_bytes = allocation_bytes(std::alloc::Layout::new::<Document>())
            + capacity_bytes::<RetainedGeometryObserver>(1)
            + capacity_bytes::<u8>("registry-owned-key".len());
        let live = || {
            let usage = scope.usage();
            (usage.payload_bytes + usage.tracking_bytes) as usize
        };
        assert_eq!(live(), 2 * record_bytes);
        drop(first);
        prune(&mut registry);
        assert_eq!(live(), record_bytes);
        drop(second);
        prune(&mut registry);
        assert_eq!(live(), 0);
        assert!(registry.head.is_none());
    }

    #[test]
    fn document_payload_aggregates_distinct_scenes_deduplicates_clones_and_releases() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.scene.scene_id = "cpu-document-aggregation-proof".into();
        let first = RetainedScene::from_workspace(&state, 960, 720);
        let observer = first.geometry_observer();
        let payload = first.heap_payload_bytes().unwrap();
        let initial = usage(&observer);
        assert!(initial > payload, "observer storage is included");
        let clone = first.clone();
        assert_eq!(usage(&observer), initial);
        let second = RetainedScene::from_workspace(&state, 960, 720);
        assert!(usage(&observer) >= initial + payload);
        let both = usage(&observer);
        check_limit(&observer, both).unwrap();
        assert!(check_limit(&observer, both - 1).is_err());
        state.scene.scene_id = "cpu-document-isolation-proof".into();
        let other = RetainedScene::from_workspace(&state, 960, 720);
        assert_eq!(
            usage(&observer),
            both,
            "another document is not charged here"
        );
        drop(first);
        assert_eq!(
            usage(&observer),
            both,
            "external clone retains complete payload"
        );
        drop(clone);
        assert!(usage(&observer) < both);
        assert!(usage(&observer) >= second.heap_payload_bytes().unwrap());
        drop(second);
        assert_eq!(usage(&observer), 0);
        assert!(usage(&other.geometry_observer()) > 0);
    }
}
