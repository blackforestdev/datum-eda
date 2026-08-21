use datum_terminal_core::{
    KeyCode as CoreKeyCode, KeyEventKind as CoreKeyEventKind, KeyInput as CoreKeyInput,
    KeyModifiers as CoreKeyModifiers, KeypadKey,
};
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{Key, KeyCode, ModifiersState, NamedKey, PhysicalKey},
};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum TerminalKeyAction {
    CoreKey(CoreKeyInput),
    NewSession,
    RestartSession,
    TerminateSession,
    CloseSession,
    ScrollbackPageUp,
    ScrollbackPageDown,
    ScrollbackTop,
    ScrollbackBottom,
    CopyClipboard,
    PasteClipboard,
    Search,
    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TerminalClipboardShortcut {
    Copy,
    Paste,
}

pub(super) fn terminal_clipboard_shortcut(
    state: ElementState,
    repeat: bool,
    physical_key: PhysicalKey,
    modifiers: ModifiersState,
) -> Option<TerminalClipboardShortcut> {
    if state != ElementState::Pressed || repeat || !modifiers.shift_key() {
        return None;
    }
    match physical_key {
        PhysicalKey::Code(KeyCode::KeyC) if modifiers.control_key() => {
            Some(TerminalClipboardShortcut::Copy)
        }
        PhysicalKey::Code(KeyCode::KeyV) if modifiers.control_key() => {
            Some(TerminalClipboardShortcut::Paste)
        }
        PhysicalKey::Code(KeyCode::Insert) => Some(TerminalClipboardShortcut::Paste),
        _ => None,
    }
}

pub(super) fn terminal_key_action(
    event: &KeyEvent,
    modifiers: ModifiersState,
    application_cursor_keys: bool,
    application_keypad: bool,
) -> TerminalKeyAction {
    if event.state == ElementState::Released {
        return terminal_core_key_input(event, modifiers)
            .map(TerminalKeyAction::CoreKey)
            .unwrap_or(TerminalKeyAction::Ignore);
    }
    if terminal_new_session_shortcut(event.state, event.repeat, event.physical_key, modifiers) {
        return TerminalKeyAction::NewSession;
    }
    if terminal_search_shortcut(event.state, event.repeat, event.physical_key, modifiers) {
        return TerminalKeyAction::Search;
    }
    if let Some(shortcut) =
        terminal_clipboard_shortcut(event.state, event.repeat, event.physical_key, modifiers)
    {
        return match shortcut {
            TerminalClipboardShortcut::Copy => TerminalKeyAction::CopyClipboard,
            TerminalClipboardShortcut::Paste => TerminalKeyAction::PasteClipboard,
        };
    }
    if modifiers.control_key() {
        if modifiers.shift_key() && matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyR)) {
            return TerminalKeyAction::RestartSession;
        }
        if modifiers.shift_key() && matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyK)) {
            return TerminalKeyAction::TerminateSession;
        }
        if modifiers.shift_key() && matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyW)) {
            return TerminalKeyAction::CloseSession;
        }
    }
    if modifiers.shift_key()
        && let Key::Named(key) = &event.logical_key
        && let Some(action) = terminal_shift_named_key_action(*key)
    {
        return action;
    }
    let _ = (application_cursor_keys, application_keypad);
    terminal_core_key_input(event, modifiers)
        .map(TerminalKeyAction::CoreKey)
        .unwrap_or(TerminalKeyAction::Ignore)
}

pub(super) fn terminal_core_key_input(
    event: &KeyEvent,
    modifiers: ModifiersState,
) -> Option<CoreKeyInput> {
    let code = match &event.logical_key {
        Key::Character(text) => CoreKeyCode::Text(text.to_string()),
        Key::Named(NamedKey::Space) => CoreKeyCode::Text(" ".into()),
        Key::Named(NamedKey::Escape) => CoreKeyCode::Escape,
        Key::Named(NamedKey::Enter) => CoreKeyCode::Enter,
        Key::Named(NamedKey::Tab) => CoreKeyCode::Tab,
        Key::Named(NamedKey::Backspace) => CoreKeyCode::Backspace,
        Key::Named(NamedKey::Insert) => CoreKeyCode::Insert,
        Key::Named(NamedKey::Delete) => CoreKeyCode::Delete,
        Key::Named(NamedKey::ArrowLeft) => CoreKeyCode::Left,
        Key::Named(NamedKey::ArrowRight) => CoreKeyCode::Right,
        Key::Named(NamedKey::ArrowUp) => CoreKeyCode::Up,
        Key::Named(NamedKey::ArrowDown) => CoreKeyCode::Down,
        Key::Named(NamedKey::PageUp) => CoreKeyCode::PageUp,
        Key::Named(NamedKey::PageDown) => CoreKeyCode::PageDown,
        Key::Named(NamedKey::Home) => CoreKeyCode::Home,
        Key::Named(NamedKey::End) => CoreKeyCode::End,
        Key::Named(key) => named_function_key(*key).map(CoreKeyCode::Function)?,
        _ => physical_keypad(event.physical_key).map(CoreKeyCode::Keypad)?,
    };
    Some(CoreKeyInput {
        code,
        shifted_key: None,
        base_layout_key: None,
        modifiers: CoreKeyModifiers {
            shift: modifiers.shift_key(),
            alt: modifiers.alt_key(),
            control: modifiers.control_key(),
            super_key: modifiers.super_key(),
            hyper: false,
            meta: false,
        },
        kind: if event.state == ElementState::Released {
            CoreKeyEventKind::Release
        } else if event.repeat {
            CoreKeyEventKind::Repeat
        } else {
            CoreKeyEventKind::Press
        },
    })
}

fn named_function_key(key: NamedKey) -> Option<u8> {
    Some(match key {
        NamedKey::F1 => 1,
        NamedKey::F2 => 2,
        NamedKey::F3 => 3,
        NamedKey::F4 => 4,
        NamedKey::F5 => 5,
        NamedKey::F6 => 6,
        NamedKey::F7 => 7,
        NamedKey::F8 => 8,
        NamedKey::F9 => 9,
        NamedKey::F10 => 10,
        NamedKey::F11 => 11,
        NamedKey::F12 => 12,
        NamedKey::F13 => 13,
        NamedKey::F14 => 14,
        NamedKey::F15 => 15,
        NamedKey::F16 => 16,
        NamedKey::F17 => 17,
        NamedKey::F18 => 18,
        NamedKey::F19 => 19,
        NamedKey::F20 => 20,
        NamedKey::F21 => 21,
        NamedKey::F22 => 22,
        NamedKey::F23 => 23,
        NamedKey::F24 => 24,
        NamedKey::F25 => 25,
        NamedKey::F26 => 26,
        NamedKey::F27 => 27,
        NamedKey::F28 => 28,
        NamedKey::F29 => 29,
        NamedKey::F30 => 30,
        NamedKey::F31 => 31,
        NamedKey::F32 => 32,
        NamedKey::F33 => 33,
        NamedKey::F34 => 34,
        NamedKey::F35 => 35,
        _ => return None,
    })
}

fn physical_keypad(key: PhysicalKey) -> Option<KeypadKey> {
    let PhysicalKey::Code(key) = key else {
        return None;
    };
    Some(match key {
        KeyCode::Numpad0 => KeypadKey::Digit(0),
        KeyCode::Numpad1 => KeypadKey::Digit(1),
        KeyCode::Numpad2 => KeypadKey::Digit(2),
        KeyCode::Numpad3 => KeypadKey::Digit(3),
        KeyCode::Numpad4 => KeypadKey::Digit(4),
        KeyCode::Numpad5 => KeypadKey::Digit(5),
        KeyCode::Numpad6 => KeypadKey::Digit(6),
        KeyCode::Numpad7 => KeypadKey::Digit(7),
        KeyCode::Numpad8 => KeypadKey::Digit(8),
        KeyCode::Numpad9 => KeypadKey::Digit(9),
        KeyCode::NumpadDecimal => KeypadKey::Decimal,
        KeyCode::NumpadDivide => KeypadKey::Divide,
        KeyCode::NumpadMultiply => KeypadKey::Multiply,
        KeyCode::NumpadSubtract => KeypadKey::Subtract,
        KeyCode::NumpadAdd => KeypadKey::Add,
        KeyCode::NumpadEnter => KeypadKey::Enter,
        KeyCode::NumpadEqual => KeypadKey::Equal,
        _ => return None,
    })
}

pub(super) fn terminal_new_session_shortcut(
    state: ElementState,
    repeat: bool,
    physical_key: PhysicalKey,
    modifiers: ModifiersState,
) -> bool {
    state == ElementState::Pressed
        && !repeat
        && modifiers.control_key()
        && modifiers.shift_key()
        && matches!(physical_key, PhysicalKey::Code(KeyCode::KeyT))
}

pub(super) fn terminal_search_shortcut(
    state: ElementState,
    repeat: bool,
    physical_key: PhysicalKey,
    modifiers: ModifiersState,
) -> bool {
    state == ElementState::Pressed
        && !repeat
        && modifiers.control_key()
        && modifiers.shift_key()
        && matches!(physical_key, PhysicalKey::Code(KeyCode::KeyF))
}

#[cfg(test)]
#[path = "terminal_input_legacy.rs"]
mod legacy;
#[cfg(test)]
#[path = "terminal_input_legacy_mouse.rs"]
mod legacy_mouse;
#[cfg(test)]
use legacy::*;
#[cfg(test)]
pub(crate) use legacy::{
    terminal_character_sequence, terminal_space_sequence, terminal_tab_sequence,
};
#[cfg(test)]
use legacy_mouse::*;
fn terminal_shift_named_key_action(key: NamedKey) -> Option<TerminalKeyAction> {
    match key {
        NamedKey::PageUp => Some(TerminalKeyAction::ScrollbackPageUp),
        NamedKey::PageDown => Some(TerminalKeyAction::ScrollbackPageDown),
        NamedKey::Home => Some(TerminalKeyAction::ScrollbackTop),
        NamedKey::End => Some(TerminalKeyAction::ScrollbackBottom),
        _ => None,
    }
}

#[cfg(test)]
#[allow(dead_code)]
fn consumes_release(event: &KeyEvent) -> bool {
    matches!(
        event.logical_key,
        Key::Named(
            NamedKey::Enter
                | NamedKey::Backspace
                | NamedKey::Tab
                | NamedKey::ArrowLeft
                | NamedKey::ArrowRight
                | NamedKey::ArrowUp
                | NamedKey::ArrowDown
                | NamedKey::Home
                | NamedKey::End
                | NamedKey::Insert
                | NamedKey::Delete
                | NamedKey::PageUp
                | NamedKey::PageDown
                | NamedKey::F1
                | NamedKey::F2
                | NamedKey::F3
                | NamedKey::F4
                | NamedKey::F5
                | NamedKey::F6
                | NamedKey::F7
                | NamedKey::F8
                | NamedKey::F9
                | NamedKey::F10
                | NamedKey::F11
                | NamedKey::F12
                | NamedKey::Escape
        )
    ) || matches!(
        event.physical_key,
        PhysicalKey::Code(
            KeyCode::KeyC
                | KeyCode::KeyV
                | KeyCode::KeyK
                | KeyCode::KeyW
                | KeyCode::KeyR
                | KeyCode::KeyT
                | KeyCode::NumpadEnter
                | KeyCode::Numpad0
                | KeyCode::Numpad1
                | KeyCode::Numpad2
                | KeyCode::Numpad3
                | KeyCode::Numpad4
                | KeyCode::Numpad5
                | KeyCode::Numpad6
                | KeyCode::Numpad7
                | KeyCode::Numpad8
                | KeyCode::Numpad9
                | KeyCode::NumpadDecimal
                | KeyCode::NumpadComma
                | KeyCode::NumpadSubtract
                | KeyCode::NumpadAdd
                | KeyCode::NumpadMultiply
                | KeyCode::NumpadStar
                | KeyCode::NumpadDivide
                | KeyCode::NumpadEqual
        )
    )
}

#[cfg(test)]
#[path = "terminal_workspace_hotkey_tests.rs"]
mod terminal_workspace_hotkey_tests;

#[cfg(test)]
#[path = "terminal_input_tests.rs"]
mod tests;
