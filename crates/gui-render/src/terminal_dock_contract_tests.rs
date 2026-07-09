use super::*;

#[test]
fn terminal_dock_surfaces_copy_and_paste_shortcuts() {
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

    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.text.contains("codex: substrate review")),
        "terminal dock should surface PTY-provided OSC title state"
    );
    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.text.contains("BELL 2")),
        "terminal dock should surface PTY bell alert count"
    );
    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.text.contains("CWD /home/user/Datum Project/layout")),
        "terminal dock should surface PTY-provided OSC 7 current working directory"
    );
    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.text.contains("SIZE 132x37")),
        "terminal dock should surface active PTY geometry"
    );
    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.text.contains("FOCUS EVENTS")),
        "terminal dock should surface PTY focus-event reporting mode"
    );
    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.text.contains("APP CURSOR")),
        "terminal dock should surface DEC application cursor-key mode"
    );
    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.text.contains("APP KEYPAD")),
        "terminal dock should surface DEC application keypad mode"
    );
    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.text.contains("MOUSE BUTTON_EVENT SGR")),
        "terminal dock should surface PTY mouse reporting mode"
    );
    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.text.contains("COPY SCROLLBACK CTRL+SHIFT+C")),
        "terminal dock should expose its native scrollback copy shortcut"
    );
    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.text.contains("PASTE CTRL+V")),
        "terminal dock should expose its paste shortcut"
    );
    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.text.contains("SCROLL SHIFT+PGUP/PGDN")),
        "terminal dock should expose keyboard scrollback shortcuts"
    );
    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.text.contains("[layout shell R1]")),
        "terminal dock should render active terminal session restart count"
    );
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
    for target in [
        HitTarget::TerminalSessionNew,
        HitTarget::TerminalSessionRenameActive,
        HitTarget::TerminalSessionRestartActive,
        HitTarget::TerminalSessionDetachActive,
        HitTarget::TerminalSessionCloseActive,
    ] {
        assert!(
            prepared
                .hit_regions
                .iter()
                .any(|region| region.target == target),
            "terminal dock should expose {target:?}"
        );
    }
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

    state.ui.terminal.rename_session_id = Some("terminal-a".to_string());
    state.ui.terminal.input = "layout edit".to_string();
    state.ui.terminal.cursor = 6;
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
            .any(|run| run.text.contains("[layout| edit]")),
        "terminal dock should render inline tab rename editor"
    );
    assert!(
        prepared
            .text_runs
            .iter()
            .any(|run| run.text.contains("ENTER SAVE  ESC CANCEL")),
        "terminal dock should render rename save/cancel affordance"
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

    let terminal_label = prepared
        .text_runs
        .iter()
        .find(|run| run.text == "TERMINAL")
        .expect("TERMINAL tab label");
    assert_eq!(
        terminal_label.color, TEXT_PRIMARY,
        "TERMINAL should render as the active command lane"
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
    state.ui.terminal.lines = vec!["ERR ok".to_string()];
    state.ui.terminal.styled_lines = vec![datum_gui_protocol::TerminalStyledLine {
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

    let err_run = prepared
        .text_runs
        .iter()
        .find(|run| run.text == "ERR")
        .expect("styled terminal span should render as its own text run");
    let ok_run = prepared
        .text_runs
        .iter()
        .find(|run| run.text == " ok")
        .expect("unstyled terminal suffix should render as its own text run");
    assert_ne!(
        err_run.color, ok_run.color,
        "styled terminal output should not collapse to one default color"
    );
}

#[test]
fn terminal_dock_uses_inverse_background_as_visible_terminal_span_color() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;
    state.ui.terminal.lines = vec!["INV ok".to_string()];
    state.ui.terminal.styled_lines = vec![datum_gui_protocol::TerminalStyledLine {
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

    let inv_run = prepared
        .text_runs
        .iter()
        .find(|run| run.text == "INV")
        .expect("inverse terminal span should render separately");
    let ok_run = prepared
        .text_runs
        .iter()
        .find(|run| run.text == " ok")
        .expect("unstyled terminal suffix should render separately");
    assert_ne!(
        inv_run.color, ok_run.color,
        "inverse/background terminal metadata should affect visible terminal color"
    );
}

#[test]
fn terminal_dock_renders_protocol_screen_cursor_when_visible() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;
    state.ui.terminal.lines = vec!["prompt".to_string()];
    state.ui.terminal.screen_cursor_row = 0;
    state.ui.terminal.screen_cursor_col = 6;
    state.ui.terminal.screen_cursor_visible = true;

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );

    assert!(
        prepared.text_runs.iter().any(|run| run.text == "█"),
        "terminal dock should render the PTY screen cursor from protocol state"
    );
}

#[test]
fn terminal_dock_renders_protocol_cursor_shape() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;
    state.ui.terminal.lines = vec!["prompt".to_string()];
    state.ui.terminal.screen_cursor_row = 0;
    state.ui.terminal.screen_cursor_col = 6;
    state.ui.terminal.screen_cursor_visible = true;
    state.ui.terminal.screen_cursor_style = Some("steady_bar".to_string());

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );

    assert!(
        prepared.text_runs.iter().any(|run| run.text == "|"),
        "terminal dock should render bar cursor style from protocol state"
    );
    assert!(
        prepared.text_runs.iter().all(|run| run.text != "█"),
        "bar cursor style should not fall back to block cursor"
    );
}

#[test]
fn terminal_dock_suppresses_protocol_screen_cursor_when_hidden() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;
    state.ui.terminal.lines = vec!["prompt".to_string()];
    state.ui.terminal.screen_cursor_row = 0;
    state.ui.terminal.screen_cursor_col = 6;
    state.ui.terminal.screen_cursor_visible = false;

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );

    assert!(
        prepared.text_runs.iter().all(|run| run.text != "█"),
        "terminal dock should honor hidden cursor mode"
    );
}
