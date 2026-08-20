use crate::{
    KeyCode, KeyEventKind, KeyInput, KeyModifiers, KeypadKey, MediaKey, ModifierKey, TerminalCore,
};

const KITTY_DISAMBIGUATE: u8 = 1;
const KITTY_REPORT_EVENTS: u8 = 2;
const KITTY_REPORT_ALTERNATE: u8 = 4;
const KITTY_REPORT_ALL: u8 = 8;
const KITTY_ASSOCIATED_TEXT: u8 = 16;

pub(crate) fn encode_key(core: &TerminalCore, input: &KeyInput) -> Option<Vec<u8>> {
    let flags = core.state.kitty_keyboard.flags;
    if should_encode_kitty(input, flags) {
        return kitty(input, flags);
    }
    legacy(core, input)
}

fn should_encode_kitty(input: &KeyInput, flags: u8) -> bool {
    if input.kind == KeyEventKind::Release && flags & KITTY_REPORT_EVENTS == 0 {
        return false;
    }
    if flags & KITTY_REPORT_ALL != 0 {
        return true;
    }
    if flags & KITTY_REPORT_EVENTS != 0 && input.kind != KeyEventKind::Press {
        return true;
    }
    flags & KITTY_DISAMBIGUATE != 0
        && (!matches!(input.code, KeyCode::Text(_))
            || input.modifiers.control
            || input.modifiers.alt
            || input.modifiers.super_key
            || input.modifiers.hyper
            || input.modifiers.meta)
        && !matches!(
            input.code,
            KeyCode::Enter | KeyCode::Tab | KeyCode::Backspace
        )
}

fn kitty(input: &KeyInput, flags: u8) -> Option<Vec<u8>> {
    let code = kitty_code(&input.code)?;
    let modifiers = modifier_parameter(input.modifiers);
    let event = match input.kind {
        KeyEventKind::Press => 1,
        KeyEventKind::Repeat => 2,
        KeyEventKind::Release => 3,
    };
    let mut sequence = format!("\x1b[{code}");
    if flags & KITTY_REPORT_ALTERNATE != 0 {
        if let Some(shifted) = input.shifted_key {
            sequence.push(':');
            sequence.push_str(&shifted.to_string());
        }
        if let Some(base_layout) = input.base_layout_key {
            if input.shifted_key.is_none() {
                sequence.push(':');
            }
            sequence.push(':');
            sequence.push_str(&base_layout.to_string());
        }
    }
    sequence.push_str(&format!(";{modifiers}:{event}"));
    if flags & KITTY_ASSOCIATED_TEXT != 0
        && let KeyCode::Text(text) = &input.code
        && !text.is_empty()
    {
        sequence.push(';');
        for (index, character) in text.chars().enumerate() {
            if index != 0 {
                sequence.push(':');
            }
            sequence.push_str(&(character as u32).to_string());
        }
    }
    sequence.push('u');
    Some(sequence.into_bytes())
}

fn kitty_code(code: &KeyCode) -> Option<u32> {
    Some(match code {
        KeyCode::Text(text) => text.chars().next()? as u32,
        KeyCode::Escape => 27,
        KeyCode::Enter => 13,
        KeyCode::Tab => 9,
        KeyCode::Backspace => 127,
        KeyCode::Insert => 57_348,
        KeyCode::Delete => 57_349,
        KeyCode::Left => 57_350,
        KeyCode::Right => 57_351,
        KeyCode::Up => 57_352,
        KeyCode::Down => 57_353,
        KeyCode::PageUp => 57_354,
        KeyCode::PageDown => 57_355,
        KeyCode::Home => 57_356,
        KeyCode::End => 57_357,
        KeyCode::Function(number @ 1..=35) => 57_363 + u32::from(*number),
        KeyCode::Function(_) => return None,
        KeyCode::Keypad(key) => kitty_keypad_code(*key)?,
        KeyCode::CapsLock => 57_358,
        KeyCode::ScrollLock => 57_359,
        KeyCode::NumLock => 57_360,
        KeyCode::PrintScreen => 57_361,
        KeyCode::Pause => 57_362,
        KeyCode::Menu => 57_363,
        KeyCode::Media(key) => 57_428 + media_offset(*key),
        KeyCode::Modifier(key) => 57_441 + modifier_offset(*key),
    })
}

fn legacy(core: &TerminalCore, input: &KeyInput) -> Option<Vec<u8>> {
    if input.kind == KeyEventKind::Release {
        return None;
    }
    let modifiers = input.modifiers;
    if let KeyCode::Text(text) = &input.code {
        let mut bytes = if modifiers.control {
            control_text(text)?
        } else {
            text.as_bytes().to_vec()
        };
        if modifiers.alt {
            bytes.insert(0, 0x1b);
        }
        return Some(bytes);
    }
    if let KeyCode::Keypad(key) = input.code {
        return keypad(key, core.state.modes.application_keypad);
    }
    let parameter = modifier_parameter(modifiers);
    let modified = parameter != 1;
    let bytes = match input.code {
        KeyCode::Escape => vec![0x1b],
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Tab if modifiers.shift => b"\x1b[Z".to_vec(),
        KeyCode::Tab => vec![b'\t'],
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Up | KeyCode::Down | KeyCode::Right | KeyCode::Left => {
            let final_byte = match input.code {
                KeyCode::Up => 'A',
                KeyCode::Down => 'B',
                KeyCode::Right => 'C',
                _ => 'D',
            };
            if modified {
                format!("\x1b[1;{parameter}{final_byte}").into_bytes()
            } else if core.state.modes.application_cursor {
                format!("\x1bO{final_byte}").into_bytes()
            } else {
                format!("\x1b[{final_byte}").into_bytes()
            }
        }
        KeyCode::Home | KeyCode::End => {
            let final_byte = if input.code == KeyCode::Home {
                'H'
            } else {
                'F'
            };
            if modified {
                format!("\x1b[1;{parameter}{final_byte}").into_bytes()
            } else if core.state.modes.application_cursor {
                format!("\x1bO{final_byte}").into_bytes()
            } else {
                format!("\x1b[{final_byte}").into_bytes()
            }
        }
        KeyCode::Insert | KeyCode::Delete | KeyCode::PageUp | KeyCode::PageDown => {
            let number = match input.code {
                KeyCode::Insert => 2,
                KeyCode::Delete => 3,
                KeyCode::PageUp => 5,
                _ => 6,
            };
            tilde(number, parameter, modified)
        }
        KeyCode::Function(number) => function(number, parameter, modified)?,
        KeyCode::CapsLock
        | KeyCode::ScrollLock
        | KeyCode::NumLock
        | KeyCode::PrintScreen
        | KeyCode::Pause
        | KeyCode::Menu
        | KeyCode::Media(_)
        | KeyCode::Modifier(_) => return None,
        KeyCode::Text(_) | KeyCode::Keypad(_) => unreachable!(),
    };
    Some(if modifiers.alt && !matches!(input.code, KeyCode::Escape) {
        let mut prefixed = vec![0x1b];
        prefixed.extend(bytes);
        prefixed
    } else {
        bytes
    })
}

fn modifier_parameter(modifiers: KeyModifiers) -> u8 {
    1 + u8::from(modifiers.shift)
        + 2 * u8::from(modifiers.alt)
        + 4 * u8::from(modifiers.control)
        + 8 * u8::from(modifiers.super_key)
        + 16 * u8::from(modifiers.hyper)
        + 32 * u8::from(modifiers.meta)
}

fn control_text(text: &str) -> Option<Vec<u8>> {
    let character = text.chars().next()?;
    if text.chars().count() != 1 {
        return None;
    }
    let byte = match character {
        '@' | ' ' | '2' => 0,
        'a'..='z' => character as u8 - b'a' + 1,
        'A'..='Z' => character as u8 - b'A' + 1,
        '[' | '3' => 0x1b,
        '\\' | '4' => 0x1c,
        ']' | '5' => 0x1d,
        '^' | '6' => 0x1e,
        '_' | '7' => 0x1f,
        '?' | '8' => 0x7f,
        _ => return None,
    };
    Some(vec![byte])
}

fn tilde(number: u8, parameter: u8, modified: bool) -> Vec<u8> {
    if modified {
        format!("\x1b[{number};{parameter}~").into_bytes()
    } else {
        format!("\x1b[{number}~").into_bytes()
    }
}

fn function(number: u8, parameter: u8, modified: bool) -> Option<Vec<u8>> {
    if (1..=4).contains(&number) {
        let final_byte = char::from(b'P' + number - 1);
        return Some(if modified {
            format!("\x1b[1;{parameter}{final_byte}").into_bytes()
        } else {
            format!("\x1bO{final_byte}").into_bytes()
        });
    }
    let code = match number {
        5..=10 => [15, 17, 18, 19, 20, 21][usize::from(number - 5)],
        11..=14 => [23, 24, 25, 26][usize::from(number - 11)],
        15..=16 => [28, 29][usize::from(number - 15)],
        17..=20 => [31, 32, 33, 34][usize::from(number - 17)],
        21..=35 => 42 + number - 21,
        _ => return None,
    };
    Some(tilde(code, parameter, modified))
}

fn keypad(key: KeypadKey, application: bool) -> Option<Vec<u8>> {
    if !application {
        let bytes = match key {
            KeypadKey::Left => b"\x1b[D".as_slice(),
            KeypadKey::Right => b"\x1b[C".as_slice(),
            KeypadKey::Up => b"\x1b[A".as_slice(),
            KeypadKey::Down => b"\x1b[B".as_slice(),
            KeypadKey::PageUp => b"\x1b[5~".as_slice(),
            KeypadKey::PageDown => b"\x1b[6~".as_slice(),
            KeypadKey::Home => b"\x1b[H".as_slice(),
            KeypadKey::End => b"\x1b[F".as_slice(),
            KeypadKey::Insert => b"\x1b[2~".as_slice(),
            KeypadKey::Delete => b"\x1b[3~".as_slice(),
            KeypadKey::Begin => b"\x1b[E".as_slice(),
            _ => keypad_text(key)?.as_bytes(),
        };
        return Some(bytes.to_vec());
    }
    let final_byte = match key {
        KeypadKey::Digit(value @ 0..=9) => b'p' + value,
        KeypadKey::Digit(_) => return None,
        KeypadKey::Decimal => b'n',
        KeypadKey::Divide => b'o',
        KeypadKey::Multiply => b'j',
        KeypadKey::Subtract => b'm',
        KeypadKey::Add => b'k',
        KeypadKey::Enter => b'M',
        KeypadKey::Equal => b'X',
        KeypadKey::Separator => b'l',
        KeypadKey::Left => b't',
        KeypadKey::Right => b'v',
        KeypadKey::Up => b'x',
        KeypadKey::Down => b'r',
        KeypadKey::PageUp => b'y',
        KeypadKey::PageDown => b's',
        KeypadKey::Home => b'w',
        KeypadKey::End => b'q',
        KeypadKey::Insert => b'p',
        KeypadKey::Delete => b'n',
        KeypadKey::Begin => b'u',
    };
    Some(vec![0x1b, b'O', final_byte])
}

fn keypad_text(key: KeypadKey) -> Option<&'static str> {
    Some(match key {
        KeypadKey::Digit(0) => "0",
        KeypadKey::Digit(1) => "1",
        KeypadKey::Digit(2) => "2",
        KeypadKey::Digit(3) => "3",
        KeypadKey::Digit(4) => "4",
        KeypadKey::Digit(5) => "5",
        KeypadKey::Digit(6) => "6",
        KeypadKey::Digit(7) => "7",
        KeypadKey::Digit(8) => "8",
        KeypadKey::Digit(9) => "9",
        KeypadKey::Digit(_) => return None,
        KeypadKey::Decimal => ".",
        KeypadKey::Divide => "/",
        KeypadKey::Multiply => "*",
        KeypadKey::Subtract => "-",
        KeypadKey::Add => "+",
        KeypadKey::Enter => "\r",
        KeypadKey::Equal => "=",
        KeypadKey::Separator => ",",
        KeypadKey::Left
        | KeypadKey::Right
        | KeypadKey::Up
        | KeypadKey::Down
        | KeypadKey::PageUp
        | KeypadKey::PageDown
        | KeypadKey::Home
        | KeypadKey::End
        | KeypadKey::Insert
        | KeypadKey::Delete
        | KeypadKey::Begin => return None,
    })
}

const fn kitty_keypad_code(key: KeypadKey) -> Option<u32> {
    Some(match key {
        KeypadKey::Digit(value @ 0..=9) => 57_399 + value as u32,
        KeypadKey::Digit(_) => return None,
        KeypadKey::Decimal => 57_409,
        KeypadKey::Divide => 57_410,
        KeypadKey::Multiply => 57_411,
        KeypadKey::Subtract => 57_412,
        KeypadKey::Add => 57_413,
        KeypadKey::Enter => 57_414,
        KeypadKey::Equal => 57_415,
        KeypadKey::Separator => 57_416,
        KeypadKey::Left => 57_417,
        KeypadKey::Right => 57_418,
        KeypadKey::Up => 57_419,
        KeypadKey::Down => 57_420,
        KeypadKey::PageUp => 57_421,
        KeypadKey::PageDown => 57_422,
        KeypadKey::Home => 57_423,
        KeypadKey::End => 57_424,
        KeypadKey::Insert => 57_425,
        KeypadKey::Delete => 57_426,
        KeypadKey::Begin => 57_427,
    })
}

const fn media_offset(key: MediaKey) -> u32 {
    match key {
        MediaKey::Play => 0,
        MediaKey::Pause => 1,
        MediaKey::PlayPause => 2,
        MediaKey::Reverse => 3,
        MediaKey::Stop => 4,
        MediaKey::FastForward => 5,
        MediaKey::Rewind => 6,
        MediaKey::Next => 7,
        MediaKey::Previous => 8,
        MediaKey::Record => 9,
        MediaKey::VolumeDown => 10,
        MediaKey::VolumeUp => 11,
        MediaKey::Mute => 12,
    }
}

const fn modifier_offset(key: ModifierKey) -> u32 {
    match key {
        ModifierKey::LeftShift => 0,
        ModifierKey::LeftControl => 1,
        ModifierKey::LeftAlt => 2,
        ModifierKey::LeftSuper => 3,
        ModifierKey::LeftHyper => 4,
        ModifierKey::LeftMeta => 5,
        ModifierKey::RightShift => 6,
        ModifierKey::RightControl => 7,
        ModifierKey::RightAlt => 8,
        ModifierKey::RightSuper => 9,
        ModifierKey::RightHyper => 10,
        ModifierKey::RightMeta => 11,
        ModifierKey::IsoLevel3Shift => 12,
        ModifierKey::IsoLevel5Shift => 13,
    }
}
