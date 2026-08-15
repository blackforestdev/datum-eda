//! T0-C04 workspace-shortcut regression lock (DATUM_NATIVE_TERMINAL_SPEC.md
//! §7.1; decision 027): with Terminal keyboard focus every workspace hotkey is
//! raw PTY bytes — it never routes to a workspace tool — and with Editor focus
//! hotkeys fire while no key class routes toward the PTY. The byte mappings
//! are asserted against the same private sequence builders `terminal_key_action`
//! uses, so a mapping change and a routing change both fail here.

use super::{terminal_character_sequence, terminal_space_sequence, terminal_tab_sequence};
use crate::keyboard_focus::{KeyClass, KeyboardFocus, RouteDecision, key_route};
use winit::keyboard::ModifiersState;

/// Every character workspace hotkey the editor persona binds
/// (`keyboard_focus::handle_keyboard_input`): tools s/b/v/m/x/r, fit f/t,
/// pane zoom z, crosshair c, review navigation [ / ].
const WORKSPACE_HOTKEYS: [&str; 12] = [
    "s", "b", "v", "m", "x", "r", "f", "t", "z", "c", "[", "]",
];

#[test]
fn every_workspace_hotkey_is_pty_bytes_never_a_tool_under_terminal_focus() {
    for key in WORKSPACE_HOTKEYS {
        for visible in [false, true] {
            assert_eq!(
                key_route(KeyboardFocus::Terminal, KeyClass::WorkspaceHotkey, visible),
                RouteDecision::Unrouted,
                "hotkey {key:?} must never fire a workspace tool under Terminal focus"
            );
        }
        // The literal byte the attached shell receives for this keystroke.
        assert_eq!(
            terminal_character_sequence(key, ModifiersState::empty()),
            Some(key.as_bytes().to_vec()),
            "hotkey {key:?} must reach the PTY as its literal byte"
        );
        if key.chars().all(|ch| ch.is_ascii_alphabetic()) {
            let upper = key.to_ascii_uppercase();
            assert_eq!(
                terminal_character_sequence(&upper, ModifiersState::SHIFT),
                Some(upper.as_bytes().to_vec()),
                "shifted hotkey {upper:?} must reach the PTY as its literal byte"
            );
        }
    }
}

#[test]
fn tab_cycling_and_space_pan_keys_are_pty_bytes_under_terminal_focus() {
    for visible in [false, true] {
        // Tab (pane cycling) and Space (pan chord) are workspace hotkeys only
        // for the editor; under Terminal focus neither may fire.
        assert_eq!(
            key_route(KeyboardFocus::Terminal, KeyClass::WorkspaceHotkey, visible),
            RouteDecision::Unrouted
        );
        assert_ne!(
            key_route(KeyboardFocus::Terminal, KeyClass::WorkspaceHotkey, visible),
            RouteDecision::Editor,
            "the Space pan chord is gated to editor hotkey ownership"
        );
    }
    assert_eq!(
        terminal_tab_sequence(ModifiersState::empty()),
        Some(b"\t".to_vec()),
        "Tab must reach the PTY as HT, not cycle panes"
    );
    assert_eq!(
        terminal_tab_sequence(ModifiersState::SHIFT),
        Some(b"\x1b[Z".to_vec()),
        "Shift+Tab must reach the PTY as CSI Z, not cycle panes backward"
    );
    assert_eq!(
        terminal_space_sequence(ModifiersState::empty()),
        Some(b" ".to_vec()),
        "Space must reach the PTY as a literal space, not arm the pan chord"
    );
}

#[test]
fn editor_focus_fires_hotkeys_and_no_key_class_routes_to_the_pty() {
    for visible in [false, true] {
        assert_eq!(
            key_route(KeyboardFocus::Editor, KeyClass::WorkspaceHotkey, visible),
            RouteDecision::Editor,
            "editor focus must own workspace hotkeys"
        );
        for class in [
            KeyClass::RawPty,
            KeyClass::TerminalRenameEdit,
            KeyClass::EscapeWithEmptyRename,
        ] {
            assert_eq!(
                key_route(KeyboardFocus::Editor, class, visible),
                RouteDecision::Unrouted,
                "under Editor focus {class:?} must never route toward the terminal"
            );
        }
    }
}
