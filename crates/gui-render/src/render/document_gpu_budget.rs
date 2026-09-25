//! Shared GPU admission keyed by the protocol's revision-independent scene identity.
use crate::text_gpu::budget::Budget;
use std::sync::Arc;

/// One document's world-buffer reservations, including allocation preparation
/// and submitted retirement. API-private/driver storage is a separate boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentGpuUsage {
    pub scene_id: String,
    pub budget_id: u64,
    pub lifetime_peak_reserved_bytes: u64,
    pub reserved_bytes: u64,
    pub limit_bytes: u64,
}

impl crate::Renderer {
    /// Snapshot each shared document allowance once, across renderers/revisions.
    /// Observing a row does not retain its budget or GPU allocations. Concurrent
    /// rows are sampled, not an atomic transaction across resource owners.
    pub fn world_document_gpu_usage() -> Vec<DocumentGpuUsage> {
        crate::retained_scene_owner::document_cpu::gpu_usage()
    }
}

pub(super) fn for_scene(scene_id: &str) -> Arc<Budget> {
    crate::retained_scene_owner::document_cpu::for_scene(scene_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Weak;

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
