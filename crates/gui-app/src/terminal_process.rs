//! Application adapter from Datum session context to opaque PTY transport.

use crate::{
    terminal_context::{
        DATUM_CLI, DATUM_LEGACY_CLI, tool_session_event_log_path, write_terminal_context,
        write_terminal_context_files,
    },
    terminal_session::{TerminalLaunchContext, TerminalSession},
    terminal_transport::{TerminalTransportRequest, TerminalWakeGate, prepare_terminal_transport},
};
use anyhow::Result;

pub(super) fn spawn_terminal_process(
    context: &TerminalLaunchContext,
    terminal_wake: TerminalWakeGate,
) -> Result<TerminalSession> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let mut terminal_context = write_terminal_context(context)?;
    let mut request = TerminalTransportRequest::new(&shell, terminal_context.project_root.clone())
        .env("TERM", "xterm-256color")
        .env("DATUM_PROJECT_ROOT", context.project_root.as_os_str())
        .env("DATUM_CLI", DATUM_CLI)
        .env("DATUM_LEGACY_CLI", DATUM_LEGACY_CLI)
        .env("DATUM_CONTEXT_ID", &terminal_context.context_id)
        .env("DATUM_SESSION_ID", &terminal_context.session_id)
        .env("DATUM_DISCOVERY", terminal_context.context_path.as_os_str())
        .env(
            "DATUM_TOOL_SESSION_EVENT_LOG",
            tool_session_event_log_path(&terminal_context.session_path).as_os_str(),
        )
        .env(
            "DATUM_MODEL_REVISION",
            terminal_context.model_revision.as_deref().unwrap_or(""),
        )
        .env(
            "DATUM_TERMINAL_CONTEXT",
            terminal_context.context_path.as_os_str(),
        )
        .env("DATUM_TERMINAL_SESSION_ID", &terminal_context.session_id);
    if let Some(project_id) = &terminal_context.project_id {
        request = request.env("DATUM_PROJECT_ID", project_id);
    }
    if let Some(model_revision) = &terminal_context.model_revision {
        request = request.env("DATUM_SOURCE_REVISION", model_revision);
    }

    // Preserve the existing ordering: the child exists before its process ID is
    // persisted, but output/wait publication starts only after the pid-bearing
    // context files are durable.
    let prepared = prepare_terminal_transport(request)?;
    terminal_context.process_group_id = Some(prepared.process_group_id() as i32);
    let transport = persist_context_before_start(
        || write_terminal_context_files(&terminal_context, context),
        || prepared.start(terminal_wake),
    )?;

    Ok(TerminalSession::from_transport(transport, terminal_context))
}

fn persist_context_before_start<T>(
    persist: impl FnOnce() -> Result<()>,
    start: impl FnOnce() -> T,
) -> Result<T> {
    persist()?;
    Ok(start())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn context_persistence_failure_prevents_transport_start() {
        let started = Cell::new(false);
        let result = persist_context_before_start(
            || Err(anyhow::anyhow!("context persistence failed")),
            || started.set(true),
        );
        assert!(result.is_err());
        assert!(!started.get());
    }
}
