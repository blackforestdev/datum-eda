use super::*;
use std::{
    fs,
    sync::mpsc::RecvTimeoutError,
    time::{Duration, Instant},
};

fn test_root() -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "datum-portable-pty-isolation-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create PTY isolation test root");
    root
}

fn collect_slot_until(
    registry: &TerminalSessionRegistry,
    index: usize,
    marker: &str,
    minimum_marker_count: usize,
    require_exit: bool,
) -> (String, Option<i32>) {
    let deadline = Instant::now() + Duration::from_secs(10);
    let session = &registry.sessions[index].session;
    let mut output = Vec::new();
    let mut exit = None;
    while Instant::now() < deadline {
        match session.rx.recv_timeout(Duration::from_millis(25)) {
            Ok(TerminalEvent::Output(bytes)) => output.extend(bytes),
            Ok(TerminalEvent::Exited(code)) => exit = code,
            Err(RecvTimeoutError::Timeout) => {}
            Err(error) => panic!("terminal session event channel failed: {error}"),
        }
        let text = String::from_utf8_lossy(&output);
        if text.matches(marker).count() >= minimum_marker_count && (!require_exit || exit.is_some())
        {
            break;
        }
    }
    (String::from_utf8_lossy(&output).into_owned(), exit)
}

fn write_active(registry: &TerminalSessionRegistry, command: &str) {
    registry
        .active()
        .write_bytes(command.as_bytes())
        .expect("write isolated terminal command");
}

#[test]
fn concurrent_sessions_isolate_io_resize_attachment_exit_and_teardown() {
    let root = test_root();
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry = TerminalSessionRegistry::spawn(&context).expect("spawn first PTY session");
    let first_id = registry.active().session_id().to_string();

    registry.resize_active(91, 27).expect("resize first PTY");
    write_active(
        &registry,
        "printf 'FIRST-%s\\n' INPUT; stty size; printf 'FIRST-%s\\n' SIZE-END; \
         sleep 1; printf 'FIRST-WHILE-%s\\n' DETACHED\n",
    );

    let second_id = registry
        .spawn_and_activate(&context)
        .expect("spawn second PTY session")
        .to_string();
    registry.resize_active(73, 19).expect("resize second PTY");
    write_active(
        &registry,
        "printf 'SECOND-%s\\n' INPUT; stty size; printf 'SECOND-%s\\n' SIZE-END\n",
    );

    let (second_output, second_exit) =
        collect_slot_until(&registry, 1, "SECOND-SIZE-END", 1, false);
    assert_eq!(second_exit, None);
    assert!(second_output.contains("SECOND-INPUT"), "{second_output}");
    assert!(second_output.contains("19 73"), "{second_output}");
    assert!(!second_output.contains("FIRST-INPUT"), "{second_output}");

    let mut lane = TerminalLaneState::default();
    registry
        .detach_active(&mut lane)
        .expect("detach second PTY session");
    assert!(!registry.active_attached());
    let (first_output, first_exit) =
        collect_slot_until(&registry, 0, "FIRST-WHILE-DETACHED", 1, false);
    assert_eq!(first_exit, None);
    assert!(first_output.contains("FIRST-INPUT"), "{first_output}");
    assert!(first_output.contains("27 91"), "{first_output}");
    assert!(!first_output.contains("SECOND-INPUT"), "{first_output}");

    registry.activate(&second_id).expect("reattach second PTY");
    registry.sync_lane_tabs(&mut lane);
    assert!(registry.active_attached());
    assert_eq!((lane.columns, lane.rows), (73, 19));
    assert_eq!(
        (registry.sessions[0].columns, registry.sessions[0].rows),
        (91, 27)
    );
    write_active(&registry, "printf 'SECOND-AFTER-%s\\n' REATTACH\n");
    let (reattached_output, _) =
        collect_slot_until(&registry, 1, "SECOND-AFTER-REATTACH", 1, false);
    assert!(reattached_output.contains("SECOND-AFTER-REATTACH"));

    registry.activate(&first_id).expect("activate first PTY");
    write_active(&registry, "printf 'FIRST-%s\\n' EXITING; exit 17\n");
    let (first_exit_output, first_exit) =
        collect_slot_until(&registry, 0, "FIRST-EXITING", 1, true);
    assert!(
        first_exit_output.contains("FIRST-EXITING"),
        "{first_exit_output}"
    );
    assert_eq!(first_exit, Some(17));
    assert!(registry.sessions[0].session.exited.load(Ordering::Acquire));

    registry
        .activate(&second_id)
        .expect("activate surviving PTY");
    write_active(&registry, "printf 'SECOND-AFTER-FIRST-%s\\n' EXIT\n");
    let (survivor_output, survivor_exit) =
        collect_slot_until(&registry, 1, "SECOND-AFTER-FIRST-EXIT", 1, false);
    assert_eq!(survivor_exit, None);
    assert!(survivor_output.contains("SECOND-AFTER-FIRST-EXIT"));

    registry.activate(&first_id).expect("select exited PTY tab");
    registry
        .close_active(&mut lane)
        .expect("close naturally exited PTY without signalling a stale process group");
    assert_eq!(registry.len(), 1);
    assert_eq!(registry.active().session_id(), second_id);
    assert!(registry.active_attached());

    write_active(
        &registry,
        r#"trap "printf 'SECOND-%s\n' TEARDOWN; exit 44" TERM;
            printf 'SECOND-TERM-%s\n' READY; while :; do sleep 1; done
"#,
    );
    let (ready_output, _) = collect_slot_until(&registry, 0, "SECOND-TERM-READY", 1, false);
    assert!(ready_output.contains("SECOND-TERM-READY"), "{ready_output}");
    registry
        .terminate_active(&mut lane)
        .expect("terminate surviving PTY process group");
    let (teardown_output, teardown_exit) =
        collect_slot_until(&registry, 0, "SECOND-TEARDOWN", 1, true);
    assert!(
        teardown_output.contains("SECOND-TEARDOWN"),
        "{teardown_output}"
    );
    assert_eq!(teardown_exit, Some(44));

    drop(registry);
    let _ = fs::remove_dir_all(&root);
}
