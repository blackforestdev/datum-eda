//! T0-C04 regression boundary (DATUM_NATIVE_TERMINAL_SPEC.md §7.1; decision
//! 027; bead dat-pan-trace-terminal-pollution-0j0): workspace shortcuts, Datum
//! telemetry, session lifecycle, and diagnostic paths cannot write to or
//! displace terminal cells, and keyboard-focus transfer flows through the one
//! authority — entry only through the deliberate terminal targets, restart/
//! detach/close/observation chrome never arming, and focus movement never
//! restarting or clearing the rolling shell session (owner decisions
//! 2026-08-14: tab-click focuses; a terminal is a rolling session cleared only
//! by the shell itself).

use super::*;
use crate::keyboard_focus::{
    KeyClass, KeyboardFocus, RouteDecision, hit_target_is_terminal_entry, key_route,
    workspace_action_should_fire,
};
use datum_gui_render::HitTarget;
use std::fs;
use std::time::{Duration, Instant};
use winit::event::ElementState;

use super::terminal_screen_authority_tests::DATUM_LIFECYCLE_PHRASES;

/// The T0-C04 focus-entry contract, restated as an EXHAUSTIVE match so adding
/// a `HitTarget` variant fails compilation here until it is classified as
/// deliberate terminal keyboard entry or not (spec §5: a control click gives
/// focus only when the resulting behavior expects terminal typing).
fn expected_terminal_entry(target: &HitTarget) -> bool {
    match target {
        // Deliberate entry: the shell-content cell rectangle, the dock
        // terminal tab (owner decision 2026-08-14), and the session actions
        // whose resulting behavior expects terminal typing.
        HitTarget::TerminalScreen
        | HitTarget::TerminalTab
        | HitTarget::TerminalSessionTab(_)
        | HitTarget::TerminalSessionNew
        | HitTarget::TerminalSessionRenameActive
        | HitTarget::TerminalSessionReattachActive => true,
        // Session-ending/suspending terminal chrome never arms focus.
        HitTarget::TerminalSessionRestartActive
        | HitTarget::TerminalSessionDetachActive
        | HitTarget::TerminalSessionCloseActive
        // Production handoffs write PTY bytes but are observation gestures.
        | HitTarget::ProductionOutputJobRun(_)
        | HitTarget::ProductionTerminalCommand(_)
        // Everything else belongs to the editor persona.
        | HitTarget::ReviewAction(_)
        | HitTarget::AuthoredObject(_)
        | HitTarget::FitBoard
        | HitTarget::FitReviewTarget
        | HitTarget::SetWorkspaceTool(_)
        | HitTarget::ReviewPrev
        | HitTarget::ReviewNext
        | HitTarget::ToggleShowAuthored
        | HitTarget::ToggleShowProposed
        | HitTarget::ToggleShowUnrouted
        | HitTarget::ToggleDimUnrelated
        | HitTarget::ToggleLayer(_)
        | HitTarget::ToggleSelectedBoardTextMirrored
        | HitTarget::ToggleSelectedBoardTextKeepUpright
        | HitTarget::ToggleSelectedBoardTextBold
        | HitTarget::CycleSelectedBoardTextRenderIntent
        | HitTarget::CycleSelectedBoardTextFamily
        | HitTarget::CycleSelectedBoardTextHAlign
        | HitTarget::CycleSelectedBoardTextVAlign
        | HitTarget::EditSelectedBoardTextRenderIntent
        | HitTarget::EditSelectedBoardTextFamily
        | HitTarget::EditSelectedBoardTextAlignment
        | HitTarget::DecreaseSelectedBoardTextHeight
        | HitTarget::IncreaseSelectedBoardTextHeight
        | HitTarget::RotateSelectedBoardTextCounterClockwise90
        | HitTarget::RotateSelectedBoardTextClockwise90
        | HitTarget::DecreaseSelectedBoardTextLineSpacing
        | HitTarget::IncreaseSelectedBoardTextLineSpacing
        | HitTarget::EditSelectedBoardTextContent
        | HitTarget::EditSelectedBoardTextHeight
        | HitTarget::EditSelectedBoardTextRotation
        | HitTarget::EditSelectedBoardTextLineSpacing
        | HitTarget::CheckFinding(_)
        | HitTarget::ProductionArtifact(_)
        | HitTarget::ProductionArtifactFile(_)
        | HitTarget::ArtifactPreviewZoomIn
        | HitTarget::ArtifactPreviewZoomOut
        | HitTarget::ArtifactPreviewReset
        | HitTarget::ArtifactPreviewViewport
        | HitTarget::ToggleArtifactPreviewGeometry
        | HitTarget::ToggleArtifactPreviewDrills
        | HitTarget::MenuTitle(_)
        | HitTarget::MenuItem { .. }
        | HitTarget::MarkingMenuItem { .. }
        | HitTarget::DockResizeHandle => false,
    }
}

fn sample_handoff() -> datum_gui_protocol::TerminalCommandHandoff {
    datum_gui_protocol::TerminalCommandHandoff {
        command_id: "datum.project.status".to_string(),
        mcp_alias: None,
        command: "datum-eda project status".to_string(),
    }
}

#[test]
fn terminal_focus_entry_is_exhaustively_classified_over_every_hit_target() {
    let id = || "t0c04".to_string();
    let samples = vec![
        HitTarget::ReviewAction(id()),
        HitTarget::AuthoredObject(id()),
        HitTarget::FitBoard,
        HitTarget::FitReviewTarget,
        HitTarget::SetWorkspaceTool(datum_gui_protocol::WorkspaceTool::Select),
        HitTarget::ReviewPrev,
        HitTarget::ReviewNext,
        HitTarget::ToggleShowAuthored,
        HitTarget::ToggleShowProposed,
        HitTarget::ToggleShowUnrouted,
        HitTarget::ToggleDimUnrelated,
        HitTarget::ToggleLayer(id()),
        HitTarget::ToggleSelectedBoardTextMirrored,
        HitTarget::ToggleSelectedBoardTextKeepUpright,
        HitTarget::ToggleSelectedBoardTextBold,
        HitTarget::CycleSelectedBoardTextRenderIntent,
        HitTarget::CycleSelectedBoardTextFamily,
        HitTarget::CycleSelectedBoardTextHAlign,
        HitTarget::CycleSelectedBoardTextVAlign,
        HitTarget::EditSelectedBoardTextRenderIntent,
        HitTarget::EditSelectedBoardTextFamily,
        HitTarget::EditSelectedBoardTextAlignment,
        HitTarget::DecreaseSelectedBoardTextHeight,
        HitTarget::IncreaseSelectedBoardTextHeight,
        HitTarget::RotateSelectedBoardTextCounterClockwise90,
        HitTarget::RotateSelectedBoardTextClockwise90,
        HitTarget::DecreaseSelectedBoardTextLineSpacing,
        HitTarget::IncreaseSelectedBoardTextLineSpacing,
        HitTarget::EditSelectedBoardTextContent,
        HitTarget::EditSelectedBoardTextHeight,
        HitTarget::EditSelectedBoardTextRotation,
        HitTarget::EditSelectedBoardTextLineSpacing,
        HitTarget::TerminalTab,
        HitTarget::TerminalSessionTab(id()),
        HitTarget::TerminalSessionNew,
        HitTarget::TerminalSessionRenameActive,
        HitTarget::TerminalSessionRestartActive,
        HitTarget::TerminalSessionDetachActive,
        HitTarget::TerminalSessionReattachActive,
        HitTarget::TerminalSessionCloseActive,
        HitTarget::TerminalScreen,
        HitTarget::CheckFinding(id()),
        HitTarget::ProductionArtifact(id()),
        HitTarget::ProductionArtifactFile(id()),
        HitTarget::ProductionOutputJobRun(sample_handoff()),
        HitTarget::ProductionTerminalCommand(sample_handoff()),
        HitTarget::ArtifactPreviewZoomIn,
        HitTarget::ArtifactPreviewZoomOut,
        HitTarget::ArtifactPreviewReset,
        HitTarget::ArtifactPreviewViewport,
        HitTarget::ToggleArtifactPreviewGeometry,
        HitTarget::ToggleArtifactPreviewDrills,
        HitTarget::MenuTitle(id()),
        HitTarget::MenuItem {
            menu: id(),
            label: id(),
        },
        HitTarget::MarkingMenuItem {
            menu_key: id(),
            slot: id(),
            label: id(),
        },
        HitTarget::DockResizeHandle,
    ];
    let mut entry_targets = 0usize;
    for target in &samples {
        assert_eq!(
            hit_target_is_terminal_entry(target),
            expected_terminal_entry(target),
            "focus-entry classification drifted for {target:?}"
        );
        entry_targets += usize::from(hit_target_is_terminal_entry(target));
    }
    assert_eq!(
        entry_targets, 6,
        "exactly the six deliberate targets may arm terminal keyboard focus"
    );
}

#[test]
fn only_escape_with_empty_input_releases_terminal_focus() {
    let all_classes = [
        KeyClass::RawPty,
        KeyClass::TerminalRenameEdit,
        KeyClass::WorkspaceHotkey,
        KeyClass::EscapeWithEmptyRename,
    ];
    for visible in [false, true] {
        for class in all_classes {
            let released = key_route(KeyboardFocus::Terminal, class, visible)
                == RouteDecision::ReleaseToEditor;
            assert_eq!(
                released,
                class == KeyClass::EscapeWithEmptyRename,
                "release-to-editor must be exactly the empty-input Escape class \
                 (got a release for {class:?}, visible={visible})"
            );
        }
        // Release is meaningless without terminal ownership: no other focus
        // owner ever produces it (an Editor/Overlay Escape must not re-route
        // key ownership).
        for focus in [KeyboardFocus::Editor, KeyboardFocus::Overlay] {
            for class in all_classes {
                assert_ne!(
                    key_route(focus, class, visible),
                    RouteDecision::ReleaseToEditor,
                    "{focus:?}/{class:?} must never produce a focus release"
                );
            }
        }
    }
}

/// PTY rows/columns for a dock height, exactly as `resize_terminal_to_dock`
/// derives them from the ONE shared solver (T0-C02).
fn dock_geometry(dock_height: u32) -> datum_gui_viewport::TerminalScreenGeometry {
    let shell = datum_gui_render::ShellLayout::for_surface(1280, 800, 1.0, Some(dock_height));
    datum_gui_viewport::terminal_screen_geometry(shell.bottom_strip.into())
}

fn test_root(tag: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("datum-terminal-t0c04-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("terminal test root should create");
    root
}

#[test]
fn focus_adjacent_operations_never_restart_or_clear_the_rolling_session() {
    // Owner decisions (bead dat-pan-trace-terminal-pollution-0j0, 2026-08-14):
    // the terminal behaves like any terminal — a rolling session, cleared only
    // by the shell ('clear'); restart-on-focus does not exist and never will.
    // Every state-side effect of a deliberate focus entry (tab click, session
    // tab click, screen click: dock activation, geometry resize, activity
    // refresh, tab sync, session activate) must leave the session identity and
    // the grid byte-identical.
    let root = test_root("rolling");
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry =
        TerminalSessionRegistry::spawn(&context).expect("spawn terminal session");
    let mut state = TerminalLaneState::default();

    // Simulated PTY-derived screen content (the one legal writer).
    let mut screen = crate::terminal_screen::TerminalScreen::default();
    screen.apply_bytes(
        &mut state,
        b"datum$ printf rolling-canary\r\nrolling-canary\r\ndatum$ ",
    );
    let original_session_id = registry.active().session_id().to_string();
    let pty_rows = state.grid_lines().to_vec();
    let pty_styled = state.grid_styled_lines().to_vec();

    // Focus-entry side effects, including the T0-C02 geometry resize at
    // several dock heights: the PTY size must track the shared solver exactly
    // while the grid stays untouched.
    for dock_height in [150, 320, 220] {
        let geometry = dock_geometry(dock_height);
        registry
            .resize_active(geometry.columns, geometry.rows)
            .expect("resize PTY from the shared terminal screen geometry");
        registry.sync_lane_tabs(&mut state);
        assert_eq!(
            (state.columns, state.rows),
            (geometry.columns, geometry.rows),
            "lane state must carry the shared-geometry PTY size at dock height {dock_height}"
        );
    }
    let _ = registry.active_activity_summary_lines(4);
    registry
        .activate(&original_session_id)
        .expect("re-activating the active session is a focus no-op");
    registry.sync_lane_tabs(&mut state);

    // A second session opened and the first re-activated (session-tab click).
    registry
        .spawn_and_activate(&context)
        .expect("spawn second terminal session");
    registry.sync_lane_tabs(&mut state);
    registry
        .activate(&original_session_id)
        .expect("activate first session again");
    registry.sync_lane_tabs(&mut state);

    assert_eq!(
        state.grid_lines(),
        pty_rows,
        "focus-adjacent operations must never add, remove, or edit grid rows"
    );
    assert_eq!(
        state.grid_styled_lines(),
        pty_styled,
        "focus-adjacent operations must never restyle grid rows"
    );
    let first_tab = state
        .tabs
        .iter()
        .find(|tab| tab.session_id == original_session_id)
        .expect("original session must still exist");
    assert_eq!(
        first_tab.restart_count, 0,
        "no focus-adjacent operation may restart the session"
    );
    assert!(
        first_tab.previous_session_id.is_none(),
        "the session identity must be unbroken (no restart lineage)"
    );
    assert!(first_tab.attached, "re-activated session must be attached");
    assert_eq!(first_tab.status, "running");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn narration_event_classes_route_to_console_and_leave_grid_byte_identical() {
    // T0-C04 telemetry boundary: drive every narration-producing event class
    // through the real session registry against ONE workspace UI state —
    // session open/rename/detach/restart/terminate/close, activity refresh,
    // clipboard narration, PTY-failure narration, pan/diagnostic traces, and
    // production-status narration. The console sink receives every routed
    // message; the terminal grid stays byte-identical; lifecycle truth lives
    // in chrome fields (status/tabs/activity), never in cells.
    let root = test_root("narration");
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry =
        TerminalSessionRegistry::spawn(&context).expect("spawn terminal session");
    let mut ui = workspace_ui_state();

    let mut screen = crate::terminal_screen::TerminalScreen::default();
    screen.apply_bytes(
        &mut ui.terminal,
        b"datum$ printf narration-canary\r\nnarration-canary\r\ndatum$ ",
    );
    registry.sync_lane_tabs(&mut ui.terminal);
    let pty_rows = ui.terminal.grid_lines().to_vec();
    let pty_styled = ui.terminal.grid_styled_lines().to_vec();
    let mut routed: Vec<String> = Vec::new();
    let mut narrate = |ui: &mut datum_gui_protocol::WorkspaceUiState, line: String| {
        // The same capability-limited production route used by
        // Runtime::log_review_event; its type cannot reach terminal state.
        crate::terminal_narration::route_gui_narration(&mut ui.console, line.clone());
        routed.push(line);
    };

    // Session lifecycle classes.
    let second_id = registry
        .spawn_and_activate(&context)
        .expect("open second terminal session")
        .to_string();
    narrate(&mut ui, format!("opened terminal session {second_id}"));
    registry.sync_lane_tabs(&mut ui.terminal);
    registry
        .rename(&second_id, "bench")
        .expect("rename active terminal session");
    narrate(&mut ui, "renamed active terminal session bench".to_string());
    registry.sync_lane_tabs(&mut ui.terminal);
    registry
        .detach_active(&mut ui.terminal)
        .expect("detach active terminal session");
    narrate(&mut ui, "detached active terminal session".to_string());
    registry
        .restart_active(&mut ui.terminal, &context)
        .expect("restart active terminal session");
    narrate(&mut ui, "terminal session restarted".to_string());
    registry
        .terminate_active(&mut ui.terminal)
        .expect("terminate active terminal session");
    narrate(&mut ui, "terminal exited 0".to_string());
    registry
        .close_active(&mut ui.terminal)
        .expect("close active terminal session");
    narrate(&mut ui, "terminal session ended".to_string());
    registry.sync_lane_tabs(&mut ui.terminal);

    // Activity telemetry refresh: summary lines live in chrome state only.
    let _ = registry.active_activity_summary_lines(4);
    registry.sync_lane_tabs(&mut ui.terminal);

    // Clipboard narration classes (Runtime clipboard paths).
    narrate(&mut ui, "terminal scrollback copied".to_string());
    narrate(&mut ui, "clipboard copy failed".to_string());
    narrate(&mut ui, "clipboard cut failed".to_string());
    narrate(&mut ui, "clipboard paste failed".to_string());

    // PTY/transport failure narration classes.
    narrate(&mut ui, "terminal write failed: broken pipe".to_string());
    narrate(&mut ui, "terminal status response failed: broken pipe".to_string());
    narrate(&mut ui, "terminal focus report failed: broken pipe".to_string());
    narrate(&mut ui, "terminal mouse report failed: broken pipe".to_string());
    narrate(
        &mut ui,
        "terminal session is detached; activate the tab to reattach".to_string(),
    );

    // Pan/diagnostic traces route to the diagnostic log, never state.
    crate::append_gui_verbose_diagnostic_line(
        "pan key physical=Code(Space) state=Pressed consumed=true",
    );
    crate::append_gui_verbose_diagnostic_line("terminal resize begin 158x9");

    // Production-status narration classes.
    narrate(&mut ui, "workspace scene/status refreshed".to_string());
    narrate(
        &mut ui,
        "production status refresh failed: engine offline".to_string(),
    );

    assert_eq!(
        ui.terminal.grid_lines(),
        pty_rows,
        "narration-producing event classes must leave the grid byte-identical"
    );
    assert_eq!(
        ui.terminal.grid_styled_lines(),
        pty_styled,
        "narration-producing event classes must not restyle grid rows"
    );
    for line in &routed {
        assert!(
            ui.console.lines.contains(line),
            "console sink must receive the routed narration {line:?}"
        );
    }
    for line in ui.terminal.grid_lines() {
        for phrase in DATUM_LIFECYCLE_PHRASES {
            assert!(
                !line.contains(phrase),
                "terminal grid row {line:?} carries non-PTY telemetry {phrase:?}"
            );
        }
    }
    // Lifecycle/activity truth lives in chrome fields, not cells.
    assert_eq!(ui.terminal.tabs.len(), 1, "close must leave one session tab");
    for summary_line in &ui.terminal.activity_summary {
        assert!(
            !ui.terminal
                .grid_lines()
                .iter()
                .any(|row| row.contains(summary_line.as_str())),
            "activity summary line {summary_line:?} must not appear in the grid"
        );
    }
    let _ = fs::remove_dir_all(&root);
}

/// Pump PTY output through the production drain body (see the T0-C03 canary),
/// accumulating terminal status-response write-backs, until `stop` observes
/// the goal state or the deadline passes.
fn drain_output(
    registry: &mut TerminalSessionRegistry,
    state: &mut TerminalLaneState,
    response_bytes_written: &mut usize,
    deadline: Instant,
    stop: &mut dyn FnMut(&TerminalLaneState) -> bool,
) -> bool {
    while Instant::now() < deadline {
        let Ok(event) = registry
            .active()
            .rx
            .recv_timeout(Duration::from_millis(25))
        else {
            continue;
        };
        let TerminalEvent::Output(bytes) = event else {
            return false;
        };
        let _ = crate::terminal_session_events::record_terminal_output_event(
            registry.active(),
            &bytes,
        );
        let responses = registry
            .active_screen_mut()
            .apply_bytes_with_responses(state, &bytes);
        for response in responses {
            *response_bytes_written += response.len();
            let _ = registry.active().write_bytes(&response);
        }
        let _ = registry.active_activity_summary_lines(4);
        if stop(state) {
            return true;
        }
    }
    false
}

/// Recorded keyboard-origin input bytes: total input minus the tracked status
/// responses — the exact-once accounting the T0-C03 canary established.
fn recorded_input_bytes(registry: &TerminalSessionRegistry) -> usize {
    let event_log = fs::read_to_string(registry.active_event_log_path())
        .expect("read terminal session event log");
    event_log
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|event| event["event"] == "terminal_io" && event["direction"] == "input")
        .map(|event| event["byte_count"].as_u64().unwrap_or(0) as usize)
        .sum()
}

#[test]
fn workspace_hotkeys_reach_the_pty_exactly_once_and_editor_focus_writes_zero_bytes() {
    // T0-C04 shortcut boundary against a REAL shell through the production
    // session machinery: under Editor focus the raw-input gate blocks every
    // hotkey (exact-once accounting proves zero keyboard bytes reach the
    // child); under Terminal focus each hotkey keystroke reaches the child
    // exactly once as its literal byte (proven by the echoed prompt line) and
    // never restarts or clears the rolling session.
    let root = test_root("hotkeys");
    let context = TerminalLaunchContext::for_project_root(&root);
    let mut registry =
        TerminalSessionRegistry::spawn(&context).expect("spawn terminal session");
    let mut state = TerminalLaneState::default();
    let geometry = dock_geometry(220);
    registry
        .resize_active(geometry.columns, geometry.rows)
        .expect("size PTY from the shared terminal screen geometry");
    registry.sync_lane_tabs(&mut state);
    let mut response_bytes_written = 0usize;

    // Wait for the real prompt, then let start-up output settle.
    assert!(
        drain_output(
            &mut registry,
            &mut state,
            &mut response_bytes_written,
            Instant::now() + Duration::from_secs(8),
            &mut |state| !state.grid_lines().is_empty(),
        ),
        "real shell prompt output must become visible in the terminal grid"
    );
    let _ = drain_output(
        &mut registry,
        &mut state,
        &mut response_bytes_written,
        Instant::now() + Duration::from_millis(400),
        &mut |_| false,
    );

    // Phase A — Editor focus: the production raw-input gate is the key_route
    // decision; no hotkey may pass it, so zero keyboard bytes are written.
    let hotkeys = ["s", "b", "v", "m", "x", "r", "f", "t", "z", "c", "[", "]"];
    for key in hotkeys {
        for visible in [false, true] {
            assert!(workspace_action_should_fire(
                KeyboardFocus::Editor,
                visible,
                ElementState::Pressed,
                false,
            ));
            assert_ne!(
                key_route(KeyboardFocus::Editor, KeyClass::RawPty, visible),
                RouteDecision::Terminal,
                "Editor focus must never route hotkey {key:?} to the PTY"
            );
        }
    }
    assert_eq!(
        recorded_input_bytes(&registry),
        response_bytes_written,
        "with Editor focus, recorded input must be exactly the tracked status \
         responses — zero keyboard bytes"
    );

    // Phase B — Terminal focus: each hotkey keystroke goes through the
    // production write path exactly once (no Enter — the shell echoes the
    // pending line, executing nothing).
    assert_eq!(
        key_route(KeyboardFocus::Terminal, KeyClass::RawPty, true),
        RouteDecision::Terminal
    );
    assert!(!workspace_action_should_fire(
        KeyboardFocus::Terminal,
        true,
        ElementState::Pressed,
        false,
    ));
    let mut keyboard_bytes = 0usize;
    for key in hotkeys {
        registry
            .active()
            .write_bytes(key.as_bytes())
            .expect("write hotkey byte through the production input path");
        keyboard_bytes += key.len();
    }
    let expected_echo: String = hotkeys.concat();
    assert!(
        drain_output(
            &mut registry,
            &mut state,
            &mut response_bytes_written,
            Instant::now() + Duration::from_secs(8),
            &mut |state| state
                .grid_lines()
                .iter()
                .any(|line| line.contains(&expected_echo)),
        ),
        "hotkey bytes must echo on the shell prompt line; rows: {:?}",
        state.grid_lines()
    );
    let echo_rows = state
        .grid_lines()
        .iter()
        .filter(|line| line.contains(&expected_echo))
        .count();
    assert_eq!(
        echo_rows, 1,
        "the hotkey payload must appear exactly once (exactly-once delivery)"
    );
    assert_eq!(
        recorded_input_bytes(&registry),
        keyboard_bytes + response_bytes_written,
        "recorded input bytes must equal written hotkey bytes plus tracked \
         responses exactly (no duplicated or dropped keystrokes)"
    );

    // Shortcut traffic never restarts or clears the rolling session, and no
    // Datum telemetry entered the cells alongside it.
    let tab = &state.tabs[0];
    assert_eq!(tab.restart_count, 0);
    assert_eq!(tab.status, "running");
    for line in state.grid_lines() {
        for phrase in DATUM_LIFECYCLE_PHRASES {
            assert!(
                !line.contains(phrase),
                "terminal grid row {line:?} carries non-PTY telemetry {phrase:?}"
            );
        }
    }
    let _ = fs::remove_dir_all(&root);
}

fn workspace_ui_state() -> datum_gui_protocol::WorkspaceUiState {
    datum_gui_protocol::WorkspaceUiState {
        active_dock_tab: Some(datum_gui_protocol::DockTab::Terminal),
        active_menu: None,
        marking_menu: None,
        dock_height_px: 220,
        hovered_object: None,
        cursor_pos: None,
        crosshair_style: datum_gui_protocol::CrosshairStyle::default(),
        filters: datum_gui_protocol::WorkspaceFilterState {
            show_authored: true,
            show_proposed: true,
            show_unrouted: true,
            dim_unrelated: false,
            active_layer_id: None,
            layer_visibility: std::collections::BTreeMap::new(),
        },
        terminal: TerminalLaneState::default(),
        console: datum_gui_protocol::ConsoleLaneState::default(),
        artifact_preview: datum_gui_protocol::ArtifactPreviewViewportState::default(),
        layout: datum_gui_protocol::WorkspaceLayout::default(),
    }
}
