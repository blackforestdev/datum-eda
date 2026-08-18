use super::*;

#[path = "terminal_hit_ownership_tests.rs"]
mod terminal_hit_ownership_tests;

#[test]
fn terminal_dock_omits_persistent_header_and_routine_diagnostics() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;
    state.ui.terminal.title = Some("codex: substrate review".to_string());
    state.ui.terminal.current_working_directory =
        Some("/home/user/Datum Project/layout".to_string());
    state.ui.terminal.bell_count = 2;
    state.ui.terminal.columns = 132;
    state.ui.terminal.rows = 37;
    state.ui.terminal.application_cursor_keys = true;
    state.ui.terminal.application_keypad = true;
    state.ui.terminal.focus_event_reporting = true;
    state.ui.terminal.mouse_reporting_mode = Some("button_event".to_string());
    state.ui.terminal.mouse_coordinate_encoding = Some("sgr".to_string());
    state.ui.terminal.tabs = vec![
        datum_gui_protocol::TerminalTabState {
            session_id: "terminal-a".to_string(),
            previous_session_id: Some("terminal-a-prev".to_string()),
            label: "layout shell".to_string(),
            event_log_path: "/tmp/datum-terminal-a.jsonl".to_string(),
            activity_event_count: 3,
            activity_summary: vec!["#1 check datum.check.run in:0B out:12B".to_string()],
            active: true,
            attached: true,
            status: "running".to_string(),
            restart_count: 1,
        },
        datum_gui_protocol::TerminalTabState {
            session_id: "terminal-b".to_string(),
            previous_session_id: None,
            label: "fab shell".to_string(),
            event_log_path: "/tmp/datum-terminal-b.jsonl".to_string(),
            activity_event_count: 0,
            activity_summary: Vec::new(),
            active: false,
            attached: false,
            status: "running".to_string(),
            restart_count: 0,
        },
    ];

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );

    for diagnostic in [
        "PROJECT TERMINAL",
        "COPY SCROLLBACK CTRL+SHIFT+C",
        "SCROLL SHIFT+PGUP/PGDN",
        "PASTE CTRL+V",
        "SHELL SESSION",
        "codex: substrate review",
        "BELL 2",
        "CWD /home/user/Datum Project/layout",
        "SIZE 132x37",
        "FOCUS EVENTS",
        "APP CURSOR",
        "APP KEYPAD",
        "MOUSE BUTTON_EVENT SGR",
    ] {
        assert!(
            !prepared
                .text_runs
                .iter()
                .any(|run| run.text.contains(diagnostic)),
            "routine terminal diagnostic must not consume persistent chrome: {diagnostic}"
        );
    }
    for redundant in ["SESSIONS", "+NEW", "RENAME", "RESTART", "CLOSE"] {
        assert!(
            !prepared.text_runs.iter().any(|run| run.text == redundant),
            "terminal dock must not retain redundant session-menu item {redundant}"
        );
    }
    let session_region = prepared.hit_regions.iter().find(|region| {
        matches!(
            &region.target,
            HitTarget::TerminalSessionTab(session_id) if session_id == "terminal-b"
        )
    });
    assert!(
        session_region.is_some(),
        "terminal dock should expose clickable session tab hit region"
    );
    let rect = session_region.unwrap().rect;
    assert!(matches!(
        prepared.hit_test(rect.x + 1.0, rect.y + 1.0),
        Some(HitTarget::TerminalSessionTab(session_id)) if session_id == "terminal-b"
    ));
    assert!(
        prepared
            .hit_regions
            .iter()
            .any(|region| region.target == HitTarget::TerminalSessionNew)
    );
    assert_eq!(
        prepared
            .hit_regions
            .iter()
            .filter(|region| matches!(region.target, HitTarget::TerminalSessionTab(_)))
            .count(),
        state.ui.terminal.tabs.len(),
        "each session must have exactly one tab target in the top strip"
    );
    for command_id in [
        "datum.journal.list",
        "datum.journal.undo",
        "datum.journal.redo",
    ] {
        assert!(
            !prepared.hit_regions.iter().any(|region| matches!(
                &region.target,
                HitTarget::ProductionTerminalCommand(handoff)
                    if handoff.command_id == command_id
            )),
            "terminal dock must not expose {command_id} as a CLI handoff"
        );
    }
}

#[test]
fn terminal_screen_rect_is_the_dedicated_content_hit_target() {
    // T0-C02: the exact visible cell rectangle is exposed as its own hit
    // target, sized from the one shared geometry, inside the dock content.
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );

    let shell = ShellLayout::for_window(1280, 800, Some(260));
    let geometry = datum_gui_viewport::terminal_screen_geometry(shell.bottom_strip.into());
    let region = prepared
        .hit_regions
        .iter()
        .find(|region| region.target == HitTarget::TerminalScreen)
        .expect("terminal dock should expose the screen cell-rectangle hit target");
    assert_eq!(region.rect, geometry.screen.into());
    let content: RectPx = geometry.content.into();
    assert!(
        region.rect.x >= content.x
            && region.rect.y >= content.y
            && region.rect.x + region.rect.width <= content.x + content.width
            && region.rect.y + region.rect.height <= content.y + content.height,
        "screen rect must never overflow the dock content rect"
    );
    let center = prepared.hit_test(
        region.rect.x + region.rect.width * 0.5,
        region.rect.y + region.rect.height * 0.5,
    );
    assert_eq!(center, Some(&HitTarget::TerminalScreen));
}

#[test]
fn terminal_top_boundary_is_the_visible_resize_cursor_target() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    let shell = ShellLayout::for_window(1280, 800, Some(260));
    let handle = prepared
        .hit_regions
        .iter()
        .find(|region| region.target == HitTarget::DockResizeHandle)
        .expect("open terminal dock must expose its top resize boundary");

    assert_eq!(handle.rect.y, shell.bottom_strip.y);
    assert_eq!(handle.rect.height, 6.0);
    assert_eq!(handle.rect.width, shell.bottom_strip.width);
    assert_eq!(
        prepared.hit_test(
            handle.rect.x + handle.rect.width * 0.5,
            handle.rect.y + handle.rect.height * 0.5,
        ),
        Some(&HitTarget::DockResizeHandle)
    );
}

#[test]
fn terminal_lane_draws_exactly_the_shared_geometry_row_count() {
    // T0-C02: the renderer draws exactly the rows the shared geometry solved
    // — the same count the PTY resize path derives — so drawn rows always
    // equal PTY rows and the screen never spills below the content rect.
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;
    *state.ui.terminal.pty_grid_mut().lines = (0..40)
        .map(|index| format!("pty-canary-{index:02}"))
        .collect();

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );

    let shell = ShellLayout::for_window(1280, 800, Some(260));
    let geometry = datum_gui_viewport::terminal_screen_geometry(shell.bottom_strip.into());
    let drawn = prepared
        .text_runs
        .iter()
        .filter(|run| run.text.starts_with("pty-canary-"))
        .count();
    assert_eq!(
        drawn, geometry.rows as usize,
        "drawn terminal rows must equal the shared-geometry row authority"
    );
    let screen: RectPx = geometry.screen.into();
    for run in prepared
        .text_runs
        .iter()
        .filter(|run| run.text.starts_with("pty-canary-"))
    {
        assert!(
            run.y >= screen.y && run.y < screen.y + screen.height,
            "terminal grid row at y={} must stay inside the screen rect",
            run.y
        );
    }
}

#[test]
fn terminal_lane_renders_no_activity_summary_rows() {
    // T0-C02 / owner directive (dat-pan-trace-terminal-pollution-0j0):
    // application summaries consume zero terminal rows and zero lane space.
    // Activity data may exist in state, but the lane renders none of it.
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;
    state.ui.terminal.activity_summary =
        vec!["#3 command datum.artifact.generate in:7B out:12B".to_string()];

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );

    assert!(
        !prepared
            .text_runs
            .iter()
            .any(|run| run.text.contains("ACTIVITY")),
        "terminal lane must not render an activity block"
    );
    assert!(
        !prepared
            .text_runs
            .iter()
            .any(|run| run.text.contains("datum.artifact.generate")),
        "terminal lane must not render activity summary lines"
    );
}

#[test]
fn terminal_dock_does_not_render_output_lane_findings() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 300;
    state.checks = datum_gui_protocol::check_run_review_state_from_json(
        r#"{
          "contract": "check_run_v1",
          "check_run_id": "00000000-0000-0000-0000-00000000chk2",
          "profile_id": "standards",
          "status": "error",
          "finding_count": 1,
          "findings": [{
            "finding_id": "00000000-0000-0000-0000-00000000f002",
            "source": "drc",
            "code": "pad_mask_expansion_missing",
            "severity": "error",
            "fingerprint": "sha256:process-aperture",
            "domain": "drc",
            "rule_id": "process_aperture_policy",
            "status": "active",
            "evidence": [{
              "evidence_kind": "standards_basis",
              "basis_id": "datum.process_aperture_and_geometry.current"
            }]
          }]
        }"#,
    )
    .expect("check-run fixture should decode");

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );

    assert!(
        !prepared
            .text_runs
            .iter()
            .any(|run| run.text.contains("BASIS DATUM.PROCESS_APERTURE")),
        "Phase 1 terminal dock must not render the retired Output lane"
    );
}

#[test]
fn dock_exposes_terminal_tab_only() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );

    // The tab uses the session's own (lower/mixed-case) name, not an uppercased
    // constant; the fixture terminal has no title, so it reads "terminal".
    let terminal_label = prepared
        .text_runs
        .iter()
        .find(|run| run.text == "terminal")
        .expect("terminal tab label");
    assert_eq!(
        terminal_label.color, TEXT_PRIMARY,
        "the active terminal lane tab should render in the active color"
    );
    assert!(
        !prepared.text_runs.iter().any(|run| run.text == "OUTPUT"),
        "Phase 1 dock must not render an Output tab"
    );
}

#[test]
fn terminal_dock_renders_styled_terminal_spans_as_colored_runs() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;
    *state.ui.terminal.pty_grid_mut().lines = vec!["ERR ok".to_string()];
    *state.ui.terminal.pty_grid_mut().styled_lines = vec![datum_gui_protocol::TerminalStyledLine {
        text: "ERR ok".to_string(),
        spans: vec![datum_gui_protocol::TerminalStyleSpan {
            start: 0,
            end: 3,
            fg: Some("red".to_string()),
            bg: None,
            bold: true,
            dim: false,
            italic: false,
            underline: false,
            overline: false,
            blink: false,
            strikethrough: false,
            conceal: false,
            inverse: false,
        }],
    }];

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );

    let line_run = prepared
        .text_runs
        .iter()
        .find(|run| run.text == "ERR ok")
        .expect("styled terminal line should render as one shaped text run");
    assert_eq!(
        line_run
            .rich_spans
            .iter()
            .map(|span| span.text.as_str())
            .collect::<Vec<_>>(),
        vec!["ERR", " ok"]
    );
    assert_ne!(
        line_run.rich_spans[0].color, line_run.rich_spans[1].color,
        "styled terminal output should retain distinct colors in one shaped line"
    );
}

#[test]
fn terminal_dock_uses_inverse_background_as_visible_terminal_span_color() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;
    *state.ui.terminal.pty_grid_mut().lines = vec!["INV ok".to_string()];
    *state.ui.terminal.pty_grid_mut().styled_lines = vec![datum_gui_protocol::TerminalStyledLine {
        text: "INV ok".to_string(),
        spans: vec![datum_gui_protocol::TerminalStyleSpan {
            start: 0,
            end: 3,
            fg: Some("red".to_string()),
            bg: Some("green".to_string()),
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            overline: false,
            blink: false,
            strikethrough: false,
            conceal: false,
            inverse: true,
        }],
    }];

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );

    let line_run = prepared
        .text_runs
        .iter()
        .find(|run| run.text == "INV ok")
        .expect("inverse terminal line should render as one shaped text run");
    assert_eq!(line_run.rich_spans.len(), 2);
    assert_ne!(
        line_run.rich_spans[0].color, line_run.rich_spans[1].color,
        "inverse/background terminal metadata should affect visible terminal color"
    );
}

#[test]
fn terminal_dock_renders_protocol_screen_cursor_when_visible() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;
    *state.ui.terminal.pty_grid_mut().lines = vec!["prompt".to_string()];
    state.ui.terminal.screen_cursor_row = 0;
    state.ui.terminal.screen_cursor_col = 6;
    state.ui.terminal.screen_cursor_visible = true;

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let visible = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    state.ui.terminal.screen_cursor_visible = false;
    let hidden = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );

    assert_eq!(
        visible.panel_vertices().len(),
        hidden.panel_vertices().len() + 24,
        "unfocused PTY cursor should add a four-quad hollow outline"
    );
}

#[test]
fn terminal_dock_renders_protocol_cursor_shape() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;
    *state.ui.terminal.pty_grid_mut().lines = vec!["prompt".to_string()];
    state.ui.terminal.screen_cursor_row = 0;
    state.ui.terminal.screen_cursor_col = 6;
    state.ui.terminal.screen_cursor_visible = true;
    state.ui.focus = datum_gui_protocol::ApplicationFocus::Terminal;
    state.ui.terminal.screen_cursor_style = Some("steady_bar".to_string());

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let bar = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    state.ui.terminal.screen_cursor_style = None;
    let block = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );

    assert_ne!(
        bar.panel_vertices(),
        block.panel_vertices(),
        "child-selected bar geometry must not be replaced by a block cursor"
    );
}

#[test]
fn terminal_dock_suppresses_protocol_screen_cursor_when_hidden() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;
    *state.ui.terminal.pty_grid_mut().lines = vec!["prompt".to_string()];
    state.ui.terminal.screen_cursor_row = 0;
    state.ui.terminal.screen_cursor_col = 6;
    state.ui.terminal.screen_cursor_visible = true;

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let visible = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    state.ui.terminal.screen_cursor_visible = false;
    let hidden = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );

    assert!(
        hidden.panel_vertices().len() < visible.panel_vertices().len(),
        "terminal dock should honor hidden cursor mode"
    );
}

#[test]
fn terminal_dock_renders_exact_global_shutdown_survivor_identity() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;
    state.ui.terminal.application_shutdown_blocked =
        Some("shutdown blocked: agent shell: pid=4242 pgid=4200 sid=4100".to_string());

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );

    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| { run.text.contains("pid=4242 pgid=4200 sid=4100") })
    );
    assert!(
        prepared
            .hit_regions
            .iter()
            .any(|region| { region.target == HitTarget::TerminalSessionRetryTermination })
    );
    assert!(
        prepared
            .hit_regions
            .iter()
            .any(|region| { region.target == HitTarget::TerminalShutdownCancel })
    );
}
