use crate::terminal_transport::{
    ShutdownPhase, TerminalExitStatus, TerminalTransportEvent, TerminalTransportRequest,
    TerminalWakeGate, prepare_terminal_transport,
};
use std::{
    fs,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

const P06_LIFECYCLE_CYCLES: usize = 100;
const RESOURCE_RETURN_DEADLINE: Duration = Duration::from_secs(1);
const SESSION_COMPLETION_DEADLINE: Duration = Duration::from_secs(3);
const RESOURCE_SETTLE_ALLOWANCE: usize = 2;
const HELPER_ENV: &str = "DATUM_P06_RESOURCE_HELPER";
const HELPER_TEST: &str =
    "terminal_session::terminal_session_p06_stress_tests::p06_resource_helper";

fn proc_entry_count(path: impl AsRef<Path>) -> usize {
    fs::read_dir(path)
        .expect("read Linux process resource directory")
        .count()
}

fn wait_for_resource_return(
    baseline_fds: usize,
    baseline_tasks: usize,
    cycle: usize,
) -> (usize, usize) {
    let deadline = Instant::now() + RESOURCE_RETURN_DEADLINE;
    loop {
        let fds = proc_entry_count("/proc/self/fd");
        let tasks = proc_entry_count("/proc/self/task");
        if fds <= baseline_fds + RESOURCE_SETTLE_ALLOWANCE
            && tasks <= baseline_tasks + RESOURCE_SETTLE_ALLOWANCE
        {
            return (fds, tasks);
        }
        assert!(
            Instant::now() < deadline,
            "cycle {cycle} retained resources: fds {fds} > {} or tasks {tasks} > {}",
            baseline_fds + RESOURCE_SETTLE_ALLOWANCE,
            baseline_tasks + RESOURCE_SETTLE_ALLOWANCE,
        );
        std::thread::yield_now();
    }
}

fn run_lifecycle_resource_proof() {
    let baseline_fds = proc_entry_count("/proc/self/fd");
    let baseline_tasks = proc_entry_count("/proc/self/task");
    let mut settled_samples = Vec::with_capacity(P06_LIFECYCLE_CYCLES);

    for cycle in 0..P06_LIFECYCLE_CYCLES {
        let request =
            TerminalTransportRequest::new("/bin/sh", "/tmp".into()).args(["-c", "exit 0"]);
        let session = prepare_terminal_transport(request)
            .unwrap_or_else(|error| panic!("cycle {cycle} failed to spawn: {error:#}"))
            .start(TerminalWakeGate::new(None));
        let deadline = Instant::now() + SESSION_COMPLETION_DEADLINE;
        let mut exit_status = None;

        while Instant::now() < deadline {
            match session.recv_event_timeout(Duration::from_millis(20)) {
                Ok(TerminalTransportEvent::Output(_)) => {}
                Ok(TerminalTransportEvent::Exited(status)) => exit_status = Some(status),
                Ok(TerminalTransportEvent::Error(error)) => {
                    panic!("cycle {cycle} transport error: {error:?}")
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    panic!("cycle {cycle} transport disconnected before completion")
                }
            }
            if exit_status.is_some() && session.presentation_complete() {
                break;
            }
        }

        assert_eq!(
            exit_status,
            Some(TerminalExitStatus::Code(0)),
            "cycle {cycle}"
        );
        assert!(
            session.presentation_complete(),
            "cycle {cycle} presentation"
        );
        let snapshot = session
            .shutdown_snapshot()
            .unwrap_or_else(|| panic!("cycle {cycle} lacks shutdown state"));
        assert_eq!(snapshot.phase, ShutdownPhase::Closed, "cycle {cycle}");
        assert!(
            snapshot.leader_reaped,
            "cycle {cycle} leader was not reaped"
        );
        assert!(
            snapshot.surviving_processes.is_empty(),
            "cycle {cycle} retained original-SID processes: {:?}",
            snapshot.surviving_processes
        );
        drop(session);

        settled_samples.push(wait_for_resource_return(
            baseline_fds,
            baseline_tasks,
            cycle,
        ));
    }

    assert_eq!(settled_samples.len(), P06_LIFECYCLE_CYCLES);
    assert!(
        settled_samples
            .iter()
            .all(|(fds, _)| *fds <= baseline_fds + RESOURCE_SETTLE_ALLOWANCE),
        "descriptor samples grew above the governed settled allowance: {settled_samples:?}"
    );
    assert!(
        settled_samples
            .iter()
            .all(|(_, tasks)| *tasks <= baseline_tasks + RESOURCE_SETTLE_ALLOWANCE),
        "worker samples grew above the governed settled allowance: {settled_samples:?}"
    );
}

#[test]
fn p06_resource_helper() {
    if std::env::var_os(HELPER_ENV).is_some() {
        run_lifecycle_resource_proof();
        return;
    }

    let status = Command::new(std::env::current_exe().expect("resolve current test binary"))
        .args(["--exact", HELPER_TEST, "--nocapture", "--test-threads=1"])
        .env(HELPER_ENV, "1")
        .status()
        .expect("launch isolated P06 resource helper");
    assert!(
        status.success(),
        "isolated P06 resource helper failed: {status}"
    );
}
