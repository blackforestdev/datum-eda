use super::*;
use crate::terminal_session::PendingTerminalPlacement;
use crate::{
    terminal_activity_snapshot::TerminalActivitySummaryCache,
    terminal_core_adapter::TerminalCoreSessionAdapter,
    terminal_session::{
        PendingTerminalSpawn, TerminalLaunchContext, TerminalSession, TerminalSessionSlot,
    },
    terminal_transport::{TerminalExitStatus, TerminalTransportSession, TerminalWakeGate},
};
use std::time::{Duration, Instant};
use std::{
    cell::Cell,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
};

static SYNTHETIC_REGISTRY_ID: AtomicU64 = AtomicU64::new(1);

#[test]
fn seventeenth_session_is_refused_by_preallocation_guard() {
    assert!(super::super::ensure_session_capacity(15).is_ok());
    assert!(super::super::ensure_session_capacity(16).is_err());
}

#[test]
fn active_pending_tab_keeps_previous_session_output_in_its_parked_projection() {
    let mut registry = synthetic_registry(1);
    let (_sender, result) = mpsc::channel();
    registry.pending_spawns.push(PendingTerminalSpawn {
        pending_id: "pending-shell-2".to_string(),
        label: "shell 2".to_string(),
        result,
        canceled: false,
        placement: PendingTerminalPlacement::NewTab,
    });
    registry.active_pending_id = Some("pending-shell-2".to_string());
    registry.sessions[0]
        .session
        .transport
        .push_synthetic_output(b"old-shell-output");

    let fixed_now = Instant::now();
    let mut lane = TerminalLaneState::default();
    let report = registry.drain_with_clock(&mut lane, || fixed_now);
    assert!(!report.active_projection_changed);
    assert_eq!(registry.test_active_text().trim_end(), "old-shell-output");
}

#[test]
fn one_gui_turn_never_exceeds_owner_ratified_output_limits() {
    let root =
        std::env::temp_dir().join(format!("datum-terminal-drain-limit-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
    let fixed_now = Instant::now();
    let mut lane = TerminalLaneState::default();
    registry
        .active()
        .write_bytes(b"head -c 200000 /dev/zero | tr '\\0' x\n")
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while !registry.active().has_pending_event() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    let report = registry.drain_with_clock(&mut lane, || fixed_now);
    assert!(report.output_events <= GUI_DRAIN_EVENT_LIMIT);
    assert!(report.output_bytes <= GUI_DRAIN_BYTE_LIMIT);
    let _ = std::fs::remove_dir_all(root);
}

fn synthetic_registry(session_count: usize) -> TerminalSessionRegistry {
    let wake = TerminalWakeGate::new(None);
    let root = std::env::temp_dir().join(format!(
        "datum-terminal-synthetic-drain-{}-{}",
        std::process::id(),
        SYNTHETIC_REGISTRY_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let sessions = (0..session_count)
        .map(|index| {
            let id = format!("synthetic-{index}");
            TerminalSessionSlot {
                session: TerminalSession {
                    transport: TerminalTransportSession::synthetic(wake.clone()),
                    context_path: root.join(format!("{id}-context.json")),
                    latest_context_path: root.join("latest.json"),
                    session_path: root.join(format!("{id}-session.json")),
                    session_id: id.clone(),
                    context_id: format!("context-{index}"),
                    terminal_profile: crate::terminal_profile::TerminalLaunchProfile::default(),
                    active_execution_id: Arc::new(Mutex::new(None)),
                    finished_scan_offset: Cell::new(0),
                },
                core: TerminalCoreSessionAdapter::new(
                    id.clone(),
                    format!("context-{index}"),
                    80,
                    24,
                )
                .unwrap(),
                label: id,
                label_is_explicit: false,
                status: "running".to_string(),
                attached: index == 0,
                previous_session_id: None,
                restart_count: 0,
                columns: 80,
                rows: 24,
                activity: TerminalActivitySummaryCache::default(),
                pending_drain_output: Vec::new(),
                parked_lane: TerminalLaneState::default(),
                disconnected_reported: false,
                termination_failure_reported: false,
                close_confirmation_armed: false,
                pending_restart: false,
                remove_when_closed: false,
                hidden_after_close: false,
                exact_exit_status: None,
                unread_output: false,
                seen_bell_count: 0,
            }
        })
        .collect();
    TerminalSessionRegistry {
        sessions,
        terminal_tabs: (0..session_count)
            .map(|index| {
                datum_gui_protocol::TerminalTabLayout::single(format!("synthetic-{index}"))
            })
            .collect(),
        pending_spawns: Vec::new(),
        active_pending_id: None,
        active_index: 0,
        next_session_ordinal: session_count + 1,
        terminal_wake: wake,
        next_drain_index: 0,
        next_apply_index: 0,
        projection_managed: true,
    }
}

#[test]
fn control_priority_round_robin_cursor_and_exact_global_caps_are_literal() {
    let mut registry = synthetic_registry(3);
    registry.sessions[1]
        .session
        .transport
        .push_synthetic_error();
    for round in 0..43 {
        for index in 0..3 {
            registry.sessions[index]
                .session
                .transport
                .push_synthetic_output(&vec![b'a' + index as u8; 512]);
        }
        assert!(round < 43);
    }
    let fixed_now = Instant::now();
    let mut lane = TerminalLaneState::default();
    let first = registry.drain_with_clock(&mut lane, || fixed_now);
    assert_eq!(first.serviced[0], (1, "control", 0));
    assert_eq!(first.output_events, GUI_DRAIN_EVENT_LIMIT);
    assert_eq!(first.output_bytes, GUI_DRAIN_BYTE_LIMIT);
    assert!(first.pending);
    assert_eq!(
        first
            .serviced
            .iter()
            .filter(|(_, kind, _)| *kind == "output")
            .take(6)
            .map(|(index, _, _)| *index)
            .collect::<Vec<_>>(),
        vec![2, 0, 1, 2, 0, 1]
    );
    assert_eq!(registry.next_drain_index, 1);

    let mut total = first.output_bytes;
    while registry
        .sessions
        .iter()
        .any(|slot| !slot.pending_drain_output.is_empty() || slot.session.has_pending_event())
    {
        let next = registry.drain_with_clock(&mut lane, || fixed_now);
        total += next.output_bytes;
        assert!(next.output_bytes <= GUI_DRAIN_BYTE_LIMIT);
        assert!(next.events <= 256);
    }
    assert_eq!(total, 129 * 512);
}

#[test]
fn inactive_output_and_bell_mark_only_the_originating_tab_unread() {
    let mut registry = synthetic_registry(2);
    registry.sessions[1]
        .session
        .transport
        .push_synthetic_output(b"background\x07");
    let fixed_now = Instant::now();
    let mut lane = TerminalLaneState::default();

    let report = registry.drain_with_clock(&mut lane, || fixed_now);
    assert_eq!(report.output_bytes, 11);
    registry.sync_lane_tabs(&mut lane);
    assert!(!lane.tabs[0].unread_output);
    assert_eq!(lane.tabs[0].unread_bell_count, 0);
    assert!(lane.tabs[1].unread_output);
    assert_eq!(lane.tabs[1].unread_bell_count, 1);
}

#[test]
fn tiny_chunk_flood_is_applied_once_per_session_per_turn() {
    let mut registry = synthetic_registry(1);
    for _ in 0..GUI_DRAIN_EVENT_LIMIT {
        registry.sessions[0]
            .session
            .transport
            .push_synthetic_output(b"x");
    }
    let fixed_now = Instant::now();
    let mut lane = TerminalLaneState::default();
    let report = registry.drain_with_clock(&mut lane, || fixed_now);
    assert_eq!(report.output_events, GUI_DRAIN_EVENT_LIMIT);
    assert_eq!(report.output_bytes, GUI_DRAIN_EVENT_LIMIT);
    assert_eq!(report.output_batches, 1);
    assert_eq!(
        registry.test_active_text().replace('\n', ""),
        "x".repeat(GUI_DRAIN_EVENT_LIMIT)
    );
    let event_log = crate::terminal_session_events::io_event_log::read_event_log_family_text(
        &registry.sessions[0].session.event_log_path(),
    );
    let output_records = event_log
        .lines()
        .filter(|line| line.contains("\"direction\":\"output\""))
        .collect::<Vec<_>>();
    assert_eq!(output_records.len(), 1);
    assert!(output_records[0].contains(&format!("\"byte_count\":{}", GUI_DRAIN_EVENT_LIMIT)));
}

#[test]
fn osc52_becomes_a_typed_session_scoped_request_without_changing_cells() {
    let mut registry = synthetic_registry(2);
    registry.sessions[0]
        .session
        .transport
        .push_synthetic_output(b"visible\x1b]52;c;RGF0dW0=\x07\x1b]9;build done\x07");
    registry.sessions[1]
        .session
        .transport
        .push_synthetic_output(b"peer");
    let fixed_now = Instant::now();
    let mut lane = TerminalLaneState::default();
    let report = registry.drain_with_clock(&mut lane, || fixed_now);

    assert_eq!(report.clipboard_requests.len(), 1);
    let request = &report.clipboard_requests[0];
    assert_eq!(request.session_id, "synthetic-0");
    assert_eq!(
        request.selection,
        datum_terminal_core::ClipboardSelection::Clipboard
    );
    assert_eq!(request.encoded_contents, b"RGF0dW0=");
    assert_eq!(report.notifications.len(), 1);
    assert_eq!(report.notifications[0].session_id, "synthetic-0");
    assert_eq!(report.notifications[0].text, "build done");
    assert_eq!(lane.latest_notification.as_deref(), Some("build done"));
    assert!(registry.test_session_text(0).contains("visible"));
    assert!(!registry.test_session_text(0).contains("RGF0dW0"));
    assert!(registry.test_session_text(1).contains("peer"));
}

#[test]
fn osc7_and_osc133_are_untrusted_session_metadata_not_design_authority() {
    let mut registry = synthetic_registry(2);
    registry.sessions[0]
        .session
        .transport
        .push_synthetic_output(
            b"visible\x1b]7;file://host/work/project\x07\x1b]133;A\x07\x1b]133;B\x07\x1b]133;C\x07\x1b]133;D;17\x07",
        );
    registry.sessions[1]
        .session
        .transport
        .push_synthetic_output(b"peer\x1b]133;A\x07");

    let fixed_now = Instant::now();
    let mut lane = TerminalLaneState::default();
    let report = registry.drain_with_clock(&mut lane, || fixed_now);
    assert!(report.notices.is_empty(), "{:?}", report.notices);
    assert_eq!(
        lane.current_working_directory.as_deref(),
        Some("file://host/work/project")
    );
    assert_eq!(
        lane.shell_metadata.phase,
        datum_gui_protocol::TerminalShellPhase::CommandFinished
    );
    assert_eq!(lane.shell_metadata.last_exit_code, Some(17));
    assert_eq!(lane.shell_metadata.event_sequence, 4);
    assert_eq!(
        registry.sessions[1].parked_lane.shell_metadata.phase,
        datum_gui_protocol::TerminalShellPhase::Prompt
    );
    assert_eq!(
        registry.sessions[1]
            .parked_lane
            .shell_metadata
            .event_sequence,
        1
    );
    assert!(registry.test_session_text(0).contains("visible"));
    assert!(!registry.test_session_text(0).contains("file://host"));

    let event_log = crate::terminal_session_events::io_event_log::read_event_log_family_text(
        &registry.sessions[0].session.event_log_path(),
    );
    let metadata = event_log
        .lines()
        .filter(|line| line.contains("\"event\":\"terminal_shell_metadata\""))
        .collect::<Vec<_>>();
    assert_eq!(metadata.len(), 5);
    assert!(metadata.iter().all(|line| {
        line.contains("\"origin\":\"pty_osc\"") && line.contains("\"trust\":\"untrusted\"")
    }));
    assert!(metadata.iter().any(|line| {
        line.contains("\"kind\":\"command_finished\"") && line.contains("\"process_exit_code\":17")
    }));
    for forbidden in [
        "terminal_command_handoff",
        "terminal_command_lifecycle",
        "approval",
        "operation",
    ] {
        assert!(!event_log.contains(forbidden));
    }
}

#[test]
fn split_control_and_utf8_chunks_batch_without_cross_session_leakage() {
    let mut registry = synthetic_registry(2);
    for bytes in [b"\x1b[31".as_slice(), b"mred".as_slice()] {
        registry.sessions[0]
            .session
            .transport
            .push_synthetic_output(bytes);
    }
    for bytes in [b"\xe2\x94".as_slice(), b"\x8c".as_slice()] {
        registry.sessions[1]
            .session
            .transport
            .push_synthetic_output(bytes);
    }
    let fixed_now = Instant::now();
    let mut lane = TerminalLaneState::default();
    let report = registry.drain_with_clock(&mut lane, || fixed_now);
    assert_eq!(report.output_batches, 2);
    assert!(registry.test_session_text(0).contains("red"));
    assert!(registry.test_session_text(1).contains('┌'));
    assert!(!registry.test_session_text(0).contains('┌'));
    assert!(!registry.test_session_text(1).contains("red"));
}

#[test]
fn same_session_output_is_applied_before_its_final_exit_control() {
    let mut registry = synthetic_registry(2);
    registry.sessions[0]
        .session
        .transport
        .push_synthetic_output(b"unrelated");
    registry.sessions[1]
        .session
        .transport
        .push_synthetic_output(b"final-tail");
    registry.sessions[1]
        .session
        .transport
        .push_synthetic_child_exit(TerminalExitStatus::Code(23));
    registry.sessions[1]
        .session
        .transport
        .finish_synthetic_reader();

    let fixed_now = Instant::now();
    let mut lane = TerminalLaneState::default();
    let mut report = registry.drain_with_clock(&mut lane, || fixed_now);
    report
        .serviced
        .extend(registry.drain_with_clock(&mut lane, || fixed_now).serviced);
    let controlled_apply = report
        .serviced
        .iter()
        .position(|entry| *entry == (1, "apply", 10))
        .unwrap();
    let exit = report
        .serviced
        .iter()
        .position(|entry| *entry == (1, "control", 0))
        .unwrap();
    let unrelated_apply = report
        .serviced
        .iter()
        .position(|entry| *entry == (0, "apply", 9))
        .unwrap();
    assert!(controlled_apply < exit);
    assert_ne!(controlled_apply, unrelated_apply);
    assert!(registry.test_session_text(1).contains("final-tail"));
}

#[test]
fn budget_yield_retains_bytes_rotates_application_and_cannot_overtake_exit() {
    let mut registry = synthetic_registry(2);
    registry.active_index = 1; // Preserve the exited inactive tab for content assertions.
    registry.sessions[0].pending_drain_output =
        [vec![b'x'; 4095], "┌tail".as_bytes().to_vec()].concat();
    registry.sessions[1].pending_drain_output = b"peer".to_vec();
    registry.sessions[0]
        .session
        .transport
        .push_synthetic_child_exit(TerminalExitStatus::Code(23));
    registry.sessions[0]
        .session
        .transport
        .finish_synthetic_reader();
    let mut lane = TerminalLaneState::default();
    let start = Instant::now();
    let mut calls = 0;
    let first = registry.drain_with_clock(&mut lane, || {
        calls += 1;
        start
            + if calls <= 2 {
                Duration::ZERO
            } else {
                Duration::from_millis(2)
            }
    });
    assert!(first.pending);
    assert_eq!(first.serviced, vec![(0, "apply", 4096)]);
    assert_eq!(registry.sessions[0].pending_drain_output.len(), 6);
    assert_eq!(registry.sessions[1].pending_drain_output, b"peer");
    assert!(registry.sessions[0].exact_exit_status.is_none());
    calls = 0;
    let second = registry.drain_with_clock(&mut lane, || {
        calls += 1;
        start
            + if calls <= 2 {
                Duration::ZERO
            } else {
                Duration::from_millis(2)
            }
    });
    assert_eq!(second.serviced, vec![(1, "apply", 4)]);
    assert!(second.pending);
    let final_turn = registry.drain_with_clock(&mut lane, || start);
    assert_eq!(final_turn.serviced[0], (0, "apply", 6));
    assert_eq!(final_turn.serviced[1], (0, "control", 0));
    assert!(!final_turn.pending);
    assert!(registry.test_session_text(0).contains("┌tail"));
    assert!(registry.test_session_text(1).contains("peer"));
    assert!(!registry.test_session_text(1).contains("tail"));
}

#[test]
fn spare_dispatch_time_rotates_batches_with_one_shared_application_byte_cap() {
    let mut registry = synthetic_registry(2);
    registry.sessions[0].pending_drain_output = vec![b'a'; GUI_DRAIN_BYTE_LIMIT / 2];
    registry.sessions[1].pending_drain_output = vec![b'b'; GUI_DRAIN_BYTE_LIMIT / 2];
    for slot in &registry.sessions {
        slot.session
            .transport
            .push_synthetic_output(&vec![b'c'; GUI_DRAIN_BYTE_LIMIT / 4]);
    }
    let start = Instant::now();
    let mut lane = TerminalLaneState::default();
    let report = registry.drain_with_clock(&mut lane, || start);
    assert_eq!(report.applied_bytes, GUI_DRAIN_BYTE_LIMIT);
    assert_eq!(report.output_bytes, GUI_DRAIN_BYTE_LIMIT / 2);
    assert!(report.pending);
    let applied: Vec<_> = report
        .serviced
        .iter()
        .filter(|(_, kind, _)| *kind == "apply")
        .collect();
    assert_eq!(applied.len(), GUI_DRAIN_BYTE_LIMIT / APPLY_BATCH_BYTES);
    for (turn, (index, _, bytes)) in applied.into_iter().enumerate() {
        assert_eq!(*index, turn % 2);
        assert_eq!(*bytes, APPLY_BATCH_BYTES);
    }
    for slot in &registry.sessions {
        assert_eq!(slot.pending_drain_output.len(), GUI_DRAIN_BYTE_LIMIT / 4);
    }
    let second = registry.drain_with_clock(&mut lane, || start);
    assert_eq!(second.applied_bytes, GUI_DRAIN_BYTE_LIMIT / 2);
    assert!(!second.pending);
}

#[test]
fn render_panes_borrow_matching_active_and_parked_lane_projections() {
    use datum_gui_protocol::{TerminalSplitDirection, TerminalSplitNode};
    let mut registry = synthetic_registry(2);
    registry.terminal_tabs.truncate(1);
    registry.terminal_tabs[0].root = TerminalSplitNode::Split {
        direction: TerminalSplitDirection::SideBySide,
        ratio_millis: 500,
        first: Box::new(TerminalSplitNode::session("synthetic-0")),
        second: Box::new(TerminalSplitNode::session("synthetic-1")),
    };
    registry.terminal_tabs[0].focused_session_id = "synthetic-1".into();
    registry.sessions[1].parked_lane.status = "parked output".into();
    let parked = &registry.sessions[1].parked_lane as *const TerminalLaneState;
    let lane = TerminalLaneState {
        status: "active output".into(),
        ..Default::default()
    };
    let panes = registry.take_active_tab_render_states(&lane).unwrap();
    assert_eq!(panes.len(), 2);
    assert_eq!(panes[0].session_id, "synthetic-0");
    assert!(std::ptr::eq(panes[0].lane, &lane));
    assert!(!panes[0].focused);
    assert_eq!(panes[1].session_id, "synthetic-1");
    assert!(std::ptr::eq(panes[1].lane, parked));
    assert_eq!(panes[1].lane.status, "parked output");
    assert!(panes[1].focused);
    drop(panes);
    registry.active_pending_id = Some("pending".into());
    assert!(
        registry
            .take_active_tab_render_states(&lane)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn refused_scene_restores_consumed_terminal_damage_for_retry() {
    let mut registry = synthetic_registry(1);
    let lane = TerminalLaneState::default();
    let panes = registry.take_active_tab_render_states(&lane).unwrap();
    assert!(!panes[0].damage.is_empty());
    let expected = panes[0].damage.clone();
    let damage = panes
        .into_iter()
        .map(|pane| (pane.session_id, pane.damage))
        .collect();
    registry.restore_render_damage(damage);
    let retry = registry.take_active_tab_render_states(&lane).unwrap();
    assert_eq!(retry[0].damage, expected);
    drop(retry);
    assert!(
        registry.take_active_tab_render_states(&lane).unwrap()[0]
            .damage
            .is_empty()
    );
}
