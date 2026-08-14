//! Screen-authority regression for T0-C01 (DATUM_NATIVE_TERMINAL_SPEC.md) and
//! decision 027 FT-001: only PTY bytes interpreted by the terminal core may
//! mutate terminal cells. Session lifecycle and GUI events route their
//! narration to the console sink, never the grid.

use super::*;
use std::fs;

#[test]
fn terminal_grid_holds_only_pty_rows_across_session_lifecycle_events() {
    // T0-C01 (DATUM_NATIVE_TERMINAL_SPEC.md) / decision 027 FT-001 regression:
    // the terminal grid may be written only by PTY bytes interpreted by the
    // terminal core. GUI/session lifecycle events that historically injected
    // notice rows (open, restart, detach, close, tab sync) must leave the
    // grid byte-identical.
    let root = std::env::temp_dir().join(format!(
        "datum-terminal-screen-authority-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("terminal test root should create");
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry =
        TerminalSessionRegistry::spawn(&context).expect("spawn initial terminal session");
    let mut state = TerminalLaneState::default();

    // The grid starts empty — no seeded "ready" rows.
    assert!(
        state.grid_lines().is_empty() && state.grid_styled_lines().is_empty(),
        "terminal grid must start empty; only PTY output may create rows"
    );

    // The one legal writer: PTY bytes interpreted by the terminal core.
    let mut screen = crate::terminal_screen::TerminalScreen::default();
    screen.apply_bytes(
        &mut state,
        b"datum$ printf t0-canary\r\nt0-canary\r\ndatum$ ",
    );
    let pty_rows = state.grid_lines().to_vec();
    assert!(
        pty_rows.iter().any(|line| line.contains("t0-canary")),
        "PTY-derived canary rows should be visible in the grid"
    );

    // Session lifecycle and GUI-side refresh events that previously wrote the
    // grid.
    registry.sync_lane_tabs(&mut state);
    registry
        .spawn_and_activate(&context)
        .expect("spawn second terminal session");
    registry.sync_lane_tabs(&mut state);
    assert!(registry.resize_active(101, 29).is_ok());
    registry
        .restart_active(&mut state, &context)
        .expect("restart active terminal session");
    registry
        .detach_active(&mut state)
        .expect("detach active terminal session");
    registry
        .close_active(&mut state)
        .expect("close active terminal session");

    assert_eq!(
        state.grid_lines(),
        pty_rows,
        "session lifecycle events must not add, remove, or edit terminal grid rows"
    );
    let lifecycle_phrases = [
        "opened terminal session",
        "terminal restarted",
        "renamed active terminal session",
        "detached active terminal session",
        "terminal session",
        "activity summary",
        "workspace scene/status refreshed",
    ];
    for line in state.grid_lines() {
        for phrase in lifecycle_phrases {
            assert!(
                !line.contains(phrase),
                "terminal grid row {line:?} carries non-PTY lifecycle text {phrase:?}"
            );
        }
    }
    let _ = fs::remove_dir_all(&root);
}
