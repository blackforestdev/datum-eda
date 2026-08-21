use super::{TerminalEvent, TerminalSession};
use crate::{
    terminal_context::{TerminalContext, tool_session_event_log_path},
    terminal_session_events::record_terminal_input_accepted_event,
    terminal_transport::{ShutdownSnapshot, TerminalTransportSession},
};
use anyhow::Result;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};

impl TerminalSession {
    pub(crate) fn from_transport(
        transport: TerminalTransportSession,
        context: TerminalContext,
        terminal_profile: crate::terminal_profile::TerminalLaunchProfile,
    ) -> Self {
        Self {
            transport,
            context_path: context.context_path,
            latest_context_path: context.latest_context_path,
            session_path: context.session_path,
            session_id: context.session_id,
            context_id: context.context_id,
            terminal_profile,
            active_execution_id: Arc::new(Mutex::new(None)),
            finished_scan_offset: std::cell::Cell::new(0),
        }
    }

    pub(crate) fn write_bytes(&self, bytes: &[u8]) -> Result<()> {
        self.transport.write_bytes(bytes)?;
        let _ = record_terminal_input_accepted_event(self, bytes);
        Ok(())
    }
    pub(crate) fn try_recv_control_event(&self) -> Option<TerminalEvent> {
        self.transport.try_recv_control_event()
    }
    pub(crate) fn try_recv_output(&self, max_bytes: usize) -> Option<Vec<u8>> {
        self.transport.try_recv_output(max_bytes)
    }
    pub(crate) fn has_pending_event(&self) -> bool {
        self.transport.has_pending_event()
    }
    #[cfg(test)]
    pub(crate) fn recv_event_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> Result<TerminalEvent, std::sync::mpsc::RecvTimeoutError> {
        self.transport.recv_event_timeout(timeout)
    }
    pub(crate) fn process_group_id(&self) -> libc::pid_t {
        self.transport.process_group_id()
    }
    pub(crate) fn session_id(&self) -> &str {
        &self.session_id
    }
    pub(crate) fn event_log_path(&self) -> PathBuf {
        tool_session_event_log_path(&self.session_path)
    }
    pub(crate) fn set_active_execution_id(&self, execution_id: String) {
        if let Ok(mut active) = self.active_execution_id.lock() {
            *active = Some(execution_id);
        }
    }
    pub(crate) fn active_execution_id(&self) -> Option<String> {
        self.active_execution_id
            .lock()
            .ok()
            .and_then(|active| active.clone())
    }
    pub(crate) fn clear_active_execution_id(&self, execution_id: &str) {
        if let Ok(mut active) = self.active_execution_id.lock()
            && active.as_deref() == Some(execution_id)
        {
            *active = None;
        }
    }
    pub(crate) fn finished_scan_offset(&self) -> u64 {
        self.finished_scan_offset.get()
    }
    pub(crate) fn set_finished_scan_offset(&self, offset: u64) {
        self.finished_scan_offset.set(offset);
    }
    pub(crate) fn terminate(&self) -> Result<()> {
        self.transport.terminate()
    }
    pub(crate) fn terminate_by(&self, deadline: Instant) -> Result<()> {
        self.transport.terminate_by(deadline)
    }
    pub(crate) fn force_kill(&self) {
        self.transport.force_kill();
    }
    pub(crate) fn retry_termination_by(&self, deadline: Instant) {
        self.transport.retry_termination_by(deadline);
    }
    pub(crate) fn shutdown_snapshot(&self) -> Option<ShutdownSnapshot> {
        self.transport.shutdown_snapshot()
    }
    pub(crate) fn presentation_complete(&self) -> bool {
        self.transport.presentation_complete()
    }
    pub(crate) fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        self.transport.resize(cols, rows)
    }
}
