//! Application adapter from Datum session context to opaque PTY transport.

use crate::{
    terminal_capability::{DATUM_TERM, DATUM_TERM_PROGRAM, install_session_terminfo},
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
    let profile = context
        .terminal_profile
        .resolve(&context.project_root, &context.launch_working_directory);
    let mut terminal_context = write_terminal_context(context)?;
    let terminfo_root = install_session_terminfo(&terminal_context.session_path)?;
    let mut request =
        TerminalTransportRequest::new(&profile.executable, profile.cwd).args(profile.args);
    for (key, value) in profile.environment {
        request = match value {
            Some(value) => request.env(key, value),
            None => request.env_remove(key),
        };
    }
    let mut request = apply_datum_terminal_identity(request, &terminfo_root)
        .env("DATUM_TERMINAL_PROFILE", &profile.name)
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

    Ok(TerminalSession::from_transport(
        transport,
        terminal_context,
        context.terminal_profile.clone(),
    ))
}

/// Remove capability and owner identity inherited from the launching emulator.
/// DTC-P27 applies Datum's own values after this sanitization step.
fn apply_interactive_terminal_environment(
    request: TerminalTransportRequest,
) -> TerminalTransportRequest {
    request
        .env_remove("TERM")
        .env_remove("TERMINFO")
        .env_remove("TERM_PROGRAM")
        .env_remove("TERM_PROGRAM_VERSION")
}

fn apply_datum_terminal_identity(
    request: TerminalTransportRequest,
    terminfo_root: &std::path::Path,
) -> TerminalTransportRequest {
    apply_interactive_terminal_environment(request)
        .env("TERM", DATUM_TERM)
        .env("TERMINFO", terminfo_root.as_os_str())
        .env("TERM_PROGRAM", DATUM_TERM_PROGRAM)
        .env("TERM_PROGRAM_VERSION", env!("CARGO_PKG_VERSION"))
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
    fn interactive_terminal_environment_removes_foreign_capability_identity() {
        let request = TerminalTransportRequest::new("/bin/sh", "/tmp".into())
            .env("NO_COLOR", "1")
            .env("TERM", "owner-terminal")
            .env("COLORTERM", "owner-color-policy")
            .env("TERM_PROGRAM", "ghostty")
            .env("TERM_PROGRAM_VERSION", "fixture");
        let (command, _, _) = apply_interactive_terminal_environment(request).into_command();
        let env = command.get_envs().collect::<Vec<_>>();
        assert!(env.contains(&(std::ffi::OsStr::new("TERM"), None)));
        assert!(env.contains(&(std::ffi::OsStr::new("TERMINFO"), None)));
        assert!(env.contains(&(
            std::ffi::OsStr::new("COLORTERM"),
            Some(std::ffi::OsStr::new("owner-color-policy"))
        )));
        assert!(env.contains(&(
            std::ffi::OsStr::new("NO_COLOR"),
            Some(std::ffi::OsStr::new("1"))
        )));
        for key in ["TERM_PROGRAM", "TERM_PROGRAM_VERSION"] {
            assert!(env.contains(&(std::ffi::OsStr::new(key), None)));
        }
    }

    #[test]
    fn datum_terminal_identity_replaces_the_launching_emulator_exactly() {
        let root = std::path::Path::new("/tmp/datum-session/terminfo");
        let request = TerminalTransportRequest::new("/bin/sh", "/tmp".into())
            .env("TERM", "foreign-256color")
            .env("TERMINFO", "/foreign/terminfo")
            .env("TERM_PROGRAM", "foreign-terminal")
            .env("TERM_PROGRAM_VERSION", "99");
        let (command, _, _) = apply_datum_terminal_identity(request, root).into_command();
        let env = command.get_envs().collect::<Vec<_>>();
        for (key, value) in [
            ("TERM", DATUM_TERM),
            ("TERMINFO", root.to_str().expect("fixture path is UTF-8")),
            ("TERM_PROGRAM", DATUM_TERM_PROGRAM),
            ("TERM_PROGRAM_VERSION", env!("CARGO_PKG_VERSION")),
        ] {
            assert_eq!(
                env.iter()
                    .rev()
                    .find(|(candidate, _)| *candidate == std::ffi::OsStr::new(key))
                    .and_then(|(_, value)| *value),
                Some(std::ffi::OsStr::new(value)),
                "{key} must end with Datum's authoritative value"
            );
        }
    }

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
