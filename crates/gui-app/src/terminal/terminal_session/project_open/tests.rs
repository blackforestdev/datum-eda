#[path = "../../../../../test-harness/src/development_workspace.rs"]
mod development_workspace;
use super::*;
use std::sync::atomic::AtomicU64;
use std::time::{Duration, Instant};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);

fn registry() -> (TerminalSessionRegistry, TerminalLaunchContext) {
    let root = development_workspace::path("open-project-terminal-tests").join(format!(
        "{}-{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&root).unwrap();
    let context = TerminalLaunchContext::for_project_root(&root);
    (TerminalSessionRegistry::spawn(&context).unwrap(), context)
}

fn finish(registry: &mut TerminalSessionRegistry, lane: &mut TerminalLaneState) {
    let deadline =
        Instant::now() + Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS);
    registry.terminate_all_by(deadline);
    while !registry.all_sessions_closed() && Instant::now() < deadline {
        registry.drain_all(lane);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        registry.all_sessions_closed(),
        "owned terminal cleanup did not finish"
    );
}

#[test]
fn staged_cancel_retains_child_and_capacity_until_verified_reaping() {
    let (mut registry, context) = registry();
    let old = registry.active().session_id().to_owned();
    let tabs = registry.terminal_tabs.clone();
    let prepared = registry
        .reserve_project_terminal()
        .unwrap()
        .prepare(context)
        .unwrap();
    assert_eq!(
        registry.project_open_reservations.load(Ordering::Acquire),
        1
    );
    assert!(
        registry.project_preparation_pending(),
        "host naming is blocked during actual terminal preparation"
    );
    drop(prepared);
    assert_eq!(registry.active().session_id(), old);
    assert_eq!(registry.terminal_tabs, tabs);
    assert_eq!(registry.sessions.len(), 1);
    assert_eq!(registry.rejected_project_terminals.lock().unwrap().len(), 1);
    assert_eq!(
        registry.project_open_reservations.load(Ordering::Acquire),
        1
    );
    assert!(
        registry.project_preparation_pending(),
        "cancelled prepared child still owns host-action protection before verified reaping"
    );
    let mut lane = TerminalLaneState::default();
    finish(&mut registry, &mut lane);
    assert!(
        !registry.project_preparation_pending(),
        "host-action protection releases only after actual child cleanup"
    );
    assert!(
        registry
            .rejected_project_terminals
            .lock()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        registry.project_open_reservations.load(Ordering::Acquire),
        0
    );
}

#[test]
fn reservations_enforce_existing_capacity_before_child_allocation() {
    let (mut registry, _) = registry();
    let reservations: Vec<_> = (1..MAX_LIVE_SESSIONS)
        .map(|_| registry.reserve_project_terminal().unwrap())
        .collect();
    assert!(registry.reserve_project_terminal().is_err());
    assert_eq!(registry.sessions.len(), 1);
    assert!(
        registry
            .rejected_project_terminals
            .lock()
            .unwrap()
            .is_empty()
    );
    drop(reservations);
    assert_eq!(
        registry.project_open_reservations.load(Ordering::Acquire),
        0
    );
    finish(&mut registry, &mut TerminalLaneState::default());
}

#[test]
fn pending_tab_and_split_refuse_publication_without_double_parking_old_lane() {
    for split in [false, true] {
        let (mut registry, context) = registry();
        let old = registry.active().session_id().to_owned();
        let prepared = registry
            .reserve_project_terminal()
            .unwrap()
            .prepare(context)
            .unwrap();
        let (_sender, receiver) = std::sync::mpsc::channel();
        registry.pending_spawns.push(PendingTerminalSpawn {
            pending_id: "held-pending".to_owned(),
            label: "held".to_owned(),
            result: receiver,
            canceled: false,
            placement: if split {
                PendingTerminalPlacement::Split {
                    source_session_id: old.clone(),
                    direction: datum_gui_protocol::TerminalSplitDirection::SideBySide,
                }
            } else {
                PendingTerminalPlacement::NewTab
            },
        });
        registry.active_pending_id = Some("held-pending".to_owned());
        registry.sessions[0].parked_lane.title = Some("original parked screen".to_owned());
        let mut lane = TerminalLaneState {
            title: Some("pending screen".to_owned()),
            ..Default::default()
        };
        let before = lane.clone();
        assert!(
            registry
                .publish_project_terminal(prepared, &mut lane)
                .is_err()
        );
        assert_eq!(lane, before);
        assert_eq!(registry.active().session_id(), old);
        assert_eq!(
            registry.sessions[0].parked_lane.title.as_deref(),
            Some("original parked screen")
        );
        registry.pending_spawns.clear();
        registry.active_pending_id = None;
        finish(&mut registry, &mut lane);
    }
}

#[test]
fn publication_preserves_old_session_origin_and_workspace_owned_chrome() {
    let (mut registry, old_context) = registry();
    let old = registry.active().session_id().to_owned();
    let destination = old_context
        .project_root
        .as_ref()
        .unwrap()
        .join("destination with space");
    std::fs::create_dir_all(&destination).unwrap();
    let mut new_context = TerminalLaunchContext::for_project_root(&destination);
    new_context.project_id = old_context.project_id.clone();
    new_context.source_revision = old_context.source_revision.clone();
    let prepared = registry
        .reserve_project_terminal()
        .unwrap()
        .prepare(new_context.clone())
        .unwrap();
    let mut lane = TerminalLaneState {
        title: Some("old screen".to_owned()),
        font_scale_millis: 1400,
        project_open_cleanup_failure: Some("owned rejected child failure".to_owned()),
        ..Default::default()
    };
    registry
        .publish_project_terminal(prepared, &mut lane)
        .unwrap();
    assert_ne!(registry.active().session_id(), old);
    assert_eq!(registry.sessions[0].session.session_id(), old);
    assert_eq!(
        registry.sessions[0].parked_lane.title.as_deref(),
        Some("old screen")
    );
    assert_eq!(lane.font_scale_millis, 1400);
    assert_eq!(
        lane.project_open_cleanup_failure.as_deref(),
        Some("owned rejected child failure")
    );
    assert_eq!(
        registry.origin_contexts[&old].project_root,
        old_context.project_root
    );
    assert_eq!(
        registry.active_origin_context().unwrap().project_root,
        Some(destination)
    );
    assert_eq!(
        registry.restart_origin(&old, &new_context).project_root,
        old_context.project_root
    );
    finish(&mut registry, &mut lane);
    assert!(lane.project_open_cleanup_failure.is_none());
}

#[test]
fn actual_child_context_and_core_failure_keep_supervised_cleanup_ownership() {
    for context_failure in [true, false] {
        let (mut registry, context) = registry();
        let old = registry.active().session_id().to_owned();
        let reservation = registry.reserve_project_terminal().unwrap();
        if context_failure {
            let (result, rejected) =
                crate::terminal_process::prepare_terminal_process_with_persistence(
                    &context,
                    registry.terminal_wake.clone(),
                    |pid_context, _| {
                        assert!(pid_context.process_group_id.is_some());
                        anyhow::bail!("injected PID-context persistence refusal")
                    },
                );
            assert!(result.is_err());
            reservation.reject(rejected.expect("real child must have a supervised owner"));
        } else {
            let session = spawn_terminal_process(&context, registry.terminal_wake.clone()).unwrap();
            assert!(
                reservation
                    .finish_prepare(
                        session,
                        context,
                        Err(anyhow::anyhow!("injected core creation failure"))
                    )
                    .is_err()
            );
        }
        assert_eq!(registry.active().session_id(), old);
        assert_eq!(registry.sessions.len(), 1);
        assert_eq!(
            registry.project_open_reservations.load(Ordering::Acquire),
            1
        );
        assert_eq!(registry.rejected_project_terminals.lock().unwrap().len(), 1);
        finish(&mut registry, &mut TerminalLaneState::default());
        assert_eq!(
            registry.project_open_reservations.load(Ordering::Acquire),
            0
        );
    }
}

#[test]
fn rejected_failure_survives_tab_projection_and_retry_clears_only_after_reaping() {
    let (mut registry, context) = registry();
    let old = registry.active().session_id().to_owned();
    let mut prepared = registry
        .reserve_project_terminal()
        .unwrap()
        .prepare(context)
        .unwrap();
    let slot = prepared.slot.take().unwrap();
    slot.session.terminate_by(Instant::now()).unwrap();
    let deadline =
        Instant::now() + Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS);
    while slot.session.shutdown_snapshot().unwrap().phase
        != crate::terminal_transport::ShutdownPhase::Failed
    {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    // Retain the actual failed child directly; rejecting it anew would itself
    // request another graceful attempt and conceal the failure under test.
    let mut reservation = prepared.reservation.take().unwrap();
    let queue = reservation.rejected.take().unwrap();
    queue.lock().unwrap().push(RejectedProjectTerminal {
        session: slot.session,
        failure_reported: false,
        _reservation: reservation,
    });
    let mut lane = TerminalLaneState::default();
    registry.drain_all(&mut lane);
    assert!(lane.project_open_cleanup_failure.is_some());
    assert_eq!(registry.active().session_id(), old);
    assert_eq!(registry.sessions[0].status, "running");
    let mut parked = TerminalLaneState::default();
    lane.swap_session_projection(&mut parked);
    assert!(lane.project_open_cleanup_failure.is_some());
    assert!(parked.project_open_cleanup_failure.is_none());
    registry.retry_failed_terminations();
    let deadline =
        Instant::now() + Duration::from_millis(crate::terminal_transport::GLOBAL_SHUTDOWN_MS);
    while !registry
        .rejected_project_terminals
        .lock()
        .unwrap()
        .is_empty()
    {
        assert!(Instant::now() < deadline);
        registry.drain_all(&mut lane);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(lane.project_open_cleanup_failure.is_none());
    assert_eq!(registry.active().session_id(), old);
    assert_eq!(
        registry.project_open_reservations.load(Ordering::Acquire),
        0
    );
    finish(&mut registry, &mut lane);
}
