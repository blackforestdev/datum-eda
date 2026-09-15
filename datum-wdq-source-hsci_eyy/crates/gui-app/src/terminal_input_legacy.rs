//! Legacy key encoder reference retained only for shadow-equivalence tests.

use crate::terminal_control_input::control_character_sequence;
use winit::keyboard::{KeyCode, ModifiersState, NamedKey};

#[cfg(test)]
pub(super) fn cursor_key_sequence(application_cursor_keys: bool, final_byte: u8) -> Vec<u8> {
    if application_cursor_keys {
        vec![b'\x1b', b'O', final_byte]
    } else {
        vec![b'\x1b', b'[', final_byte]
    }
}

#[cfg(test)]
pub(super) fn arrow_key_sequence(
    application_cursor_keys: bool,
    modifiers: ModifiersState,
    final_byte: u8,
) -> Vec<u8> {
    if let Some(modifier_param) = xterm_modifier_param(modifiers) {
        return format!("\x1b[1;{}{}", modifier_param, final_byte as char).into_bytes();
    }
    cursor_key_sequence(application_cursor_keys, final_byte)
}

#[cfg(test)]
pub(super) fn xterm_modifier_param(modifiers: ModifiersState) -> Option<u8> {
    let shift = modifiers.shift_key() as u8;
    let alt = modifiers.alt_key() as u8;
    let control = modifiers.control_key() as u8;
    let bits = shift + (alt << 1) + (control << 2);
    (bits > 0).then_some(bits + 1)
}

#[cfg(test)]
pub(crate) fn terminal_character_sequence(
    text: &str,
    modifiers: ModifiersState,
) -> Option<Vec<u8>> {
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

#[cfg(test)]
pub(crate) fn terminal_space_sequence(modifiers: ModifiersState) -> Option<Vec<u8>> {
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

#[cfg(test)]
pub(crate) fn terminal_tab_sequence(modifiers: ModifiersState) -> Option<Vec<u8>> {
    if modifiers.alt_key() {
        return None;
    }
    if modifiers.shift_key() {
        return Some(b"\x1b[Z".to_vec());
    }
    Some(b"\t".to_vec())
}

#[cfg(test)]
pub(super) fn terminal_named_key_sequence(
    key: NamedKey,
    modifiers: ModifiersState,
) -> Option<Vec<u8>> {
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

#[cfg(test)]
pub(super) fn xterm_tilde_sequence(param: u8, modifiers: ModifiersState) -> Vec<u8> {
    if let Some(modifier_param) = xterm_modifier_param(modifiers) {
        format!("\x1b[{};{}~", param, modifier_param).into_bytes()
    } else {
        format!("\x1b[{}~", param).into_bytes()
    }
}

#[cfg(test)]
pub(super) fn xterm_function_sequence(final_byte: u8, modifiers: ModifiersState) -> Vec<u8> {
    if let Some(modifier_param) = xterm_modifier_param(modifiers) {
        format!("\x1b[1;{}{}", modifier_param, final_byte as char).into_bytes()
    } else {
        vec![b'\x1b', b'O', final_byte]
    }
}

#[cfg(test)]
pub(super) fn application_keypad_sequence(key: KeyCode) -> Option<Vec<u8>> {
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
