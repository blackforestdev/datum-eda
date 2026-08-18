use super::*;
use std::{
    fs,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const SESSION_COUNT: usize = 8;
const TEST_DEADLINE: Duration = Duration::from_secs(12);

fn unique_test_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should follow the Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "datum-terminal-p06-isolation-{}-{nonce}",
        std::process::id()
    ))
}

fn projection_text(
    registry: &TerminalSessionRegistry,
    lane: &TerminalLaneState,
    index: usize,
) -> String {
    if index == registry.active_index && registry.active_pending_id.is_none() {
        lane.grid_lines().join("\n")
    } else {
        registry.sessions[index].parked_lane.grid_lines().join("\n")
    }
}

fn drain_until(
    registry: &mut TerminalSessionRegistry,
    lane: &mut TerminalLaneState,
    description: &str,
    ready: impl Fn(&TerminalSessionRegistry, &TerminalLaneState) -> bool,
) {
    let deadline = Instant::now() + TEST_DEADLINE;
    while Instant::now() < deadline {
        registry.drain_all(lane);
        if ready(registry, lane) {
            return;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    panic!("timed out waiting for {description}");
}

#[test]
fn eight_real_sessions_isolate_io_resize_exit_termination_and_restart() {
    let root = unique_test_root();
    fs::create_dir_all(&root).expect("create P06 isolation root");
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry = TerminalSessionRegistry::spawn(&context).expect("spawn shell 1");
    let mut lane = TerminalLaneState::default();

    for ordinal in 2..=SESSION_COUNT {
        registry
            .spawn_and_activate_with_lane(&context, &mut lane)
            .unwrap_or_else(|error| panic!("spawn shell {ordinal}: {error:#}"));
    }
    assert_eq!(registry.len(), SESSION_COUNT);
    let original_ids = registry
        .sessions
        .iter()
        .map(|slot| slot.session.session_id().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        original_ids
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        SESSION_COUNT
    );

    // Configure distinct kernel geometry and enqueue unique FIFO input for all
    // eight real PTYs before draining any of them. This makes every child live
    // concurrently while the registry's persistent round-robin drain owns
    // presentation into active and inactive projections.
    for (index, session_id) in original_ids.iter().enumerate() {
        registry
            .activate_with_lane(session_id, &mut lane)
            .expect("activate session for resize");
        let columns = 90 + index as u16;
        let rows = 20 + index as u16;
        registry
            .active()
            .write_bytes(
                format!(
                    "read -r first; read -r second; size=$(stty size); printf 'DTC06B-{index}:%s:%s:%s\\n' \"$first\" \"$second\" \"$size\"\n"
                )
                .as_bytes(),
            )
            .expect("enqueue read/print command");
        // The shell is now running a foreground read while Datum mutates this
        // PTY's geometry, so the resize occurs under live concurrent input/I/O
        // rather than as a pre-launch setup operation.
        registry
            .resize_active(columns, rows)
            .expect("resize isolated PTY");
        registry
            .active()
            .write_bytes(format!("input-{index}-a\ninput-{index}-b\n").as_bytes())
            .expect("enqueue exact session input");
    }

    drain_until(
        &mut registry,
        &mut lane,
        "all eight isolated markers",
        |registry, lane| {
            (0..SESSION_COUNT).all(|index| {
                projection_text(registry, lane, index).contains(&format!(
                    "DTC06B-{index}:input-{index}-a:input-{index}-b:{} {}",
                    20 + index,
                    90 + index
                ))
            })
        },
    );

    for index in 0..SESSION_COUNT {
        let text = projection_text(&registry, &lane, index);
        assert!(text.contains(&format!("DTC06B-{index}:")));
        for peer in 0..SESSION_COUNT {
            if peer != index {
                assert!(
                    !text.contains(&format!("DTC06B-{peer}:input-{peer}-a")),
                    "session {index} projection contains session {peer} output"
                );
            }
        }
        let slot = &registry.sessions[index];
        assert_eq!(
            (slot.columns, slot.rows),
            (90 + index as u16, 20 + index as u16)
        );
        assert!(slot.attached, "P06 must not recreate a detached PTY state");
    }

    // Every inactive projection must restore byte-identically when selected.
    for (index, session_id) in original_ids.iter().enumerate() {
        registry
            .activate_with_lane(session_id, &mut lane)
            .expect("reactivate isolated session");
        assert!(
            lane.grid_lines()
                .join("\n")
                .contains(&format!("DTC06B-{index}:"))
        );
    }

    // A natural exit remains exact and reviewable while its seven peers stay
    // owned. Restart is forbidden until the output/session closure barrier is
    // complete, then creates a fresh identity with explicit lineage.
    let exited_index = 2;
    let exited_id = original_ids[exited_index].clone();
    registry
        .activate_with_lane(&original_ids[SESSION_COUNT - 1], &mut lane)
        .expect("keep exiting session inactive");
    registry.sessions[exited_index]
        .session
        .write_bytes(b"exit 23\n")
        .expect("request exact natural exit");
    drain_until(
        &mut registry,
        &mut lane,
        "exact inactive exit and presentation barrier",
        |registry, _| {
            registry.sessions[exited_index].exact_exit_status.as_deref() == Some("exited 23")
                && registry.sessions[exited_index]
                    .session
                    .presentation_complete()
        },
    );
    registry
        .activate_with_lane(&exited_id, &mut lane)
        .expect("activate exited session for restart");
    registry
        .restart_active(&mut lane, &context)
        .expect("restart verified-closed session");
    let restarted_id = registry.active().session_id().to_string();
    assert_ne!(restarted_id, exited_id);
    assert_eq!(
        registry.sessions[registry.active_index]
            .previous_session_id
            .as_deref(),
        Some(exited_id.as_str())
    );
    assert_eq!(registry.sessions[registry.active_index].restart_count, 1);
    assert_eq!(
        (
            registry.sessions[registry.active_index].columns,
            registry.sessions[registry.active_index].rows,
        ),
        (92, 22)
    );

    // Explicitly terminate one session, then prove a different peer remains
    // writable. Teardown of one owned SID must not affect any other PTY.
    let terminated_id = original_ids[4].clone();
    registry
        .activate_with_lane(&terminated_id, &mut lane)
        .expect("activate termination target");
    registry
        .terminate_active(&mut lane)
        .expect("terminate one isolated session");
    drain_until(
        &mut registry,
        &mut lane,
        "isolated termination",
        |registry, _| {
            !registry
                .sessions
                .iter()
                .any(|slot| slot.session.session_id() == terminated_id)
        },
    );
    let peer_id = original_ids[6].clone();
    registry
        .activate_with_lane(&peer_id, &mut lane)
        .expect("activate surviving peer");
    registry
        .active()
        .write_bytes(b"printf 'DTC06B-peer-survived\\n'\n")
        .expect("write to surviving peer");
    drain_until(
        &mut registry,
        &mut lane,
        "surviving peer output",
        |_, lane| {
            lane.grid_lines()
                .join("\n")
                .contains("DTC06B-peer-survived")
        },
    );

    // End with the same verified no-orphan contract used by controlled app
    // shutdown. All original-SID processes and transport presentation barriers
    // must close before the fixture removes its context directory.
    registry.terminate_all_by(Instant::now() + Duration::from_secs(7));
    drain_until(
        &mut registry,
        &mut lane,
        "verified closure of all eight sessions",
        |registry, _| registry.all_sessions_closed(),
    );
    fs::remove_dir_all(root).expect("remove P06 isolation root");
}
