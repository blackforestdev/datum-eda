use super::*;
use std::time::{Duration, Instant};

#[test]
fn already_exited_sole_tab_closes_immediately_without_signaling() {
    let root = std::env::temp_dir().join(format!(
        "datum-terminal-close-exited-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
    let mut lane = TerminalLaneState::default();
    registry.active().write_bytes(b"exit 0\n").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline && !registry.active().presentation_complete() {
        registry.drain_all(&mut lane);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(registry.active().presentation_complete());
    registry.close_active(&mut lane).unwrap();
    assert!(lane.tabs.is_empty());
    assert_eq!(lane.active_session_id, None);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn lifecycle_only_supervisor_wake_changes_visible_status() {
    let root =
        std::env::temp_dir().join(format!("datum-terminal-phase-wake-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
    let mut lane = TerminalLaneState::default();
    registry
        .active()
        .write_bytes(b"trap '' HUP TERM; while :; do sleep 1; done\n")
        .unwrap();
    std::thread::sleep(Duration::from_millis(30));
    while registry.active().has_pending_event() {
        registry.drain_all(&mut lane);
    }
    registry.active().terminate().unwrap();
    let report = registry.drain_all(&mut lane);
    assert!(report.tabs_changed || report.active_projection_changed);
    assert!(lane.status.starts_with("terminating"));
    registry.active().force_kill();
    let _ = std::fs::remove_dir_all(root);
}
