//! Bounded allocation-history delivery. Records hold no GPU or owner handles.
//! Registration follows API creation: the tracked-capacity peak is not an exact
//! API creation-time peak. Pre-creation reservation peaks are separate bounds.
use super::{Identity, NEXT_ID, PROCESS_ALLOCATIONS, Record};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::Instant;

static ENABLED: AtomicBool = AtomicBool::new(false);
static OBSERVER: Mutex<Option<Buffer>> = Mutex::new(None);
static NEXT_OBSERVATION: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transition {
    Registered,
    Consumers,
    Requested,
    PreparedReference,
    SubmissionReference,
    ReferenceReleased,
    Retired,
    SubmittedUpload,
    Released,
}

#[derive(Clone, Copy, Debug)]
pub struct Event {
    pub sequence: u64,
    pub elapsed_ns: u128,
    pub transition: Transition,
    pub allocation: Record,
}

pub struct Batch {
    pub observation_id: u64,
    /// One means observation started before any tracked owner was constructed.
    pub first_identity_id: u64,
    pub events: Vec<Event>,
    pub total_events: u64,
    pub dropped_events: u64,
    pub active_allocations: u64,
    pub tracked_capacity_bytes: u64,
    pub tracked_capacity_peak_bytes: u64,
    pub invalid_accounting: bool,
    pub capacity: usize,
    pub buffer_capacity_bytes: usize,
}

struct Buffer {
    id: u64,
    first_identity_id: u64,
    started: Instant,
    events: Vec<Event>,
    capacity: usize,
    total: u64,
    dropped: u64,
    active: u64,
    bytes: u64,
    peak: u64,
    invalid: bool,
}

impl Buffer {
    fn push(&mut self, transition: Transition, allocation: Record) {
        let next = match transition {
            Transition::Registered => self
                .active
                .checked_add(1)
                .zip(self.bytes.checked_add(allocation.bytes)),
            Transition::Released => self
                .active
                .checked_sub(1)
                .zip(self.bytes.checked_sub(allocation.bytes)),
            _ => Some((self.active, self.bytes)),
        };
        if let Some((active, bytes)) = next {
            self.active = active;
            self.bytes = bytes;
            self.peak = self.peak.max(bytes);
        } else {
            self.invalid = true;
        }
        self.total += 1;
        if self.events.len() == self.capacity {
            self.dropped += 1;
            return;
        }
        self.events.push(Event {
            sequence: self.total,
            elapsed_ns: self.started.elapsed().as_nanos(),
            transition,
            allocation,
        });
    }

    fn batch(&self, events: Vec<Event>) -> Batch {
        let buffer_capacity_bytes = events.capacity() * std::mem::size_of::<Event>();
        Batch {
            observation_id: self.id,
            first_identity_id: self.first_identity_id,
            events,
            total_events: self.total,
            dropped_events: self.dropped,
            active_allocations: self.active,
            tracked_capacity_bytes: self.bytes,
            tracked_capacity_peak_bytes: self.peak,
            invalid_accounting: self.invalid,
            capacity: self.capacity,
            buffer_capacity_bytes,
        }
    }
}

// Serialize each observed metadata mutation with its resulting record. Taking a
// snapshot after unlocking the mutation would invent an order under concurrency.
pub(super) fn change<T>(
    identity: &Identity,
    transition: Transition,
    work: impl FnOnce() -> T,
) -> T {
    if !ENABLED.load(Ordering::Acquire) {
        return work();
    }
    let mut observer = OBSERVER.lock().unwrap_or_else(|e| e.into_inner());
    let result = work();
    if let Some(buffer) = observer.as_mut() {
        buffer.push(transition, identity.snapshot());
    }
    result
}

fn storage(capacity: usize) -> anyhow::Result<Vec<Event>> {
    crate::cpu_alloc::observation_storage(|| {
        let mut events = Vec::new();
        events.try_reserve_exact(capacity)?;
        Ok(events)
    })
}

/// Call at a quiescent application boundary. Existing allocations reject startup;
/// first_identity_id also exposes earlier, already released owners. A caller must
/// not start/stop while another thread is constructing an unregistered API object.
pub fn start(capacity: usize) -> anyhow::Result<u64> {
    anyhow::ensure!(
        (1..=65_536).contains(&capacity),
        "invalid GPU trace capacity"
    );
    let events = storage(capacity)?;
    let entries = PROCESS_ALLOCATIONS
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    anyhow::ensure!(
        !entries.iter().any(|entry| entry
            .upgrade()
            .is_some_and(|identity| !identity.metadata.released.load(Ordering::Acquire))),
        "GPU allocations already live"
    );
    let mut observer = OBSERVER.lock().unwrap_or_else(|e| e.into_inner());
    anyhow::ensure!(observer.is_none(), "GPU observation already active");
    let id = NEXT_OBSERVATION
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .map_err(|_| anyhow::anyhow!("GPU observation identity exhausted"))?;
    *observer = Some(Buffer {
        id,
        first_identity_id: NEXT_ID.load(Ordering::Acquire),
        started: Instant::now(),
        events,
        capacity,
        total: 0,
        dropped: 0,
        active: 0,
        bytes: 0,
        peak: 0,
        invalid: false,
    });
    ENABLED.store(true, Ordering::Release);
    Ok(id)
}

pub fn has_pending(id: u64) -> anyhow::Result<bool> {
    let observer = OBSERVER.lock().unwrap_or_else(|e| e.into_inner());
    let buffer = observer
        .as_ref()
        .filter(|b| b.id == id)
        .ok_or_else(|| anyhow::anyhow!("GPU observation identity mismatch"))?;
    Ok(!buffer.events.is_empty())
}

/// Replacement storage is allocated outside the event lock. At most two buffers
/// belong to one serial drainer; allocator/JSON/I/O storage must be reported too.
pub fn drain(id: u64) -> anyhow::Result<Batch> {
    let capacity = OBSERVER
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .filter(|b| b.id == id)
        .map(|b| b.capacity)
        .ok_or_else(|| anyhow::anyhow!("GPU observation identity mismatch"))?;
    let replacement = storage(capacity)?;
    let mut observer = OBSERVER.lock().unwrap_or_else(|e| e.into_inner());
    let buffer = observer
        .as_mut()
        .filter(|b| b.id == id)
        .ok_or_else(|| anyhow::anyhow!("GPU observation changed during drain"))?;
    let events = std::mem::replace(&mut buffer.events, replacement);
    Ok(buffer.batch(events))
}

/// Stop only after all tracked resources and completion holds have been released.
pub fn stop(id: u64) -> anyhow::Result<Batch> {
    let mut observer = OBSERVER.lock().unwrap_or_else(|e| e.into_inner());
    let buffer = observer
        .as_ref()
        .filter(|b| b.id == id)
        .ok_or_else(|| anyhow::anyhow!("GPU observation identity mismatch"))?;
    anyhow::ensure!(buffer.active == 0, "GPU observation has live allocations");
    let mut buffer = observer.take().expect("checked observer");
    ENABLED.store(false, Ordering::Release);
    let events = std::mem::take(&mut buffer.events);
    Ok(buffer.batch(events))
}

/// Lifetime peaks of admitted capacity, including pending API creation and shared
/// batch reservations. These are conservative capacity bounds, not API-live or
/// physical driver-residency measurements. Private-call peaks remain independent.
#[derive(Clone, Copy, Debug)]
pub struct ReservationPeaks {
    pub gpu_process: u64,
    pub atlas_process: u64,
    pub staging_process: u64,
    pub terminal_process: u64,
}
pub fn reservation_peaks() -> ReservationPeaks {
    let budget = &super::super::budget::gpu_process();
    ReservationPeaks {
        gpu_process: budget.peak(),
        atlas_process: super::super::budget::process().peak(),
        staging_process: super::super::budget::staging_process().peak(),
        terminal_process: super::super::budget::terminal_process().peak(),
    }
}

#[cfg(test)]
#[path = "lifetime_observation_tests.rs"]
mod tests;
