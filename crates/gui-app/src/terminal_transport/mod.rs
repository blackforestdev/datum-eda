//! Datum-owned terminal process transport.
//!
//! This module owns PTY allocation, child attachment, byte transport, process
//! identity, resize, and teardown. It deliberately knows nothing about cells,
//! escape parsing, rendering, selection, or Datum design state.

mod event;
mod linux;
mod reader;
mod request;
mod session_handle;
mod wake;

use anyhow::{Context, Result};
pub(super) use event::TerminalTransportEvent;
pub(super) use request::TerminalTransportRequest;
pub(super) use session_handle::{PreparedTerminalTransport, TerminalTransportSession};
pub(super) use wake::TerminalWakeGate;

pub(super) fn prepare_terminal_transport(
    request: TerminalTransportRequest,
) -> Result<PreparedTerminalTransport> {
    let spawn_failure_context = request.spawn_failure_context();
    let pty = linux::pty::open_pty_pair().context("open terminal PTY")?;
    let reader = pty
        .master
        .try_clone()
        .context("clone terminal PTY master for reader")?;
    let mut command = request.into_command();
    linux::spawn::attach_child_pty(&mut command, pty.slave_path, pty.master_fd);
    let child = command.spawn().with_context(|| spawn_failure_context)?;
    let process_group_id = child.id() as libc::pid_t;
    Ok(PreparedTerminalTransport::new(
        pty.master,
        reader,
        child,
        pty.master_fd,
        process_group_id,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn spawn_failure_preserves_shell_and_project_root_context() {
        let request =
            TerminalTransportRequest::new("/datum-test/nonexistent-shell", PathBuf::from("/tmp"));
        let error = match prepare_terminal_transport(request) {
            Ok(_) => panic!("spawn must fail"),
            Err(error) => error,
        };
        assert_eq!(
            error.to_string(),
            "spawn PTY terminal shell /datum-test/nonexistent-shell in /tmp"
        );
    }
}
