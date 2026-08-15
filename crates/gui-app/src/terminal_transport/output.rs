use super::{TerminalWakeGate, limits};
use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
};

#[derive(Default)]
struct OutputState {
    chunks: VecDeque<Box<[u8]>>,
    queued_bytes: usize,
    reserved_chunks: usize,
    reserved_bytes: usize,
    closed: bool,
}

pub(super) struct OutputBacklog {
    state: Mutex<OutputState>,
    not_full: Condvar,
    wake: TerminalWakeGate,
}

pub(super) struct OutputPermit {
    backlog: Arc<OutputBacklog>,
    committed: bool,
}

impl OutputBacklog {
    pub(super) fn new(wake: TerminalWakeGate) -> Self {
        Self {
            state: Mutex::new(OutputState::default()),
            not_full: Condvar::new(),
            wake,
        }
    }

    pub(super) fn reserve(self: &Arc<Self>) -> Option<OutputPermit> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while !state.closed
            && (state.chunks.len() + state.reserved_chunks == limits::MAX_OUTPUT_CHUNKS
                || state.queued_bytes + state.reserved_bytes + limits::MAX_OUTPUT_CHUNK_BYTES
                    > limits::MAX_OUTPUT_BYTES)
        {
            state = self
                .not_full
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        if state.closed {
            return None;
        }
        state.reserved_chunks += 1;
        state.reserved_bytes += limits::MAX_OUTPUT_CHUNK_BYTES;
        Some(OutputPermit {
            backlog: self.clone(),
            committed: false,
        })
    }

    #[cfg(test)]
    pub(super) fn push_lossless(self: &Arc<Self>, bytes: Box<[u8]>) -> bool {
        self.reserve().is_some_and(|permit| permit.publish(bytes))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn try_pop(&self) -> Option<Vec<u8>> {
        self.try_pop_if_fits(usize::MAX)
    }

    pub(super) fn try_pop_if_fits(&self, remaining: usize) -> Option<Vec<u8>> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if remaining == 0 {
            return None;
        }
        let bytes = state.chunks.pop_front()?;
        if bytes.len() > remaining {
            let mut bytes = bytes.into_vec();
            let suffix = bytes.split_off(remaining);
            bytes.shrink_to_fit();
            state.chunks.push_front(suffix.into_boxed_slice());
            state.queued_bytes -= bytes.len();
            self.not_full.notify_one();
            return Some(bytes);
        }
        state.queued_bytes -= bytes.len();
        self.not_full.notify_one();
        Some(bytes.into_vec())
    }

    pub(super) fn has_pending(&self) -> bool {
        !self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .chunks
            .is_empty()
    }

    pub(super) fn close(&self) {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .closed = true;
        self.not_full.notify_all();
    }

    fn release_reservation(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.reserved_chunks -= 1;
        state.reserved_bytes -= limits::MAX_OUTPUT_CHUNK_BYTES;
        self.not_full.notify_one();
    }
}

impl OutputPermit {
    pub(super) fn publish(mut self, bytes: Box<[u8]>) -> bool {
        assert!(bytes.len() <= limits::MAX_OUTPUT_CHUNK_BYTES);
        let mut state = self
            .backlog
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.reserved_chunks -= 1;
        state.reserved_bytes -= limits::MAX_OUTPUT_CHUNK_BYTES;
        if state.closed {
            self.committed = true;
            return false;
        }
        state.queued_bytes += bytes.len();
        state.chunks.push_back(bytes);
        self.committed = true;
        drop(state);
        self.backlog.wake.request();
        true
    }
}

impl Drop for OutputPermit {
    fn drop(&mut self) {
        if !self.committed {
            self.backlog.release_reservation();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::{Arc, mpsc},
        time::Duration,
    };

    #[test]
    fn full_output_backlog_blocks_reservation_until_consumer_pop() {
        let backlog = Arc::new(OutputBacklog::new(TerminalWakeGate::new(None)));
        for value in 0..limits::MAX_OUTPUT_CHUNKS {
            assert!(backlog.push_lossless(
                vec![value as u8; limits::MAX_OUTPUT_CHUNK_BYTES].into_boxed_slice()
            ));
        }
        let (tx, rx) = mpsc::channel();
        let producer = backlog.clone();
        std::thread::spawn(move || tx.send(producer.reserve().is_some()).unwrap());
        assert!(rx.recv_timeout(Duration::from_millis(20)).is_err());
        assert_eq!(
            backlog.try_pop().unwrap().len(),
            limits::MAX_OUTPUT_CHUNK_BYTES
        );
        assert_eq!(rx.recv_timeout(Duration::from_secs(1)), Ok(true));
        let mut bytes = 0;
        while let Some(chunk) = backlog.try_pop() {
            bytes += chunk.len();
        }
        assert_eq!(
            bytes,
            limits::MAX_OUTPUT_BYTES - limits::MAX_OUTPUT_CHUNK_BYTES
        );
    }

    #[test]
    fn non_divisor_chunks_fill_the_exact_gui_byte_budget_without_reordering() {
        let backlog = Arc::new(OutputBacklog::new(TerminalWakeGate::new(None)));
        for value in 0..7_u8 {
            assert!(backlog.push_lossless(vec![value; 10_000].into_boxed_slice()));
        }
        let mut drained = Vec::new();
        let mut remaining = limits::GUI_DRAIN_BYTE_LIMIT;
        while let Some(bytes) = backlog.try_pop_if_fits(remaining) {
            remaining -= bytes.len();
            drained.extend(bytes);
        }
        assert_eq!(drained.len(), limits::GUI_DRAIN_BYTE_LIMIT);
        assert_eq!(&drained[60_000..], &vec![6; 5_536]);
        assert_eq!(backlog.try_pop(), Some(vec![6; 4_464]));
    }
}
