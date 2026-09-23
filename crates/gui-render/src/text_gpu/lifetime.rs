//! Allocation identities and explicit submission holds for the shared text owner.
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Texture,
    Instances,
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
}

/// Retain observation across renderer close without retaining GPU resources.
#[derive(Clone)]
pub struct Observer(Owner);

impl Observer {
    pub fn allocations(&self) -> Vec<Record> {
        self.0.records()
    }
}

#[derive(Clone)]
pub(crate) struct Owner(Arc<State>);

impl Owner {
    pub fn observer(&self) -> Observer {
        Observer(self.clone())
    }

    pub fn new() -> Self {
        Self(Arc::new(State {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            allocations: Mutex::new(Vec::new()),
        }))
    }

    pub(super) fn track<T>(
        &self,
        resource: T,
        bytes: u64,
        generation: u64,
        kind: Kind,
    ) -> Tracked<T> {
        let identity = Arc::new(Identity {
            record: Record {
                id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
                owner: self.0.id,
                generation,
                bytes,
                kind,
                retiring: false,
            },
            active: AtomicBool::new(true),
        });
        let mut entries = self.0.allocations.lock().unwrap_or_else(|e| e.into_inner());
        entries.retain(|entry| entry.strong_count() != 0);
        entries.push(Arc::downgrade(&identity));
        Tracked(Arc::new(Allocation { resource, identity }))
    }

    /// API-owned bytes, including submitted allocations whose CPU owner retired.
    /// Driver residency, allocation metadata and staging are separate boundaries.
    pub fn records(&self) -> Vec<Record> {
        let mut entries = self.0.allocations.lock().unwrap_or_else(|e| e.into_inner());
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
}

// Resource drops before identity: a record cannot disappear while this owner
// still holds the API object. Submission references own this SAME allocation.
struct Allocation<T> {
    resource: T,
    identity: Arc<Identity>,
}

pub(super) struct Tracked<T>(Arc<Allocation<T>>);

impl<T> Tracked<T> {
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
