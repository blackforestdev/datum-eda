//! Allocation identities and explicit submission holds for the shared GPU owners.
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};

static PROCESS_ALLOCATIONS: Mutex<Vec<Weak<Identity>>> = Mutex::new(Vec::new());

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
    pub generation: u64,
    pub bytes: u64,
    pub kind: Kind,
    pub retiring: bool,
}

struct Identity {
    record: Record,
    active: AtomicBool,
}

struct State {
    id: u64,
    allocations: Mutex<Vec<Weak<Identity>>>,
    uploads: Mutex<super::upload_totals::UploadTotals>,
}

/// Retain observation across renderer close without retaining GPU resources.
#[derive(Clone)]
pub struct Observer(Owner);

impl Observer {
    pub fn submitted_upload_totals(&self) -> super::upload_totals::UploadTotals {
        *self.0.0.uploads.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn allocations(&self) -> Vec<Record> {
        self.0.records()
    }
}

#[derive(Clone)]
pub(crate) struct Owner(Arc<State>);

impl Owner {
    pub(crate) fn record_upload(&self, totals: super::upload_totals::UploadTotals) {
        self.0
            .uploads
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .add(totals);
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
        }))
    }

    #[cfg(test)]
    pub(crate) fn track<T>(
        &self,
        resource: T,
        bytes: u64,
        generation: u64,
        kind: Kind,
    ) -> Tracked<T> {
        self.track_reserved(
            resource,
            generation,
            kind,
            super::budget::GpuReservation::new(bytes, Vec::new()).expect("test GPU reservation"),
        )
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
                generation,
                bytes: reservation.bytes(),
                kind,
                retiring: false,
            },
            active: AtomicBool::new(true),
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
        if let Some(identity) = entry.upgrade() {
            records.push(Record {
                retiring: !identity.active.load(Ordering::Acquire),
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
}

pub(crate) struct Tracked<T>(Arc<Allocation<T>>);

impl<T> Tracked<T> {
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

impl<T: Send + Sync + 'static> Tracked<T> {
    pub fn submission_ref(&self) -> SubmissionRef {
        SubmissionRef {
            _resource: self.0.clone(),
        }
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
        self.0.identity.active.store(false, Ordering::Release);
    }
}

pub(crate) struct SubmissionRef {
    _resource: Arc<dyn Send + Sync>,
}

/// Call immediately AFTER the submission that consumes these resources/writes.
/// Before that point the caller must retain the refs, including failed-prepare
/// allocations with queued writes. A reset is not a submission or completion.
pub(crate) fn hold_until_done(queue: &wgpu::Queue, resources: Vec<SubmissionRef>) {
    queue.on_submitted_work_done(move || drop(resources));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texture_reservation_survives_owner_and_every_submission_hold() {
        let budget = super::super::budget::Budget::new(64);
        let owner = Owner::new();
        let texture = owner.track_reserved(
            vec![0_u8; 64],
            1,
            Kind::Texture,
            super::super::budget::GpuReservation::new(64, vec![budget.reserve(64).unwrap()])
                .unwrap(),
        );
        let id = texture.id();
        let first = texture.submission_ref();
        let second = texture.submission_ref();
        drop(texture);
        drop(owner);
        assert_eq!(budget.used(), 64);
        assert!(budget.reserve(1).is_err());
        let records = crate::Renderer::text_gpu_process_allocations();
        assert!(
            records
                .iter()
                .any(|record| record.id == id && record.retiring)
        );
        drop(first);
        assert_eq!(budget.used(), 64);
        drop(second);
        assert_eq!(budget.used(), 0);
        assert!(
            !crate::Renderer::text_gpu_process_allocations()
                .iter()
                .any(|record| record.id == id)
        );
    }

    #[test]
    fn split_upload_reservation_survives_mapped_owner_and_packet_submissions() {
        use super::super::budget::{Budget, GpuReservation};
        let budget = Budget::new(96);
        let mut reservation = GpuReservation::new(96, vec![budget.reserve(96).unwrap()]).unwrap();
        assert!(reservation.split(97).is_err());
        assert_eq!(reservation.bytes(), 96);
        let owner = Owner::new();
        let mapped = owner.track_reserved((), 1, Kind::Staging, reservation.split(64).unwrap());
        let packet = owner.track_reserved((), 1, Kind::Staging, reservation.split(32).unwrap());
        assert_eq!(reservation.bytes(), 0);
        assert_eq!(owner.records().iter().map(|r| r.bytes).sum::<u64>(), 96);
        let observer = owner.observer();
        let first = packet.submission_ref();
        let second = packet.submission_ref();
        drop(reservation);
        drop(mapped);
        drop(packet);
        drop(owner);
        assert_eq!(budget.used(), 96);
        assert!(budget.reserve(1).is_err());
        assert_eq!(observer.allocations().len(), 1);
        assert_eq!(observer.allocations()[0].bytes, 32);
        assert!(observer.allocations()[0].retiring);
        drop(first);
        assert_eq!(budget.used(), 96);
        drop(second);
        assert_eq!(budget.used(), 0);
        assert!(observer.allocations().is_empty());
        assert!(budget.reserve(96).is_ok());
    }

    #[test]
    fn submitted_allocation_stays_charged_after_cpu_replacement() {
        let owner = Owner::new();
        let observer = owner.observer();
        let first = owner.track(vec![0_u8; 64], 64, 1, Kind::Instances);
        let first_id = first.id();
        let submitted = first.submission_ref();
        let submitted_again = first.submission_ref();
        let replacement = owner.track(vec![1_u8; 64], 64, 2, Kind::Instances);
        assert_ne!(first_id, replacement.id());
        drop(first);
        let records = owner.records();
        assert_eq!(records.iter().map(|r| r.bytes).sum::<u64>(), 128);
        assert!(records.iter().find(|r| r.id == first_id).unwrap().retiring);
        drop(submitted);
        assert_eq!(
            owner.records().len(),
            2,
            "later submission still pins old allocation"
        );
        drop(submitted_again);
        assert_eq!(owner.records().len(), 1);
        drop(owner);
        assert_eq!(observer.allocations().len(), 1);
        drop(replacement);
        assert!(observer.allocations().is_empty());
    }
}
