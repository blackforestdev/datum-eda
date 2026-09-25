//! Allocation-event windows for opaque synchronous text calls. Reservations and
//! observations share one lock, so overlapping hosts use simultaneous totals.
use super::{Scope, with_current};
use crate::text_gpu::budget::{Budget, Permit};
use std::sync::{Arc, Mutex};

static LEDGER: Mutex<Ledger> = Mutex::new(Ledger {
    next: 1,
    calls: Vec::new(),
});

#[derive(Clone, Copy, Debug, Default)]
pub struct Report {
    pub call_id: u64,
    pub owner_id: u64,
    pub allocator_installed: bool,
    /// Admission occupancy includes held reservations as well as observed growth.
    /// The call's own initial/final/peak fields count actual scoped allocations.
    pub host_initial_bytes: u64,
    pub host_final_bytes: u64,
    pub process_initial_bytes: u64,
    pub process_final_bytes: u64,
    pub initial_bytes: u64,
    pub final_bytes: u64,
    pub peak_bytes: u64,
    pub host_peak_bytes: u64,
    pub process_peak_bytes: u64,
    pub exceeded: bool,
}

#[derive(Debug)]
pub struct Overrun(pub Report);
impl std::fmt::Display for Overrun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "private text construction exceeded memory budget: {:?}",
            self.0
        )
    }
}
impl std::error::Error for Overrun {}

struct Entry {
    id: u64,
    owner: u64,
    live: u64,
    excluded: u64,
    credit: u64,
    host: Arc<Budget>,
    process: Arc<Budget>,
    report: Report,
}
pub(crate) struct Ledger {
    next: u64,
    calls: Vec<Entry>,
}
impl Ledger {
    pub(crate) fn used(&self, budget: &Budget) -> u64 {
        let mut bytes = budget.used();
        for call in &self.calls {
            if std::ptr::eq(budget, &*call.host) || std::ptr::eq(budget, &*call.process) {
                bytes = bytes.saturating_sub(call.credit);
            }
        }
        for call in &self.calls {
            if std::ptr::eq(budget, &*call.host) || std::ptr::eq(budget, &*call.process) {
                // A private call may exceed its reservations, but cannot lend
                // still-owned pre-call capacity to concurrent admissions.
                bytes =
                    bytes.saturating_add(call.live.saturating_sub(call.excluded).max(call.credit));
            }
        }
        bytes
    }
    fn refresh(&mut self) {
        for index in 0..self.calls.len() {
            let call = &self.calls[index];
            let host = self.used(&call.host);
            let process = self.used(&call.process);
            let exceeded = host > call.host.limit() || process > call.process.limit();
            let call = &mut self.calls[index];
            let live = call.live.saturating_sub(call.excluded);
            call.report.final_bytes = live;
            call.report.host_final_bytes = host;
            call.report.process_final_bytes = process;
            call.report.peak_bytes = call.report.peak_bytes.max(live);
            call.report.host_peak_bytes = call.report.host_peak_bytes.max(host);
            call.report.process_peak_bytes = call.report.process_peak_bytes.max(process);
            call.report.exceeded |= exceeded;
        }
    }
    pub(super) fn allocation(&mut self, owner: u64, added: u64, removed: u64) {
        for call in &mut self.calls {
            if call.owner == owner {
                call.live = call.live + added - removed;
            }
        }
    }
}

/// No allocations or drops of scoped resources may occur inside the closure.
/// Registry storage itself is unscoped and is not allocator-hook recursion.
pub(crate) fn transaction<T>(work: impl FnOnce(&mut Ledger) -> T) -> T {
    with_current(std::ptr::null(), || {
        let mut ledger = LEDGER.lock().unwrap_or_else(|e| e.into_inner());
        let result = work(&mut ledger);
        ledger.refresh();
        result
    })
}

pub(crate) struct Call {
    id: Option<u64>,
}
impl Call {
    /// `excluded` is existing fixed/input/output ownership outside scratch;
    /// `credit` replaces the existing cache and temporary scratch reservations.
    pub fn begin(
        scope: &Scope,
        host: Arc<Budget>,
        process: Arc<Budget>,
        excluded: u64,
        credit: u64,
    ) -> Self {
        transaction(|ledger| {
            let usage = scope.usage();
            assert!(
                !ledger.calls.iter().any(|c| c.owner == usage.owner_id),
                "overlapping calls on one mutable text owner"
            );
            let id = ledger.next;
            ledger.next += 1;
            let live = usage.payload_bytes + usage.tracking_bytes;
            let initial = live.saturating_sub(excluded);
            ledger.calls.push(Entry {
                id,
                owner: usage.owner_id,
                live,
                excluded,
                credit,
                host,
                process,
                report: Report {
                    call_id: id,
                    owner_id: usage.owner_id,
                    allocator_installed: usage.allocator_installed,
                    initial_bytes: initial,
                    final_bytes: initial,
                    peak_bytes: initial,
                    ..Report::default()
                },
            });
            let index = ledger.calls.len() - 1;
            let host = ledger.used(&ledger.calls[index].host);
            let process = ledger.used(&ledger.calls[index].process);
            ledger.calls[index].report.host_initial_bytes = host;
            ledger.calls[index].report.process_initial_bytes = process;
            Self { id: Some(id) }
        })
    }

    /// Atomically transfer monitored construction to ordinary retained permits.
    /// Old/temporary permits are consumed under the same lock, without a gap or
    /// double charge visible to another host. Refusal is distinct from no glyph.
    pub fn finish(
        mut self,
        retained: u64,
        mut old: Option<[Permit; 2]>,
        mut temporary: Option<[Permit; 2]>,
    ) -> anyhow::Result<([Permit; 2], Report)> {
        let id = self.id.take().expect("active text call");
        let (result, _call) = transaction(|ledger| {
            let index = ledger
                .calls
                .iter()
                .position(|c| c.id == id)
                .expect("registered text call");
            let call = ledger.calls.remove(index);
            for pair in [&mut old, &mut temporary].into_iter().flatten() {
                for permit in pair {
                    permit.release_in_transaction();
                }
            }
            let host = ledger.used(&call.host).saturating_add(retained);
            let process = ledger.used(&call.process).saturating_add(retained);
            let mut report = call.report;
            report.exceeded |= host > call.host.limit() || process > call.process.limit();
            if report.exceeded {
                return (Err(Overrun(report)), call);
            }
            let result = Ok((
                [
                    Permit::admitted(call.host.clone(), retained),
                    Permit::admitted(call.process.clone(), retained),
                ],
                report,
            ));
            (result, call)
        });
        result.map_err(Into::into)
    }
}
impl Drop for Call {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            let _removed = transaction(|ledger| {
                let index = ledger
                    .calls
                    .iter()
                    .position(|call| call.id == id)
                    .expect("registered text call");
                ledger.calls.remove(index)
            });
        }
    }
}

#[cfg(test)]
#[path = "private_text_call_tests.rs"]
mod tests;
