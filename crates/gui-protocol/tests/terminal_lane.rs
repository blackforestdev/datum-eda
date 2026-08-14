#[test]
fn terminal_default_grid_is_empty_and_session_status_lives_in_chrome() {
    // T0-C01 (DATUM_NATIVE_TERMINAL_SPEC.md) / decision 027 FT-001: the
    // terminal grid starts with zero rows — no seeded "ready"/description
    // copy. Only PTY bytes interpreted by the terminal core may create cells;
    // session status is chrome state (`status`), not grid content.
    let workspace = datum_gui_protocol::load_fixture_workspace_state();
    assert!(
        workspace.ui.terminal.grid_lines().is_empty(),
        "seed terminal grid must be empty; only PTY output may create rows"
    );
    assert!(workspace.ui.terminal.grid_styled_lines().is_empty());
    assert_eq!(workspace.ui.terminal.status, "running");
}
