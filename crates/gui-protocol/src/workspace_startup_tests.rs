use super::*;

#[test]
fn unopened_state_has_no_source_authority_and_shared_commands_refuse_project_actions() {
    let state = ReviewWorkspaceState::unopened();
    assert!(state.backing.is_none());
    assert!(state.scene.project_uuid.is_empty());
    assert!(state.scene.scene_id.is_empty());
    assert!(state.review.project_root.is_none());
    assert!(state.library.basis.is_none());
    assert!(camera_scene_for_pane(&state, state.ui.layout.focused).is_none());
    assert!(gui_menu_model::action_registry::admit("view.fit", &state).is_err());
    assert!(gui_menu_model::action_registry::admit("library.open", &state).is_err());
}

#[test]
fn unopened_origin_cannot_publish_across_loaded_or_new_session_generation() {
    let origin = ProjectOpenOrigin::capture_unopened(9);
    assert!(origin.matches_unopened(9));
    assert!(!origin.matches_unopened(10));
    assert!(!origin.matches(std::path::Path::new("."), 9));
}

#[test]
fn loaded_unbacked_review_retains_camera_authority() {
    let mut state = crate::load_fixture_workspace_state();
    state.backing = None;
    assert!(!state.project_unopened);
    assert!(camera_scene_for_pane(&state, state.ui.layout.focused).is_some());
    assert!(gui_menu_model::action_registry::admit("view.fit", &state).is_ok());
}
