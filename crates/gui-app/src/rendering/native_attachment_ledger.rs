//! Native attachment reference/retirement accounting for one shared queue epoch.
//! Metadata alone survives host closure; it never retains a native window or GPU view.
use std::collections::BTreeMap;

#[derive(Clone, Copy, serde::Serialize)]
pub(super) struct Allocation {
    pub host: u64,
    pub owner: u64,
    pub allocation: u64,
    pub payload_bytes: Option<u64>,
    pub last_submission: u64,
    pub release_reason: Option<&'static str>,
}

#[derive(Default)]
pub(super) struct Ledger {
    records: BTreeMap<(u64, u64), Allocation>,
    observed: u64,
    retired: u64,
    device_retired: u64,
    device_lost: bool,
    peak_payload_bytes: u64,
}

#[derive(serde::Serialize)]
pub(super) struct Snapshot {
    pub current_payload_bytes: u64,
    pub retiring_payload_bytes: u64,
    pub peak_payload_bytes: u64,
    pub unknown_payload_allocations: usize,
    pub observed_allocations: u64,
    pub completed_retirements: u64,
    pub device_loss_retirements: u64,
    pub device_loss_confirmed: bool,
    pub allocations: Vec<Allocation>,
}

impl Ledger {
    pub(super) fn observe_host(
        &mut self,
        host: u64,
        allocations: impl IntoIterator<Item = Allocation>,
        completed: u64,
    ) {
        self.reap(completed);
        let mut current = [None; datum_gui_render::Renderer::SURFACE_ATTACHMENT_USAGE_CAPACITY];
        for (index, allocation) in allocations.into_iter().enumerate() {
            assert_eq!(allocation.host, host);
            let key = (allocation.owner, allocation.allocation);
            current[index] = allocation.release_reason.is_none().then_some(key);
            if let Some(previous) = self.records.get_mut(&key) {
                assert_eq!(previous.host, host, "native attachment changed host");
                assert_eq!(previous.payload_bytes, allocation.payload_bytes);
                assert!(
                    allocation.release_reason.is_some() || previous.release_reason.is_none(),
                    "retired attachment made current again"
                );
                previous.last_submission = previous.last_submission.max(allocation.last_submission);
            } else {
                self.records.insert(key, allocation);
                self.observed = self
                    .observed
                    .checked_add(1)
                    .expect("attachment count exhausted");
            }
        }
        // Observe the coherent image set before retiring missing references. A
        // later image in the same bundle does not supersede an earlier image.
        self.peak_payload_bytes = self.peak_payload_bytes.max(self.known_payload());
        for (id, previous) in &mut self.records {
            if previous.host == host
                && !current.contains(&Some(*id))
                && previous.release_reason.is_none()
            {
                previous.release_reason = Some("replacement");
            }
        }
        self.reap(completed);
    }

    pub(super) fn close(&mut self, host: u64, completed: u64) {
        for allocation in self.records.values_mut() {
            if allocation.host == host && allocation.release_reason.is_none() {
                allocation.release_reason = Some("host_closed");
            }
        }
        self.reap(completed);
    }

    pub(super) fn confirm_device_loss(&mut self, completed: u64) {
        self.device_lost = true;
        self.reap(completed);
    }

    fn known_payload(&self) -> u64 {
        self.records
            .values()
            .filter_map(|a| a.payload_bytes)
            .try_fold(0_u64, u64::checked_add)
            .expect("attachment bytes exhausted")
    }

    pub(super) fn reap(&mut self, completed: u64) {
        self.records.retain(|_, allocation| {
            let release = allocation.release_reason.is_some()
                && (allocation.last_submission <= completed || self.device_lost);
            if release {
                let count = if allocation.last_submission <= completed {
                    &mut self.retired
                } else {
                    &mut self.device_retired
                };
                *count = count
                    .checked_add(1)
                    .expect("attachment retirements exhausted");
            }
            !release
        });
        debug_assert_eq!(
            self.observed,
            self.retired + self.device_retired + self.records.len() as u64
        );
    }

    pub(super) fn snapshot(&mut self, completed: u64) -> Snapshot {
        self.reap(completed);
        let mut snapshot = Snapshot {
            current_payload_bytes: 0,
            retiring_payload_bytes: 0,
            peak_payload_bytes: self.peak_payload_bytes,
            unknown_payload_allocations: 0,
            observed_allocations: self.observed,
            completed_retirements: self.retired,
            device_loss_retirements: self.device_retired,
            device_loss_confirmed: self.device_lost,
            allocations: self.records.values().copied().collect(),
        };
        for allocation in &snapshot.allocations {
            let Some(bytes) = allocation.payload_bytes else {
                snapshot.unknown_payload_allocations += 1;
                continue;
            };
            let total = if allocation.release_reason.is_some() {
                &mut snapshot.retiring_payload_bytes
            } else {
                &mut snapshot.current_payload_bytes
            };
            *total = total
                .checked_add(bytes)
                .expect("attachment bytes exhausted");
        }
        snapshot
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    impl Ledger {
        fn observe(&mut self, allocation: Allocation, completed: u64) {
            self.observe_host(allocation.host, [allocation], completed);
        }
    }
    #[test]
    fn coherent_pair_observation_retires_only_missing_images() {
        let mut ledger = Ledger::default();
        let mut a = allocation(1, Some(1024), 1);
        let mut b = allocation(2, Some(1024), 1);
        ledger.observe_host(a.host, [a, b], 0);
        let both = ledger.snapshot(0);
        assert_eq!(both.current_payload_bytes, 2048);
        assert_eq!(both.retiring_payload_bytes, 0);
        b.last_submission = 2;
        ledger.observe_host(b.host, [b], 0);
        let evicted = ledger.snapshot(0);
        assert_eq!(evicted.current_payload_bytes, 1024);
        assert_eq!(evicted.retiring_payload_bytes, 1024);
        assert_eq!(ledger.snapshot(1).completed_retirements, 1);
        a.allocation = 3;
        a.last_submission = 3;
        b.last_submission = 3;
        ledger.observe_host(b.host, [a, b], 1);
        assert_eq!(ledger.snapshot(1).current_payload_bytes, 2048);
        ledger.close(b.host, 2);
        assert_eq!(ledger.snapshot(2).retiring_payload_bytes, 2048);
        assert!(ledger.snapshot(3).allocations.is_empty());
    }
    #[test]
    fn evicted_image_used_by_new_submission_updates_retirement_serial() {
        let mut ledger = Ledger::default();
        let mut a = allocation(1, Some(1024), 1);
        let mut b = allocation(2, Some(1024), 1);
        ledger.observe_host(a.host, [a, b], 0);
        a.last_submission = 3;
        a.release_reason = Some("optional_evicted_before_submission");
        b.last_submission = 3;
        ledger.observe_host(a.host, [a, b], 1);
        assert_eq!(ledger.snapshot(2).retiring_payload_bytes, 1024);
        assert_eq!(ledger.snapshot(3).completed_retirements, 1);
        assert_eq!(ledger.snapshot(3).current_payload_bytes, 1024);
    }
    fn allocation(id: u64, bytes: Option<u64>, submission: u64) -> Allocation {
        Allocation {
            host: 1,
            owner: 7,
            allocation: id,
            payload_bytes: bytes,
            last_submission: submission,
            release_reason: None,
        }
    }

    #[test]
    fn preparation_abort_releases_unsubmitted_replacement_without_completing_old_work() {
        let mut ledger = Ledger::default();
        ledger.observe(allocation(1, Some(1024), 7), 0);
        // Replacement is now owned before fallible scene preparation, not only
        // after rendering. No submission is invented for an aborted frame.
        ledger.observe(allocation(2, Some(2048), 0), 0);
        let prepared = ledger.snapshot(0);
        assert_eq!(prepared.current_payload_bytes, 2048);
        assert_eq!(prepared.retiring_payload_bytes, 1024);
        assert_eq!(prepared.observed_allocations, 2);
        ledger.close(1, 0);
        let aborted = ledger.snapshot(0);
        assert_eq!(aborted.current_payload_bytes, 0);
        assert_eq!(aborted.retiring_payload_bytes, 1024);
        assert_eq!(aborted.completed_retirements, 1);
        assert_eq!(aborted.allocations[0].allocation, 1);
        assert!(ledger.snapshot(7).allocations.is_empty());
    }

    #[test]
    fn replacement_and_close_preserve_bytes_until_their_own_gpu_completion() {
        let mut ledger = Ledger::default();
        ledger.observe(allocation(1, Some(1024), 1), 0);
        ledger.observe(allocation(1, Some(1024), 2), 0);
        ledger.observe(allocation(2, Some(2048), 3), 0);
        let before = ledger.snapshot(1);
        assert_eq!(
            (before.current_payload_bytes, before.retiring_payload_bytes),
            (2048, 1024)
        );
        assert_eq!(before.observed_allocations, 2);
        assert_eq!(before.peak_payload_bytes, 3072);
        assert_eq!(before.allocations[0].release_reason, Some("replacement"));
        ledger.close(1, 2);
        let closed = ledger.snapshot(2);
        assert_eq!(
            (closed.current_payload_bytes, closed.retiring_payload_bytes),
            (0, 2048)
        );
        assert_eq!(closed.completed_retirements, 1);
        assert_eq!(closed.allocations[0].release_reason, Some("host_closed"));
        let finished = ledger.snapshot(3);
        assert_eq!(finished.retiring_payload_bytes, 0);
        assert_eq!(finished.completed_retirements, 2);
        assert!(finished.allocations.is_empty());
    }

    #[test]
    fn confirmed_device_loss_retires_only_dropped_references_without_faking_completion() {
        let mut ledger = Ledger::default();
        ledger.observe(allocation(1, Some(1024), 9), 0);
        ledger.confirm_device_loss(0);
        let alive = ledger.snapshot(0);
        assert!(alive.device_loss_confirmed);
        assert_eq!(alive.current_payload_bytes, 1024);
        assert_eq!(alive.completed_retirements, 0);
        assert_eq!(alive.device_loss_retirements, 0);
        ledger.close(1, 0);
        let retired = ledger.snapshot(0);
        assert_eq!(retired.device_loss_retirements, 1);
        assert_eq!(retired.completed_retirements, 0);
        assert!(retired.allocations.is_empty());
        assert_eq!(ledger.snapshot(9).device_loss_retirements, 1);
        assert_eq!(ledger.snapshot(9).completed_retirements, 0);
    }

    #[test]
    fn completed_use_keeps_current_reference_and_unknown_bytes_stay_explicit() {
        let mut ledger = Ledger::default();
        ledger.observe(allocation(1, Some(1024), 1), 1);
        assert_eq!(ledger.snapshot(100).current_payload_bytes, 1024);
        ledger.observe(allocation(2, None, 0), 100);
        let snapshot = ledger.snapshot(100);
        assert_eq!(snapshot.unknown_payload_allocations, 1);
        assert_eq!(snapshot.completed_retirements, 1);
        ledger.close(1, 100);
        assert_eq!(ledger.snapshot(100).unknown_payload_allocations, 0);
    }
}
