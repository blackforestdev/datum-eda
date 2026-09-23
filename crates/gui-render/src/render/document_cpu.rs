//! Shared retained payload accounting for revision-independent document owners.
//! Includes owned registry and registered history metadata. Construction peaks
//! and admission remain separate. Observation never retains scene payloads.
use super::{RetainedGeometryObserver, RetainedScene};
use crate::cpu_alloc::heap::{allocation_bytes, capacity_bytes};
use crate::text_gpu::budget::Budget;
use std::sync::{Arc, Mutex, Weak};

pub(crate) const DOCUMENT_LIMIT: usize = 64 * 1024 * 1024;
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
                reserved_bytes: budget.used(),
                limit_bytes: DOCUMENT_LIMIT as u64,
            });
        }
        cursor = document.next.as_deref();
    }
    result.sort_unstable_by(|a, b| a.scene_id.cmp(&b.scene_id));
    result
}

pub(super) fn register(
    scene: &RetainedScene,
    scope: &crate::cpu_alloc::Scope,
    limit: usize,
) -> anyhow::Result<()> {
    let observer = scene.geometry_observer();
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
        .saturating_add(growth as u64);
    anyhow::ensure!(
        required <= limit as u64,
        "retained scene registry publication exceeds document CPU budget: {required} bytes including constructor and replacement storage; limit {limit}"
    );
    if capacity != 0 {
        document
            .scenes
            .try_reserve_exact(capacity - document.scenes.len())?;
    }
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
            bytes.saturating_add(scene.heap_bytes_excluding(&document.scenes[..index]))
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
    fn pruning_reuses_charged_capacity_without_allocating_and_releases_empty_storage() {
        let mut state = datum_gui_protocol::load_fixture_workspace_state();
        state.scene.scene_id = "registry-pruning-reuse".into();
        let scene = RetainedScene::from_workspace(&state, 960, 720);
        let observer = scene.geometry_observer();
        let budget = scene.world_vertices.document_budget().unwrap().clone();
        let mut registry = Registry::default();
        let mut scenes = Vec::with_capacity(32);
        scenes.push(observer.clone());
        let pointer = scenes.as_ptr();
        registry.insert(
            "registry-pruning-reuse".into(),
            Arc::downgrade(&budget),
            scenes,
        );
        let before = document_bytes(registry.head.as_ref().unwrap());
        let scope = crate::cpu_alloc::Scope::new("registry-pruning-allocation");
        scope.with(|| prune(&mut registry));
        let document = registry.head.as_ref().unwrap();
        assert_eq!(document.scenes.as_ptr(), pointer);
        assert_eq!(document.scenes.capacity(), 32);
        assert_eq!(document_bytes(document), before);
        assert_eq!(scope.usage().peak_payload_bytes, 0);
        drop(scene);
        scope.with(|| prune(&mut registry));
        assert_eq!(registry.head.as_ref().unwrap().scenes.capacity(), 0);
        assert_eq!(scope.usage().peak_payload_bytes, 0);
        drop(budget);
        prune(&mut registry);
        assert!(registry.head.is_none());
    }

    #[test]
    fn publication_refusal_preserves_registry_and_retry_registers_unique_metadata() {
        let mut state = crate::gpu_surface_pass::board_fixture_state();
        state.scene.scene_id = "construction-publication-refusal".into();
        let scene = RetainedScene::from_workspace(&state, 960, 720);
        let scope = crate::cpu_alloc::Scope::new("publication-candidate");
        let mut candidate = scene.clone();
        candidate.draw_commands = scope.with(|| Arc::new(scene.draw_commands.as_ref().clone()));
        let before = usage(&scene.geometry_observer());
        let candidate_bytes = scope.usage();
        let error = register(&candidate, &scope, 0).unwrap_err();
        assert!(error.to_string().contains("registry publication exceeds"));
        assert_eq!(usage(&scene.geometry_observer()), before);
        assert_eq!(scope.usage().allocations, candidate_bytes.allocations);
        register(&candidate, &scope, DOCUMENT_LIMIT).unwrap();
        assert!(usage(&scene.geometry_observer()) > before);
        drop(candidate);
        drop(scene);
        gpu_usage();
        assert_eq!(scope.usage().allocations, 0);
    }

    #[test]
    fn shared_owner_refusal_allocates_no_owners_and_preserves_staging() {
        let mut state = crate::gpu_surface_pass::board_fixture_state();
        state.scene.scene_id = "construction-owner-refusal".into();
        let scene = RetainedScene::from_workspace(&state, 960, 720);
        let budget = scene.world_vertices.document_budget().unwrap();
        let scope = crate::cpu_alloc::Scope::new("shared-owner-staging");
        let mut vertices = scope.with(|| scene.world_vertices().to_vec());
        scope.with(|| vertices.reserve_exact(7));
        let strokes = scope.with(|| scene.world_strokes.to_vec());
        let before = scope.usage();
        let error =
            RetainedScene::admit_shared_owners(&vertices, &strokes, budget, &scope, 0).unwrap_err();
        assert!(error.to_string().contains("shared owners exceeds"));
        RetainedScene::admit_shared_owners(&vertices, &strokes, budget, &scope, DOCUMENT_LIMIT)
            .unwrap();
        assert_eq!(scope.usage().allocations, before.allocations);
        assert_eq!(vertices.as_slice(), scene.world_vertices());
    }

    #[test]
    fn real_hit_index_admission_includes_staging_and_matches_allocator() {
        let mut state = crate::gpu_surface_pass::board_fixture_state();
        state.scene.scene_id = "construction-hit-index-admission".into();
        let scene = RetainedScene::from_workspace(&state, 960, 720);
        let budget = scene.world_vertices.document_budget().unwrap();
        let scope = crate::cpu_alloc::Scope::new("hit-index-construction-proof");
        let regions = scope.with(|| scene.world_hit_index.regions().to_vec());
        assert!(!regions.is_empty());
        let layouts =
            datum_gui_viewport::SpatialHitIndex::<crate::HitTarget>::construction_layouts(
                regions.len(),
            )
            .unwrap();
        let extra: usize = layouts.into_iter().map(allocation_bytes).sum();
        let staging = scope.usage();
        let limit = usage(&scene.geometry_observer())
            + (staging.payload_bytes + staging.tracking_bytes) as usize
            + extra;
        let error =
            RetainedScene::admitted_hit_index(regions, budget, &scope, limit - 1).unwrap_err();
        assert!(error.to_string().contains("hit index exceeds"));
        assert_eq!(scope.usage().allocations, 0);
        let regions = scope.with(|| scene.world_hit_index.regions().to_vec());
        let index = scope
            .with(|| RetainedScene::admitted_hit_index(regions, budget, &scope, limit))
            .unwrap();
        let bytes = index
            .heap_bytes_with(
                |target| {
                    Some(match target {
                        crate::HitTarget::AuthoredObject(id) => capacity_bytes::<u8>(id.capacity()),
                        _ => 0,
                    })
                },
                |layout| Some(allocation_bytes(layout)),
            )
            .unwrap();
        let live = scope.usage();
        assert_eq!(bytes as u64, live.payload_bytes + live.tracking_bytes);
        drop(index);
        assert_eq!(scope.usage().allocations, 0);
    }

    #[test]
    fn board_and_companion_constructors_refuse_budget_and_allow_retry() {
        let mut state = crate::gpu_surface_pass::board_fixture_state();
        state.scene.scene_id = "construction-board-refusal".into();
        let error = RetainedScene::from_workspace_bounded(&state, 960, 720, 1.0, 0).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("geometry emission exceeds document CPU budget")
        );
        let board = RetainedScene::try_from_workspace_for_surface(&state, 960, 720, 1.0).unwrap();
        assert!(!board.world_vertices().is_empty() || !board.world_strokes().is_empty());
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../engine/testdata/import/kicad/simple-demo.kicad_sch");
        let mut schematic =
            datum_gui_protocol::load_kicad_schematic_workspace_state(&path).unwrap();
        schematic.scene.scene_id = "construction-schematic-refusal".into();
        state.schematic_scene = Some(schematic.scene);
        let error =
            RetainedScene::schematic_workspace_bounded(&state, 960, 720, 1.0, 0).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("geometry emission exceeds document CPU budget")
        );
        assert!(
            RetainedScene::try_from_workspace_schematic_for_surface(&state, 960, 720, 1.0)
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn vertex_expansion_admission_includes_live_owners_staging_and_requested_capacity() {
        let budget = for_scene("construction-expansion-boundary");
        let scope = crate::cpu_alloc::Scope::new("construction-boundary-proof");
        let staging = scope.with(|| vec![0u8; 4096]);
        let before = scope.usage();
        let existing = usage_for_identity(&Arc::downgrade(&budget));
        let limit = existing
            + (before.payload_bytes + before.tracking_bytes) as usize
            + capacity_bytes::<crate::Vertex>(6);
        admit_vertex_expansion(&budget, &scope, 1, limit).unwrap();
        assert!(admit_vertex_expansion(&budget, &scope, 1, limit - 1).is_err());
        assert!(admit_vertex_expansion(&budget, &scope, usize::MAX, limit).is_err());
        assert_eq!(scope.usage().allocations, before.allocations);
        assert_eq!(staging.len(), 4096);
    }

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
        let construction_owner = crate::cpu_alloc::usage()
            .into_iter()
            .filter(|value| {
                value.label == "retained-board-construction"
                    && value.owner_id > scope.usage().owner_id
            })
            .map(|value| value.owner_id)
            .max()
            .expect("constructor owner");
        let live = || {
            let value = scope.usage();
            let construction = crate::cpu_alloc::usage()
                .into_iter()
                .find(|value| value.owner_id == construction_owner)
                .map_or(0, |value| value.payload_bytes + value.tracking_bytes);
            (value.payload_bytes + value.tracking_bytes + construction) as usize
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
