use super::{
    KeyClass, KeyboardFocus, RouteDecision, focus_after_canvas_click,
    focus_after_hit_target, hit_target_is_terminal_entry, key_route, pre_raw_escape_route,
    terminal_focus_report_transition,
};
use datum_gui_render::HitTarget;

#[test]
fn default_focus_is_editor() {
    assert_eq!(KeyboardFocus::default(), KeyboardFocus::Editor);
}

#[test]
fn terminal_focus_routes_text_to_terminal_and_never_to_workspace() {
    for visible in [false, true] {
        assert_eq!(
            key_route(KeyboardFocus::Terminal, KeyClass::DockLineEdit, visible),
            RouteDecision::Terminal
        );
        assert_eq!(
            key_route(KeyboardFocus::Terminal, KeyClass::WorkspaceHotkey, visible),
            RouteDecision::Unrouted
        );
    }
    assert_eq!(
        key_route(KeyboardFocus::Terminal, KeyClass::RawPty, true),
        RouteDecision::Terminal
    );
}

#[test]
fn editor_focus_routes_hotkeys_and_never_to_terminal() {
    for visible in [false, true] {
        assert_eq!(
            key_route(KeyboardFocus::Editor, KeyClass::WorkspaceHotkey, visible),
            RouteDecision::Editor
        );
        assert_eq!(
            key_route(KeyboardFocus::Editor, KeyClass::DockLineEdit, visible),
            RouteDecision::Unrouted
        );
        assert_eq!(
            key_route(KeyboardFocus::Editor, KeyClass::RawPty, visible),
            RouteDecision::Unrouted
        );
    }
}

#[test]
fn dock_visibility_never_changes_routing_except_raw_pty() {
    for focus in [
        KeyboardFocus::Editor,
        KeyboardFocus::Terminal,
        KeyboardFocus::Overlay,
    ] {
        for class in [
            KeyClass::DockLineEdit,
            KeyClass::WorkspaceHotkey,
            KeyClass::EscapeWithEmptyInput,
        ] {
            assert_eq!(
                key_route(focus, class, false),
                key_route(focus, class, true),
                "dock visibility changed routing for {focus:?}/{class:?}"
            );
        }
    }
    assert_eq!(
        key_route(KeyboardFocus::Terminal, KeyClass::RawPty, false),
        RouteDecision::Unrouted
    );
    assert_eq!(
        key_route(KeyboardFocus::Terminal, KeyClass::RawPty, true),
        RouteDecision::Terminal
    );
}

#[test]
fn escape_under_terminal_focus_releases_to_editor_when_input_empty() {
    for visible in [false, true] {
        assert_eq!(
            key_route(
                KeyboardFocus::Terminal,
                KeyClass::EscapeWithEmptyInput,
                visible
            ),
            RouteDecision::ReleaseToEditor
        );
    }
    assert_eq!(
        pre_raw_escape_route(KeyboardFocus::Terminal, true, true),
        Some(RouteDecision::ReleaseToEditor),
        "Escape release must be classified before attached-shell raw routing",
    );
    assert_eq!(
        pre_raw_escape_route(KeyboardFocus::Terminal, true, false),
        None,
        "Escape press remains raw PTY input",
    );
    assert_eq!(
        pre_raw_escape_route(KeyboardFocus::Editor, true, true),
        None,
        "editor Escape is not a terminal focus exit",
    );
}

#[test]
fn terminal_screen_click_is_entry_and_observation_chrome_is_not() {
    assert!(hit_target_is_terminal_entry(&HitTarget::TerminalScreen));
    assert!(hit_target_is_terminal_entry(&HitTarget::TerminalTab));
    assert!(hit_target_is_terminal_entry(&HitTarget::TerminalSessionTab(
        "terminal-a".to_string()
    )));
    assert!(hit_target_is_terminal_entry(&HitTarget::TerminalSessionNew));
    assert!(hit_target_is_terminal_entry(
        &HitTarget::TerminalSessionRenameActive
    ));
    for target in [
        HitTarget::TerminalSessionRestartActive,
        HitTarget::TerminalSessionDetachActive,
        HitTarget::TerminalSessionCloseActive,
        HitTarget::DockResizeHandle,
    ] {
        assert!(
            !hit_target_is_terminal_entry(&target),
            "{target:?} must not arm terminal keyboard focus"
        );
    }
}

#[test]
fn production_click_transition_functions_are_the_single_focus_authority() {
    assert_eq!(focus_after_canvas_click(), KeyboardFocus::Editor);
    assert_eq!(
        focus_after_hit_target(KeyboardFocus::Editor, true, &HitTarget::TerminalScreen),
        KeyboardFocus::Terminal,
    );
    let handoff = datum_gui_protocol::TerminalCommandHandoff {
        command_id: "datum.project.status".to_string(),
        mcp_alias: None,
        command: "datum-eda project status".to_string(),
    };
    for target in [
        HitTarget::ProductionOutputJobRun(handoff.clone()),
        HitTarget::ProductionTerminalCommand(handoff),
    ] {
        assert_eq!(
            focus_after_hit_target(KeyboardFocus::Editor, true, &target),
            KeyboardFocus::Editor,
            "programmatic run-in-terminal handoff must not steal keyboard focus",
        );
    }
    assert_eq!(
        focus_after_hit_target(KeyboardFocus::Overlay, false, &HitTarget::TerminalScreen),
        KeyboardFocus::Overlay,
        "an unhandled hit cannot change focus",
    );
}

#[test]
fn overlay_focus_routes_nothing_through_the_focus_classes() {
    for visible in [false, true] {
        for class in [
            KeyClass::RawPty,
            KeyClass::DockLineEdit,
            KeyClass::WorkspaceHotkey,
            KeyClass::EscapeWithEmptyInput,
        ] {
            assert_eq!(
                key_route(KeyboardFocus::Overlay, class, visible),
                RouteDecision::Unrouted
            );
        }
    }
}

#[test]
fn focus_reports_follow_terminal_keyboard_ownership_only() {
    assert_eq!(
        terminal_focus_report_transition(KeyboardFocus::Editor, KeyboardFocus::Terminal),
        Some(true),
    );
    assert_eq!(
        terminal_focus_report_transition(KeyboardFocus::Terminal, KeyboardFocus::Editor),
        Some(false),
    );
    assert_eq!(
        terminal_focus_report_transition(KeyboardFocus::Terminal, KeyboardFocus::Overlay),
        Some(false),
    );
    assert_eq!(
        terminal_focus_report_transition(KeyboardFocus::Editor, KeyboardFocus::Overlay),
        None,
    );
    assert_eq!(
        terminal_focus_report_transition(KeyboardFocus::Terminal, KeyboardFocus::Terminal),
        None,
    );
}
