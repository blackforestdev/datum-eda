use super::*;
use crate::terminal_session::TerminalLaunchContext;
use std::time::{Duration, Instant};

#[test]
fn inactive_and_active_sessions_drain_into_isolated_projections() {
    let root =
        std::env::temp_dir().join(format!("datum-terminal-fair-drain-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry = TerminalSessionRegistry::spawn(&context).unwrap();
    let first_id = registry.active().session_id().to_string();
    let mut lane = TerminalLaneState::default();
    registry
        .spawn_and_activate_with_lane(&context, &mut lane)
        .unwrap();
    registry.sessions[0]
        .session
        .write_bytes(b"printf 'alpha-session\\n'\n")
        .unwrap();
    registry
        .active()
        .write_bytes(b"printf 'beta-session\\n'\n")
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        registry.drain_all(&mut lane);
        let inactive = registry.sessions[0].parked_lane.grid_lines().join("\n");
        let active = lane.grid_lines().join("\n");
        if inactive.contains("alpha-session") && active.contains("beta-session") {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        registry.sessions[0]
            .parked_lane
            .grid_lines()
            .join("\n")
            .contains("alpha-session")
    );
    assert!(lane.grid_lines().join("\n").contains("beta-session"));
    assert!(!lane.grid_lines().join("\n").contains("alpha-session"));
    registry.sync_lane_tabs(&mut lane);
    let inactive_tab = lane
        .tabs
        .iter()
        .find(|tab| tab.session_id == first_id)
        .unwrap();
    assert!(inactive_tab.activity_event_count > 0);

    registry.activate_with_lane(&first_id, &mut lane).unwrap();
    assert!(lane.grid_lines().join("\n").contains("alpha-session"));
    assert!(!lane.grid_lines().join("\n").contains("beta-session"));
    let _ = std::fs::remove_dir_all(root);
}
