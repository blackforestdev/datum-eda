use super::*;

#[test]
fn source_free_launch_has_no_project_root_or_terminal_process() {
    let args = GuiArgs::parse_from(["datum-gui"]);
    let launch = args.unopened_launch_state(None).unwrap();
    assert!(launch.request.is_none());
    assert!(launch.state.project_unopened);
    assert!(launch.state.backing.is_none());
    assert!(launch.terminal_launch_context.is_none());
    assert_eq!(launch.terminal_sessions.len(), 0);
    assert!(!launch.terminal_sessions.active_attached());
}

#[test]
fn explicit_invalid_source_never_becomes_empty_startup() {
    for flag in ["--project-root", "--board", "--schematic", "--artifact"] {
        let args = GuiArgs::parse_from(["datum-gui", flag, "/nonexistent/datum-source"]);
        assert!(!args.has_no_project_source(), "{flag}");
    }
    assert!(!GuiArgs::parse_from(["datum-gui", "--demo-known-good"]).has_no_project_source());
    assert!(!GuiArgs::parse_from(["datum-gui", "--net", "unbound"]).has_no_project_source());
}

#[test]
fn empty_terminal_projection_render_resize_and_lifecycle_are_safe() {
    let mut registry = TerminalSessionRegistry::empty(None);
    let mut lane = datum_gui_protocol::TerminalLaneState::default();
    registry.sync_lane_tabs(&mut lane);
    assert!(lane.active_session_id.is_none());
    assert!(registry.active_origin_context().is_none());
    assert!(lane.tabs.is_empty());
    assert!(lane.tab_layouts.is_empty());
    assert_eq!(registry.active_render_row_count(), 0);
    assert!(
        registry
            .take_active_tab_render_states(&lane)
            .unwrap()
            .is_empty()
    );
    registry.resize_active_surface(80, 24, 640, 480).unwrap();
    assert!(registry.copy_active_selection().is_err());
    assert!(registry.close_active(&mut lane).is_err());
    assert!(registry.terminate_active(&mut lane).is_err());
    assert!(!registry.active_close_confirmation_armed());
    assert!(registry.confirm_close_active(&mut lane).is_err());
    registry.force_kill_active();
    registry.handle_close_confirmation_input(b"\x1b", &mut lane);
}
