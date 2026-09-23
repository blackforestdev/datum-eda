//! Shared retained payload accounting for revision-independent document owners.
//! Includes owned registry and registered history metadata. Construction peaks
//! and admission remain separate. Observation never retains scene payloads.
use super::{RetainedGeometryObserver, RetainedScene};
use crate::cpu_alloc::heap::{allocation_bytes, capacity_bytes};
use crate::text_gpu::budget::Budget;
use std::sync::{Arc, Mutex, Weak};

const DOCUMENT_LIMIT: usize = 64 * 1024 * 1024;
struct Document {
    identity: Weak<Budget>,
    scenes: Vec<RetainedGeometryObserver>,
    metadata_bytes: usize,
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
    fn insert(&mut self, identity: Weak<Budget>, observer: RetainedGeometryObserver) {
        self.head = Some(Box::new(Document {
            identity,
            scenes: vec![observer],
            metadata_bytes: 0,
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
        if document.scenes.is_empty() && document.metadata_bytes == 0 {
            *cursor = document.next.take();
        } else {
            *cursor = Some(document);
            cursor = &mut cursor.as_mut().expect("retained document").next;
        }
    }
}

pub(super) fn register(scene: &RetainedScene) {
    let observer = scene.geometry_observer();
    let Some(identity) = observer.document.clone() else {
        return;
    };
    let mut documents = DOCUMENTS.lock().unwrap_or_else(|e| e.into_inner());
    prune(&mut documents);
    if let Some(document) = documents.find(&identity) {
        if observer.heap_bytes_excluding(&document.scenes) != 0 {
            document.scenes.push(observer);
        }
    } else {
        documents.insert(identity, observer);
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
        }
        prune(&mut documents);
    }
}

impl RetainedGeometryObserver {
    /// Register metadata already owned by the caller. Admission must include the
    /// candidate bytes before publication; the charge keeps cross-owner checks exact.
    pub fn charge_document_metadata(&self, bytes: usize) -> DocumentCpuCharge {
        let mut documents = DOCUMENTS.lock().unwrap_or_else(|e| e.into_inner());
        let identity = self.document.as_ref().and_then(|identity| {
            let live_identity = identity.upgrade()?;
            let document = documents.find(identity)?;
            document.metadata_bytes = document
                .metadata_bytes
                .checked_add(bytes)
                .expect("document CPU metadata overflow");
            Some(live_identity)
        });
        DocumentCpuCharge { identity, bytes }
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
            scope.with(|| registry.insert(observer.document.clone().unwrap(), observer));
        }
        let record_bytes = allocation_bytes(std::alloc::Layout::new::<Document>())
            + capacity_bytes::<RetainedGeometryObserver>(1);
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
