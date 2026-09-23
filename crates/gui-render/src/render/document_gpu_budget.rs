//! Shared GPU admission keyed by the protocol's revision-independent scene identity.
use crate::text_gpu::budget::Budget;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, Weak},
};

const DOCUMENT_LIMIT: u64 = 64 * 1024 * 1024;
static DOCUMENTS: Mutex<BTreeMap<String, Weak<Budget>>> = Mutex::new(BTreeMap::new());

/// One document's world-buffer reservations, including allocation preparation
/// and submitted retirement. API-private/driver storage is a separate boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentGpuUsage {
    pub scene_id: String,
    pub reserved_bytes: u64,
    pub limit_bytes: u64,
}

impl crate::Renderer {
    /// Snapshot each shared document allowance once, across renderers/revisions.
    /// Observing a row does not retain its budget or GPU allocations. Concurrent
    /// rows are sampled, not an atomic transaction across resource owners.
    pub fn world_document_gpu_usage() -> Vec<DocumentGpuUsage> {
        let mut documents = DOCUMENTS.lock().unwrap_or_else(|error| error.into_inner());
        documents.retain(|_, budget| budget.strong_count() != 0);
        documents
            .iter()
            .filter_map(|(scene_id, budget)| {
                let budget = budget.upgrade()?;
                Some(DocumentGpuUsage {
                    scene_id: scene_id.clone(),
                    reserved_bytes: budget.used(),
                    limit_bytes: DOCUMENT_LIMIT,
                })
            })
            .collect()
    }
}

pub(super) fn for_scene(scene_id: &str) -> Arc<Budget> {
    let mut documents = DOCUMENTS.lock().unwrap_or_else(|error| error.into_inner());
    documents.retain(|_, budget| budget.strong_count() != 0);
    if let Some(budget) = documents.get(scene_id).and_then(Weak::upgrade) {
        return budget;
    }
    let budget = Budget::new(DOCUMENT_LIMIT);
    documents.insert(scene_id.to_owned(), Arc::downgrade(&budget));
    budget
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocation_permit_keeps_document_identity_after_cpu_owner_closes() {
        let first = for_scene("document-registry-retirement");
        let weak = Arc::downgrade(&first);
        let permit = first.reserve(16).unwrap();
        drop(first);
        let reopened = for_scene("document-registry-retirement");
        assert!(Arc::ptr_eq(&reopened, &weak.upgrade().unwrap()));
        assert_eq!(reopened.used(), 16);
        drop(permit);
        assert_eq!(reopened.used(), 0);
        drop(reopened);
        assert!(weak.upgrade().is_none());
        let fresh = for_scene("document-registry-retirement");
        assert!(!Weak::ptr_eq(&weak, &Arc::downgrade(&fresh)));
    }
}
