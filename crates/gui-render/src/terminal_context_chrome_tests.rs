use super::*;
use datum_gui_protocol::{TerminalSessionBinding, TerminalSessionContext};

fn state() -> ReviewWorkspaceState {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.ui.terminal.tabs = (0..2)
        .map(|i| TerminalTabState {
            session_id: format!("session-{i}"),
            previous_session_id: None,
            label: if i == 0 {
                "Home".into()
            } else {
                "northwind-environmental-sensor-node-rev-c-low-power".into()
            },
            event_log_path: String::new(),
            activity_event_count: 0,
            activity_summary: vec![],
            active: i == 1,
            attached: true,
            status: "running".into(),
            restart_count: 0,
            unread_output: false,
            unread_bell_count: 0,
        })
        .collect();
    state.ui.terminal.session_context = Some(TerminalSessionContext {
        binding: TerminalSessionBinding::Project {
            name: "northwind-environmental-sensor-node-rev-c-low-power".into(),
            root: "/customers/northwind-instruments/2026-programs/long-program-name".into(),
        },
        default_tab_label: "northwind-environmental-sensor-node-rev-c-low-power".into(),
        directory: "/customers/northwind-instruments/2026-programs/long-program-name/board".into(),
        directory_reported: false,
    });
    state
}

fn render(state: &ReviewWorkspaceState, width: f32) -> (Vec<Quad>, Vec<TextRun>, Vec<HitRegion>) {
    let (mut quads, mut text, mut hits) = (vec![], vec![], vec![]);
    render_terminal_tab_strip(
        state,
        RectPx {
            x: 0.0,
            y: 0.0,
            width,
            height: 44.0,
        },
        1.0,
        &mut quads,
        &mut text,
        &mut hits,
    );
    (quads, text, hits)
}

#[test]
fn terminal_context_full_independent_binding_uses_free_room_and_replaces_routine_hint() {
    let mut state = state();
    state.ui.terminal.tabs.truncate(1);
    state.ui.terminal.tabs[0].active = true;
    let context = state.ui.terminal.session_context.as_mut().unwrap();
    context.binding = TerminalSessionBinding::Independent;
    context.directory = "/tmp".into();
    context.directory_reported = true;
    state.ui.terminal.status = "running · no Project bound".into();
    let (_, text, hits) = render(&state, 1040.0);
    assert!(text.iter().any(|t| t.text == "Independent shell"));
    assert!(text.iter().any(|t| t.text == "/tmp"));
    assert!(
        !text
            .iter()
            .any(|t| t.text.contains("Ctrl+Shift+T") || t.text.contains("no Project bound"))
    );
    let plus = hits
        .iter()
        .find(|h| h.target == HitTarget::TerminalSessionNew)
        .unwrap();
    let binding = text.iter().find(|t| t.text == "Independent shell").unwrap();
    assert!(binding.x > plus.rect.x + plus.rect.width);
}

#[test]
fn terminal_context_narrow_long_binding_and_cwd_ellipsize_before_lifecycle_controls() {
    let mut state = state();
    state.ui.terminal.status = "close terminal? Enter confirms; Escape cancels".into();
    let (_, text, hits) = render(&state, 620.0);
    assert!(
        text.iter().any(|t| t.text == "close terminal?"),
        "compact prompt must use its measured available width"
    );
    let context: Vec<_> = text
        .iter()
        .filter(|t| t.text.starts_with("Project:") || t.text.starts_with("Started"))
        .collect();
    assert_eq!(context.len(), 2);
    assert!(context.iter().all(|t| t.text.ends_with('…')));
    for target in [
        HitTarget::TerminalSessionNew,
        HitTarget::TerminalSessionTerminateActive,
        HitTarget::TerminalShutdownCancel,
    ] {
        let hit = hits.iter().find(|h| h.target == target).unwrap();
        assert!(hit.rect.width > 0.0 && hit.rect.x >= 0.0 && hit.rect.x + hit.rect.width <= 620.0);
    }
    let action = hits
        .iter()
        .find(|h| h.target == HitTarget::TerminalSessionTerminateActive)
        .unwrap();
    assert!(
        context
            .iter()
            .all(|t| t.clip_bounds.unwrap().x + t.clip_bounds.unwrap().width < action.rect.x)
    );
    assert!(
        state
            .ui
            .terminal
            .session_context
            .as_ref()
            .unwrap()
            .description()
            .contains("/customers/northwind-instruments/2026-programs/long-program-name/board")
    );
}

#[test]
fn terminal_context_closed_strip_keeps_selected_label_without_open_accent() {
    let mut state = state();
    state.ui.active_dock_tab = None;
    let (quads, text, hits) = render(&state, 1040.0);
    let selected = hits
        .iter()
        .find(|h| h.target == HitTarget::TerminalSessionTab("session-1".into()))
        .unwrap();
    let label = text
        .iter()
        .find(|t| t.text.starts_with("northwind"))
        .unwrap();
    assert_eq!(label.color, TEXT_PRIMARY);
    assert!(
        !quads.iter().any(|q| q.color == TEXT_ACCENT
            && q.points.iter().any(|&(x, y)| selected.rect.contains(x, y)))
    );
    assert!(text.iter().any(|t| t.text.starts_with("Project:")));
}
