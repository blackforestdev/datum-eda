use super::*;

#[test]
fn new_terminal_tabs_append_left_to_right_with_close_targets_and_plus_after_last() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;
    state.ui.terminal.tabs = (1..=3)
        .map(|index| datum_gui_protocol::TerminalTabState {
            session_id: format!("terminal-{index}"),
            previous_session_id: None,
            label: format!("shell {index}"),
            event_log_path: format!("/tmp/terminal-{index}.jsonl"),
            activity_event_count: 0,
            activity_summary: Vec::new(),
            active: index == 3,
            attached: true,
            status: "running".to_string(),
            restart_count: 0,
        })
        .collect();

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    let strip = ShellLayout::for_window(1280, 800, Some(260)).bottom_strip;
    let top_tabs = prepared
        .hit_regions
        .iter()
        .filter(|region| {
            region.rect.y < strip.y + 40.0
                && matches!(region.target, HitTarget::TerminalSessionTab(_))
        })
        .collect::<Vec<_>>();
    assert_eq!(top_tabs.len(), 3);
    for (index, region) in top_tabs.iter().enumerate() {
        assert!(matches!(
            &region.target,
            HitTarget::TerminalSessionTab(session_id)
                if session_id == &format!("terminal-{}", index + 1)
        ));
        if let Some(previous) = index.checked_sub(1).map(|i| top_tabs[i].rect) {
            assert!(
                previous.x + previous.width < region.rect.x,
                "tab {} must be placed to the right of tab {}",
                index + 1,
                index
            );
        }
    }
    let close_targets = prepared
        .hit_regions
        .iter()
        .filter(|region| matches!(region.target, HitTarget::TerminalSessionClose(_)))
        .collect::<Vec<_>>();
    assert_eq!(
        close_targets.len(),
        3,
        "every session tab needs one close target"
    );
    for (index, close) in close_targets.iter().enumerate() {
        assert!(matches!(
            &close.target,
            HitTarget::TerminalSessionClose(session_id)
                if session_id == &format!("terminal-{}", index + 1)
        ));
        let tab = top_tabs[index].rect;
        assert_eq!(tab.x + tab.width, close.rect.x);
        if index + 1 < top_tabs.len() {
            assert_eq!(
                close.rect.x + close.rect.width + crate::terminal_tab_strip::TAB_GAP_PX,
                top_tabs[index + 1].rect.x
            );
        }
        assert!(matches!(
            prepared.hit_test(close.rect.x + 2.0, close.rect.y + 2.0),
            Some(HitTarget::TerminalSessionClose(session_id))
                if session_id == &format!("terminal-{}", index + 1)
        ));
    }
    let plus = prepared
        .hit_regions
        .iter()
        .find(|region| {
            region.target == HitTarget::TerminalSessionNew && region.rect.y < strip.y + 40.0
        })
        .expect("top-strip new-session affordance");
    let last = top_tabs.last().expect("last session tab").rect;
    assert!(last.x + last.width < plus.rect.x);
}

#[test]
fn guarded_tab_close_uses_dedicated_strip_chrome_without_covering_terminal_text() {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(datum_gui_protocol::DockTab::Terminal);
    state.ui.dock_height_px = 260;
    state.ui.terminal.status = "close terminal? Enter confirms; Escape cancels".to_string();
    state.ui.terminal.tabs = vec![datum_gui_protocol::TerminalTabState {
        session_id: "terminal-1".to_string(),
        previous_session_id: None,
        label: "shell 1".to_string(),
        event_log_path: "/tmp/terminal-1.jsonl".to_string(),
        activity_event_count: 0,
        activity_summary: Vec::new(),
        active: true,
        attached: true,
        status: state.ui.terminal.status.clone(),
        restart_count: 0,
    }];
    *state.ui.terminal.pty_grid_mut().lines = vec!["visible-shell-prompt$".to_string()];

    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let prepared = PreparedScene::from_workspace(
        &state,
        1280,
        800,
        CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    let geometry = datum_gui_viewport::terminal_screen_geometry(
        ShellLayout::for_window(1280, 800, Some(260))
            .bottom_strip
            .into(),
    );
    let confirmation = prepared
        .text_runs
        .iter()
        .find(|run| run.text.contains("Enter confirms"))
        .expect("guarded close must explain the Enter/Escape choice");
    let prompt = prepared
        .text_runs
        .iter()
        .find(|run| run.text == "visible-shell-prompt$")
        .expect("terminal content must remain rendered");
    assert!(confirmation.y < geometry.screen.y);
    assert!(prompt.y >= geometry.screen.y);
    assert!(
        prepared
            .hit_regions
            .iter()
            .any(|region| { region.target == HitTarget::TerminalSessionTerminateActive })
    );
    assert!(
        prepared
            .hit_regions
            .iter()
            .any(|region| { region.target == HitTarget::TerminalShutdownCancel })
    );

    let close = prepared
        .hit_regions
        .iter()
        .find(|region| matches!(region.target, HitTarget::TerminalSessionClose(_)))
        .expect("active tab close target");
    assert!(
        close.rect.width >= 28.0,
        "tab close target must be easy to hit"
    );
    let close_glyph = prepared
        .text_runs
        .iter()
        .find(|run| run.text == "×")
        .expect("visible tab close glyph");
    assert!(close_glyph.size >= 15.0);
}
