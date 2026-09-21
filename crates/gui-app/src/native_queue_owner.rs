//! One event-thread owner for native configuration admission and GPU receipts.
//! Callbacks publish only a completion watermark; they never retain a window.
use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

#[derive(Clone, Default)]
pub(super) struct QueueOwner(Rc<RefCell<State>>);
#[derive(Default)]
struct State {
    next_host: u64,
    submitted: u64,
    configurations: u64,
    completed: Arc<AtomicU64>,
    tickets: VecDeque<u64>,
}
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Admission {
    Frame,
    Configure,
    Wait,
}

impl QueueOwner {
    pub(super) fn register(&self) -> u64 {
        let mut state = self.0.borrow_mut();
        state.next_host = state
            .next_host
            .checked_add(1)
            .expect("queue host generation exhausted");
        state.next_host
    }
    pub(super) fn cancel(&self, host: u64) {
        self.0.borrow_mut().tickets.retain(|id| *id != host);
    }
    pub(super) fn admit(&self, host: u64, configuration: bool, in_flight: u64) -> Admission {
        let mut state = self.0.borrow_mut();
        if configuration && !state.tickets.contains(&host) {
            state.tickets.push_back(host);
        }
        if !configuration {
            state.tickets.retain(|id| *id != host);
        }
        let completed = state.completed.load(Ordering::Acquire);
        if let Some(first) = state.tickets.front() {
            if *first != host || completed < state.submitted {
                return Admission::Wait;
            }
            return Admission::Configure;
        }
        if completed < in_flight {
            Admission::Wait
        } else {
            Admission::Frame
        }
    }
    pub(super) fn configured(&self, host: u64) {
        let mut state = self.0.borrow_mut();
        assert_eq!(state.tickets.pop_front(), Some(host));
        let completed = state.completed.load(Ordering::Acquire);
        assert!(completed >= state.submitted);
        state.configurations += 1;
        super::append_gui_diagnostic_line(format!(
            "native queue configured host={host} configurations={} submitted={} completed={completed}",
            state.configurations, state.submitted
        ));
    }
    pub(super) fn submitted(&self, queue: &wgpu::Queue) -> u64 {
        let (serial, completion) = self.submission_receipt();
        queue.on_submitted_work_done(move || {
            completion.fetch_max(serial, Ordering::Release);
        });
        serial
    }
    fn submission_receipt(&self) -> (u64, Arc<AtomicU64>) {
        let mut state = self.0.borrow_mut();
        state.submitted = state
            .submitted
            .checked_add(1)
            .expect("queue submission generation exhausted");
        (state.submitted, Arc::clone(&state.completed))
    }
}

/// A bounded nonblocking drain; full drawable-active recovery is owned by the
/// frame coordinator migration. No timer is scheduled when there is no waiter.
pub(super) fn check_drain(start: Instant, now: Instant) -> anyhow::Result<()> {
    anyhow::ensure!(
        now.duration_since(start) < Duration::from_secs(2),
        "native queue drain exceeded two seconds; stop submissions"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn configuration_ticket_prevents_busy_host_overtaking_and_preserves_fifo() {
        let owner = QueueOwner::default();
        let a = owner.register();
        let b = owner.register();
        let c = owner.register();
        let (serial, completion) = owner.submission_receipt();
        assert_eq!(owner.admit(b, true, 0), Admission::Wait);
        for _ in 0..20 {
            assert_eq!(owner.admit(a, false, serial), Admission::Wait);
            assert_eq!(owner.admit(c, true, 0), Admission::Wait);
            assert_eq!(owner.admit(b, true, 0), Admission::Wait);
        }
        assert_eq!(owner.0.borrow().tickets.len(), 2);
        completion.store(serial, Ordering::Release);
        assert_eq!(owner.admit(b, true, 0), Admission::Configure);
        owner.configured(b);
        assert_eq!(owner.admit(b, false, 0), Admission::Wait);
        assert_eq!(owner.admit(a, false, serial), Admission::Wait);
        assert_eq!(owner.admit(c, true, 0), Admission::Configure);
        owner.configured(c);
        assert_eq!(owner.admit(a, false, serial), Admission::Frame);
    }
    #[test]
    fn closed_host_releases_ticket_but_not_outstanding_gpu_ownership() {
        let owner = QueueOwner::default();
        let a = owner.register();
        let b = owner.register();
        let (serial, completion) = owner.submission_receipt();
        assert_eq!(owner.admit(a, true, serial), Admission::Wait);
        owner.cancel(a);
        assert_eq!(owner.admit(b, true, 0), Admission::Wait);
        completion.store(serial, Ordering::Release);
        assert_eq!(owner.admit(b, true, 0), Admission::Configure);
        owner.configured(b);
        assert_ne!(owner.register(), a);
    }
    #[test]
    fn completed_watermark_does_not_complete_later_submission() {
        let owner = QueueOwner::default();
        let host = owner.register();
        let (old, completion) = owner.submission_receipt();
        let (new, _) = owner.submission_receipt();
        completion.fetch_max(old, Ordering::Release);
        assert_eq!(owner.admit(host, false, new), Admission::Wait);
        completion.fetch_max(new, Ordering::Release);
        assert_eq!(owner.admit(host, false, new), Admission::Frame);
        let now = Instant::now();
        assert!(check_drain(now, now + Duration::from_secs(2)).is_err());
    }
}
