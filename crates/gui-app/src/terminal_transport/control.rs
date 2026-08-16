use super::{
    TerminalTransportEvent, TerminalWakeGate, event::TerminalIoError,
    process_status::TerminalExitStatus,
};
use std::{collections::VecDeque, sync::Mutex};

#[derive(Default)]
struct ControlState {
    events: VecDeque<TerminalTransportEvent>,
    reader_failed: bool,
    writer_failed: bool,
    wait_failed: bool,
    reader_finished: bool,
    writer_finished: bool,
    child_exit: Option<TerminalExitStatus>,
    exit_published: bool,
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

    pub(super) fn wait_failed(&self, error: TerminalIoError) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.wait_failed {
            return;
        }
        state.wait_failed = true;
        state.events.push_back(TerminalTransportEvent::Error(error));
        drop(state);
        self.wake.request();
    }

    pub(super) fn child_exited(&self, status: TerminalExitStatus) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.child_exit.is_none() {
            state.child_exit = Some(status);
            drop(state);
            self.wake.request();
        }
    }

    pub(super) fn reader_finished(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !state.reader_finished {
            state.reader_finished = true;
            drop(state);
            self.wake.request();
        }
    }

    pub(super) fn writer_finished(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !state.writer_finished {
            state.writer_finished = true;
            drop(state);
            self.wake.request();
        }
    }

    pub(super) fn try_pop(
        &self,
        output_pending: bool,
        session_closed: bool,
    ) -> Option<TerminalTransportEvent> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(event) = state.events.pop_front() {
            return Some(event);
        }
        if session_closed
            && !output_pending
            && state.reader_finished
            && state.writer_finished
            && !state.exit_published
        {
            let status = state.child_exit?;
            state.exit_published = true;
            return Some(TerminalTransportEvent::Exited(status));
        }
        None
    }

    pub(super) fn has_pending(&self, session_closed: bool) -> bool {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        !state.events.is_empty()
            || (session_closed
                && state.reader_finished
                && state.writer_finished
                && state.child_exit.is_some()
                && !state.exit_published)
    }

    pub(super) fn presentation_complete(&self) -> bool {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.reader_finished
            && state.writer_finished
            && state.child_exit.is_some()
            && state.exit_published
    }
}
