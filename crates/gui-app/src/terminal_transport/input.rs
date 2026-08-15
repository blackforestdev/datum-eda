use super::{
    TerminalInputError, TerminalWakeGate, control::ControlBacklog, event::TerminalIoError, limits,
    linux::io as descriptor_io,
};
use std::{
    collections::VecDeque,
    fs::File,
    io::{self, Write},
    os::fd::AsRawFd,
    sync::{Arc, Condvar, Mutex, TryLockError},
    thread,
};

struct InputRequest {
    chunks: VecDeque<Box<[u8]>>,
    front_offset: usize,
    accepted_bytes: usize,
}

impl InputRequest {
    fn new(bytes: &[u8]) -> Self {
        Self {
            chunks: bytes
                .chunks(limits::MAX_OUTPUT_CHUNK_BYTES)
                .map(|chunk| chunk.to_vec().into_boxed_slice())
                .collect(),
            front_offset: 0,
            accepted_bytes: bytes.len(),
        }
    }

    fn current(&self) -> &[u8] {
        &self.chunks.front().expect("pending input chunk")[self.front_offset..]
    }

    fn advance(&mut self, count: usize) -> usize {
        self.front_offset += count;
        let front_len = self.chunks.front().expect("pending input chunk").len();
        if self.front_offset == front_len {
            self.front_offset = 0;
            self.chunks
                .pop_front()
                .expect("completed input chunk")
                .len()
        } else {
            0
        }
    }

    fn remaining_bytes(&self) -> usize {
        self.chunks
            .iter()
            .enumerate()
            .map(|(index, chunk)| chunk.len() - if index == 0 { self.front_offset } else { 0 })
            .sum()
    }
}

#[derive(Default)]
struct InputState {
    requests: VecDeque<InputRequest>,
    queued_bytes: usize,
    resident_bytes: usize,
    request_count: usize,
    closed: bool,
}

pub(super) struct InputQueue {
    state: Mutex<InputState>,
    available: Condvar,
}

impl InputQueue {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new(InputState::default()),
            available: Condvar::new(),
        }
    }

    pub(super) fn try_enqueue(&self, bytes: &[u8]) -> Result<(), TerminalInputError> {
        if bytes.is_empty() {
            return Ok(());
        }
        if bytes.len() > limits::MAX_INPUT_BYTES {
            return Err(TerminalInputError::RequestTooLarge);
        }
        let mut state = match self.state.try_lock() {
            Ok(state) => state,
            Err(TryLockError::WouldBlock) => return Err(TerminalInputError::Busy),
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
        };
        if state.closed {
            return Err(TerminalInputError::Closed);
        }
        if state.request_count == limits::MAX_INPUT_REQUESTS {
            return Err(TerminalInputError::RequestLimit);
        }
        if state.queued_bytes.saturating_add(bytes.len()) > limits::MAX_INPUT_BYTES
            || state.resident_bytes.saturating_add(bytes.len()) > limits::MAX_INPUT_BYTES
        {
            return Err(TerminalInputError::ByteLimit);
        }
        state.queued_bytes += bytes.len();
        state.resident_bytes += bytes.len();
        state.request_count += 1;
        state.requests.push_back(InputRequest::new(bytes));
        self.available.notify_one();
        Ok(())
    }

    fn take(&self) -> Option<InputRequest> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while !state.closed && state.requests.is_empty() {
            state = self
                .available
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        state.requests.pop_front()
    }

    fn account_progress(&self, written: usize, released_resident: usize) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.queued_bytes = state.queued_bytes.saturating_sub(written);
        state.resident_bytes = state.resident_bytes.saturating_sub(released_resident);
    }

    fn finish_request(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.request_count = state.request_count.saturating_sub(1);
    }

    fn fail_and_close(&self, unwritten_current: usize) -> (usize, usize) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let queued_tail = state
            .requests
            .iter()
            .map(InputRequest::remaining_bytes)
            .sum::<usize>();
        let undelivered_requests = state.request_count;
        let total_undelivered_bytes = unwritten_current + queued_tail;
        state.queued_bytes = 0;
        state.resident_bytes = 0;
        state.request_count = 0;
        state.requests.clear();
        state.closed = true;
        self.available.notify_all();
        (undelivered_requests, total_undelivered_bytes)
    }

    pub(super) fn close(&self) {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .closed = true;
        self.available.notify_all();
    }
}

pub(super) fn spawn_writer(
    writer: File,
    queue: Arc<InputQueue>,
    control: Arc<ControlBacklog>,
    _wake: TerminalWakeGate,
) {
    thread::spawn(move || {
        let _ = write_input(FileWriter(writer), queue, control);
    });
}

trait PtyWriteIo {
    fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<usize>;
    fn wait_writable(&mut self) -> io::Result<()>;
}

struct FileWriter(File);

impl PtyWriteIo for FileWriter {
    fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.write(bytes)
    }
    fn wait_writable(&mut self) -> io::Result<()> {
        descriptor_io::wait_writable(self.0.as_raw_fd()).map(|_| ())
    }
}

fn write_input<W: PtyWriteIo>(
    mut writer: W,
    queue: Arc<InputQueue>,
    control: Arc<ControlBacklog>,
) -> W {
    while let Some(mut request) = queue.take() {
        let accepted = request.accepted_bytes;
        let mut written = 0;
        while !request.chunks.is_empty() {
            match writer.write_bytes(request.current()) {
                Ok(0) => {
                    let error =
                        io::Error::new(io::ErrorKind::WriteZero, "PTY write made no progress");
                    let (requests, bytes) = queue.fail_and_close(request.remaining_bytes());
                    control.writer_failed(TerminalIoError::write(
                        &error, accepted, written, requests, bytes,
                    ));
                    return writer;
                }
                Ok(count) => {
                    written += count;
                    let released_resident = request.advance(count);
                    queue.account_progress(count, released_resident);
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    if let Err(wait_error) = writer.wait_writable() {
                        let (requests, bytes) = queue.fail_and_close(request.remaining_bytes());
                        control.writer_failed(TerminalIoError::write(
                            &wait_error,
                            accepted,
                            written,
                            requests,
                            bytes,
                        ));
                        return writer;
                    }
                }
                Err(error) => {
                    let (requests, bytes) = queue.fail_and_close(request.remaining_bytes());
                    control.writer_failed(TerminalIoError::write(
                        &error, accepted, written, requests, bytes,
                    ));
                    return writer;
                }
            }
        }
        queue.finish_request();
    }
    writer
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn input_admission_is_atomic_at_request_and_byte_limits() {
        let requests = InputQueue::new();
        for _ in 0..limits::MAX_INPUT_REQUESTS {
            assert_eq!(requests.try_enqueue(&[1]), Ok(()));
        }
        assert_eq!(
            requests.try_enqueue(&[2]),
            Err(TerminalInputError::RequestLimit)
        );

        let bytes = InputQueue::new();
        assert_eq!(bytes.try_enqueue(&vec![3; limits::MAX_INPUT_BYTES]), Ok(()));
        assert_eq!(bytes.try_enqueue(&[4]), Err(TerminalInputError::ByteLimit));
        assert_eq!(
            bytes.try_enqueue(&vec![5; limits::MAX_INPUT_BYTES + 1]),
            Err(TerminalInputError::RequestTooLarge)
        );
    }

    #[test]
    fn closed_input_rejects_without_accepting_a_prefix() {
        let queue = InputQueue::new();
        queue.close();
        assert_eq!(
            queue.try_enqueue(&[0, 0x1b, 0xff]),
            Err(TerminalInputError::Closed)
        );
    }

    #[test]
    fn partial_progress_releases_pending_and_completed_chunk_resident_budget() {
        let queue = InputQueue::new();
        assert_eq!(queue.try_enqueue(&vec![7; limits::MAX_INPUT_BYTES]), Ok(()));
        let mut request = queue.take().unwrap();
        assert_eq!(queue.try_enqueue(&[8]), Err(TerminalInputError::ByteLimit));
        let released = request.advance(limits::MAX_OUTPUT_CHUNK_BYTES);
        queue.account_progress(limits::MAX_OUTPUT_CHUNK_BYTES, released);
        assert_eq!(queue.try_enqueue(&[8]), Ok(()));
        assert_eq!(queue.state.lock().unwrap().request_count, 2);
    }

    #[test]
    fn write_failure_accounts_for_current_suffix_and_every_accepted_tail() {
        let queue = InputQueue::new();
        queue.try_enqueue(&[1; 10]).unwrap();
        queue.try_enqueue(&[2; 20]).unwrap();
        let _current = queue.take().unwrap();
        assert_eq!(queue.fail_and_close(6), (2, 26));
        assert_eq!(queue.try_enqueue(&[3]), Err(TerminalInputError::Closed));
    }

    enum Step {
        Write(usize),
        Interrupted,
        WouldBlock,
    }

    struct ScriptedWriter {
        steps: VecDeque<Step>,
        written: Vec<u8>,
        waits: usize,
    }

    impl PtyWriteIo for ScriptedWriter {
        fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<usize> {
            match self.steps.pop_front().expect("scripted write step") {
                Step::Write(count) => {
                    self.written.extend_from_slice(&bytes[..count]);
                    Ok(count)
                }
                Step::Interrupted => Err(io::Error::from(io::ErrorKind::Interrupted)),
                Step::WouldBlock => Err(io::Error::from(io::ErrorKind::WouldBlock)),
            }
        }
        fn wait_writable(&mut self) -> io::Result<()> {
            self.waits += 1;
            Ok(())
        }
    }

    #[test]
    fn writer_preserves_exact_bytes_across_partial_eintr_and_would_block() {
        let queue = Arc::new(InputQueue::new());
        queue.try_enqueue(b"abcdef").unwrap();
        queue.close();
        let wake = TerminalWakeGate::new(None);
        let control = Arc::new(ControlBacklog::new(wake));
        let writer = ScriptedWriter {
            steps: VecDeque::from([
                Step::Write(2),
                Step::Interrupted,
                Step::WouldBlock,
                Step::Write(4),
            ]),
            written: Vec::new(),
            waits: 0,
        };
        let writer = write_input(writer, queue, control);
        assert_eq!(writer.written, b"abcdef");
        assert_eq!(writer.waits, 1);
    }
}
