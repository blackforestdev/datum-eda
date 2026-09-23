//! Shared retained payload accounting for revision-independent document owners.
//! This supplements local history caps; construction, spare history capacity and
//! global registry admission remain separate. Observation never retains payloads.
use super::{RetainedGeometryObserver, RetainedScene};
use crate::cpu_alloc::heap::capacity_bytes;
use crate::text_gpu::budget::Budget;
use std::sync::{Mutex, Weak};

const DOCUMENT_LIMIT: usize = 64 * 1024 * 1024;
struct Document {
    identity: Weak<Budget>,
    scenes: Vec<RetainedGeometryObserver>,
    metadata_bytes: usize,
}
static DOCUMENTS: Mutex<Vec<Document>> = Mutex::new(Vec::new());

fn prune(documents: &mut Vec<Document>) {
    documents.retain_mut(|document| {
        document.scenes.retain(RetainedGeometryObserver::is_live);
        if document.scenes.capacity() > document.scenes.len().saturating_mul(4) {
            document.scenes.shrink_to_fit();
        }
        !document.scenes.is_empty() || document.metadata_bytes != 0
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
            metadata_bytes: 0,
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
        capacity_bytes::<RetainedGeometryObserver>(document.scenes.capacity())
            .saturating_add(document.metadata_bytes),
        |bytes, (index, scene)| {
            bytes.saturating_add(scene.heap_bytes_excluding(&document.scenes[..index]))
        },
    )
}

/// Exclusive lifetime charge for a history owner's separately allocated metadata.
/// Moving a charge transfers ownership; dropping it releases the document charge.
pub struct DocumentCpuCharge {
    identity: Option<Weak<Budget>>,
    bytes: usize,
}
impl Drop for DocumentCpuCharge {
    fn drop(&mut self) {
        let Some(identity) = &self.identity else {
            return;
        };
        let mut documents = DOCUMENTS.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(document) = documents.iter_mut().find(|d| d.identity.ptr_eq(identity)) {
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
            let document = documents.iter_mut().find(|d| d.identity.ptr_eq(identity))?;
            document.metadata_bytes = document
                .metadata_bytes
                .checked_add(bytes)
                .expect("document CPU metadata overflow");
            Some(identity.clone())
        });
        DocumentCpuCharge { identity, bytes }
    }

    /// Retained document payload, registered history metadata and observer storage.
    /// Excludes global registry storage and construction scratch.
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
    /// Spare history capacity, global storage and construction remain separate.
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
