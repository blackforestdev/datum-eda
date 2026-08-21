#[test]
fn terminal_default_session_projection_contains_only_chrome_state() {
    let workspace = datum_gui_protocol::load_fixture_workspace_state();
    assert_eq!(workspace.ui.terminal.title, None);
    assert_eq!(workspace.ui.terminal.bell_count, 0);
    assert_eq!(workspace.ui.terminal.status, "running");
}
