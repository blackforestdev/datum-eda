//! T0-C04 workspace-shortcut regression lock (DATUM_NATIVE_TERMINAL_SPEC.md
//! §7.1; decision 027): with Terminal keyboard focus every workspace hotkey is
//! raw PTY bytes — it never routes to a workspace tool — and with Editor focus
//! hotkeys fire while no key class routes toward the PTY. The byte mappings
//! are asserted against the same private sequence builders `terminal_key_action`
//! uses, so a mapping change and a routing change both fail here.

use super::{
    TerminalClipboardShortcut, terminal_character_sequence, terminal_clipboard_shortcut,
    terminal_new_session_shortcut, terminal_search_shortcut, terminal_space_sequence,
    terminal_split_shortcut, terminal_tab_sequence,
};
use crate::keyboard_focus::{KeyClass, RouteDecision, key_route};
use datum_gui_protocol::{ApplicationFocus as KeyboardFocus, PaneId};
use winit::{
    event::ElementState,
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
};

/// Every character workspace hotkey the editor persona binds
/// (`keyboard_focus::handle_keyboard_input`): tools s/b/v/m/x/r, fit f/t,
/// pane zoom z, crosshair c, review navigation [ / ].
const WORKSPACE_HOTKEYS: [&str; 12] = ["s", "b", "v", "m", "x", "r", "f", "t", "z", "c", "[", "]"];

#[test]
fn ctrl_shift_t_is_the_nonrepeating_new_session_shortcut() {
    let modifiers = ModifiersState::CONTROL | ModifiersState::SHIFT;
    let key = PhysicalKey::Code(KeyCode::KeyT);
    assert!(terminal_new_session_shortcut(
        ElementState::Pressed,
        false,
        key,
        modifiers,
    ));
    assert!(!terminal_new_session_shortcut(
        ElementState::Pressed,
        true,
        key,
        modifiers,
    ));
    assert!(!terminal_new_session_shortcut(
        ElementState::Released,
        false,
        key,
        modifiers,
    ));
    assert!(!terminal_new_session_shortcut(
        ElementState::Pressed,
        false,
        key,
        ModifiersState::CONTROL,
    ));
}

#[test]
fn ctrl_shift_o_and_e_are_nonrepeating_split_shortcuts() {
    let modifiers = ModifiersState::CONTROL | ModifiersState::SHIFT;
    assert_eq!(
        terminal_split_shortcut(
            ElementState::Pressed,
            false,
            PhysicalKey::Code(KeyCode::KeyO),
            modifiers,
        ),
        Some(datum_gui_protocol::TerminalSplitDirection::SideBySide)
    );
    assert_eq!(
        terminal_split_shortcut(
            ElementState::Pressed,
            false,
            PhysicalKey::Code(KeyCode::KeyE),
            modifiers,
        ),
        Some(datum_gui_protocol::TerminalSplitDirection::Stacked)
    );
    assert!(
        terminal_split_shortcut(
            ElementState::Pressed,
            true,
            PhysicalKey::Code(KeyCode::KeyO),
            modifiers,
        )
        .is_none()
    );
}

#[test]
fn ctrl_shift_f_is_a_terminal_local_search_shortcut() {
    assert!(terminal_search_shortcut(
        ElementState::Pressed,
        false,
        PhysicalKey::Code(KeyCode::KeyF),
        ModifiersState::CONTROL | ModifiersState::SHIFT,
    ));
    assert!(!terminal_search_shortcut(
        ElementState::Pressed,
        false,
        PhysicalKey::Code(KeyCode::KeyF),
        ModifiersState::CONTROL,
    ));
}

#[test]
fn clipboard_shortcuts_fire_once_on_press_and_preserve_plain_control_bytes() {
    let modifiers = ModifiersState::CONTROL | ModifiersState::SHIFT;
    for (key, expected) in [
        (KeyCode::KeyC, TerminalClipboardShortcut::Copy),
        (KeyCode::KeyV, TerminalClipboardShortcut::Paste),
    ] {
        assert_eq!(
            terminal_clipboard_shortcut(
                ElementState::Pressed,
                false,
                PhysicalKey::Code(key),
                modifiers,
            ),
            Some(expected)
        );
        assert_eq!(
            terminal_clipboard_shortcut(
                ElementState::Released,
                false,
                PhysicalKey::Code(key),
                modifiers,
            ),
            None
        );
        assert_eq!(
            terminal_clipboard_shortcut(
                ElementState::Pressed,
                true,
                PhysicalKey::Code(key),
                modifiers,
            ),
            None
        );
    }
    assert_eq!(
        terminal_clipboard_shortcut(
            ElementState::Pressed,
            false,
            PhysicalKey::Code(KeyCode::Insert),
            ModifiersState::SHIFT,
        ),
        Some(TerminalClipboardShortcut::Paste)
    );
    assert_eq!(
        terminal_clipboard_shortcut(
            ElementState::Pressed,
            false,
            PhysicalKey::Code(KeyCode::KeyV),
            ModifiersState::CONTROL,
        ),
        None
    );
    assert_eq!(
        terminal_character_sequence("v", ModifiersState::CONTROL),
        Some(vec![0x16]),
        "plain Ctrl+V remains the shell's literal-next byte"
    );
}

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
            key_route(
                KeyboardFocus::Editor(PaneId(0)),
                KeyClass::WorkspaceHotkey,
                visible
            ),
            RouteDecision::Editor,
            "editor focus must own workspace hotkeys"
        );
        for class in [KeyClass::RawPty, KeyClass::TerminalFocusExit] {
            assert_eq!(
                key_route(KeyboardFocus::Editor(PaneId(0)), class, visible),
                RouteDecision::Unrouted,
                "under Editor focus {class:?} must never route toward the terminal"
            );
        }
    }
}
