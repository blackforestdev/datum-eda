//! Allocation identities and explicit submission holds for the shared GPU owners.
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};

#[path = "lifetime_state.rs"]
mod state;
use state::{Metadata, ReferenceKind};
pub use state::{ReleasedAllocations, RetirementReason};

static PROCESS_ALLOCATIONS: Mutex<Vec<Weak<Identity>>> = Mutex::new(Vec::new());

static PROCESS_RELEASED: Mutex<[ReleasedAllocations; 5]> = Mutex::new(
    [ReleasedAllocations {
        allocations: 0,
        capacity_bytes: 0,
    }; 5],
);

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Texture,
    Instances,
    Vertex,
    TerminalTexture,
    Attachment,
    Uniform,
    Staging,
    Query,
    QueryResolve,
    Readback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    pub id: u64,
    pub owner: u64,
    /// Renderer instance that created this resource. None means creation outside
    /// a renderer (for example a standalone capture target), never inferred ownership.
    pub renderer_id: Option<u64>,
    pub generation: u64,
    /// Allocated API capacity; never inferred from upload traffic.
    pub bytes: u64,
    /// Latest requested logical buffer extent, or full requested texture/query/storage extent.
    /// This describes prepared ownership, not submitted content or atlas occupancy.
    pub requested_bytes: u64,
    pub prepared_references: u64,
    pub submission_references: u64,
    pub retirement_reason: Option<RetirementReason>,
    pub kind: Kind,
    pub retiring: bool,
    /// Submitted source payload; buffer ranges already include copy alignment.
    pub submitted_source_bytes: u64,
    /// API upload representation, including texture row padding or scatter indices.
    /// Not driver bus traffic, completion timing, or allocated staging capacity.
    pub submitted_transfer_bytes: u64,
    pub last_upload: Option<super::upload_totals::AllocationUploadFrame>,
}

struct Identity {
    record: Record,
    metadata: Metadata,
    owner: Arc<State>,
    uploads: Mutex<super::upload_totals::AllocationUploads>,
}

struct State {
    id: u64,
    allocations: Mutex<Vec<Weak<Identity>>>,
    uploads: Mutex<super::upload_totals::Accounting>,
    released: Mutex<[ReleasedAllocations; 5]>,
}

/// Retain observation across renderer close without retaining GPU resources.
#[derive(Clone)]
pub struct Observer(Owner);

impl Observer {
    /// Final GPU-owner releases, grouped by first retirement cause. Fixed-size
    /// cumulative counters survive resource teardown without retaining resources.
    pub fn released_allocations(&self) -> [(RetirementReason, ReleasedAllocations); 5] {
        let totals = *self.0.0.released.lock().unwrap_or_else(|e| e.into_inner());
        state::released_snapshot(totals)
    }

    pub fn submitted_upload_totals(&self) -> super::upload_totals::UploadTotals {
        self.0
            .0
            .uploads
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .total
    }

    pub fn allocations(&self) -> Vec<Record> {
        self.0.records()
    }
}

#[derive(Clone)]
pub(crate) struct Owner(Arc<State>);

impl Owner {
    pub(crate) fn begin_upload_frame(&self) {
        self.0
            .uploads
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .begin(self.id());
    }

    pub(crate) fn finish_upload_frame(&self, rendered: Option<bool>) {
        self.0
            .uploads
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .finish(rendered);
    }

    pub(crate) fn last_upload_frame(&self) -> Option<super::upload_totals::UploadFrame> {
        self.0
            .uploads
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .latest()
    }

    pub(crate) fn record_upload(
        &self,
        totals: super::upload_totals::UploadTotals,
    ) -> Option<(u64, u64)> {
        self.0
            .uploads
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .record(totals)
    }

    pub fn id(&self) -> u64 {
        self.0.id
    }

    pub fn observer(&self) -> Observer {
        Observer(self.clone())
    }

    pub fn new() -> Self {
        Self(Arc::new(State {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            allocations: Mutex::new(Vec::new()),
            uploads: Mutex::new(Default::default()),
            released: Mutex::new(Default::default()),
        }))
    }

    pub(crate) fn track_reserved<T>(
        &self,
        resource: T,
        generation: u64,
        kind: Kind,
        reservation: super::budget::GpuReservation,
    ) -> Tracked<T> {
        let identity = Arc::new(Identity {
            record: Record {
                id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
                owner: self.0.id,
                renderer_id: super::allocation_host::current(),
                generation,
                bytes: reservation.bytes(),
                requested_bytes: reservation.bytes(),
                prepared_references: 0,
                submission_references: 0,
                retirement_reason: None,
                kind,
                retiring: false,
                submitted_source_bytes: 0,
                submitted_transfer_bytes: 0,
                last_upload: None,
            },
            metadata: Metadata::new(reservation.bytes()),
            owner: self.0.clone(),
            uploads: Mutex::new(Default::default()),
        });
        let mut entries = self.0.allocations.lock().unwrap_or_else(|e| e.into_inner());
        entries.retain(|entry| entry.strong_count() != 0);
        entries.push(Arc::downgrade(&identity));
        let mut process = PROCESS_ALLOCATIONS
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        process.retain(|entry| entry.strong_count() != 0);
        process.push(Arc::downgrade(&identity));
        Tracked(Arc::new(Allocation {
            resource,
            _reservation: reservation,
            _shared_permit: None,
            _released: FinalRelease(identity.clone()),
            identity,
        }))
    }

    /// API-owned bytes, including submitted allocations whose CPU owner retired.
    /// Driver residency, allocation metadata and staging are separate boundaries.
    pub fn records(&self) -> Vec<Record> {
        records(&self.0.allocations)
    }
}

fn records(source: &Mutex<Vec<Weak<Identity>>>) -> Vec<Record> {
    let mut entries = source.lock().unwrap_or_else(|e| e.into_inner());
    let mut records = Vec::new();
    entries.retain(|entry| {
        if let Some(identity) = entry.upgrade()
            && !identity.metadata.released.load(Ordering::Acquire)
        {
            let uploads = identity.uploads.lock().unwrap_or_else(|e| e.into_inner());
            let reason = identity.metadata.reason();
            records.push(Record {
                retiring: reason.is_some(),
                retirement_reason: reason,
                requested_bytes: identity.metadata.payload.load(Ordering::Acquire),
                prepared_references: identity.metadata.prepared.load(Ordering::Acquire),
                submission_references: identity.metadata.submitted.load(Ordering::Acquire),
                submitted_source_bytes: uploads.source_bytes,
                submitted_transfer_bytes: uploads.transfer_bytes,
                last_upload: uploads.latest,
                ..identity.record
            });
            true
        } else {
            false
        }
    });
    if entries.capacity() > entries.len().saturating_mul(4) {
        *entries = std::mem::take(&mut *entries).into_boxed_slice().into_vec();
    }
    records
}

impl crate::Renderer {
    /// Final tracked API allocation releases across all migrated owners. Fixed-size
    /// process totals; no driver residency or per-allocation event history.
    pub fn gpu_process_released_allocations() -> [(RetirementReason, ReleasedAllocations); 5] {
        state::released_snapshot(*PROCESS_RELEASED.lock().unwrap_or_else(|e| e.into_inner()))
    }

    /// This host's screen vertices, glyph instances and uniform buffer capacities.
    /// Includes submission retirement; texture and staging subcaps remain separate.
    pub fn screen_gpu_reserved_bytes(&self) -> u64 {
        self.screen_budget.used()
    }

    /// All live and submission-retiring text texture/instance API allocations.
    /// Weak process observation does not prolong GPU resource lifetime.
    pub fn text_gpu_process_allocations() -> Vec<Record> {
        Self::gpu_process_allocations()
            .into_iter()
            .filter(|r| matches!(r.kind, Kind::Texture | Kind::Instances))
            .collect()
    }

    /// Migrated renderer resources, explicit staging and opt-in query/readback storage.
    /// Query bytes describe requested timestamp capacity, not backend-private storage.
    /// Driver residency and allocator metadata remain separate.
    pub fn gpu_process_allocations() -> Vec<Record> {
        records(&PROCESS_ALLOCATIONS)
    }

    /// Reserved migrated GPU capacity, including pending creation and retirement.
    /// Includes logical query capacity and exact resolve/readback buffer capacities;
    /// backend-private storage and driver residency remain separate.
    pub fn gpu_process_reserved_bytes() -> u64 {
        super::budget::gpu_process().used()
    }

    /// Shared GPU staging, pending glyph pixels and retained private layout scratch.
    /// Other CPU pending payload, tracking metadata and scratch remain separate.
    pub fn upload_staging_reserved_bytes(&self) -> u64 {
        self.atlas.staging_budget.used()
    }

    pub fn upload_staging_process_reserved_bytes() -> u64 {
        super::budget::staging_process().used()
    }

    /// Terminal image texture and quad capacities, including submitted retirement.
    /// Shared glyph/text resources and upload staging have separate subcaps.
    pub fn terminal_graphics_gpu_reserved_bytes() -> u64 {
        super::budget::terminal_process().used()
    }

    /// This renderer's atlas capacity, including pages held by submitted work.
    pub fn text_atlas_reserved_bytes(&self) -> u64 {
        self.atlas.reserved_texture_bytes()
    }

    /// Texture bytes reserved or allocated, including retiring submissions.
    /// This is API capacity, not physical driver residency or upload staging.
    pub fn text_atlas_process_bytes() -> u64 {
        super::budget::process().used()
    }
}

// Resource drops before identity: a record cannot disappear while this owner
// still holds the API object. Submission references own this SAME allocation.
struct Allocation<T> {
    resource: T,
    _shared_permit: Option<Arc<super::budget::Permit>>,
    _reservation: super::budget::GpuReservation,
    identity: Arc<Identity>,
    _released: FinalRelease,
}

struct FinalRelease(Arc<Identity>);
impl Drop for FinalRelease {
    fn drop(&mut self) {
        self.0.metadata.released.store(true, Ordering::Release);
        let reason = self
            .0
            .metadata
            .reason()
            .unwrap_or(RetirementReason::OwnerDropped);
        let mut totals = self
            .0
            .owner
            .released
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let total = &mut totals[reason as usize - 1];
        total.allocations += 1;
        total.capacity_bytes += self.0.record.bytes;
        drop(totals);
        let mut process = PROCESS_RELEASED.lock().unwrap_or_else(|e| e.into_inner());
        let total = &mut process[reason as usize - 1];
        total.allocations += 1;
        total.capacity_bytes += self.0.record.bytes;
    }
}

pub(crate) struct Tracked<T>(Arc<Allocation<T>>);

/// Borrowed during planning; retaining a receipt retains accounting, not GPU data.
#[derive(Clone, Copy)]
pub(crate) struct UploadTarget<'a>(&'a Arc<Identity>);
pub(crate) struct SubmittedUpload {
    identity: Arc<Identity>,
    source: u64,
    transfer: u64,
}
impl UploadTarget<'_> {
    pub fn receipt(self, source: u64, transfer: u64) -> SubmittedUpload {
        SubmittedUpload {
            identity: self.0.clone(),
            source,
            transfer,
        }
    }
}
impl SubmittedUpload {
    pub fn commit(self, attempt: Option<(u64, u64)>) {
        self.identity
            .uploads
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .record(attempt, self.source, self.transfer);
    }
}

impl<T> Tracked<T> {
    pub(crate) fn upload_target(&self) -> UploadTarget<'_> {
        UploadTarget(&self.0.identity)
    }

    /// A queue completion callback now owns this handle instead of the producer.
    pub(crate) fn mark_retiring(&self) {
        if self.0.identity.metadata.retire(RetirementReason::Submitted) {
            self.0
                .identity
                .metadata
                .submitted
                .fetch_add(1, Ordering::AcqRel);
        }
    }

    pub(crate) fn retire(&self, reason: RetirementReason) {
        self.0.identity.metadata.retire(reason);
    }

    pub(crate) fn set_requested_bytes(&self, bytes: u64) {
        assert!(
            bytes <= self.0.identity.record.bytes,
            "payload exceeds GPU capacity"
        );
        self.0
            .identity
            .metadata
            .payload
            .store(bytes, Ordering::Release);
    }

    pub(crate) fn with_shared_permit(mut self, permit: Arc<super::budget::Permit>) -> Self {
        Arc::get_mut(&mut self.0)
            .expect("attach shared permit before publishing allocation")
            ._shared_permit = Some(permit);
        self
    }

    #[cfg(test)]
    pub fn id(&self) -> u64 {
        self.0.identity.record.id
    }
}

trait ResourceIdentity: Send + Sync {
    fn identity(&self) -> &Identity;
}
impl<T: Send + Sync> ResourceIdentity for Allocation<T> {
    fn identity(&self) -> &Identity {
        &self.identity
    }
}
impl<T: Send + Sync + 'static> Tracked<T> {
    fn reference(&self, kind: ReferenceKind) -> SubmissionRef {
        self.0
            .identity
            .metadata
            .counter(kind)
            .fetch_add(1, Ordering::AcqRel);
        SubmissionRef {
            allocation_id: self.0.identity.record.id,
            kind,
            resource: self.0.clone(),
        }
    }
    pub fn submission_ref(&self) -> SubmissionRef {
        self.reference(ReferenceKind::Submission)
    }
    pub fn prepared_ref(&self) -> SubmissionRef {
        self.reference(ReferenceKind::Prepared)
    }
}

impl<T> Deref for Tracked<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0.resource
    }
}

impl<T> Drop for Tracked<T> {
    fn drop(&mut self) {
        if self.0.identity.metadata.reason() == Some(RetirementReason::Submitted) {
            self.0
                .identity
                .metadata
                .submitted
                .fetch_sub(1, Ordering::AcqRel);
        }
        self.retire(RetirementReason::OwnerDropped);
    }
}

pub(crate) struct SubmissionRef {
    pub(crate) allocation_id: u64,
    kind: ReferenceKind,
    resource: Arc<dyn ResourceIdentity>,
}

impl Drop for SubmissionRef {
    fn drop(&mut self) {
        self.resource
            .identity()
            .metadata
            .counter(self.kind)
            .fetch_sub(1, Ordering::AcqRel);
    }
}

/// Call immediately AFTER the submission that consumes these resources/writes.
/// Before that point the caller must retain the refs, including failed-prepare
/// allocations with queued writes. A reset is not a submission or completion.
pub(crate) fn hold_until_done(queue: &wgpu::Queue, resources: Vec<SubmissionRef>) {
    queue.on_submitted_work_done(move || drop(resources));
}

#[cfg(test)]
#[path = "lifetime_tests.rs"]
mod tests;
