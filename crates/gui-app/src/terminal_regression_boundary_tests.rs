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
    KeyClass, RouteDecision, hit_target_is_terminal_entry, key_route, workspace_action_should_fire,
};
use datum_gui_protocol::{ApplicationFocus as KeyboardFocus, PaneId};
use datum_gui_render::HitTarget;
use std::fs;
use std::time::{Duration, Instant};
use winit::event::ElementState;

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
        | HitTarget::TerminalPaneScreen(_)
        | HitTarget::TerminalTab
        | HitTarget::TerminalSessionTab(_)
        | HitTarget::TerminalSessionNew => true,
        // Session-ending/suspending terminal chrome never arms focus.
        HitTarget::TerminalSessionClose(_)
        | HitTarget::TerminalSessionTerminateActive
        | HitTarget::TerminalSessionForceKillActive
        | HitTarget::TerminalSessionRetryTermination
        | HitTarget::TerminalShutdownCancel
        | HitTarget::TerminalClipboardCopy
        | HitTarget::TerminalClipboardPaste
        | HitTarget::TerminalProfileNext
        | HitTarget::TerminalThemeNext
        | HitTarget::TerminalLinkCopy
        | HitTarget::TerminalLinkOpen
        | HitTarget::TerminalLinkConfirmOpen
        | HitTarget::TerminalLinkCancel
        | HitTarget::TerminalClipboardConfirmWrite
        | HitTarget::TerminalClipboardCancelWrite
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
        | HitTarget::DockResizeHandle
        | HitTarget::TerminalSplitDivider(_) => false,
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
        HitTarget::TerminalSessionClose(id()),
        HitTarget::TerminalSessionNew,
        HitTarget::TerminalSessionTerminateActive,
        HitTarget::TerminalSessionForceKillActive,
        HitTarget::TerminalSessionRetryTermination,
        HitTarget::TerminalShutdownCancel,
        HitTarget::TerminalClipboardCopy,
        HitTarget::TerminalClipboardPaste,
        HitTarget::TerminalLinkCopy,
        HitTarget::TerminalLinkOpen,
        HitTarget::TerminalLinkConfirmOpen,
        HitTarget::TerminalLinkCancel,
        HitTarget::TerminalClipboardConfirmWrite,
        HitTarget::TerminalClipboardCancelWrite,
        HitTarget::TerminalScreen,
        HitTarget::TerminalPaneScreen(id()),
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
        HitTarget::TerminalSplitDivider(Vec::new()),
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
        entry_targets, 5,
        "exactly the five deliberate targets may arm terminal keyboard focus"
    );
}

#[test]
fn only_terminal_escape_release_returns_focus_to_the_editor() {
    let all_classes = [
        KeyClass::RawPty,
        KeyClass::WorkspaceHotkey,
        KeyClass::TerminalFocusExit,
    ];
    for visible in [false, true] {
        for class in all_classes {
            let released = key_route(KeyboardFocus::Terminal, class, visible)
                == RouteDecision::ReleaseToEditor;
            assert_eq!(
                released,
                class == KeyClass::TerminalFocusExit,
                "release-to-editor must be exactly the terminal Escape-release class \
                 (got a release for {class:?}, visible={visible})"
            );
        }
        // Release is meaningless without terminal ownership: no other focus
        // owner ever produces it (an Editor/Overlay Escape must not re-route
        // key ownership).
        for focus in [KeyboardFocus::Editor(PaneId(0)), KeyboardFocus::Overlay] {
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
    let root =
        std::env::temp_dir().join(format!("datum-terminal-t0c04-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("terminal test root should create");
    root
}

/// Pump PTY output through the production drain body (see the T0-C03 canary),
/// accumulating terminal status-response write-backs, until `stop` observes
/// the goal state or the deadline passes.
fn drain_output(
    registry: &mut TerminalSessionRegistry,
    state: &mut TerminalLaneState,
    response_bytes_written: &mut usize,
    deadline: Instant,
    stop: &mut dyn FnMut(&TerminalSessionRegistry) -> bool,
) -> bool {
    while Instant::now() < deadline {
        let Ok(event) = registry
            .active()
            .recv_event_timeout(Duration::from_millis(25))
        else {
            continue;
        };
        let TerminalEvent::Output(bytes) = event else {
            return false;
        };
        let _ =
            crate::terminal_session_events::record_terminal_output_event(registry.active(), &bytes);
        let update = registry
            .active_core_mut()
            .apply_output(state, &bytes)
            .expect("TerminalCore must accept PTY output");
        for response in update.replies {
            *response_bytes_written += response.len();
            let _ = registry.active().write_bytes(&response);
        }
        let _ = registry.active_activity_summary_lines(4);
        if stop(registry) {
            return true;
        }
    }
    false
}

/// Recorded keyboard-origin input bytes: total input minus the tracked status
/// responses — the exact-once accounting the T0-C03 canary established.
fn recorded_input_bytes(registry: &TerminalSessionRegistry) -> usize {
    let event_log = crate::terminal_session_events::io_event_log::read_event_log_family_text(
        &registry.active_event_log_path(),
    );
    event_log
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|event| event["event"] == "terminal_io" && event["direction"] == "input_accepted")
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
    let mut registry = TerminalSessionRegistry::spawn(&context).expect("spawn terminal session");
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
            &mut |registry| !registry.test_active_text().is_empty(),
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
                KeyboardFocus::Editor(PaneId(0)),
                visible,
                ElementState::Pressed,
                false,
            ));
            assert_ne!(
                key_route(KeyboardFocus::Editor(PaneId(0)), KeyClass::RawPty, visible),
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
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            match registry.active().write_bytes(key.as_bytes()) {
                Ok(()) => break,
                Err(error)
                    if error.to_string().contains("queue is busy") && Instant::now() < deadline =>
                {
                    std::thread::yield_now();
                }
                Err(error) => {
                    panic!("write hotkey byte through the production input path: {error}")
                }
            }
        }
        keyboard_bytes += key.len();
    }
    let expected_echo: String = hotkeys.concat();
    assert!(
        drain_output(
            &mut registry,
            &mut state,
            &mut response_bytes_written,
            Instant::now() + Duration::from_secs(8),
            &mut |registry| registry.test_active_text().contains(&expected_echo),
        ),
        "hotkey bytes must echo on the shell prompt line; rows: {:?}",
        registry.test_active_text()
    );
    let echo_text = registry.test_active_text();
    let echo_rows = echo_text
        .lines()
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

    // Shortcut traffic never restarts or clears the rolling session.
    let tab = &state.tabs[0];
    assert_eq!(tab.restart_count, 0);
    assert_eq!(tab.status, "running");
    let _ = fs::remove_dir_all(&root);
}
