//! Datum-owned terminal process transport.
//!
//! This module owns PTY allocation, child attachment, byte transport, process
//! identity, resize, and teardown. It deliberately knows nothing about cells,
//! escape parsing, rendering, selection, or Datum design state.

mod control;
mod event;
mod input;
mod launch_error;
mod limits;
mod linux;
mod output;
mod process_status;
mod process_supervisor;
mod reader;
mod request;
mod session_handle;
mod shutdown;
mod wake;

use anyhow::{Context, Result};
pub(super) use event::{TerminalInputError, TerminalTransportEvent};
use launch_error::{TerminalLaunchError, TerminalLaunchStage};
#[cfg(test)]
pub(super) use limits::{MAX_OUTPUT_CHUNK_BYTES, MAX_OUTPUT_CHUNKS};
pub(super) use limits::{
    GLOBAL_SHUTDOWN_MS, GUI_DRAIN_BYTE_LIMIT, GUI_DRAIN_EVENT_LIMIT, MAX_LIVE_SESSIONS,
};
pub(super) use process_status::TerminalExitStatus;
pub(super) use process_supervisor::{ShutdownProcessIdentity, ShutdownSnapshot};
pub(super) use request::TerminalTransportRequest;
pub(super) use session_handle::{PreparedTerminalTransport, TerminalTransportSession};
pub(super) use shutdown::ShutdownPhase;
pub(super) use wake::TerminalWakeGate;

pub(super) fn prepare_terminal_transport(
    request: TerminalTransportRequest,
) -> Result<PreparedTerminalTransport> {
    let spawn_failure_context = request.spawn_failure_context();
    request
        .validate_cwd()
        .map_err(|error| TerminalLaunchError::new(TerminalLaunchStage::WorkingDirectory, error))?;
    let (mut command, columns, rows) = request.into_command();
    let pty = linux::pty::open_pty_pair()?;
    use std::os::fd::AsRawFd;
    linux::termios::configure_interactive(pty.slave.as_raw_fd())
        .map_err(|error| TerminalLaunchError::new(TerminalLaunchStage::ConfigureTermios, error))?;
    linux::job_control::resize_fd(pty.slave.as_raw_fd(), columns, rows)
        .map_err(|error| TerminalLaunchError::new(TerminalLaunchStage::InitialSize, error))?;
    let reader = pty
        .master
        .try_clone()
        .map_err(|error| TerminalLaunchError::new(TerminalLaunchStage::CloneReader, error))?;
    let control = pty
        .master
        .try_clone()
        .map_err(|error| TerminalLaunchError::new(TerminalLaunchStage::CloneControl, error))?;
    linux::spawn::attach_child_pty(&mut command, pty.slave.as_raw_fd(), pty.master_fd);
    let child = command
        .spawn()
        .map_err(|error| TerminalLaunchError::new(TerminalLaunchStage::Spawn, error))
        .with_context(|| spawn_failure_context)?;
    let process_group_id = child.id() as libc::pid_t;
    Ok(PreparedTerminalTransport::new(
        pty.master,
        reader,
        control,
        child,
        process_group_id,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn run_request(request: TerminalTransportRequest) -> (Vec<u8>, Option<TerminalExitStatus>) {
        let session = prepare_terminal_transport(request)
            .unwrap()
            .start(TerminalWakeGate::new(None));
        let mut output = Vec::new();
        let mut exit_code = None;
        loop {
            let timeout = if exit_code.is_some() {
                std::time::Duration::from_millis(50)
            } else {
                std::time::Duration::from_secs(2)
            };
            let Ok(event) = session.recv_event_timeout(timeout) else {
                return (output, exit_code);
            };
            match event {
                TerminalTransportEvent::Output(bytes) => output.extend(bytes),
                TerminalTransportEvent::Exited(status) => exit_code = Some(status),
                TerminalTransportEvent::Error(error) => panic!("transport error: {error:?}"),
            }
        }
    }

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
        let launch = error
            .chain()
            .find_map(|cause| cause.downcast_ref::<TerminalLaunchError>())
            .unwrap();
        assert_eq!(launch.stage, TerminalLaunchStage::Spawn);
    }

    #[test]
    fn invalid_cwd_has_a_typed_launch_stage() {
        let request = TerminalTransportRequest::new(
            "/bin/sh",
            PathBuf::from("/datum-test/nonexistent-directory"),
        );
        let error = match prepare_terminal_transport(request) {
            Ok(_) => panic!("invalid cwd must fail"),
            Err(error) => error,
        };
        let launch = error
            .chain()
            .find_map(|cause| cause.downcast_ref::<TerminalLaunchError>())
            .unwrap();
        assert_eq!(launch.stage, TerminalLaunchStage::WorkingDirectory);
        assert_eq!(launch.kind, std::io::ErrorKind::NotFound);
        assert!(launch.os_code.is_some());
    }

    #[test]
    fn explicit_argv_is_not_interpreted_by_an_implicit_shell() {
        let marker =
            std::env::temp_dir().join(format!("datum-no-shell-injection-{}", std::process::id()));
        let literal = format!("$(touch {})", marker.display());
        let request = TerminalTransportRequest::new("/usr/bin/printf", PathBuf::from("/tmp"))
            .args(["%s", literal.as_str()]);
        let (output, code) = run_request(request);
        assert_eq!(code, Some(TerminalExitStatus::Code(0)));
        assert_eq!(String::from_utf8_lossy(&output), literal);
        assert!(!marker.exists());
    }

    #[test]
    fn child_starts_with_requested_geometry_and_no_master_fd() {
        let script = "printf 'SIZE:'; stty size; printf 'TTY:'; readlink /proc/self/fd/0; \
                      printf 'FDS:'; for f in /proc/self/fd/*; do readlink \"$f\" || true; done";
        let request = TerminalTransportRequest::new("/bin/sh", PathBuf::from("/tmp"))
            .args(["-c", script])
            .initial_size(111, 37);
        let (output, code) = run_request(request);
        let output = String::from_utf8_lossy(&output);
        assert_eq!(code, Some(TerminalExitStatus::Code(0)));
        assert!(output.contains("SIZE:37 111"), "{output}");
        assert!(output.contains("TTY:/dev/pts/"), "{output}");
        assert!(!output.contains("/dev/ptmx"), "{output}");
        assert_eq!(output.matches("/dev/pts/").count(), 4, "{output}");
    }

    #[test]
    fn child_inherits_process_credentials() {
        let request = TerminalTransportRequest::new("/bin/sh", PathBuf::from("/tmp")).args([
            "-c",
            "printf '%s:%s:%s:%s' \"$(id -ru)\" \"$(id -u)\" \"$(id -rg)\" \"$(id -g)\"",
        ]);
        let (output, code) = run_request(request);
        assert_eq!(code, Some(TerminalExitStatus::Code(0)));
        let expected = format!(
            "{}:{}:{}:{}",
            unsafe { libc::getuid() },
            unsafe { libc::geteuid() },
            unsafe { libc::getgid() },
            unsafe { libc::getegid() },
        );
        assert_eq!(String::from_utf8_lossy(&output), expected);
    }

    #[test]
    fn child_observes_environment_overlay_removal_groups_and_umask() {
        use std::os::unix::ffi::OsStringExt;

        let opaque = std::ffi::OsString::from_vec(vec![b'x', 0xff, b'y']);
        let request = TerminalTransportRequest::new("/bin/sh", PathBuf::from("/tmp"))
            .args([
                "-c",
                "printf 'PATH_PRESENT:%s\\n' \"${PATH:+yes}\"; id -G; grep '^Umask:' /proc/self/status; /usr/bin/env",
            ])
            .env("DATUM_OPAQUE_TEST", &opaque)
            .env("DATUM_REMOVE_TEST", "must-not-survive")
            .env_remove("DATUM_REMOVE_TEST");
        let (output, code) = run_request(request);
        assert_eq!(code, Some(TerminalExitStatus::Code(0)));
        assert!(output.windows(16).any(|part| part == b"PATH_PRESENT:yes"));
        assert!(
            output
                .windows(b"DATUM_OPAQUE_TEST=x\xffy".len())
                .any(|part| part == b"DATUM_OPAQUE_TEST=x\xffy")
        );
        assert!(!output.windows(17).any(|part| part == b"DATUM_REMOVE_TEST"));

        let text = String::from_utf8_lossy(&output).replace('\r', "");
        let expected_groups = String::from_utf8_lossy(
            &std::process::Command::new("id")
                .arg("-G")
                .output()
                .unwrap()
                .stdout,
        )
        .trim()
        .to_string();
        assert!(text.lines().any(|line| line == expected_groups));
        let parent_status = std::fs::read_to_string("/proc/self/status").unwrap();
        let parent_umask = parent_status
            .lines()
            .find(|line| line.starts_with("Umask:"))
            .unwrap();
        assert!(text.lines().any(|line| line == parent_umask));
    }
}
