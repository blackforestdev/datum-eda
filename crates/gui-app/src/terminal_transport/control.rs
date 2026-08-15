use super::{TerminalTransportEvent, TerminalWakeGate, event::TerminalIoError};
use std::{collections::VecDeque, sync::Mutex};

#[derive(Default)]
struct ControlState {
    events: VecDeque<TerminalTransportEvent>,
    reader_failed: bool,
    writer_failed: bool,
    exited: bool,
}

pub(super) struct ControlBacklog {
    state: Mutex<ControlState>,
    wake: TerminalWakeGate,
}

impl ControlBacklog {
    pub(super) fn new(wake: TerminalWakeGate) -> Self {
        Self {
            state: Mutex::new(ControlState::default()),
            wake,
        }
    }

    pub(super) fn reader_failed(&self, error: TerminalIoError) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.reader_failed {
            return;
        }
        state.reader_failed = true;
        state.events.push_back(TerminalTransportEvent::Error(error));
        drop(state);
        self.wake.request();
    }

    pub(super) fn writer_failed(&self, error: TerminalIoError) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.writer_failed {
            return;
        }
        state.writer_failed = true;
        state.events.push_back(TerminalTransportEvent::Error(error));
        drop(state);
        self.wake.request();
    }

    pub(super) fn exited(&self, code: Option<i32>) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.exited {
            return;
        }
        state.exited = true;
        state.events.push_back(TerminalTransportEvent::Exited(code));
        drop(state);
        self.wake.request();
    }

    pub(super) fn try_pop(&self) -> Option<TerminalTransportEvent> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .events
            .pop_front()
    }

    pub(super) fn has_pending(&self) -> bool {
        !self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .events
            .is_empty()
    }
}
