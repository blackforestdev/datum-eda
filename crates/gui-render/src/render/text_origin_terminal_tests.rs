//! Real TerminalCore snapshots through production split composition and row reuse.
use super::*;
use crate::{
    CameraState, PreparedScene, RetainedScene, TerminalPaneRenderState, TerminalRenderCache,
};
use datum_gui_protocol::{DockTab, TerminalSplitDirection, TerminalSplitNode, TerminalTabLayout};

#[test]
#[ignore = "requires DATUM_RESOURCE_TRACE=1 for production source identity capture"]
fn terminal_origin_tracks_session_reorder_and_close() {
    assert!(std::env::var_os("DATUM_RESOURCE_TRACE").is_some());
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.active_dock_tab = Some(DockTab::Terminal);
    state.ui.dock_height_px = 220;
    state.ui.terminal.active_tab_id = Some("test-tab".into());
    state.ui.terminal.active_session_id = Some("left-session".into());
    state.ui.terminal.tab_layouts = vec![TerminalTabLayout {
        tab_id: "test-tab".into(),
        focused_session_id: "left-session".into(),
        root: TerminalSplitNode::Split {
            direction: TerminalSplitDirection::SideBySide,
            ratio_millis: 500,
            first: Box::new(TerminalSplitNode::session("left-session")),
            second: Box::new(TerminalSplitNode::session("right-session")),
        },
    }];
    let lane = state.ui.terminal.clone();
    let mut panes = vec![
        TerminalPaneRenderState {
            session_id: "left-session".into(),
            focused: true,
            lane: &lane,
            snapshot: crate::terminal_core_render::test_snapshot(b"LEFT-OWNER"),
            damage: Vec::new(),
        },
        TerminalPaneRenderState {
            session_id: "right-session".into(),
            focused: false,
            lane: &lane,
            snapshot: crate::terminal_core_render::test_snapshot(b"RIGHT-OWNER"),
            damage: Vec::new(),
        },
    ];
    let retained = RetainedScene::from_workspace(&state, 1280, 800);
    let mut cache = TerminalRenderCache::new();
    let prepare = |state: &datum_gui_protocol::ReviewWorkspaceState,
                   panes: &[TerminalPaneRenderState<'_>],
                   cache: &mut TerminalRenderCache| {
        PreparedScene::from_workspace_with_terminal_renderer(
            state,
            1280,
            800,
            1.0,
            CameraState::fit_to_bounds(&state.scene.bounds),
            &retained,
            panes,
            Some(cache),
            false,
        )
        .unwrap()
    };
    let check = |prepared: &PreparedScene, marker: &str, session: &str| {
        let mut text = String::new();
        for run in &prepared.text_runs {
            if let TextOrigin::TerminalLeaf(index) = run.origin {
                let source = prepared
                    .admission_sources()
                    .unwrap()
                    .terminal_sessions
                    .get(index)
                    .expect("terminal origin must have a source identity");
                if source == session {
                    text.push_str(&run.text);
                }
            }
        }
        assert!(
            text.contains(marker),
            "missing terminal marker {marker} for {session}: {text:?}"
        );
        let other = if marker == "LEFT-OWNER" {
            "RIGHT-OWNER"
        } else {
            "LEFT-OWNER"
        };
        assert!(
            !text.contains(other),
            "another session's text leaked into {session}"
        );
    };
    let first = prepare(&state, &panes, &mut cache);
    check(&first, "LEFT-OWNER", "left-session");
    check(&first, "RIGHT-OWNER", "right-session");
    let rebuilt = cache.rebuilt_rows();
    assert!(rebuilt > 0);
    panes.reverse();
    let reversed = prepare(&state, &panes, &mut cache);
    assert_eq!(
        cache.rebuilt_rows(),
        rebuilt,
        "input ordering must reuse session-keyed row plans"
    );
    check(&reversed, "LEFT-OWNER", "left-session");
    check(&reversed, "RIGHT-OWNER", "right-session");
    assert_eq!(
        reversed.admission_sources().unwrap().terminal_sessions,
        ["right-session", "left-session"]
    );
    assert_eq!(
        first.admission_sources().unwrap().terminal_sessions,
        ["left-session", "right-session"]
    );
    panes.retain(|pane| pane.session_id == "left-session");
    state.ui.terminal.tab_layouts[0].root = TerminalSplitNode::session("left-session");
    let closed = prepare(&state, &panes, &mut cache);
    check(&closed, "LEFT-OWNER", "left-session");
    let closed_text: String = closed
        .text_runs
        .iter()
        .filter(|run| matches!(run.origin, TextOrigin::TerminalLeaf(_)))
        .map(|run| run.text.as_str())
        .collect();
    assert!(!closed_text.contains("RIGHT-OWNER"));
    assert_eq!(
        closed.admission_sources().unwrap().terminal_sessions,
        ["left-session"]
    );
    assert!(
        closed
            .text_runs
            .iter()
            .all(|run| !matches!(run.origin, TextOrigin::TerminalLeaf(index) if index != 0))
    );
    check(&first, "RIGHT-OWNER", "right-session");
}
