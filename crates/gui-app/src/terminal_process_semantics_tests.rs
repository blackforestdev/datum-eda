use super::*;
use std::{
    fs,
    sync::mpsc::RecvTimeoutError,
    time::{Duration, Instant},
};

fn spawn_script(root: &std::path::Path, script: &str) -> TerminalSession {
    let context = TerminalLaunchContext::for_project_root(root);
    let args = [OsString::from("-lc"), OsString::from(script)];
    spawn_terminal_process_argv(
        &context,
        TerminalWakeGate::new(None),
        OsStr::new("/bin/sh"),
        &args,
    )
    .expect("spawn portable PTY process-semantics script")
}

fn collect_until(
    session: &TerminalSession,
    timeout: Duration,
    output_marker: &str,
    require_exit: bool,
) -> (String, Option<i32>) {
    let deadline = Instant::now() + timeout;
    let mut output = Vec::new();
    let mut exit = None;
    while Instant::now() < deadline {
        match session.rx.recv_timeout(Duration::from_millis(25)) {
            Ok(TerminalEvent::Output(bytes)) => output.extend(bytes),
            Ok(TerminalEvent::Exited(code)) => exit = code,
            Err(RecvTimeoutError::Timeout) => {}
            Err(error) => panic!("portable PTY event channel failed: {error}"),
        }
        let text = String::from_utf8_lossy(&output);
        if text.contains(output_marker) && (!require_exit || exit.is_some()) {
            break;
        }
    }
    (String::from_utf8_lossy(&output).into_owned(), exit)
}

fn test_root(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "datum-{name}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create process-semantics test root");
    root
}

#[test]
fn pipeline_process_group_and_nonzero_exit_are_reported() {
    let root = test_root("portable-pty-pipeline");
    let session = spawn_script(
        &root,
        "printf 'alpha\\nbeta\\n' | grep beta | tr '[:lower:]' '[:upper:]'; \
         printf 'pgid:%s\\n' \"$(ps -o pgid= -p $$ | tr -d ' ')\"; exit 23",
    );
    let expected_pgid = format!("pgid:{}", session.process_group_id);
    let (output, exit) = collect_until(&session, Duration::from_secs(8), &expected_pgid, true);
    assert!(output.contains("BETA"), "pipeline output missing: {output}");
    assert!(
        output.contains(&expected_pgid),
        "process group mismatch: {output}"
    );
    assert_eq!(exit, Some(23), "nonzero child exit must remain exact");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn ctrl_c_reaches_the_foreground_pipeline_through_the_pty() {
    let root = test_root("portable-pty-interrupt");
    let session = spawn_script(
        &root,
        r#"trap "printf 'interrupt-ok\n'; exit 130" INT;
            printf 'interrupt-ready\n'; while :; do sleep 1; done"#,
    );
    let (ready_output, _) =
        collect_until(&session, Duration::from_secs(5), "interrupt-ready", false);
    assert!(ready_output.contains("interrupt-ready"), "{ready_output}");
    session.interrupt().expect("write terminal Ctrl-C");
    let (output, exit) = collect_until(&session, Duration::from_secs(8), "interrupt-ok", true);
    assert!(
        output.contains("interrupt-ok"),
        "foreground SIGINT missing: {output}"
    );
    assert_eq!(exit, Some(130));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn explicit_termination_stops_the_process_group_and_reports_exit() {
    let root = test_root("portable-pty-terminate");
    let session = spawn_script(
        &root,
        r#"trap "printf 'terminate-ok\n'; exit 42" TERM;
            printf 'terminate-ready\n'; while :; do sleep 1; done"#,
    );
    let (ready_output, _) =
        collect_until(&session, Duration::from_secs(5), "terminate-ready", false);
    assert!(ready_output.contains("terminate-ready"), "{ready_output}");
    session
        .terminate()
        .expect("terminate terminal process group");
    let (output, exit) = collect_until(&session, Duration::from_secs(8), "terminate-ok", true);
    assert!(
        output.contains("terminate-ok"),
        "group termination missing: {output}"
    );
    assert_eq!(exit, Some(42));
    let _ = fs::remove_dir_all(&root);
}
