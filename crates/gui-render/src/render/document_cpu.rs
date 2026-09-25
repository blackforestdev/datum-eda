//! Shared retained payload accounting for revision-independent document owners.
//! Includes owned registry and registered history metadata. Construction peaks
//! and admission remain separate. Observation never retains scene payloads.
use super::{ObserverLifetime, RetainedGeometryObserver, RetainedScene, observer_lifetime_bytes};
use crate::cpu_alloc::heap::{allocation_bytes, capacity_bytes};
use crate::text_gpu::budget::Budget;
use std::sync::{Arc, Mutex, Weak};

pub(crate) const DOCUMENT_LIMIT: usize = 64 * 1024 * 1024;
const DOCUMENT_HISTORY_LIMIT: usize = 6;
struct Document {
    budget_id: u64,
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
// Cold constructors must not each spend the same uncommitted document headroom.
// Native hosts already construct serially; this also protects other API callers.
static CONSTRUCTION: Mutex<()> = Mutex::new(());

pub(crate) fn with_constructor<T>(scope: &crate::cpu_alloc::Scope, build: impl FnOnce() -> T) -> T {
    let _construction = CONSTRUCTION.lock().unwrap_or_else(|e| e.into_inner());
    scope.with(build)
}

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
            budget_id: identity
                .upgrade()
                .expect("document registration has a live budget")
                .id(),
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
        document.scenes.retain(|scene| {
            scene.is_live()
                || scene
                    .lifetime
                    .as_ref()
                    .is_some_and(|owner| Arc::strong_count(owner) > 1)
        });
        // Pruning must not allocate while observing or enforcing a budget.
        // Keep admitted capacity for reuse; release it when no scene remains.
        if document.scenes.is_empty() {
            document.scenes = Vec::new();
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
                budget_id: budget.id(),
                lifetime_peak_reserved_bytes: budget.peak(),
                reserved_bytes: budget.used(),
                limit_bytes: DOCUMENT_LIMIT as u64,
            });
        }
        cursor = document.next.as_deref();
    }
    result.sort_unstable_by(|a, b| a.scene_id.cmp(&b.scene_id));
    result
}

pub(crate) fn cpu_usage() -> Vec<crate::resource_observation::DocumentCpuUsage> {
    let mut documents = DOCUMENTS.lock().unwrap_or_else(|e| e.into_inner());
    prune(&mut documents);
    let mut result = Vec::new();
    let mut cursor = documents.head.as_deref();
    while let Some(document) = cursor {
        result.push(crate::resource_observation::DocumentCpuUsage {
            scene_id: document.scene_id.clone(),
            budget_id: document.budget_id,
            retained_bytes: document_bytes(document),
            history_entries: document.history_entries,
            limit_bytes: DOCUMENT_LIMIT,
        });
        cursor = document.next.as_deref();
    }
    result.sort_unstable_by(|a, b| {
        (a.scene_id.as_str(), a.budget_id).cmp(&(b.scene_id.as_str(), b.budget_id))
    });
    result
}

/// Reuse the registered observer lease without allocating on observation.
/// Its shared lifetime keeps weak-only Arc containers in document accounting.
pub(super) fn observe(observer: RetainedGeometryObserver) -> RetainedGeometryObserver {
    let Some(identity) = &observer.document else {
        return observer;
    };
    let mut documents = DOCUMENTS.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(document) = documents.find(identity)
        && let Some(existing) = document.scenes.iter().find(|existing| {
            existing.vertices.ptr_eq(&observer.vertices)
                && existing.strokes.ptr_eq(&observer.strokes)
                && existing.commands.ptr_eq(&observer.commands)
                && existing.hits.ptr_eq(&observer.hits)
        })
    {
        return existing.clone();
    }
    observer
}

pub(super) fn register(
    scene: &RetainedScene,
    scope: &crate::cpu_alloc::Scope,
    limit: usize,
) -> anyhow::Result<()> {
    let mut observer = scene.geometry_observer();
    let Some(identity) = observer.document.clone() else {
        return Ok(());
    };
    let mut documents = DOCUMENTS.lock().unwrap_or_else(|e| e.into_inner());
    prune(&mut documents);
    let document = documents
        .find(&identity)
        .expect("scene owns registered document identity");
    if observer.heap_bytes_excluding(&document.scenes) == 0 {
        return Ok(());
    }
    let capacity = if document.scenes.len() == document.scenes.capacity() {
        document.scenes.capacity().saturating_mul(2).max(1)
    } else {
        0
    };
    let growth = allocation_bytes(std::alloc::Layout::array::<RetainedGeometryObserver>(
        capacity,
    )?);
    let staging = scope.usage();
    let required = (document_bytes(document) as u64)
        .saturating_add(staging.payload_bytes)
        .saturating_add(staging.tracking_bytes)
        .saturating_add(growth as u64)
        .saturating_add(observer_lifetime_bytes() as u64);
    anyhow::ensure!(
        required <= limit as u64,
        "retained scene registry publication exceeds document CPU budget: {required} bytes including constructor and replacement storage; limit {limit}"
    );
    if capacity != 0 {
        document
            .scenes
            .try_reserve_exact(capacity - document.scenes.len())?;
    }
    observer.lifetime = Some(Arc::new(ObserverLifetime {
        _document: identity.upgrade(),
    }));
    document.scenes.push(observer);
    Ok(())
}

fn usage(observer: &RetainedGeometryObserver) -> usize {
    let Some(identity) = &observer.document else {
        return 0;
    };
    usage_for_identity(identity)
}

fn usage_for_identity(identity: &Weak<Budget>) -> usize {
    let mut documents = DOCUMENTS.lock().unwrap_or_else(|e| e.into_inner());
    prune(&mut documents);
    let Some(document) = documents.find(identity) else {
        return 0;
    };
    document_bytes(document)
}

fn document_bytes(document: &Document) -> usize {
    document.scenes.iter().enumerate().fold(
        capacity_bytes::<RetainedGeometryObserver>(document.scenes.capacity())
            .saturating_add(document.metadata_bytes)
            .saturating_add(capacity_bytes::<u8>(document.scene_id.capacity()))
            .saturating_add(Budget::cpu_allocation_bytes())
            .saturating_add(allocation_bytes(std::alloc::Layout::new::<Document>())),
        |bytes, (index, scene)| {
            bytes
                .saturating_add(scene.heap_bytes_excluding(&document.scenes[..index]))
                .saturating_add(usize::from(scene.lifetime.is_some()) * observer_lifetime_bytes())
        },
    )
}

/// Check the live constructor staging plus proposed quad-to-vertex expansion
/// against the document allowance before allocating the expanded vertex buffer.
pub(crate) fn admit_vertex_expansion(
    budget: &Arc<Budget>,
    scope: &crate::cpu_alloc::Scope,
    quads: usize,
    limit: usize,
) -> anyhow::Result<()> {
    let count = quads
        .checked_mul(6)
        .ok_or_else(|| anyhow::anyhow!("retained vertex count overflow"))?;
    let layout = std::alloc::Layout::array::<crate::Vertex>(count)?;
    admit_constructor_allocation(
        budget,
        scope,
        allocation_bytes(layout),
        limit,
        "vertex expansion",
    )
}

pub(crate) fn admit_constructor_allocation(
    budget: &Arc<Budget>,
    scope: &crate::cpu_alloc::Scope,
    additional_bytes: usize,
    limit: usize,
    operation: &str,
) -> anyhow::Result<()> {
    let usage = scope.usage();
    let staging = usage.payload_bytes.saturating_add(usage.tracking_bytes);
    let existing = usage_for_identity(&Arc::downgrade(budget));
    let required = (existing as u64)
        .saturating_add(staging)
        .saturating_add(additional_bytes as u64);
    anyhow::ensure!(
        required <= limit as u64,
        "retained scene {operation} exceeds document CPU budget: {required} bytes including live owners and constructor staging; limit {limit}"
    );
    Ok(())
}

/// Exclusive lifetime charge for a history owner's separately allocated metadata.
/// Moving a charge transfers ownership; dropping it releases the document charge.
pub struct DocumentCpuCharge {
    identity: Option<Arc<Budget>>,
    bytes: usize,
    history_entry: bool,
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
    /// Reserve separately owned history metadata before allocating its storage.
    pub fn try_charge_document_storage(&self, bytes: usize) -> Option<DocumentCpuCharge> {
        self.charge_metadata(bytes, false)
    }

    /// Atomically admit one historical entry across all owners of this document.
    /// Active scenes and externally pinned payloads are not historical entries.
    pub fn try_charge_document_history(&self, bytes: usize) -> Option<DocumentCpuCharge> {
        self.charge_metadata(bytes, true)
    }

    /// Reserve an entry and optional replacement vector together. The old vector
    /// remains charged by its existing owner until the caller installs the new one.
    pub fn try_charge_document_history_storage(
        &self,
        key_bytes: usize,
        storage_bytes: usize,
    ) -> Option<(DocumentCpuCharge, Option<DocumentCpuCharge>)> {
        let mut entry = self.try_charge_document_history(key_bytes.checked_add(storage_bytes)?)?;
        let storage = (storage_bytes != 0).then(|| DocumentCpuCharge {
            identity: entry.identity.clone(),
            bytes: storage_bytes,
            history_entry: false,
        });
        entry.bytes = key_bytes;
        Some((entry, storage))
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
        // Reservations must not spend headroom currently used by a constructor.
        let _construction = CONSTRUCTION.lock().unwrap_or_else(|e| e.into_inner());
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
        if (history_entry && document.history_entries >= DOCUMENT_HISTORY_LIMIT)
            || document_bytes(document)
                .checked_add(bytes)
                .is_none_or(|required| required > DOCUMENT_LIMIT)
        {
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
#[path = "document_cpu_tests.rs"]
mod tests;
