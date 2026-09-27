//! Test-only backend delivery hold for the existing production completion closure.
use super::*;
use std::sync::Weak;
use winit::window::Window;

type Completion = Box<dyn FnOnce() + Send>;
#[derive(Default)]
pub(crate) struct CallbackHold {
    delivered: Mutex<Vec<Completion>>,
    // The negative captures only at registration, never in this test owner.
    strong_capture: Option<Weak<Window>>,
}
impl CallbackHold {
    pub(crate) fn new(strong_capture: Option<Weak<Window>>) -> Arc<Self> {
        Arc::new(Self {
            strong_capture,
            ..Self::default()
        })
    }
    pub(super) fn register(
        self: Arc<Self>,
        queue: &wgpu::Queue,
        callback: impl FnOnce() + Send + 'static,
    ) {
        let retained = self.strong_capture.as_ref().and_then(Weak::upgrade);
        queue.on_submitted_work_done(move || {
            self.delivered.lock().unwrap().push(Box::new(move || {
                callback();
                drop(retained);
            }));
        });
    }
    pub(crate) fn release(&self) -> usize {
        let callbacks = std::mem::take(&mut *self.delivered.lock().unwrap());
        let count = callbacks.len();
        for callback in callbacks {
            callback();
        }
        count
    }
}
impl QueueOwner {
    pub(crate) fn assert_healthy_for_test(&self) {
        let state = self.0.borrow();
        assert!(
            !state.progress.failed(),
            "queue progress failed during lifetime proof"
        );
        assert!(
            !state.device_lost.load(Ordering::Acquire),
            "device lost during lifetime proof"
        );
    }
    pub(crate) fn hold_callbacks_for_test(&self, hold: Option<Arc<CallbackHold>>) {
        self.0.borrow_mut().callback_hold = hold;
    }
    pub(crate) fn attachment_snapshot_for_test(&self) -> serde_json::Value {
        self.with_attachments(|ledger, completed| {
            serde_json::to_value(ledger.snapshot(completed)).unwrap()
        })
    }
}
