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
