#[test]
fn terminal_default_copy_describes_project_shell() {
    let workspace = datum_gui_protocol::load_fixture_workspace_state();
    assert!(
        workspace
            .ui
            .terminal
            .lines
            .iter()
            .any(|line| line.contains("shell session starts"))
    );
    assert!(
        !workspace
            .ui
            .terminal
            .lines
            .iter()
            .any(|line| line.contains("read-only"))
    );
    assert_eq!(workspace.ui.terminal.status, "running");
}
