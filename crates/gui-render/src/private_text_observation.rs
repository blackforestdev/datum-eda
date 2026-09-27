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
    Allocation,
}

/// A requested block lifetime transition, including Datum header/alignment bytes.
/// Addresses can be reused; the ordered allocation event identifies each lifetime.
#[derive(Clone, Copy, Debug)]
pub struct Allocation {
    pub address: usize,
    pub bytes: u64,
    pub allocated: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Event {
    pub sequence: u64,
    pub allocation: Option<Allocation>,
    pub scope_live_bytes: u64,
    pub elapsed_ns: u128,
    pub phase: Phase,
    /// For Phase::Allocation only call_id, owner_id and allocator_installed are
    /// authoritative here. Accounting fields can precede the transition; use
    /// allocation + scope_live_bytes for it, and Finished for call peak/refusal.
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
            allocation: None,
            scope_live_bytes: call.live,
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

/// Compact transitions carry only fields authoritative at allocation time.
#[derive(Clone, Copy, Debug)]
pub struct AllocationRecord {
    pub sequence: u64,
    pub elapsed_ns: u128,
    pub call_id: u64,
    pub owner_id: u64,
    pub allocator_installed: bool,
    pub scope_live_bytes: u64,
    pub allocation: Allocation,
}

pub enum Record {
    Call(Box<Event>),
    Allocation(AllocationRecord),
}
impl Record {
    pub fn sequence(&self) -> u64 {
        match self {
            Self::Call(event) => event.sequence,
            Self::Allocation(event) => event.sequence,
        }
    }
}

// Preserve the previous maximum full-event vector payload allowance. Count
// vector tracking overhead and boxed call records inside this ceiling too.
pub const MAX_STORAGE_BYTES: usize = 65_536 * std::mem::size_of::<Event>();
fn call_bytes() -> usize {
    crate::cpu_alloc::heap::allocation_bytes(std::alloc::Layout::new::<Event>())
}
fn vector_bytes(capacity: usize) -> usize {
    crate::cpu_alloc::heap::capacity_bytes::<Record>(capacity)
}

pub struct Batch {
    pub observation_id: u64,
    pub first_call_id: u64,
    pub events: Vec<Record>,
    /// Cumulative, sticky loss count; a later successful drain cannot erase it.
    pub dropped_events: u64,
    pub total_events: u64,
    pub active_calls: usize,
    pub capacity: usize,
    /// Requested vector and boxed-call storage including Datum tracking bytes.
    /// A drain may own two bounded buffers; JSON/output/RSS remain separate.
    pub buffer_capacity_bytes: usize,
}

pub(super) struct Buffer {
    id: u64,
    first_call_id: u64,
    started: Instant,
    events: Vec<Record>,
    limit: usize,
    boxed_bytes: usize,
    total: u64,
    dropped: u64,
    pub(super) allocations: bool,
}
impl Buffer {
    pub(super) fn push(&mut self, mut event: Event) {
        self.total += 1;
        event.sequence = self.total;
        event.elapsed_ns = self.started.elapsed().as_nanos();
        let extra = if event.allocation.is_some() {
            0
        } else {
            call_bytes()
        };
        if self.events.len() == self.limit
            || vector_bytes(self.events.capacity()) + self.boxed_bytes + extra > MAX_STORAGE_BYTES
        {
            self.dropped += 1;
            return;
        }
        let record = if let Some(allocation) = event.allocation {
            Record::Allocation(AllocationRecord {
                sequence: event.sequence,
                elapsed_ns: event.elapsed_ns,
                call_id: event.report.call_id,
                owner_id: event.report.owner_id,
                allocator_installed: event.report.allocator_installed,
                scope_live_bytes: event.scope_live_bytes,
                allocation,
            })
        } else {
            // transaction() clears allocation attribution: this measurement box
            // cannot recurse into the scoped allocator or the call ledger.
            self.boxed_bytes += extra;
            Record::Call(Box::new(event))
        };
        self.events.push(record);
    }
    fn batch(&self, events: Vec<Record>, active_calls: usize) -> Batch {
        let buffer_capacity_bytes = vector_bytes(events.capacity())
            + events
                .iter()
                .filter(|r| matches!(r, Record::Call(_)))
                .count()
                * call_bytes();
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

fn storage(capacity: usize) -> anyhow::Result<Vec<Record>> {
    super::with_current(std::ptr::null(), || {
        let mut events = Vec::new();
        events.try_reserve_exact(capacity)?;
        anyhow::ensure!(
            vector_bytes(events.capacity()) <= MAX_STORAGE_BYTES,
            "private trace storage ceiling exceeded"
        );
        Ok(events)
    })
}

/// Start at a quiescent call boundary. The first call ID exposes earlier calls;
/// consumers requiring startup coverage must reject any value other than one.
pub fn start(capacity: usize) -> anyhow::Result<u64> {
    start_mode(capacity, false)
}

/// Include compact block transitions within monitored calls. Event count and
/// storage-byte exhaustion both fail delivery; this requires an allocation-aware
/// inference from final totals. Allocator slack/native mappings remain separate.
pub fn start_with_allocations(capacity: usize) -> anyhow::Result<u64> {
    start_mode(capacity, true)
}

fn start_mode(capacity: usize, allocations: bool) -> anyhow::Result<u64> {
    anyhow::ensure!(
        (1..=if allocations { 131_072 } else { 65_536 }).contains(&capacity),
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
        boxed_bytes: 0,
        total: 0,
        dropped: 0,
        allocations,
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
        buffer.boxed_bytes = 0;
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
