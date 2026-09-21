//! One event-thread owner for native configuration admission and GPU receipts.
//! Callbacks publish only a completion watermark; they never retain a window.
use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::Rc,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

#[path = "native_attachment_ledger.rs"]
mod attachment;

#[derive(Clone, Default)]
pub(super) struct QueueOwner(Rc<RefCell<State>>);
struct State {
    epoch: u64,
    next_host: u64,
    submitted: u64,
    configurations: u64,
    completed: Arc<AtomicU64>,
    device_lost: Arc<AtomicBool>,
    tickets: VecDeque<u64>,
    attachments: Arc<Mutex<attachment::Ledger>>,
}
impl Default for State {
    fn default() -> Self {
        static EPOCH: AtomicU64 = AtomicU64::new(1);
        Self {
            epoch: EPOCH
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
                .expect("native queue epoch exhausted"),
            next_host: 0,
            submitted: 0,
            configurations: 0,
            completed: Arc::default(),
            device_lost: Arc::default(),
            tickets: VecDeque::new(),
            attachments: Arc::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Admission {
    Frame,
    Configure,
    Wait,
}

impl QueueOwner {
    pub(super) fn with_device_loss(signal: Arc<AtomicBool>) -> Self {
        let owner = Self::default();
        owner.0.borrow_mut().device_lost = signal;
        owner
    }

    fn with_attachments<R>(&self, operation: impl FnOnce(&mut attachment::Ledger, u64) -> R) -> R {
        let state = self.0.borrow();
        let completed = state.completed.load(Ordering::Acquire);
        let mut ledger = state
            .attachments
            .lock()
            .expect("attachment ledger poisoned");
        if state.device_lost.load(Ordering::Acquire) {
            ledger.confirm_device_loss(completed);
        }
        operation(&mut ledger, completed)
    }

    /// Native frame submission receipts and their GPU completion watermark, not a count of
    /// every raw queue submission (renderer initialization may also submit).
    pub(super) fn snapshot(&self) -> (u64, u64, u64) {
        let state = self.0.borrow();
        (
            state.epoch,
            state.submitted,
            state.completed.load(Ordering::Acquire),
        )
    }

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
    pub(super) fn observe_attachment(
        &self,
        host: u64,
        owner: u64,
        allocation: u64,
        payload_bytes: Option<u64>,
        submission: u64,
    ) {
        assert!(submission <= self.0.borrow().submitted);
        self.with_attachments(|ledger, completed| {
            ledger.observe(
                attachment::Allocation {
                    host,
                    owner,
                    allocation,
                    payload_bytes,
                    last_submission: submission,
                    release_reason: None,
                },
                completed,
            )
        });
    }

    pub(super) fn close_host(&self, host: u64) {
        self.cancel(host);
        self.with_attachments(|ledger, completed| ledger.close(host, completed));
    }

    pub(super) fn trace_attachments(&self) {
        if std::env::var_os("DATUM_GUI_VERBOSE_LOG").is_none() {
            return;
        }
        let epoch = self.0.borrow().epoch;
        let snapshot = self.with_attachments(|ledger, completed| ledger.snapshot(completed));
        super::append_gui_diagnostic_line(format!(
            "native attachment ledger {}",
            serde_json::json!({ "queue_epoch": epoch, "attachments": snapshot })
        ));
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
        queue.on_submitted_work_done(self.completion_callback(serial, completion));
        serial
    }
    fn completion_callback(
        &self,
        serial: u64,
        completion: Arc<AtomicU64>,
    ) -> impl FnOnce() + Send + 'static {
        // Keep only accounting metadata alive across last-host closure. The
        // callback still publishes one watermark; no window or GPU view is held.
        let attachments = self.0.borrow().attachments.clone();
        move || {
            completion.fetch_max(serial, Ordering::Release);
            drop(attachments);
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn backend_loss_retires_old_epoch_without_faking_or_advancing_new_completion() {
        let loss = Arc::new(AtomicBool::new(false));
        let old = QueueOwner::with_device_loss(loss.clone());
        let new = QueueOwner::with_device_loss(Arc::default());
        let a = old.register();
        let b = new.register();
        let (old_serial, old_completion) = old.submission_receipt();
        let (new_serial, _) = new.submission_receipt();
        old.observe_attachment(a, 1, 1, Some(1024), old_serial);
        new.observe_attachment(b, 2, 1, Some(2048), new_serial);
        loss.store(true, Ordering::Release);
        old.close_host(a);
        new.close_host(b);
        let retired = old.with_attachments(|ledger, completed| ledger.snapshot(completed));
        assert_eq!(retired.device_loss_retirements, 1);
        assert_eq!(retired.completed_retirements, 0);
        old_completion.store(old_serial, Ordering::Release);
        assert_eq!(
            old.with_attachments(|ledger, completed| ledger.snapshot(completed))
                .completed_retirements,
            0
        );
        let pending = new.with_attachments(|ledger, completed| ledger.snapshot(completed));
        assert!(!pending.device_loss_confirmed);
        assert_eq!(pending.retiring_payload_bytes, 2048);
        assert_eq!(new.snapshot().2, 0);
    }

    #[test]
    fn completion_callback_keeps_closed_host_accounting_until_gpu_completion() {
        let owner = QueueOwner::default();
        let host = owner.register();
        let (serial, completion) = owner.submission_receipt();
        let callback = owner.completion_callback(serial, completion);
        owner.observe_attachment(host, 1, 1, Some(4096), serial);
        owner.close_host(host);
        let weak = Arc::downgrade(&owner.0.borrow().attachments);
        drop(owner);
        {
            let ledger = weak
                .upgrade()
                .expect("callback must retain retiring metadata");
            let snapshot = ledger.lock().unwrap().snapshot(0);
            assert_eq!(snapshot.current_payload_bytes, 0);
            assert_eq!(snapshot.retiring_payload_bytes, 4096);
        }
        callback();
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn closed_attachment_retirement_cannot_release_a_later_host_submission() {
        let owner = QueueOwner::default();
        let a = owner.register();
        let b = owner.register();
        let (first, completion) = owner.submission_receipt();
        let (second, _) = owner.submission_receipt();
        owner.observe_attachment(a, 1, 1, Some(1024), first);
        owner.observe_attachment(b, 2, 1, Some(2048), second);
        owner.close_host(a);
        owner.close_host(b);
        completion.store(first, Ordering::Release);
        let state = owner.0.borrow();
        let mut ledger = state.attachments.lock().unwrap();
        let snapshot = ledger.snapshot(completion.load(Ordering::Acquire));
        assert_eq!(snapshot.retiring_payload_bytes, 2048);
        assert_eq!(snapshot.allocations[0].host, b);
        assert_eq!(snapshot.completed_retirements, 1);
        completion.store(second, Ordering::Release);
        assert!(ledger.snapshot(second).allocations.is_empty());
    }

    #[test]
    fn queue_epochs_distinguish_replacement_from_late_old_completion() {
        let old = QueueOwner::default();
        let shared = old.clone();
        let replacement = QueueOwner::default();
        let (serial, completion) = old.submission_receipt();
        assert_eq!(old.snapshot(), shared.snapshot());
        assert_ne!(old.snapshot().0, replacement.snapshot().0);
        completion.store(serial, Ordering::Release);
        assert_eq!(old.snapshot().2, serial);
        assert_eq!(replacement.snapshot().1, 0);
        assert_eq!(replacement.snapshot().2, 0);
    }

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
    }
}
