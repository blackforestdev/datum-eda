use super::*;
use crate::terminal_session_context::TerminalSessionContextSummary;
use datum_gui_protocol::{
    CheckRunReviewState, DatumCursorContext, DatumProjectionContext, DatumSelectionContext,
    ProductionStatus, TerminalLaneState,
};
use std::fs;

fn terminal_launch_context_for_project_root(
    project_root: &std::path::Path,
) -> TerminalLaunchContext {
    TerminalLaunchContext {
        project_root: project_root.to_path_buf(),
        project_id: None,
        project_name: None,
        board_id: None,
        board_name: None,
        scene_id: None,
        source_revision: None,
        production_status: ProductionStatus::default(),
        source_shard_status: datum_gui_protocol::SourceShardStatusSummary::default(),
        check_status: CheckRunReviewState::default(),
        selection_context: DatumSelectionContext {
            kind: "none".to_string(),
            id: None,
        },
        cursor_context: DatumCursorContext {
            screen_px: None,
            hovered_object_id: None,
            active_dock_tab: None,
            active_tool: "select".to_string(),
        },
        projection_context: DatumProjectionContext {
            scene_id: "test-scene".to_string(),
            board_id: None,
            board_name: None,
            scene_bounds_nm: None,
            active_projection_id: None,
        },
        terminal_sessions: TerminalSessionContextSummary::default(),
    }
}

#[test]
fn terminal_context_projects_protocol_terminal_tabs_for_agents() {
    let root = std::env::temp_dir().join(format!(
        "datum-terminal-context-tabs-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("terminal context tab test root should create");
    let context = terminal_launch_context_for_project_root(&root);
    let mut registry =
        TerminalSessionRegistry::spawn(&context).expect("spawn initial terminal session");
    let first_session_id = registry.active().session_id().to_string();
    registry
        .rename(&first_session_id, "rules shell")
        .expect("rename first terminal tab");
    let second_session_id = registry
        .spawn_and_activate(&context)
        .expect("spawn second terminal session")
        .to_string();
    registry
        .rename(&second_session_id, "layout agent")
        .expect("rename second terminal tab");
    let mut terminal_state = TerminalLaneState::default();
    registry
        .resize_active(132, 37)
        .expect("resize active terminal");
    registry.sync_lane_tabs(&mut terminal_state);
    terminal_state.current_working_directory = Some(root.join("layout").display().to_string());

    let mut workspace = datum_gui_protocol::load_fixture_workspace_state();
    workspace.ui.terminal = terminal_state;
    let refreshed_context = terminal_launch_context_from_state(&root, &workspace);
    refresh_terminal_session_context(registry.active(), &refreshed_context)
        .expect("refresh terminal context with protocol tab state");

    let latest_context_path = root.join(".datum/gui-terminal-context.json");
    let latest_context: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&latest_context_path).unwrap()).unwrap();
    assert_eq!(
        latest_context["terminal_sessions"]["active_session_id"],
        second_session_id
    );
    assert_eq!(
        latest_context["terminal_sessions"]["active_label"],
        "layout agent"
    );
    assert_eq!(
        latest_context["terminal_sessions"]["active_status"],
        "running"
    );
    assert_eq!(
        latest_context["terminal_sessions"]["active_attached"],
        serde_json::json!(true)
    );
    assert_eq!(
        latest_context["terminal_sessions"]["active_working_directory"],
        root.join("layout").display().to_string()
    );
    assert_eq!(latest_context["terminal_sessions"]["tab_count"], 2);
    assert_eq!(latest_context["terminal_sessions"]["columns"], 132);
    assert_eq!(latest_context["terminal_sessions"]["rows"], 37);
    assert_eq!(
        latest_context["terminal_sessions"]["tabs"][0]["session_id"],
        first_session_id
    );
    assert_eq!(
        latest_context["terminal_sessions"]["tabs"][0]["label"],
        "rules shell"
    );
    assert_eq!(
        latest_context["terminal_sessions"]["tabs"][0]["attached"],
        serde_json::json!(false)
    );
    assert_eq!(
        latest_context["terminal_sessions"]["tabs"][1]["session_id"],
        second_session_id
    );
    assert_eq!(
        latest_context["terminal_sessions"]["tabs"][1]["label"],
        "layout agent"
    );
    assert!(
        latest_context["terminal_sessions"]["tabs"][0]["activity_event_count"]
            .as_u64()
            .unwrap()
            >= 1,
        "detached tab activity should be visible to agents: {latest_context}"
    );
    assert!(latest_context["active_context_commands"]["artifact_show"].is_null());
    assert!(latest_context["active_context_commands"]["check_show"].is_null());
    let _ = fs::remove_dir_all(&root);
}
