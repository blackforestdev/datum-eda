//! Screen-authority regressions for T0-C01 (DATUM_NATIVE_TERMINAL_SPEC.md)
//! and decision 027 FT-001: terminal cells are mutated only by PTY bytes
//! interpreted by the terminal core. Datum notices, diagnostics, activity
//! summaries, and lifecycle messages route to chrome, notifications, logs, or
//! the console sink — never the terminal grid.

use super::load_fixture_workspace_state;

#[test]
fn workspace_seed_terminal_grid_is_empty() {
    let state = load_fixture_workspace_state();
    assert!(
        state.ui.terminal.grid_lines().is_empty(),
        "the terminal grid must start with zero rows — no seeded ready/notice \
         rows; only PTY output may create terminal cells (T0-C01)"
    );
    assert!(
        state.ui.terminal.grid_styled_lines().is_empty(),
        "the styled terminal grid must start empty (T0-C01)"
    );
}

#[test]
fn lifecycle_and_diagnostic_narration_routes_to_console_not_terminal_grid() {
    let mut state = load_fixture_workspace_state();
    let grid_before = state.ui.terminal.grid_lines().to_vec();

    // The exact classes of messages that previously polluted the grid.
    let routed = [
        "opened terminal session s-1",
        "terminal session restarted",
        "terminal write failed: broken pipe",
        "workspace scene/status refreshed",
        "selected terminal activity span: #1 terminal_io in:1B out:565B",
        "detached active terminal session",
    ];
    for message in routed {
        state.ui.push_console_line(message.to_string());
    }

    for message in routed {
        assert!(
            state.ui.console.lines.iter().any(|line| line == message),
            "console sink should have received the routed message {message:?}"
        );
        assert!(
            !state
                .ui
                .terminal
                .grid_lines()
                .iter()
                .any(|line| line.contains(message)),
            "routed message {message:?} must never appear in the terminal grid"
        );
    }
    assert_eq!(
        state.ui.terminal.grid_lines(),
        grid_before,
        "console routing must leave the terminal grid byte-identical"
    );
}
