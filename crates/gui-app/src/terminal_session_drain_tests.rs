use super::*;
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
    });
    registry.active_pending_id = Some("pending-shell-2".to_string());
    registry.sessions[0]
        .session
        .transport
        .push_synthetic_output(b"old-shell-output");

    let mut lane = TerminalLaneState::default();
    let report = registry.drain_all(&mut lane);
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
    let mut lane = TerminalLaneState::default();
    registry
        .active()
        .write_bytes(b"head -c 200000 /dev/zero | tr '\\0' x\n")
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while !registry.active().has_pending_event() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    let report = registry.drain_all(&mut lane);
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
    let mut lane = TerminalLaneState::default();
    let first = registry.drain_all(&mut lane);
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

    let second = registry.drain_all(&mut lane);
    assert_eq!(second.serviced[0].0, 1);
    assert_eq!(second.serviced[0].1, "output");
    assert_eq!(second.output_events, 1);
    assert_eq!(second.output_bytes, 512);
    assert!(!second.pending);
}

#[test]
fn inactive_output_and_bell_mark_only_the_originating_tab_unread() {
    let mut registry = synthetic_registry(2);
    registry.sessions[1]
        .session
        .transport
        .push_synthetic_output(b"background\x07");
    let mut lane = TerminalLaneState::default();

    let report = registry.drain_all(&mut lane);
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
    let mut lane = TerminalLaneState::default();
    let report = registry.drain_all(&mut lane);
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
    let mut lane = TerminalLaneState::default();
    let report = registry.drain_all(&mut lane);

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
    let mut lane = TerminalLaneState::default();
    let report = registry.drain_all(&mut lane);
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

    let mut lane = TerminalLaneState::default();
    let report = registry.drain_all(&mut lane);
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
    assert!(exit < unrelated_apply);
    assert!(registry.test_session_text(1).contains("final-tail"));
}
