use winit::{
    event::{ElementState, KeyEvent, MouseButton},
    keyboard::{Key, KeyCode, ModifiersState, NamedKey, PhysicalKey},
};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum TerminalKeyAction {
    Write(Vec<u8>),
    Interrupt,
    RestartSession,
    TerminateSession,
    ScrollbackPageUp,
    ScrollbackPageDown,
    ScrollbackTop,
    ScrollbackBottom,
    LetPasteShortcutHandle,
    LetCopyShortcutHandle,
    ConsumeRelease,
    Ignore,
}

pub(super) fn terminal_key_action(
    event: &KeyEvent,
    modifiers: ModifiersState,
    application_cursor_keys: bool,
    application_keypad: bool,
) -> TerminalKeyAction {
    if event.state == ElementState::Released {
        return if consumes_release(event) {
            TerminalKeyAction::ConsumeRelease
        } else {
            TerminalKeyAction::Ignore
        };
    }
    if modifiers.control_key() {
        if modifiers.shift_key() && matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyR)) {
            return TerminalKeyAction::RestartSession;
        }
        if modifiers.shift_key() && matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyK)) {
            return TerminalKeyAction::TerminateSession;
        }
        if matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyC)) {
            return terminal_ctrl_c_action(modifiers);
        }
        if matches!(event.physical_key, PhysicalKey::Code(KeyCode::KeyV)) {
            return TerminalKeyAction::LetPasteShortcutHandle;
        }
    }
    if modifiers.shift_key()
        && let Key::Named(key) = &event.logical_key
            && let Some(action) = terminal_shift_named_key_action(*key)
        {
            return action;
        }
    if application_keypad
        && let PhysicalKey::Code(code) = event.physical_key
        && let Some(sequence) = application_keypad_sequence(code)
    {
        return TerminalKeyAction::Write(sequence);
    }
    match &event.logical_key {
        Key::Character(text) => terminal_character_sequence(text, modifiers)
            .map(TerminalKeyAction::Write)
            .unwrap_or(TerminalKeyAction::Ignore),
        Key::Named(NamedKey::Space) => terminal_space_sequence(modifiers)
            .map(TerminalKeyAction::Write)
            .unwrap_or(TerminalKeyAction::Ignore),
        Key::Named(NamedKey::Enter) => TerminalKeyAction::Write(b"\r".to_vec()),
        Key::Named(NamedKey::Backspace) => TerminalKeyAction::Write(b"\x7f".to_vec()),
        Key::Named(NamedKey::Tab) => terminal_tab_sequence(modifiers)
            .map(TerminalKeyAction::Write)
            .unwrap_or(TerminalKeyAction::Ignore),
        Key::Named(NamedKey::ArrowLeft) => {
            TerminalKeyAction::Write(arrow_key_sequence(application_cursor_keys, modifiers, b'D'))
        }
        Key::Named(NamedKey::ArrowRight) => {
            TerminalKeyAction::Write(arrow_key_sequence(application_cursor_keys, modifiers, b'C'))
        }
        Key::Named(NamedKey::ArrowUp) => {
            TerminalKeyAction::Write(arrow_key_sequence(application_cursor_keys, modifiers, b'A'))
        }
        Key::Named(NamedKey::ArrowDown) => {
            TerminalKeyAction::Write(arrow_key_sequence(application_cursor_keys, modifiers, b'B'))
        }
        Key::Named(NamedKey::Home) => {
            TerminalKeyAction::Write(arrow_key_sequence(application_cursor_keys, modifiers, b'H'))
        }
        Key::Named(NamedKey::End) => {
            TerminalKeyAction::Write(arrow_key_sequence(application_cursor_keys, modifiers, b'F'))
        }
        Key::Named(NamedKey::Escape) => TerminalKeyAction::Write(b"\x1b".to_vec()),
        Key::Named(key) => terminal_named_key_sequence(*key, modifiers)
            .map(TerminalKeyAction::Write)
            .unwrap_or(TerminalKeyAction::Ignore),
        _ => TerminalKeyAction::Ignore,
    }
}

pub(super) fn terminal_focus_event_sequence(focused: bool) -> &'static [u8] {
    if focused { b"\x1b[I" } else { b"\x1b[O" }
}

pub(super) fn terminal_sgr_mouse_button_sequence(
    button: MouseButton,
    pressed: bool,
    column: u16,
    row: u16,
) -> Option<Vec<u8>> {
    let button_code = match button {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
        _ => return None,
    };
    Some(terminal_sgr_mouse_sequence(
        button_code,
        pressed,
        column,
        row,
    ))
}

pub(super) fn terminal_sgr_mouse_wheel_sequence(
    scroll_lines: f32,
    column: u16,
    row: u16,
) -> Option<Vec<u8>> {
    if scroll_lines.abs() <= 0.01 {
        return None;
    }
    let button_code = if scroll_lines > 0.0 { 64 } else { 65 };
    Some(terminal_sgr_mouse_sequence(button_code, true, column, row))
}

pub(super) fn terminal_sgr_mouse_motion_sequence(
    held_button: Option<MouseButton>,
    column: u16,
    row: u16,
) -> Option<Vec<u8>> {
    let button_code = match held_button {
        Some(MouseButton::Left) => 32,
        Some(MouseButton::Middle) => 33,
        Some(MouseButton::Right) => 34,
        Some(_) => return None,
        None => 35,
    };
    Some(terminal_sgr_mouse_sequence(button_code, true, column, row))
}

pub(super) fn terminal_x10_mouse_button_sequence(
    button: MouseButton,
    pressed: bool,
    column: u16,
    row: u16,
) -> Option<Vec<u8>> {
    let button_code = if pressed {
        mouse_button_code(button)?
    } else {
        3
    };
    Some(terminal_x10_mouse_sequence(button_code, column, row))
}

pub(super) fn terminal_x10_mouse_wheel_sequence(
    scroll_lines: f32,
    column: u16,
    row: u16,
) -> Option<Vec<u8>> {
    if scroll_lines.abs() <= 0.01 {
        return None;
    }
    let button_code = if scroll_lines > 0.0 { 64 } else { 65 };
    Some(terminal_x10_mouse_sequence(button_code, column, row))
}

pub(super) fn terminal_x10_mouse_motion_sequence(
    held_button: MouseButton,
    column: u16,
    row: u16,
) -> Option<Vec<u8>> {
    let button_code = mouse_button_code(held_button)? + 32;
    Some(terminal_x10_mouse_sequence(button_code, column, row))
}

pub(super) fn terminal_utf8_mouse_button_sequence(
    button: MouseButton,
    pressed: bool,
    column: u16,
    row: u16,
) -> Option<Vec<u8>> {
    let button_code = if pressed {
        mouse_button_code(button)?
    } else {
        3
    };
    Some(terminal_utf8_mouse_sequence(button_code, column, row))
}

pub(super) fn terminal_utf8_mouse_wheel_sequence(
    scroll_lines: f32,
    column: u16,
    row: u16,
) -> Option<Vec<u8>> {
    if scroll_lines.abs() <= 0.01 {
        return None;
    }
    let button_code = if scroll_lines > 0.0 { 64 } else { 65 };
    Some(terminal_utf8_mouse_sequence(button_code, column, row))
}

pub(super) fn terminal_utf8_mouse_motion_sequence(
    held_button: MouseButton,
    column: u16,
    row: u16,
) -> Option<Vec<u8>> {
    let button_code = mouse_button_code(held_button)? + 32;
    Some(terminal_utf8_mouse_sequence(button_code, column, row))
}

pub(super) fn terminal_urxvt_mouse_button_sequence(
    button: MouseButton,
    pressed: bool,
    column: u16,
    row: u16,
) -> Option<Vec<u8>> {
    let button_code = if pressed {
        mouse_button_code(button)?
    } else {
        3
    };
    Some(terminal_urxvt_mouse_sequence(button_code, column, row))
}

pub(super) fn terminal_urxvt_mouse_wheel_sequence(
    scroll_lines: f32,
    column: u16,
    row: u16,
) -> Option<Vec<u8>> {
    if scroll_lines.abs() <= 0.01 {
        return None;
    }
    let button_code = if scroll_lines > 0.0 { 64 } else { 65 };
    Some(terminal_urxvt_mouse_sequence(button_code, column, row))
}

pub(super) fn terminal_urxvt_mouse_motion_sequence(
    held_button: MouseButton,
    column: u16,
    row: u16,
) -> Option<Vec<u8>> {
    let button_code = mouse_button_code(held_button)? + 32;
    Some(terminal_urxvt_mouse_sequence(button_code, column, row))
}

fn terminal_sgr_mouse_sequence(button_code: u16, pressed: bool, column: u16, row: u16) -> Vec<u8> {
    let suffix = if pressed { 'M' } else { 'm' };
    format!(
        "\x1b[<{};{};{}{}",
        button_code,
        column.max(1),
        row.max(1),
        suffix
    )
    .into_bytes()
}

fn terminal_x10_mouse_sequence(button_code: u8, column: u16, row: u16) -> Vec<u8> {
    vec![
        b'\x1b',
        b'[',
        b'M',
        button_code.saturating_add(32),
        terminal_x10_coordinate_byte(column),
        terminal_x10_coordinate_byte(row),
    ]
}

fn terminal_x10_coordinate_byte(value: u16) -> u8 {
    value.clamp(1, 223) as u8 + 32
}

fn terminal_utf8_mouse_sequence(button_code: u8, column: u16, row: u16) -> Vec<u8> {
    let mut sequence = b"\x1b[M".to_vec();
    sequence.extend(terminal_utf8_mouse_codepoint(button_code as u32 + 32));
    sequence.extend(terminal_utf8_mouse_codepoint(column.max(1) as u32 + 32));
    sequence.extend(terminal_utf8_mouse_codepoint(row.max(1) as u32 + 32));
    sequence
}

fn terminal_utf8_mouse_codepoint(value: u32) -> Vec<u8> {
    char::from_u32(value)
        .unwrap_or('\u{fffd}')
        .to_string()
        .into_bytes()
}

fn terminal_urxvt_mouse_sequence(button_code: u8, column: u16, row: u16) -> Vec<u8> {
    format!(
        "\x1b[{};{};{}M",
        button_code.saturating_add(32),
        column.max(1),
        row.max(1)
    )
    .into_bytes()
}

fn mouse_button_code(button: MouseButton) -> Option<u8> {
    match button {
        MouseButton::Left => Some(0),
        MouseButton::Middle => Some(1),
        MouseButton::Right => Some(2),
        _ => None,
    }
}

fn cursor_key_sequence(application_cursor_keys: bool, final_byte: u8) -> Vec<u8> {
    if application_cursor_keys {
        vec![b'\x1b', b'O', final_byte]
    } else {
        vec![b'\x1b', b'[', final_byte]
    }
}

fn arrow_key_sequence(
    application_cursor_keys: bool,
    modifiers: ModifiersState,
    final_byte: u8,
) -> Vec<u8> {
    if let Some(modifier_param) = xterm_modifier_param(modifiers) {
        return format!("\x1b[1;{}{}", modifier_param, final_byte as char).into_bytes();
    }
    cursor_key_sequence(application_cursor_keys, final_byte)
}

fn xterm_modifier_param(modifiers: ModifiersState) -> Option<u8> {
    let shift = modifiers.shift_key() as u8;
    let alt = modifiers.alt_key() as u8;
    let control = modifiers.control_key() as u8;
    let bits = shift + (alt << 1) + (control << 2);
    (bits > 0).then_some(bits + 1)
}

fn terminal_character_sequence(text: &str, modifiers: ModifiersState) -> Option<Vec<u8>> {
    if modifiers.control_key() {
        return (!modifiers.alt_key())
            .then(|| control_character_sequence(text))
            .flatten();
    }
    let mut bytes = Vec::new();
    if modifiers.alt_key() {
        bytes.push(b'\x1b');
    }
    bytes.extend_from_slice(text.as_bytes());
    Some(bytes)
}

fn terminal_space_sequence(modifiers: ModifiersState) -> Option<Vec<u8>> {
    if modifiers.control_key() {
        return (!modifiers.alt_key()).then_some(vec![0x00]);
    }
    let mut bytes = Vec::new();
    if modifiers.alt_key() {
        bytes.push(b'\x1b');
    }
    bytes.push(b' ');
    Some(bytes)
}

fn terminal_tab_sequence(modifiers: ModifiersState) -> Option<Vec<u8>> {
    if modifiers.alt_key() {
        return None;
    }
    if modifiers.shift_key() {
        return Some(b"\x1b[Z".to_vec());
    }
    Some(b"\t".to_vec())
}

fn control_character_sequence(text: &str) -> Option<Vec<u8>> {
    let byte = text.as_bytes().first().copied()?;
    let control = match byte {
        b'a'..=b'z' => byte - b'a' + 1,
        b'A'..=b'Z' => byte - b'A' + 1,
        b'[' => 0x1b,
        b'\\' => 0x1c,
        b']' => 0x1d,
        b'^' => 0x1e,
        b'_' => 0x1f,
        b'?' => 0x7f,
        _ => return None,
    };
    Some(vec![control])
}

fn terminal_named_key_sequence(key: NamedKey, modifiers: ModifiersState) -> Option<Vec<u8>> {
    let tilde_param = match key {
        NamedKey::Insert => Some(2),
        NamedKey::Delete => Some(3),
        NamedKey::PageUp => Some(5),
        NamedKey::PageDown => Some(6),
        NamedKey::F5 => Some(15),
        NamedKey::F6 => Some(17),
        NamedKey::F7 => Some(18),
        NamedKey::F8 => Some(19),
        NamedKey::F9 => Some(20),
        NamedKey::F10 => Some(21),
        NamedKey::F11 => Some(23),
        NamedKey::F12 => Some(24),
        _ => None,
    };
    if let Some(param) = tilde_param {
        return Some(xterm_tilde_sequence(param, modifiers));
    }
    let function_final = match key {
        NamedKey::F1 => b'P',
        NamedKey::F2 => b'Q',
        NamedKey::F3 => b'R',
        NamedKey::F4 => b'S',
        _ => return None,
    };
    Some(xterm_function_sequence(function_final, modifiers))
}

fn xterm_tilde_sequence(param: u8, modifiers: ModifiersState) -> Vec<u8> {
    if let Some(modifier_param) = xterm_modifier_param(modifiers) {
        format!("\x1b[{};{}~", param, modifier_param).into_bytes()
    } else {
        format!("\x1b[{}~", param).into_bytes()
    }
}

fn xterm_function_sequence(final_byte: u8, modifiers: ModifiersState) -> Vec<u8> {
    if let Some(modifier_param) = xterm_modifier_param(modifiers) {
        format!("\x1b[1;{}{}", modifier_param, final_byte as char).into_bytes()
    } else {
        vec![b'\x1b', b'O', final_byte]
    }
}

fn application_keypad_sequence(key: KeyCode) -> Option<Vec<u8>> {
    let final_byte = match key {
        KeyCode::Numpad0 => b'p',
        KeyCode::Numpad1 => b'q',
        KeyCode::Numpad2 => b'r',
        KeyCode::Numpad3 => b's',
        KeyCode::Numpad4 => b't',
        KeyCode::Numpad5 => b'u',
        KeyCode::Numpad6 => b'v',
        KeyCode::Numpad7 => b'w',
        KeyCode::Numpad8 => b'x',
        KeyCode::Numpad9 => b'y',
        KeyCode::NumpadDecimal => b'n',
        KeyCode::NumpadComma => b'l',
        KeyCode::NumpadSubtract => b'm',
        KeyCode::NumpadAdd => b'k',
        KeyCode::NumpadMultiply | KeyCode::NumpadStar => b'j',
        KeyCode::NumpadDivide => b'o',
        KeyCode::NumpadEnter => b'M',
        KeyCode::NumpadEqual => b'X',
        _ => return None,
    };
    Some(vec![b'\x1b', b'O', final_byte])
}

fn terminal_ctrl_c_action(modifiers: ModifiersState) -> TerminalKeyAction {
    if modifiers.shift_key() {
        TerminalKeyAction::LetCopyShortcutHandle
    } else {
        TerminalKeyAction::Interrupt
    }
}

fn terminal_shift_named_key_action(key: NamedKey) -> Option<TerminalKeyAction> {
    match key {
        NamedKey::PageUp => Some(TerminalKeyAction::ScrollbackPageUp),
        NamedKey::PageDown => Some(TerminalKeyAction::ScrollbackPageDown),
        NamedKey::Home => Some(TerminalKeyAction::ScrollbackTop),
        NamedKey::End => Some(TerminalKeyAction::ScrollbackBottom),
        _ => None,
    }
}

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
                | KeyCode::KeyR
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
mod tests {
    use super::*;
    #[test]
    fn ctrl_c_interrupts_but_ctrl_shift_c_defers_to_copy() {
        let ctrl = ModifiersState::CONTROL;
        let ctrl_shift = ModifiersState::CONTROL | ModifiersState::SHIFT;

        assert!(matches!(
            terminal_ctrl_c_action(ctrl),
            TerminalKeyAction::Interrupt
        ));
        assert!(matches!(
            terminal_ctrl_c_action(ctrl_shift),
            TerminalKeyAction::LetCopyShortcutHandle
        ));
    }

    #[test]
    fn shift_navigation_controls_terminal_scrollback() {
        for (key, action) in [
            (NamedKey::PageUp, TerminalKeyAction::ScrollbackPageUp),
            (NamedKey::PageDown, TerminalKeyAction::ScrollbackPageDown),
            (NamedKey::Home, TerminalKeyAction::ScrollbackTop),
            (NamedKey::End, TerminalKeyAction::ScrollbackBottom),
        ] {
            assert_eq!(terminal_shift_named_key_action(key), Some(action));
        }
        assert!(terminal_shift_named_key_action(NamedKey::ArrowUp).is_none());
        assert!(terminal_shift_named_key_action(NamedKey::Escape).is_none());
    }

    #[test]
    fn cursor_key_sequences_use_csi_ss3_and_xterm_modifier_params() {
        for final_byte in [b'A', b'B', b'C', b'D', b'H', b'F'] {
            assert_eq!(
                cursor_key_sequence(false, final_byte),
                vec![b'\x1b', b'[', final_byte]
            );
            assert_eq!(
                cursor_key_sequence(true, final_byte),
                vec![b'\x1b', b'O', final_byte]
            );
        }
        for final_byte in [b'D', b'H'] {
            let expected = format!("\x1b[1;5{}", final_byte as char).into_bytes();
            assert_eq!(
                arrow_key_sequence(false, ModifiersState::CONTROL, final_byte),
                expected
            );
        }
        assert_eq!(
            arrow_key_sequence(true, ModifiersState::SHIFT | ModifiersState::ALT, b'A'),
            b"\x1b[1;4A".to_vec()
        );
        assert_eq!(
            arrow_key_sequence(true, ModifiersState::empty(), b'A'),
            b"\x1bOA".to_vec()
        );
    }

    #[test]
    fn terminal_character_sequence_prefixes_alt_text_like_native_terminals() {
        assert_eq!(
            terminal_character_sequence("f", ModifiersState::empty()),
            Some(b"f".to_vec())
        );
        assert_eq!(
            terminal_character_sequence("f", ModifiersState::ALT),
            Some(b"\x1bf".to_vec())
        );
        assert_eq!(
            terminal_character_sequence("é", ModifiersState::ALT),
            Some(b"\x1b\xc3\xa9".to_vec())
        );
        assert_eq!(
            terminal_character_sequence("f", ModifiersState::CONTROL | ModifiersState::ALT),
            None
        );
    }

    #[test]
    fn terminal_character_sequence_maps_control_text_like_native_terminals() {
        assert_eq!(
            terminal_character_sequence("a", ModifiersState::CONTROL),
            Some(vec![0x01])
        );
        assert_eq!(
            terminal_character_sequence("D", ModifiersState::CONTROL),
            Some(vec![0x04])
        );
        assert_eq!(
            terminal_character_sequence("[", ModifiersState::CONTROL),
            Some(vec![0x1b])
        );
        assert_eq!(
            terminal_character_sequence("?", ModifiersState::CONTROL),
            Some(vec![0x7f])
        );
        assert_eq!(
            terminal_character_sequence("1", ModifiersState::CONTROL),
            None
        );
    }

    #[test]
    fn terminal_space_and_tab_sequences_honor_modifiers() {
        let empty = ModifiersState::empty();
        let ctrl_alt = ModifiersState::CONTROL | ModifiersState::ALT;
        assert_eq!(terminal_focus_event_sequence(true), b"\x1b[I");
        assert_eq!(terminal_focus_event_sequence(false), b"\x1b[O");
        assert_eq!(terminal_space_sequence(empty), Some(b" ".to_vec()));
        assert_eq!(
            terminal_space_sequence(ModifiersState::ALT),
            Some(b"\x1b ".to_vec())
        );
        assert_eq!(
            terminal_space_sequence(ModifiersState::CONTROL),
            Some(vec![0x00])
        );
        assert_eq!(terminal_space_sequence(ctrl_alt), None);
        assert_eq!(terminal_tab_sequence(empty), Some(b"\t".to_vec()));
        assert_eq!(
            terminal_tab_sequence(ModifiersState::SHIFT),
            Some(b"\x1b[Z".to_vec())
        );
        assert_eq!(terminal_tab_sequence(ModifiersState::ALT), None);
    }

    #[test]
    fn named_navigation_keys_emit_xterm_sequences() {
        let empty = ModifiersState::empty();
        let shift_alt = ModifiersState::SHIFT | ModifiersState::ALT;
        assert_eq!(
            terminal_named_key_sequence(NamedKey::Insert, empty).unwrap(),
            b"\x1b[2~"
        );
        assert_eq!(
            terminal_named_key_sequence(NamedKey::Delete, empty).unwrap(),
            b"\x1b[3~"
        );
        assert_eq!(
            terminal_named_key_sequence(NamedKey::PageUp, empty).unwrap(),
            b"\x1b[5~"
        );
        assert_eq!(
            terminal_named_key_sequence(NamedKey::PageDown, empty).unwrap(),
            b"\x1b[6~"
        );
        assert_eq!(
            terminal_named_key_sequence(NamedKey::PageDown, ModifiersState::CONTROL).unwrap(),
            b"\x1b[6;5~"
        );
        assert_eq!(
            terminal_named_key_sequence(NamedKey::Delete, shift_alt).unwrap(),
            b"\x1b[3;4~"
        );
        let empty = ModifiersState::empty();
        assert_eq!(
            terminal_named_key_sequence(NamedKey::F1, empty).unwrap(),
            b"\x1bOP"
        );
        assert_eq!(
            terminal_named_key_sequence(NamedKey::F12, empty).unwrap(),
            b"\x1b[24~"
        );
        assert_eq!(
            terminal_named_key_sequence(NamedKey::F1, ModifiersState::CONTROL).unwrap(),
            b"\x1b[1;5P"
        );
        assert_eq!(
            terminal_named_key_sequence(NamedKey::F12, shift_alt).unwrap(),
            b"\x1b[24;4~"
        );
    }

    #[test]
    fn application_keypad_sequence_maps_physical_numpad_keys_to_ss3() {
        assert_eq!(
            application_keypad_sequence(KeyCode::Numpad0),
            Some(b"\x1bOp".to_vec())
        );
        assert_eq!(
            application_keypad_sequence(KeyCode::Numpad9),
            Some(b"\x1bOy".to_vec())
        );
        assert_eq!(
            application_keypad_sequence(KeyCode::NumpadDecimal),
            Some(b"\x1bOn".to_vec())
        );
        assert_eq!(
            application_keypad_sequence(KeyCode::NumpadEnter),
            Some(b"\x1bOM".to_vec())
        );
        assert_eq!(
            application_keypad_sequence(KeyCode::NumpadDivide),
            Some(b"\x1bOo".to_vec())
        );
        assert_eq!(application_keypad_sequence(KeyCode::Digit1), None);
    }

    #[test]
    fn sgr_mouse_button_sequence_uses_one_based_coordinates() {
        assert_eq!(
            terminal_sgr_mouse_button_sequence(MouseButton::Left, true, 0, 0),
            Some(b"\x1b[<0;1;1M".to_vec())
        );
        assert_eq!(
            terminal_sgr_mouse_button_sequence(MouseButton::Left, false, 12, 7),
            Some(b"\x1b[<0;12;7m".to_vec())
        );
        assert_eq!(
            terminal_sgr_mouse_button_sequence(MouseButton::Right, true, 12, 7),
            Some(b"\x1b[<2;12;7M".to_vec())
        );
    }

    #[test]
    fn sgr_mouse_wheel_sequence_maps_scroll_direction() {
        assert_eq!(
            terminal_sgr_mouse_wheel_sequence(1.0, 3, 4),
            Some(b"\x1b[<64;3;4M".to_vec())
        );
        assert_eq!(
            terminal_sgr_mouse_wheel_sequence(-1.0, 3, 4),
            Some(b"\x1b[<65;3;4M".to_vec())
        );
        assert_eq!(terminal_sgr_mouse_wheel_sequence(0.0, 3, 4), None);
    }

    #[test]
    fn sgr_mouse_motion_sequence_maps_drag_and_any_motion() {
        assert_eq!(
            terminal_sgr_mouse_motion_sequence(Some(MouseButton::Left), 5, 6),
            Some(b"\x1b[<32;5;6M".to_vec())
        );
        assert_eq!(
            terminal_sgr_mouse_motion_sequence(Some(MouseButton::Right), 5, 6),
            Some(b"\x1b[<34;5;6M".to_vec())
        );
        assert_eq!(
            terminal_sgr_mouse_motion_sequence(None, 5, 6),
            Some(b"\x1b[<35;5;6M".to_vec())
        );
        assert_eq!(
            terminal_sgr_mouse_motion_sequence(Some(MouseButton::Other(9)), 5, 6),
            None
        );
    }

    #[test]
    fn x10_mouse_button_sequence_uses_legacy_coordinate_bytes() {
        assert_eq!(
            terminal_x10_mouse_button_sequence(MouseButton::Left, true, 0, 0),
            Some(vec![0x1b, b'[', b'M', b' ', b'!', b'!'])
        );
        assert_eq!(
            terminal_x10_mouse_button_sequence(MouseButton::Left, false, 12, 7),
            Some(vec![0x1b, b'[', b'M', b'#', b',', b'\''])
        );
        assert_eq!(
            terminal_x10_mouse_button_sequence(MouseButton::Right, true, 12, 7),
            Some(vec![0x1b, b'[', b'M', b'"', b',', b'\''])
        );
    }

    #[test]
    fn x10_mouse_wheel_and_motion_sequences_map_codes() {
        assert_eq!(
            terminal_x10_mouse_wheel_sequence(1.0, 3, 4),
            Some(vec![0x1b, b'[', b'M', b'`', b'#', b'$'])
        );
        assert_eq!(
            terminal_x10_mouse_wheel_sequence(-1.0, 3, 4),
            Some(vec![0x1b, b'[', b'M', b'a', b'#', b'$'])
        );
        assert_eq!(terminal_x10_mouse_wheel_sequence(0.0, 3, 4), None);
        assert_eq!(
            terminal_x10_mouse_motion_sequence(MouseButton::Left, 5, 6),
            Some(vec![0x1b, b'[', b'M', b'@', b'%', b'&'])
        );
    }

    #[test]
    fn utf8_mouse_button_sequence_matches_x10_for_ascii_coordinates() {
        assert_eq!(
            terminal_utf8_mouse_button_sequence(MouseButton::Left, true, 0, 0),
            Some(vec![0x1b, b'[', b'M', b' ', b'!', b'!'])
        );
        assert_eq!(
            terminal_utf8_mouse_button_sequence(MouseButton::Left, false, 12, 7),
            Some(vec![0x1b, b'[', b'M', b'#', b',', b'\''])
        );
    }

    #[test]
    fn utf8_mouse_sequence_encodes_extended_coordinates_as_utf8() {
        assert_eq!(
            terminal_utf8_mouse_button_sequence(MouseButton::Right, true, 200, 1),
            Some(vec![0x1b, b'[', b'M', b'"', 0xc3, 0xa8, b'!'])
        );
        assert_eq!(
            terminal_utf8_mouse_wheel_sequence(1.0, 200, 4),
            Some(vec![0x1b, b'[', b'M', b'`', 0xc3, 0xa8, b'$'])
        );
        assert_eq!(terminal_utf8_mouse_wheel_sequence(0.0, 3, 4), None);
        assert_eq!(
            terminal_utf8_mouse_motion_sequence(MouseButton::Left, 5, 6),
            Some(vec![0x1b, b'[', b'M', b'@', b'%', b'&'])
        );
    }

    #[test]
    fn urxvt_mouse_button_sequence_uses_decimal_params() {
        assert_eq!(
            terminal_urxvt_mouse_button_sequence(MouseButton::Left, true, 0, 0),
            Some(b"\x1b[32;1;1M".to_vec())
        );
        assert_eq!(
            terminal_urxvt_mouse_button_sequence(MouseButton::Left, false, 12, 7),
            Some(b"\x1b[35;12;7M".to_vec())
        );
        assert_eq!(
            terminal_urxvt_mouse_button_sequence(MouseButton::Right, true, 12, 7),
            Some(b"\x1b[34;12;7M".to_vec())
        );
    }

    #[test]
    fn urxvt_mouse_wheel_and_motion_sequences_map_codes() {
        assert_eq!(
            terminal_urxvt_mouse_wheel_sequence(1.0, 3, 4),
            Some(b"\x1b[96;3;4M".to_vec())
        );
        assert_eq!(
            terminal_urxvt_mouse_wheel_sequence(-1.0, 3, 4),
            Some(b"\x1b[97;3;4M".to_vec())
        );
        assert_eq!(terminal_urxvt_mouse_wheel_sequence(0.0, 3, 4), None);
        assert_eq!(
            terminal_urxvt_mouse_motion_sequence(MouseButton::Left, 5, 6),
            Some(b"\x1b[64;5;6M".to_vec())
        );
    }
}
