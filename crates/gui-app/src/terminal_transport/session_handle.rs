use super::{TerminalTransportEvent, TerminalWakeGate, linux::job_control, reader};
use anyhow::{Context, Result};
use std::{
    fs::File,
    io::Write,
    os::fd::RawFd,
    process::Child,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, TryRecvError},
    },
};
#[cfg(test)]
use std::{sync::mpsc::RecvTimeoutError, time::Duration};

pub(crate) struct PreparedTerminalTransport {
    writer: File,
    reader: File,
    child: Child,
    master_fd: RawFd,
    process_group_id: libc::pid_t,
}

impl PreparedTerminalTransport {
    pub(super) fn new(
        writer: File,
        reader: File,
        child: Child,
        master_fd: RawFd,
        process_group_id: libc::pid_t,
    ) -> Self {
        Self {
            writer,
            reader,
            child,
            master_fd,
            process_group_id,
        }
    }

    pub(crate) fn process_group_id(&self) -> libc::pid_t {
        self.process_group_id
    }

    pub(crate) fn start(self, wake: TerminalWakeGate) -> TerminalTransportSession {
        let events = reader::spawn_event_threads(self.reader, self.child, wake);
        TerminalTransportSession::new(
            Arc::new(Mutex::new(self.writer)),
            events,
            self.master_fd,
            self.process_group_id,
        )
    }
}

pub(crate) struct TerminalTransportSession {
    writer: Arc<Mutex<File>>,
    events: Receiver<TerminalTransportEvent>,
    master_fd: RawFd,
    process_group_id: libc::pid_t,
}

impl TerminalTransportSession {
    pub(super) fn new(
        writer: Arc<Mutex<File>>,
        events: Receiver<TerminalTransportEvent>,
        master_fd: RawFd,
        process_group_id: libc::pid_t,
    ) -> Self {
        Self {
            writer,
            events,
            master_fd,
            process_group_id,
        }
    }

    pub(crate) fn try_recv_event(&self) -> Result<TerminalTransportEvent, TryRecvError> {
        self.events.try_recv()
    }

    #[cfg(test)]
    pub(crate) fn recv_event_timeout(
        &self,
        timeout: Duration,
    ) -> Result<TerminalTransportEvent, RecvTimeoutError> {
        self.events.recv_timeout(timeout)
    }

    pub(crate) fn process_group_id(&self) -> libc::pid_t {
        self.process_group_id
    }

    pub(crate) fn write_bytes(&self, bytes: &[u8]) -> Result<()> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| anyhow::anyhow!("lock terminal PTY master"))?;
        writer
            .write_all(bytes)
            .context("write terminal PTY input")?;
        writer.flush().context("flush terminal PTY input")
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
        job_control::resize(self.master_fd, cols, rows)
    }
}

impl Drop for TerminalTransportSession {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}
