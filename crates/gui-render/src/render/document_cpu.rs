//! Shared retained payload accounting for revision-independent document owners.
//! This supplements local history caps; construction and registry/global history
//! metadata admission remain separate. Observation never retains scene payloads.
use super::{RetainedGeometryObserver, RetainedScene};
use crate::cpu_alloc::heap::capacity_bytes;
use crate::text_gpu::budget::Budget;
use std::sync::{Mutex, Weak};

const DOCUMENT_LIMIT: usize = 64 * 1024 * 1024;
struct Document {
    identity: Weak<Budget>,
    scenes: Vec<RetainedGeometryObserver>,
}
static DOCUMENTS: Mutex<Vec<Document>> = Mutex::new(Vec::new());

fn prune(documents: &mut Vec<Document>) {
    documents.retain_mut(|document| {
        document.scenes.retain(RetainedGeometryObserver::is_live);
        if document.scenes.capacity() > document.scenes.len().saturating_mul(4) {
            document.scenes.shrink_to_fit();
        }
        !document.scenes.is_empty()
    });
    if documents.capacity() > documents.len().saturating_mul(4) {
        documents.shrink_to_fit();
    }
}

pub(super) fn register(scene: &RetainedScene) {
    let observer = scene.geometry_observer();
    let Some(identity) = observer.document.clone() else {
        return;
    };
    let mut documents = DOCUMENTS.lock().unwrap_or_else(|e| e.into_inner());
    prune(&mut documents);
    if let Some(document) = documents.iter_mut().find(|d| d.identity.ptr_eq(&identity)) {
        if observer.heap_bytes_excluding(&document.scenes) != 0 {
            document.scenes.push(observer);
        }
    } else {
        documents.push(Document {
            identity,
            scenes: vec![observer],
        });
    }
}

fn usage(observer: &RetainedGeometryObserver) -> usize {
    let Some(identity) = &observer.document else {
        return 0;
    };
    let mut documents = DOCUMENTS.lock().unwrap_or_else(|e| e.into_inner());
    prune(&mut documents);
    let Some(document) = documents.iter().find(|d| d.identity.ptr_eq(identity)) else {
        return 0;
    };
    document.scenes.iter().enumerate().fold(
        capacity_bytes::<RetainedGeometryObserver>(document.scenes.capacity()),
        |bytes, (index, scene)| {
            bytes.saturating_add(scene.heap_bytes_excluding(&document.scenes[..index]))
        },
    )
}

impl RetainedGeometryObserver {
    /// Retained document payload plus per-document observer storage; excludes
    /// history keys/entries, global registry storage and construction scratch.
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
    /// Includes per-document observer storage. Local history keys/entry metadata,
    /// global registry storage and candidate construction have separate accounting.
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
