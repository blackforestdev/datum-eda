use super::{
    TerminalInputError, TerminalTransportEvent, TerminalWakeGate,
    control::ControlBacklog,
    input::{self, InputQueue},
    linux::job_control,
    output::OutputBacklog,
    reader,
};
use anyhow::Result;
use std::{
    fs::File,
    os::fd::AsRawFd,
    process::Child,
    sync::{Arc, mpsc::TryRecvError},
};
#[cfg(test)]
use std::{
    sync::mpsc::RecvTimeoutError,
    time::{Duration, Instant},
};

pub(crate) struct PreparedTerminalTransport {
    writer: File,
    reader: File,
    control_file: File,
    child: Child,
    process_group_id: libc::pid_t,
}

impl PreparedTerminalTransport {
    pub(super) fn new(
        writer: File,
        reader: File,
        control_file: File,
        child: Child,
        process_group_id: libc::pid_t,
    ) -> Self {
        Self {
            writer,
            reader,
            control_file,
            child,
            process_group_id,
        }
    }

    pub(crate) fn process_group_id(&self) -> libc::pid_t {
        self.process_group_id
    }

    pub(crate) fn start(self, wake: TerminalWakeGate) -> TerminalTransportSession {
        let output = Arc::new(OutputBacklog::new(wake.clone()));
        let control = Arc::new(ControlBacklog::new(wake.clone()));
        let input = Arc::new(InputQueue::new());
        reader::spawn_reader(self.reader, output.clone(), control.clone());
        input::spawn_writer(self.writer, input.clone(), control.clone(), wake.clone());
        let waiter_control = control.clone();
        std::thread::spawn(move || {
            let mut child = self.child;
            waiter_control.exited(child.wait().ok().and_then(|status| status.code()));
        });
        TerminalTransportSession::new(
            input,
            output,
            control,
            self.control_file,
            self.process_group_id,
        )
    }
}

pub(crate) struct TerminalTransportSession {
    input: Arc<InputQueue>,
    output: Arc<OutputBacklog>,
    control: Arc<ControlBacklog>,
    control_file: File,
    process_group_id: libc::pid_t,
}

impl TerminalTransportSession {
    pub(super) fn new(
        input: Arc<InputQueue>,
        output: Arc<OutputBacklog>,
        control: Arc<ControlBacklog>,
        control_file: File,
        process_group_id: libc::pid_t,
    ) -> Self {
        Self {
            input,
            output,
            control,
            control_file,
            process_group_id,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn try_recv_event(&self) -> Result<TerminalTransportEvent, TryRecvError> {
        if let Some(event) = self.control.try_pop() {
            return Ok(event);
        }
        if let Some(bytes) = self.output.try_pop() {
            return Ok(TerminalTransportEvent::Output(bytes));
        }
        Err(TryRecvError::Empty)
    }

    pub(crate) fn try_recv_control_event(&self) -> Option<TerminalTransportEvent> {
        self.control.try_pop()
    }

    pub(crate) fn try_recv_output(&self, max_bytes: usize) -> Option<Vec<u8>> {
        self.output.try_pop_if_fits(max_bytes)
    }

    pub(crate) fn has_pending_event(&self) -> bool {
        self.control.has_pending() || self.output.has_pending()
    }

    #[cfg(test)]
    pub(crate) fn recv_event_timeout(
        &self,
        timeout: Duration,
    ) -> Result<TerminalTransportEvent, RecvTimeoutError> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Ok(event) = self.try_recv_event() {
                return Ok(event);
            }
            if Instant::now() >= deadline {
                return Err(RecvTimeoutError::Timeout);
            }
            std::thread::yield_now();
        }
    }

    pub(crate) fn process_group_id(&self) -> libc::pid_t {
        self.process_group_id
    }

    pub(crate) fn write_bytes(&self, bytes: &[u8]) -> std::result::Result<(), TerminalInputError> {
        self.input.try_enqueue(bytes)
    }

    pub(crate) fn interrupt(&self) -> Result<()> {
        job_control::signal_process_group(
            self.process_group_id,
            libc::SIGINT,
            "interrupt terminal process group",
        )
    }

    pub(crate) fn terminate(&self) -> Result<()> {
        job_control::signal_process_group(
            self.process_group_id,
            libc::SIGTERM,
            "terminate terminal process group",
        )
    }

    pub(crate) fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        job_control::resize(self.control_file.as_raw_fd(), cols, rows)
    }
}

impl Drop for TerminalTransportSession {
    fn drop(&mut self) {
        self.input.close();
        self.output.close();
        let _ = self.terminate();
    }
}

#[cfg(test)]
impl TerminalTransportSession {
    pub(crate) fn synthetic(wake: TerminalWakeGate) -> Self {
        let control_file = File::open("/dev/null").expect("open synthetic control file");
        Self::new(
            Arc::new(InputQueue::new()),
            Arc::new(OutputBacklog::new(wake.clone())),
            Arc::new(ControlBacklog::new(wake)),
            control_file,
            libc::pid_t::MAX,
        )
    }

    pub(crate) fn push_synthetic_output(&self, bytes: &[u8]) {
        assert!(self.output.push_lossless(bytes.to_vec().into_boxed_slice()));
    }

    pub(crate) fn push_synthetic_exit(&self, code: Option<i32>) {
        self.control.exited(code);
    }
}
