use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use winit::event_loop::EventLoopProxy;

#[derive(Clone)]
pub(crate) struct TerminalWakeGate {
    proxy: Option<EventLoopProxy<()>>,
    pending: Arc<AtomicBool>,
}

impl TerminalWakeGate {
    pub(crate) fn new(proxy: Option<EventLoopProxy<()>>) -> Self {
        Self {
            proxy,
            pending: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(crate) fn request(&self) {
        request_coalesced_wake(&self.pending, || {
            self.proxy
                .as_ref()
                .is_some_and(|proxy| proxy.send_event(()).is_ok())
        });
    }

    /// Consume a request before draining so concurrent output can schedule one
    /// successor. Without a proxy, callers must keep polling explicitly.
    pub(crate) fn begin_poll(&self, deferred_work: bool) -> bool {
        begin_poll(&self.pending, deferred_work || self.proxy.is_none())
    }
}

fn begin_poll(pending: &AtomicBool, must_poll: bool) -> bool {
    // One atomic take: a request before this point is serviced by this drain;
    // a later request remains pending and schedules its own event-loop turn.
    let requested = pending.swap(false, Ordering::AcqRel);
    must_poll || requested
}

fn request_coalesced_wake(pending: &AtomicBool, wake: impl FnOnce() -> bool) {
    if pending
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
        && !wake()
    {
        pending.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn burst_output_coalesces_to_one_pending_gui_wake() {
        let gate = TerminalWakeGate::new(None);
        let wakes = AtomicUsize::new(0);
        for _ in 0..10_000 {
            request_coalesced_wake(&gate.pending, || {
                wakes.fetch_add(1, Ordering::SeqCst);
                true
            });
        }
        assert_eq!(wakes.load(Ordering::SeqCst), 1);

        assert!(begin_poll(&gate.pending, false));
        assert!(!begin_poll(&gate.pending, false));
        request_coalesced_wake(&gate.pending, || {
            wakes.fetch_add(1, Ordering::SeqCst);
            true
        });
        assert_eq!(wakes.load(Ordering::SeqCst), 2);
        assert!(begin_poll(&gate.pending, false));
        assert!(!begin_poll(&gate.pending, false));
        assert!(begin_poll(&gate.pending, true));
        assert!(gate.begin_poll(false)); // No proxy retains explicit polling.
    }

    #[test]
    fn failed_gui_wake_releases_gate_for_retry() {
        let pending = AtomicBool::new(false);
        pending.store(false, Ordering::Release);
        request_coalesced_wake(&pending, || false);
        assert!(!pending.load(Ordering::Acquire));
        let wakes = AtomicUsize::new(0);
        request_coalesced_wake(&pending, || {
            wakes.fetch_add(1, Ordering::SeqCst);
            true
        });
        assert_eq!(wakes.load(Ordering::SeqCst), 1);
    }
}
