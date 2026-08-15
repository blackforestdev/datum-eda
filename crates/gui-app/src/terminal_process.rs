use self::terminal_transport::{PortablePtyProcess, TerminalTransportLaunch, spawn_portable_pty};
use crate::{
    terminal_context::{
        DATUM_CLI, DATUM_LEGACY_CLI, tool_session_event_log_path, write_terminal_context,
        write_terminal_context_files,
    },
    terminal_session::{TerminalEvent, TerminalLaunchContext, TerminalSession},
};
use anyhow::{Context, Result};
use std::{
    ffi::{OsStr, OsString},
    io::Read,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
};
use winit::event_loop::EventLoopProxy;

#[path = "terminal_transport.rs"]
pub(super) mod terminal_transport;

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
    spawn_terminal_process_argv(context, terminal_wake, OsStr::new(&shell), &[])
}

fn spawn_terminal_process_argv(
    context: &TerminalLaunchContext,
    terminal_wake: TerminalWakeGate,
    program: &OsStr,
    args: &[OsString],
) -> Result<TerminalSession> {
    let mut terminal_context = write_terminal_context(context)?;
    let mut launch = TerminalTransportLaunch::new(program, &terminal_context.project_root);
    launch.args(args);
    launch.env("TERM", "xterm-256color");
    launch.env("DATUM_PROJECT_ROOT", &context.project_root);
    launch.env("DATUM_CLI", DATUM_CLI);
    launch.env("DATUM_LEGACY_CLI", DATUM_LEGACY_CLI);
    launch.env("DATUM_CONTEXT_ID", &terminal_context.context_id);
    launch.env("DATUM_SESSION_ID", &terminal_context.session_id);
    launch.env("DATUM_DISCOVERY", &terminal_context.context_path);
    launch.env(
        "DATUM_TOOL_SESSION_EVENT_LOG",
        tool_session_event_log_path(&terminal_context.session_path),
    );
    launch.env(
        "DATUM_MODEL_REVISION",
        terminal_context.model_revision.as_deref().unwrap_or(""),
    );
    launch.env("DATUM_TERMINAL_CONTEXT", &terminal_context.context_path);
    launch.env("DATUM_TERMINAL_SESSION_ID", &terminal_context.session_id);
    if let Some(project_id) = &terminal_context.project_id {
        launch.env("DATUM_PROJECT_ID", project_id);
    }
    if let Some(model_revision) = &terminal_context.model_revision {
        launch.env("DATUM_SOURCE_REVISION", model_revision);
    }
    let PortablePtyProcess {
        master,
        mut reader,
        writer,
        mut child,
    } = spawn_portable_pty(&launch).with_context(|| {
        format!(
            "spawn portable PTY program {program:?} in {}",
            terminal_context.project_root.display()
        )
    })?;
    let process_group_id = master
        .process_group_leader()
        .or_else(|| child.process_id().map(|pid| pid as libc::pid_t))
        .context("portable PTY child has no process identifier")?;
    terminal_context.process_group_id = Some(process_group_id);
    write_terminal_context_files(&terminal_context, context)?;
    let stdin = Arc::new(Mutex::new(writer));
    let (tx, rx) = mpsc::channel();
    let reader_tx = tx.clone();
    let reader_wake = terminal_wake.clone();
    thread::spawn(move || {
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
        let code = child.wait().ok().map(|status| status.exit_code() as i32);
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
        master,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    #[test]
    fn portable_spawn_preserves_arbitrary_argv_cwd_and_datum_context() {
        let root =
            std::env::temp_dir().join(format!("datum-portable-pty-spawn-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create portable PTY test root");
        let context = TerminalLaunchContext::for_project_root(&root);
        let args = [
            OsString::from("-lc"),
            OsString::from(
                "printf 'argv-ok:%s\\n' \"$DATUM_CLI\"; printf 'cwd-ok:%s\\n' \"$PWD\"; printf 'context-ok:%s\\n' \"$DATUM_PROJECT_ROOT\"",
            ),
        ];
        let session = spawn_terminal_process_argv(
            &context,
            TerminalWakeGate::new(None),
            OsStr::new("/bin/sh"),
            &args,
        )
        .expect("spawn arbitrary portable PTY command");

        let deadline = Instant::now() + Duration::from_secs(8);
        let mut output = Vec::new();
        let mut exit = None;
        while Instant::now() < deadline {
            match session.rx.recv_timeout(Duration::from_millis(25)) {
                Ok(TerminalEvent::Output(bytes)) => output.extend(bytes),
                Ok(TerminalEvent::Exited(code)) => exit = Some(code),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(error) => panic!("portable PTY event channel failed: {error}"),
            }
            let text = String::from_utf8_lossy(&output);
            if exit.is_some()
                && text.contains("argv-ok:datum-eda")
                && text.contains(&format!("cwd-ok:{}", root.display()))
                && text.contains(&format!("context-ok:{}", root.display()))
            {
                break;
            }
        }
        let output = String::from_utf8_lossy(&output);
        assert_eq!(exit, Some(Some(0)));
        assert!(output.contains("argv-ok:datum-eda"), "{output}");
        assert!(
            output.contains(&format!("cwd-ok:{}", root.display())),
            "{output}"
        );
        assert!(
            output.contains(&format!("context-ok:{}", root.display())),
            "{output}"
        );
        let _ = fs::remove_dir_all(&root);
    }

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
