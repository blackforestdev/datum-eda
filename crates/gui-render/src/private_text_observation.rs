//! Bounded opt-in delivery of private-call reports. This is measurement storage,
//! not another admission owner; overflow permanently invalidates the observation.
use super::{Entry, Report, transaction};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
static NEXT_OBSERVATION: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Begin,
    Finished,
    Abandoned,
}

#[derive(Clone, Copy, Debug)]
pub struct Event {
    pub sequence: u64,
    pub elapsed_ns: u128,
    pub phase: Phase,
    pub report: Report,
    pub renderer_id: Option<u64>,
    pub owner_label: &'static str,
    pub host_limit_bytes: u64,
    pub process_limit_bytes: u64,
    pub excluded_bytes: u64,
    pub credited_bytes: u64,
    pub requested_retained_bytes: Option<u64>,
}
impl Event {
    pub(super) fn new(call: &Entry, phase: Phase, requested_retained_bytes: Option<u64>) -> Self {
        Self {
            sequence: 0,
            elapsed_ns: 0,
            phase,
            report: call.report,
            renderer_id: call.renderer_id,
            owner_label: call.owner_label,
            host_limit_bytes: call.host.limit(),
            process_limit_bytes: call.process.limit(),
            excluded_bytes: call.excluded,
            credited_bytes: call.credit,
            requested_retained_bytes,
        }
    }
}

pub struct Batch {
    pub observation_id: u64,
    pub first_call_id: u64,
    pub events: Vec<Event>,
    /// Cumulative, sticky loss count; a later successful drain cannot erase it.
    pub dropped_events: u64,
    pub total_events: u64,
    pub active_calls: usize,
    pub capacity: usize,
    /// API vector storage per buffer, excluding allocator metadata. A drain may
    /// temporarily own two buffers; JSON/output buffers and RSS are separate.
    pub buffer_capacity_bytes: usize,
}

pub(super) struct Buffer {
    id: u64,
    first_call_id: u64,
    started: Instant,
    events: Vec<Event>,
    limit: usize,
    total: u64,
    dropped: u64,
}
impl Buffer {
    pub(super) fn push(&mut self, mut event: Event) {
        self.total += 1;
        event.sequence = self.total;
        event.elapsed_ns = self.started.elapsed().as_nanos();
        if self.events.len() == self.limit {
            self.dropped += 1;
        } else {
            self.events.push(event);
        }
    }
    fn batch(&self, events: Vec<Event>, active_calls: usize) -> Batch {
        let buffer_capacity_bytes = events.capacity() * std::mem::size_of::<Event>();
        Batch {
            observation_id: self.id,
            first_call_id: self.first_call_id,
            events,
            dropped_events: self.dropped,
            total_events: self.total,
            active_calls,
            capacity: self.limit,
            buffer_capacity_bytes,
        }
    }
}

fn storage(capacity: usize) -> anyhow::Result<Vec<Event>> {
    super::with_current(std::ptr::null(), || {
        let mut events = Vec::new();
        events.try_reserve_exact(capacity)?;
        Ok(events)
    })
}

/// Start at a quiescent call boundary. The first call ID exposes earlier calls;
/// consumers requiring startup coverage must reject any value other than one.
pub fn start(capacity: usize) -> anyhow::Result<u64> {
    anyhow::ensure!(
        (1..=65_536).contains(&capacity),
        "invalid private-call trace capacity"
    );
    let events = storage(capacity)?;
    let id = NEXT_OBSERVATION
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
        .map_err(|_| anyhow::anyhow!("private-call observation identity exhausted"))?;
    let mut buffer = Some(Buffer {
        id,
        first_call_id: 0,
        started: Instant::now(),
        events,
        limit: capacity,
        total: 0,
        dropped: 0,
    });
    let installed = transaction(|ledger| {
        if ledger.observer.is_some() || ledger.calls.len() != 0 {
            return false;
        }
        buffer.as_mut().expect("uninstalled observer").first_call_id = ledger.next;
        ledger.observer = buffer.take();
        true
    });
    anyhow::ensure!(
        installed,
        "private-call observation already active or calls in progress"
    );
    Ok(id)
}

/// Test delivery readiness without allocating or changing event sequences.
pub fn has_pending(id: u64) -> anyhow::Result<bool> {
    transaction(|ledger| {
        let buffer = ledger
            .observer
            .as_ref()
            .filter(|b| b.id == id)
            .ok_or_else(|| anyhow::anyhow!("private-call observation identity mismatch"))?;
        Ok(!buffer.events.is_empty())
    })
}

/// Snapshot event delivery without resetting totals. Replacement allocation is
/// outside the allocator/admission lock; active calls may finish in a later batch.
pub fn drain(id: u64) -> anyhow::Result<Batch> {
    let capacity = transaction(|ledger| {
        ledger
            .observer
            .as_ref()
            .filter(|b| b.id == id)
            .map(|b| b.limit)
    })
    .ok_or_else(|| anyhow::anyhow!("private-call observation not active"))?;
    let replacement = storage(capacity)?;
    transaction(|ledger| {
        let buffer = ledger
            .observer
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("private-call observation stopped"))?;
        anyhow::ensure!(
            buffer.limit == capacity && buffer.id == id,
            "private-call observer changed during drain"
        );
        let events = std::mem::replace(&mut buffer.events, replacement);
        Ok(buffer.batch(events, ledger.calls.len()))
    })
}

pub fn stop(id: u64) -> anyhow::Result<Batch> {
    let buffer = transaction(|ledger| {
        anyhow::ensure!(
            ledger.observer.as_ref().is_some_and(|b| b.id == id),
            "private-call observation identity mismatch"
        );
        anyhow::ensure!(
            ledger.calls.len() == 0,
            "private-call observation has active calls"
        );
        ledger
            .observer
            .take()
            .ok_or_else(|| anyhow::anyhow!("private-call observation not active"))
    })?;
    let mut buffer = buffer;
    let events = std::mem::take(&mut buffer.events);
    Ok(buffer.batch(events, 0))
}

#[cfg(test)]
#[path = "private_text_observation_tests.rs"]
mod tests;
