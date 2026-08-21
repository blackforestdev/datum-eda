use super::*;
use datum_gui_protocol::TerminalLaneState;
use std::time::{Duration, Instant};

fn write_retry(registry: &TerminalSessionRegistry, bytes: &[u8]) {
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        match registry.active().write_bytes(bytes) {
            Ok(()) => return,
            Err(error)
                if error.to_string().contains("queue is busy") && Instant::now() < deadline =>
            {
                std::thread::yield_now();
            }
            Err(error) => panic!("write terminal job-control bytes: {error}"),
        }
    }
}

fn drain_until(registry: &mut TerminalSessionRegistry, lane: &mut TerminalLaneState, needle: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        registry.drain_all(lane);
        if registry.test_active_text().contains(needle) {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!(
        "terminal output did not contain {needle:?}: {}",
        registry.test_active_text()
    );
}

#[test]
fn vintr_byte_interrupts_foreground_pipeline_and_shell_survives() {
    let root = std::env::temp_dir().join(format!("datum-terminal-vintr-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
    let mut lane = TerminalLaneState::default();

    write_retry(
        &registry,
        b"sleep 30 | python3 -c 'import os,sys,time; next(iter(lambda: os.tcgetpgrp(0) != os.getpgrp() and (time.sleep(.001) or True), False)); print(\"PIPELINE-START\", flush=True); sys.stdin.buffer.read()'\n",
    );
    drain_until(&mut registry, &mut lane, "PIPELINE-START");
    write_retry(&registry, &[0x03]);
    write_retry(&registry, b"printf 'SHELL%s\\n' '-SURVIVED'\n");
    drain_until(&mut registry, &mut lane, "SHELL-SURVIVED");
    let _ = registry.active().terminate();
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn vsusp_bg_fg_and_vintr_follow_native_shell_job_control() {
    let root = std::env::temp_dir().join(format!("datum-terminal-vsusp-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
    let mut lane = TerminalLaneState::default();

    write_retry(&registry, b"printf 'STOP%s\\n' '-START'; sleep 30\n");
    drain_until(&mut registry, &mut lane, "STOP-START");
    write_retry(&registry, &[0x1a]);
    write_retry(
        &registry,
        b"jobs -s; printf 'JOB%s\\n' '-STOPPED'; bg; printf 'BG%s\\n' '-OK'; fg\n",
    );
    drain_until(&mut registry, &mut lane, "BG-OK");
    assert!(registry.test_active_text().contains("JOB-STOPPED"));
    write_retry(&registry, &[0x03]);
    write_retry(&registry, b"printf 'FG%s\\n' '-INTERRUPTED'\n");
    drain_until(&mut registry, &mut lane, "FG-INTERRUPTED");
    let _ = registry.active().terminate();
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn tiocswinsz_reaches_the_foreground_shell_without_an_explicit_signal() {
    let root = std::env::temp_dir().join(format!("datum-terminal-winch-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
    let mut lane = TerminalLaneState::default();
    write_retry(
        &registry,
        b"python3 -c 'import os,signal,time; signal.signal(signal.SIGWINCH, lambda *_: os.write(1, f\"WINCH:{os.get_terminal_size().lines} {os.get_terminal_size().columns}\\n\".encode())); print(\"TRAP\"+\"-READY\", flush=True); time.sleep(30)'\n",
    );
    drain_until(&mut registry, &mut lane, "TRAP-READY");
    registry.resize_active(117, 33).unwrap();
    drain_until(&mut registry, &mut lane, "WINCH:33 117");
    let _ = registry.active().terminate();
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn stubborn_owned_session_escalates_to_kill_and_verifies_empty() {
    let root = std::env::temp_dir().join(format!("datum-terminal-escalate-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
    let mut lane = TerminalLaneState::default();
    write_retry(
        &registry,
        b"trap '' HUP TERM; printf 'STUBBORN%s\\n' '-READY'; while :; do sleep 1; done\n",
    );
    drain_until(&mut registry, &mut lane, "STUBBORN-READY");
    registry.active().terminate().unwrap();
    let deadline = Instant::now() + Duration::from_secs(7);
    let mut phases = Vec::new();
    while Instant::now() < deadline {
        registry.drain_all(&mut lane);
        if let Some(snapshot) = registry.active().shutdown_snapshot() {
            if phases.last() != Some(&snapshot.phase) {
                phases.push(snapshot.phase);
            }
            if snapshot.phase == crate::terminal_transport::ShutdownPhase::Closed {
                assert!(snapshot.surviving_processes.is_empty());
                assert!(snapshot.leader_reaped);
                assert!(
                    snapshot
                        .visited_phases
                        .contains(&crate::terminal_transport::ShutdownPhase::Hup)
                );
                assert!(
                    snapshot
                        .visited_phases
                        .contains(&crate::terminal_transport::ShutdownPhase::Term)
                );
                assert!(
                    snapshot
                        .visited_phases
                        .contains(&crate::terminal_transport::ShutdownPhase::Kill)
                );
                let _ = std::fs::remove_dir_all(root);
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("stubborn terminal session did not reach verified closed: {phases:?}");
}

#[test]
fn terminal_session_terminate_reports_exact_signal_exit() {
    let root =
        std::env::temp_dir().join(format!("datum-terminal-terminate-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let context = TerminalLaunchContext::for_project_root(&root);
    let session = spawn_terminal_session(&context).unwrap();
    session
        .write_bytes(b"printf 'datum%s\\n' '-terminate-ready'; trap - HUP; exec sleep 10\n")
        .unwrap();
    let mut readiness_output = Vec::new();
    for _ in 0..50 {
        if let Ok(TerminalEvent::Output(bytes)) =
            session.recv_event_timeout(Duration::from_millis(100))
        {
            readiness_output.extend_from_slice(&bytes);
            if String::from_utf8_lossy(&readiness_output).contains("datum-terminate-ready") {
                break;
            }
        }
    }
    assert!(String::from_utf8_lossy(&readiness_output).contains("datum-terminate-ready"));
    session.terminate().unwrap();
    let mut observed = None;
    for _ in 0..120 {
        if let Ok(TerminalEvent::Exited(status)) =
            session.recv_event_timeout(Duration::from_millis(100))
        {
            observed = Some(status);
            break;
        }
    }
    assert!(
        matches!(
            observed,
            Some(crate::terminal_transport::TerminalExitStatus::Signal {
                signal: libc::SIGHUP,
                ..
            })
        ),
        "unexpected terminal exit status: {observed:?}"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn new_session_descendant_is_not_signaled_and_cannot_hold_terminal_open() {
    let root = std::env::temp_dir().join(format!(
        "datum-terminal-setsid-boundary-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let pid_path = root.join("escaped.pid");
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
    let mut lane = TerminalLaneState::default();
    let script = format!(
        "exec python3 -c \"import os,time; p=os.fork(); (os.setsid(), open('{}','w').write(str(os.getpid())), time.sleep(30)) if p==0 else None\"\n",
        pid_path.display()
    );
    registry.active().write_bytes(script.as_bytes()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(4);
    while Instant::now() < deadline
        && (!registry.active().presentation_complete() || !pid_path.is_file())
    {
        registry.drain_all(&mut lane);
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(registry.active().presentation_complete());
    assert!(
        pid_path.is_file(),
        "escaped child must publish its PID before inspection"
    );
    let escaped_pid: i32 = std::fs::read_to_string(&pid_path)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert_eq!(unsafe { libc::kill(escaped_pid, 0) }, 0);
    unsafe {
        libc::kill(escaped_pid, libc::SIGKILL);
    }
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn escaped_session_holder_cannot_truncate_owned_final_output_tail() {
    let root =
        std::env::temp_dir().join(format!("datum-terminal-setsid-tail-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let pid_path = root.join("escaped.pid");
    let context = TerminalLaunchContext::for_project_root(&root);
    let session = spawn_terminal_session(&context).unwrap();
    let tail = "T".repeat(crate::terminal_transport::MAX_OUTPUT_CHUNK_BYTES + 257);
    let script = format!(
        "exec python3 -c \"import os,time; p=os.fork(); (os.setsid(), open('{}','w').write(str(os.getpid())), time.sleep(30)) if p==0 else os.write(1,b'BEGIN'+b'T'*{}+b'END')\"\n",
        pid_path.display(),
        tail.len()
    );
    session.write_bytes(script.as_bytes()).unwrap();

    let deadline = Instant::now() + Duration::from_secs(8);
    let mut output = Vec::new();
    let mut exited = false;
    while Instant::now() < deadline && !exited {
        match session.recv_event_timeout(Duration::from_millis(100)) {
            Ok(TerminalEvent::Output(bytes)) => output.extend_from_slice(&bytes),
            Ok(TerminalEvent::Exited(_)) => exited = true,
            Ok(TerminalEvent::Error(error)) => panic!("terminal transport error: {error:?}"),
            Err(_) => {}
        }
    }
    assert!(
        exited,
        "escaped slave holder prevented presentation completion"
    );
    let expected = format!("BEGIN{tail}END");
    assert!(
        String::from_utf8_lossy(&output).contains(&expected),
        "owned output tail was not preserved byte-for-byte"
    );
    let escaped_pid: i32 = std::fs::read_to_string(&pid_path)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert_eq!(unsafe { libc::kill(escaped_pid, 0) }, 0);
    unsafe {
        libc::kill(escaped_pid, libc::SIGKILL);
    }
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn termination_cancels_backpressured_input_and_closes_every_master() {
    let root = std::env::temp_dir().join(format!(
        "datum-terminal-writer-cancel-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let pid_path = root.join("escaped.pid");
    let context = TerminalLaunchContext::for_project_root(&root);
    let session = spawn_terminal_session(&context).unwrap();
    let script = format!(
        "exec python3 -c \"import os,signal,time; p=os.fork(); (os.setsid(), open('{}','w').write(str(os.getpid())), time.sleep(30)) if p==0 else (signal.signal(signal.SIGHUP,signal.SIG_IGN), signal.signal(signal.SIGTERM,signal.SIG_IGN), print('WRITER'+'-READY',flush=True), time.sleep(30))\"\n",
        pid_path.display()
    );
    session.write_bytes(script.as_bytes()).unwrap();
    let ready_deadline = Instant::now() + Duration::from_secs(4);
    let mut output = Vec::new();
    while Instant::now() < ready_deadline
        && !String::from_utf8_lossy(&output).contains("WRITER-READY")
    {
        if let Ok(TerminalEvent::Output(bytes)) =
            session.recv_event_timeout(Duration::from_millis(100))
        {
            output.extend_from_slice(&bytes);
        }
    }
    assert!(String::from_utf8_lossy(&output).contains("WRITER-READY"));
    while Instant::now() < ready_deadline && !pid_path.exists() {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        pid_path.exists(),
        "escaped child did not publish its identity"
    );
    session
        .write_bytes(&vec![b'x'; 4 * 1024 * 1024])
        .expect("owner-ratified maximum input request is admitted");
    session.terminate().unwrap();

    let deadline = Instant::now() + Duration::from_secs(8);
    let mut exited = false;
    while Instant::now() < deadline && !exited {
        if let Ok(TerminalEvent::Exited(_)) = session.recv_event_timeout(Duration::from_millis(100))
        {
            exited = true;
        }
    }
    assert!(exited, "canceled PTY writer held presentation open");
    assert!(session.presentation_complete());
    let escaped_pid: i32 = std::fs::read_to_string(&pid_path)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert_eq!(unsafe { libc::kill(escaped_pid, 0) }, 0);
    unsafe {
        libc::kill(escaped_pid, libc::SIGKILL);
    }
    let _ = std::fs::remove_dir_all(root);
}
