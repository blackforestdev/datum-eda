use super::*;
use datum_gui_protocol::TerminalLaneState;
use std::fs;

#[test]
fn default_session_labels_never_reuse_a_removed_ordinal() {
    let root = std::env::temp_dir().join(format!(
        "datum-terminal-monotonic-labels-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("terminal naming test root should create");
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry =
        TerminalSessionRegistry::spawn(&context).expect("spawn initial terminal session");
    registry
        .spawn_and_activate(&context)
        .expect("spawn shell 2");

    // Model the presentation-complete removal performed by close_active: the
    // remaining live-count must never become the next default-name authority.
    registry.sessions.remove(0);
    registry.active_index = 0;
    registry
        .spawn_and_activate(&context)
        .expect("spawn successor after shell 1 removal");

    let mut lane = TerminalLaneState::default();
    registry.sync_lane_tabs(&mut lane);
    let labels = lane
        .tabs
        .iter()
        .map(|tab| tab.label.as_str())
        .collect::<Vec<_>>();
    assert_eq!(labels, ["shell 2", "shell 3"]);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shell_titles_drive_tabs_until_the_user_renames_them() {
    let root = std::env::temp_dir().join(format!(
        "datum-terminal-title-labels-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("terminal title test root should create");
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry =
        TerminalSessionRegistry::spawn(&context).expect("spawn initial terminal session");
    let session_id = registry.active().session_id().to_string();
    let mut lane = TerminalLaneState {
        title: Some("agent workspace".to_string()),
        ..TerminalLaneState::default()
    };

    registry.sync_lane_tabs(&mut lane);
    assert_eq!(lane.tabs[0].label, "agent workspace");

    registry
        .rename(&session_id, "release shell")
        .expect("explicit rename should succeed");
    lane.title = Some("later shell title".to_string());
    registry.sync_lane_tabs(&mut lane);
    assert_eq!(lane.tabs[0].label, "release shell");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn inactive_shell_title_stays_with_its_parked_session() {
    let root = std::env::temp_dir().join(format!(
        "datum-terminal-inactive-title-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("terminal title test root should create");
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry =
        TerminalSessionRegistry::spawn(&context).expect("spawn initial terminal session");
    registry
        .spawn_and_activate(&context)
        .expect("spawn second terminal session");
    registry.sessions[0].parked_lane.title = Some("background agent".to_string());
    let mut lane = TerminalLaneState {
        title: Some("foreground shell".to_string()),
        ..TerminalLaneState::default()
    };

    registry.sync_lane_tabs(&mut lane);
    assert_eq!(lane.tabs[0].label, "background agent");
    assert_eq!(lane.tabs[1].label, "foreground shell");
    let _ = fs::remove_dir_all(root);
}
