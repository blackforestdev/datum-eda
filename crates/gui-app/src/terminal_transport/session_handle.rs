use super::{
    TerminalInputError, TerminalTransportEvent, TerminalWakeGate,
    control::ControlBacklog,
    input::{self, InputQueue},
    linux::job_control,
    output::OutputBacklog,
    process_supervisor::ProcessSupervisor,
    reader,
    shutdown::ShutdownPhase,
};
use anyhow::Result;
use std::{
    fs::File,
    os::fd::AsRawFd,
    process::Child,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::TryRecvError,
    },
    time::Instant,
};
#[cfg(test)]
use std::{sync::mpsc::RecvTimeoutError, time::Duration};

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
        let reader_stop = reader::spawn_reader(self.reader, output.clone(), control.clone());
        input::spawn_writer(self.writer, input.clone(), control.clone(), wake.clone());
        let supervisor =
            ProcessSupervisor::start(self.child, self.process_group_id, control.clone(), wake);
        TerminalTransportSession::new(
            input,
            output,
            control,
            self.control_file,
            self.process_group_id,
            Some(supervisor),
            reader_stop,
        )
    }
}

pub(crate) struct TerminalTransportSession {
    input: Arc<InputQueue>,
    output: Arc<OutputBacklog>,
    control: Arc<ControlBacklog>,
    control_file: Mutex<Option<File>>,
    process_group_id: libc::pid_t,
    supervisor: Option<Arc<ProcessSupervisor>>,
    terminating: AtomicBool,
    reader_stop: Arc<AtomicBool>,
}

impl TerminalTransportSession {
    pub(super) fn new(
        input: Arc<InputQueue>,
        output: Arc<OutputBacklog>,
        control: Arc<ControlBacklog>,
        control_file: File,
        process_group_id: libc::pid_t,
        supervisor: Option<Arc<ProcessSupervisor>>,
        reader_stop: Arc<AtomicBool>,
    ) -> Self {
        Self {
            input,
            output,
            control,
            control_file: Mutex::new(Some(control_file)),
            process_group_id,
            supervisor,
            terminating: AtomicBool::new(false),
            reader_stop,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn try_recv_event(&self) -> Result<TerminalTransportEvent, TryRecvError> {
        if let Some(event) = self
            .control
            .try_pop(self.output.has_pending(), self.session_closed())
        {
            return Ok(event);
        }
        if let Some(bytes) = self.output.try_pop() {
            return Ok(TerminalTransportEvent::Output(bytes));
        }
        Err(TryRecvError::Empty)
    }

    pub(crate) fn try_recv_control_event(&self) -> Option<TerminalTransportEvent> {
        self.control
            .try_pop(self.output.has_pending(), self.session_closed())
    }

    pub(crate) fn try_recv_output(&self, max_bytes: usize) -> Option<Vec<u8>> {
        self.output.try_pop_if_fits(max_bytes)
    }

    pub(crate) fn has_pending_event(&self) -> bool {
        self.control.has_pending(self.session_closed()) || self.output.has_pending()
    }

    fn session_closed(&self) -> bool {
        let closed = self
            .supervisor
            .as_ref()
            .is_none_or(|supervisor| supervisor.snapshot().phase == ShutdownPhase::Closed);
        if closed {
            self.input.close();
            self.reader_stop.store(true, Ordering::Release);
            self.control_file
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take();
        }
        closed
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
        if self.terminating.load(Ordering::Acquire) {
            return Err(TerminalInputError::Closed);
        }
        self.input.try_enqueue(bytes)
    }

    pub(crate) fn terminate(&self) -> Result<()> {
        self.terminate_with_deadline(None)
    }

    pub(crate) fn terminate_by(&self, deadline: Instant) -> Result<()> {
        self.terminate_with_deadline(Some(deadline))
    }

    fn terminate_with_deadline(&self, deadline: Option<Instant>) -> Result<()> {
        self.terminating.store(true, Ordering::Release);
        self.input.close();
        if let Some(supervisor) = &self.supervisor {
            if let Some(deadline) = deadline {
                supervisor.request_graceful_by(deadline);
            } else {
                supervisor.request_graceful();
            }
        }
        Ok(())
    }

    pub(crate) fn force_kill(&self) {
        self.terminating.store(true, Ordering::Release);
        self.input.close();
        if let Some(supervisor) = &self.supervisor {
            supervisor.request_force();
        }
    }

    pub(crate) fn retry_termination_by(&self, deadline: Instant) {
        self.terminating.store(true, Ordering::Release);
        self.input.close();
        if let Some(supervisor) = &self.supervisor {
            supervisor.request_retry_by(deadline);
        }
    }

    pub(crate) fn shutdown_snapshot(&self) -> Option<super::ShutdownSnapshot> {
        self.supervisor
            .as_ref()
            .map(|supervisor| supervisor.snapshot())
    }

    pub(crate) fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        if self.terminating.load(Ordering::Acquire) {
            anyhow::bail!("terminal session is terminating");
        }
        let control_file = self
            .control_file
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let control_file = control_file
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("terminal session is closed"))?;
        job_control::resize(control_file.as_raw_fd(), cols, rows)
    }

    pub(crate) fn presentation_complete(&self) -> bool {
        self.session_closed() && !self.output.has_pending() && self.control.presentation_complete()
    }
}

impl Drop for TerminalTransportSession {
    fn drop(&mut self) {
        self.input.close();
        self.output.close();
        if let Some(supervisor) = &self.supervisor {
            supervisor.request_graceful();
        }
    }
}

#[cfg(test)]
impl TerminalTransportSession {
    pub(crate) fn synthetic(wake: TerminalWakeGate) -> Self {
        let control_file = File::open("/dev/null").expect("open synthetic control file");
        let control = Arc::new(ControlBacklog::new(wake.clone()));
        control.writer_finished();
        Self::new(
            Arc::new(InputQueue::new()),
            Arc::new(OutputBacklog::new(wake.clone())),
            control,
            control_file,
            libc::pid_t::MAX,
            None,
            Arc::new(AtomicBool::new(false)),
        )
    }

    pub(crate) fn push_synthetic_output(&self, bytes: &[u8]) {
        assert!(self.output.push_lossless(bytes.to_vec().into_boxed_slice()));
    }

    pub(crate) fn push_synthetic_error(&self) {
        self.control
            .reader_failed(super::event::TerminalIoError::read(&std::io::Error::other(
                "synthetic read failure",
            )));
    }

    pub(crate) fn push_synthetic_child_exit(&self, status: super::TerminalExitStatus) {
        self.control.child_exited(status);
    }

    pub(crate) fn finish_synthetic_reader(&self) {
        self.control.reader_finished();
    }

    pub(crate) fn output_queued_bytes_for_test(&self) -> usize {
        self.output.queued_bytes()
    }

    #[cfg(test)]
    pub(crate) fn output_queued_chunks_for_test(&self) -> usize {
        self.output.queued_chunks()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal_transport::{TerminalExitStatus, TerminalTransportEvent};

    #[test]
    fn final_exit_waits_for_reader_completion_and_every_output_byte() {
        let session = TerminalTransportSession::synthetic(TerminalWakeGate::new(None));
        session.push_synthetic_child_exit(TerminalExitStatus::Code(37));
        assert!(!session.has_pending_event());

        session.push_synthetic_output(b"final-tail");
        session.finish_synthetic_reader();
        assert!(session.try_recv_control_event().is_none());
        assert_eq!(session.try_recv_output(5), Some(b"final".to_vec()));
        assert!(session.try_recv_control_event().is_none());
        assert_eq!(session.try_recv_output(usize::MAX), Some(b"-tail".to_vec()));
        assert_eq!(
            session.try_recv_control_event(),
            Some(TerminalTransportEvent::Exited(TerminalExitStatus::Code(37)))
        );
        assert!(session.try_recv_control_event().is_none());
    }

    #[test]
    fn exact_signal_exit_is_preserved() {
        let session = TerminalTransportSession::synthetic(TerminalWakeGate::new(None));
        let status = TerminalExitStatus::Signal {
            signal: libc::SIGTERM,
            core_dumped: false,
        };
        session.push_synthetic_child_exit(status);
        session.finish_synthetic_reader();
        assert_eq!(
            session.try_recv_control_event(),
            Some(TerminalTransportEvent::Exited(status))
        );
    }

    #[test]
    fn termination_atomically_rejects_input_and_resize_before_worker_progress() {
        let session = TerminalTransportSession::synthetic(TerminalWakeGate::new(None));
        session.terminate().unwrap();
        assert!(matches!(
            session.write_bytes(b"must-not-enter-pty"),
            Err(TerminalInputError::Closed)
        ));
        assert!(session.resize(100, 30).is_err());
    }
}
