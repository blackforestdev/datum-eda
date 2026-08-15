use crate::{
    terminal_context::{
        DATUM_CLI, DATUM_LEGACY_CLI, tool_session_event_log_path, write_terminal_context,
        write_terminal_context_files,
    },
    terminal_session::{TerminalEvent, TerminalLaunchContext, TerminalSession},
};
use anyhow::{Context, Result};
use std::{
    ffi::CStr,
    fs::File,
    io::{self, Read},
    os::{
        fd::{FromRawFd, RawFd},
        unix::process::CommandExt,
    },
    process::Command,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
};
use winit::event_loop::EventLoopProxy;

#[derive(Clone)]
pub(super) struct TerminalWakeGate {
    proxy: Option<EventLoopProxy<()>>,
    pending: Arc<AtomicBool>,
}

impl TerminalWakeGate {
    pub(super) fn new(proxy: Option<EventLoopProxy<()>>) -> Self {
        Self {
            proxy,
            pending: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(super) fn request(&self) {
        request_coalesced_wake(&self.pending, || {
            self.proxy
                .as_ref()
                .is_some_and(|proxy| proxy.send_event(()).is_ok())
        });
    }

    /// Clear before draining. Output arriving concurrently can then schedule
    /// exactly one successor event, avoiding both a lost wake and a queue full
    /// of redundant events ahead of keyboard input.
    pub(super) fn acknowledge(&self) {
        self.pending.store(false, Ordering::Release);
    }
}

pub(super) fn spawn_terminal_process(
    context: &TerminalLaunchContext,
    terminal_wake: TerminalWakeGate,
) -> Result<TerminalSession> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let mut terminal_context = write_terminal_context(context)?;
    let pty = open_pty_pair().context("open terminal PTY")?;
    let reader = pty
        .master
        .try_clone()
        .context("clone terminal PTY master for reader")?;
    let stdin = Arc::new(Mutex::new(pty.master));
    let slave_path = pty.slave_path;
    let master_fd = pty.master_fd;
    let mut command = Command::new(&shell);
    command
        .current_dir(&terminal_context.project_root)
        .env("TERM", "xterm-256color")
        .env("DATUM_PROJECT_ROOT", &context.project_root)
        .env("DATUM_CLI", DATUM_CLI)
        .env("DATUM_LEGACY_CLI", DATUM_LEGACY_CLI)
        .env("DATUM_CONTEXT_ID", &terminal_context.context_id)
        .env("DATUM_SESSION_ID", &terminal_context.session_id)
        .env("DATUM_DISCOVERY", &terminal_context.context_path)
        .env(
            "DATUM_TOOL_SESSION_EVENT_LOG",
            tool_session_event_log_path(&terminal_context.session_path),
        )
        .env(
            "DATUM_MODEL_REVISION",
            terminal_context.model_revision.as_deref().unwrap_or(""),
        )
        .env("DATUM_TERMINAL_CONTEXT", &terminal_context.context_path)
        .env("DATUM_TERMINAL_SESSION_ID", &terminal_context.session_id);
    if let Some(project_id) = &terminal_context.project_id {
        command.env("DATUM_PROJECT_ID", project_id);
    }
    if let Some(model_revision) = &terminal_context.model_revision {
        command.env("DATUM_SOURCE_REVISION", model_revision);
    }
    unsafe {
        command.pre_exec(move || configure_child_pty(&slave_path, master_fd));
    }
    let mut child = command.spawn().with_context(|| {
        format!(
            "spawn PTY terminal shell {shell} in {}",
            terminal_context.project_root.display()
        )
    })?;
    let process_group_id = child.id() as libc::pid_t;
    terminal_context.process_group_id = Some(process_group_id as i32);
    write_terminal_context_files(&terminal_context, context)?;
    let (tx, rx) = mpsc::channel();
    let reader_tx = tx.clone();
    let reader_wake = terminal_wake.clone();
    thread::spawn(move || {
        let mut reader = reader;
        let mut buffer = [0_u8; 4096];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    publish_terminal_event(
                        &reader_tx,
                        TerminalEvent::Output(buffer[..count].to_vec()),
                        || reader_wake.request(),
                    );
                }
                Err(_) => break,
            }
        }
    });
    thread::spawn(move || {
        let code = child.wait().ok().and_then(|status| status.code());
        publish_terminal_event(&tx, TerminalEvent::Exited(code), || terminal_wake.request());
    });
    Ok(TerminalSession {
        stdin,
        rx,
        context_path: terminal_context.context_path,
        latest_context_path: terminal_context.latest_context_path,
        session_path: terminal_context.session_path,
        session_id: terminal_context.session_id,
        context_id: terminal_context.context_id,
        master_fd,
        process_group_id,
        active_execution_id: Arc::new(Mutex::new(None)),
        finished_scan_offset: std::cell::Cell::new(0),
    })
}

fn request_coalesced_wake(pending: &AtomicBool, wake: impl FnOnce() -> bool) {
    if pending
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
        && !wake()
    {
        pending.store(false, Ordering::Release);
    }
}

/// Publish first, then wake the waiting GUI. Keeping these actions in one
/// helper makes it impossible for a new PTY event variant to be queued without
/// the corresponding event-loop wake and gives the T0-C03A contract a
/// deterministic unit boundary independent of a display server.
fn publish_terminal_event(
    tx: &mpsc::Sender<TerminalEvent>,
    event: TerminalEvent,
    wake: impl FnOnce(),
) {
    if tx.send(event).is_ok() {
        wake();
    }
}

struct PtyPair {
    master: File,
    master_fd: RawFd,
    slave_path: Vec<u8>,
}

fn open_pty_pair() -> Result<PtyPair> {
    let master_fd = unsafe { libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY) };
    if master_fd < 0 {
        return Err(io::Error::last_os_error()).context("posix_openpt");
    }
    if unsafe { libc::grantpt(master_fd) } != 0 {
        let error = io::Error::last_os_error();
        unsafe { libc::close(master_fd) };
        return Err(error).context("grantpt");
    }
    if unsafe { libc::unlockpt(master_fd) } != 0 {
        let error = io::Error::last_os_error();
        unsafe { libc::close(master_fd) };
        return Err(error).context("unlockpt");
    }
    let slave_path = slave_path(master_fd)?;
    let master = unsafe { File::from_raw_fd(master_fd) };
    Ok(PtyPair {
        master,
        master_fd,
        slave_path,
    })
}

fn slave_path(master_fd: RawFd) -> Result<Vec<u8>> {
    let mut buffer = [0 as libc::c_char; 128];
    let rc = unsafe { libc::ptsname_r(master_fd, buffer.as_mut_ptr(), buffer.len()) };
    if rc != 0 {
        return Err(io::Error::from_raw_os_error(rc)).context("ptsname_r");
    }
    let path = unsafe { CStr::from_ptr(buffer.as_ptr()) };
    Ok(path.to_bytes_with_nul().to_vec())
}

fn configure_child_pty(slave_path: &[u8], master_fd: RawFd) -> io::Result<()> {
    if unsafe { libc::setsid() } < 0 {
        return Err(io::Error::last_os_error());
    }
    let slave_fd = unsafe { libc::open(slave_path.as_ptr().cast(), libc::O_RDWR) };
    if slave_fd < 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::ioctl(slave_fd, libc::TIOCSCTTY, 0) } < 0 {
        let error = io::Error::last_os_error();
        unsafe { libc::close(slave_fd) };
        return Err(error);
    }
    for fd in [libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO] {
        if unsafe { libc::dup2(slave_fd, fd) } < 0 {
            let error = io::Error::last_os_error();
            unsafe { libc::close(slave_fd) };
            return Err(error);
        }
    }
    if slave_fd > libc::STDERR_FILENO {
        unsafe { libc::close(slave_fd) };
    }
    unsafe { libc::close(master_fd) };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn every_published_pty_event_wakes_the_consumer() {
        let (tx, rx) = mpsc::channel();
        let wakes = AtomicUsize::new(0);
        publish_terminal_event(&tx, TerminalEvent::Output(vec![1, 2, 3]), || {
            wakes.fetch_add(1, Ordering::SeqCst);
        });
        publish_terminal_event(&tx, TerminalEvent::Exited(Some(0)), || {
            wakes.fetch_add(1, Ordering::SeqCst);
        });

        assert!(matches!(rx.recv(), Ok(TerminalEvent::Output(bytes)) if bytes == [1, 2, 3]));
        assert!(matches!(rx.recv(), Ok(TerminalEvent::Exited(Some(0)))));
        assert_eq!(wakes.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn disconnected_consumer_does_not_schedule_a_spurious_wake() {
        let (tx, rx) = mpsc::channel();
        drop(rx);
        let wakes = AtomicUsize::new(0);
        publish_terminal_event(&tx, TerminalEvent::Exited(None), || {
            wakes.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(wakes.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn burst_output_coalesces_to_one_pending_gui_wake() {
        let pending = AtomicBool::new(false);
        let wakes = AtomicUsize::new(0);
        for _ in 0..10_000 {
            request_coalesced_wake(&pending, || {
                wakes.fetch_add(1, Ordering::SeqCst);
                true
            });
        }
        assert_eq!(wakes.load(Ordering::SeqCst), 1);

        pending.store(false, Ordering::Release);
        request_coalesced_wake(&pending, || {
            wakes.fetch_add(1, Ordering::SeqCst);
            true
        });
        assert_eq!(wakes.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn failed_gui_wake_releases_gate_for_retry() {
        let pending = AtomicBool::new(false);
        request_coalesced_wake(&pending, || false);
        assert!(!pending.load(Ordering::Acquire));

        let wakes = AtomicUsize::new(0);
        request_coalesced_wake(&pending, || {
            wakes.fetch_add(1, Ordering::SeqCst);
            true
        });
        assert_eq!(wakes.load(Ordering::SeqCst), 1);
    }
}
