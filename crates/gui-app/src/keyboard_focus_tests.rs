use super::{
    KeyClass, RouteDecision, TerminalInputOwner, armed_close_shortcut, focus_after_canvas_click,
    focus_after_hit_target, focus_before_terminal_mouse_press, hit_target_is_terminal_entry,
    key_route, pre_raw_escape_route, terminal_focus_report_transition, terminal_input_owner,
    terminal_mouse_report_allowed, workspace_action_should_fire,
};
use crate::terminal_input::terminal_tab_sequence;
use datum_gui_protocol::{ApplicationFocus as KeyboardFocus, PaneId};
use datum_gui_render::HitTarget;
use winit::{
    event::ElementState,
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
};

#[test]
fn default_focus_is_editor() {
    assert_eq!(KeyboardFocus::default(), KeyboardFocus::Editor(PaneId(0)));
}

#[test]
fn mouse_aware_child_cannot_swallow_terminal_focus_entry() {
    let focus =
        focus_before_terminal_mouse_press(KeyboardFocus::Editor(PaneId(0)), true, true, true);
    assert_eq!(focus, KeyboardFocus::Terminal);
    assert!(terminal_mouse_report_allowed(focus, true, true, true));
    assert_eq!(
        key_route(focus, KeyClass::RawPty, true),
        RouteDecision::Terminal
    );
    assert!(!workspace_action_should_fire(
        focus,
        true,
        ElementState::Pressed,
        false,
    ));
    assert_eq!(
        terminal_tab_sequence(ModifiersState::empty()),
        Some(b"\t".to_vec())
    );
    assert_eq!(
        terminal_tab_sequence(ModifiersState::SHIFT),
        Some(b"\x1b[Z".to_vec())
    );
    assert_eq!(
        focus_before_terminal_mouse_press(KeyboardFocus::Editor(PaneId(0)), true, true, false),
        KeyboardFocus::Editor(PaneId(0))
    );
    assert!(!terminal_mouse_report_allowed(
        KeyboardFocus::Editor(PaneId(0)),
        true,
        true,
        false,
    ));
    assert_eq!(
        focus_before_terminal_mouse_press(KeyboardFocus::Editor(PaneId(0)), false, true, true),
        KeyboardFocus::Editor(PaneId(0))
    );
}

#[test]
fn armed_close_repeat_shortcut_is_focus_independent_and_exact() {
    let key = PhysicalKey::Code(KeyCode::KeyW);
    assert!(armed_close_shortcut(
        true,
        true,
        true,
        ElementState::Pressed,
        false,
        key,
    ));
    assert!(!armed_close_shortcut(
        false,
        true,
        true,
        ElementState::Pressed,
        false,
        key,
    ));
    assert!(!armed_close_shortcut(
        true,
        true,
        false,
        ElementState::Pressed,
        false,
        key,
    ));
}

#[test]
fn terminal_input_owner_is_exclusive_to_the_visible_focused_pty() {
    assert_eq!(
        terminal_input_owner(KeyboardFocus::Terminal, true),
        TerminalInputOwner::AttachedPty
    );
    assert_eq!(
        terminal_input_owner(KeyboardFocus::Editor(PaneId(0)), true),
        TerminalInputOwner::Unowned
    );
    assert_eq!(
        terminal_input_owner(KeyboardFocus::Terminal, false),
        TerminalInputOwner::Unowned
    );
}

#[test]
fn terminal_focus_routes_text_to_terminal_and_never_to_workspace() {
    for visible in [false, true] {
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
            key_route(
                KeyboardFocus::Editor(PaneId(0)),
                KeyClass::WorkspaceHotkey,
                visible
            ),
            RouteDecision::Editor
        );
        assert_eq!(
            key_route(KeyboardFocus::Editor(PaneId(0)), KeyClass::RawPty, visible),
            RouteDecision::Unrouted
        );
    }
}

#[test]
fn workspace_actions_fire_once_on_editor_press_and_never_under_terminal_focus() {
    for visible in [false, true] {
        assert!(workspace_action_should_fire(
            KeyboardFocus::Editor(PaneId(0)),
            visible,
            ElementState::Pressed,
            false,
        ));
        assert!(!workspace_action_should_fire(
            KeyboardFocus::Editor(PaneId(0)),
            visible,
            ElementState::Released,
            false,
        ));
        assert!(!workspace_action_should_fire(
            KeyboardFocus::Editor(PaneId(0)),
            visible,
            ElementState::Pressed,
            true,
        ));
        for state in [ElementState::Pressed, ElementState::Released] {
            assert!(!workspace_action_should_fire(
                KeyboardFocus::Terminal,
                visible,
                state,
                false,
            ));
        }
    }
}

#[test]
fn dock_visibility_never_changes_routing_except_raw_pty() {
    for focus in [
        KeyboardFocus::Editor(PaneId(0)),
        KeyboardFocus::Terminal,
        KeyboardFocus::Overlay,
    ] {
        for class in [KeyClass::WorkspaceHotkey, KeyClass::TerminalFocusExit] {
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
                KeyClass::TerminalFocusExit,
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
        pre_raw_escape_route(KeyboardFocus::Editor(PaneId(0)), true, true),
        None,
        "editor Escape is not a terminal focus exit",
    );
}

#[test]
fn terminal_screen_click_is_entry_and_observation_chrome_is_not() {
    assert!(hit_target_is_terminal_entry(&HitTarget::TerminalScreen));
    assert!(hit_target_is_terminal_entry(&HitTarget::TerminalTab));
    assert!(hit_target_is_terminal_entry(
        &HitTarget::TerminalSessionTab("terminal-a".to_string())
    ));
    assert!(hit_target_is_terminal_entry(&HitTarget::TerminalSessionNew));
    for target in [
        HitTarget::TerminalSessionTerminateActive,
        HitTarget::TerminalSessionForceKillActive,
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
    assert_eq!(
        focus_after_canvas_click(PaneId(0)),
        KeyboardFocus::Editor(PaneId(0))
    );
    assert_eq!(
        focus_after_hit_target(
            KeyboardFocus::Editor(PaneId(0)),
            true,
            &HitTarget::TerminalScreen
        ),
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
            focus_after_hit_target(KeyboardFocus::Editor(PaneId(0)), true, &target),
            KeyboardFocus::Editor(PaneId(0)),
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
            KeyClass::WorkspaceHotkey,
            KeyClass::TerminalFocusExit,
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
        terminal_focus_report_transition(KeyboardFocus::Editor(PaneId(0)), KeyboardFocus::Terminal),
        Some(true),
    );
    assert_eq!(
        terminal_focus_report_transition(KeyboardFocus::Terminal, KeyboardFocus::Editor(PaneId(0))),
        Some(false),
    );
    assert_eq!(
        terminal_focus_report_transition(KeyboardFocus::Terminal, KeyboardFocus::Overlay),
        Some(false),
    );
    assert_eq!(
        terminal_focus_report_transition(KeyboardFocus::Editor(PaneId(0)), KeyboardFocus::Overlay),
        None,
    );
    assert_eq!(
        terminal_focus_report_transition(KeyboardFocus::Terminal, KeyboardFocus::Terminal),
        None,
    );
}
