//! Legacy mouse encoders retained only for shadow-equivalence tests.

use winit::event::MouseButton;

#[cfg(test)]
pub(super) fn terminal_focus_event_sequence(focused: bool) -> &'static [u8] {
    if focused { b"\x1b[I" } else { b"\x1b[O" }
}

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
pub(super) fn terminal_x10_mouse_motion_sequence(
    held_button: MouseButton,
    column: u16,
    row: u16,
) -> Option<Vec<u8>> {
    let button_code = mouse_button_code(held_button)? + 32;
    Some(terminal_x10_mouse_sequence(button_code, column, row))
}

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
pub(super) fn terminal_utf8_mouse_motion_sequence(
    held_button: MouseButton,
    column: u16,
    row: u16,
) -> Option<Vec<u8>> {
    let button_code = mouse_button_code(held_button)? + 32;
    Some(terminal_utf8_mouse_sequence(button_code, column, row))
}

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
pub(super) fn terminal_urxvt_mouse_motion_sequence(
    held_button: MouseButton,
    column: u16,
    row: u16,
) -> Option<Vec<u8>> {
    let button_code = mouse_button_code(held_button)? + 32;
    Some(terminal_urxvt_mouse_sequence(button_code, column, row))
}

#[cfg(test)]
pub(super) fn terminal_sgr_mouse_sequence(
    button_code: u16,
    pressed: bool,
    column: u16,
    row: u16,
) -> Vec<u8> {
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

#[cfg(test)]
pub(super) fn terminal_x10_mouse_sequence(button_code: u8, column: u16, row: u16) -> Vec<u8> {
    vec![
        b'\x1b',
        b'[',
        b'M',
        button_code.saturating_add(32),
        terminal_x10_coordinate_byte(column),
        terminal_x10_coordinate_byte(row),
    ]
}

#[cfg(test)]
pub(super) fn terminal_x10_coordinate_byte(value: u16) -> u8 {
    value.clamp(1, 223) as u8 + 32
}

#[cfg(test)]
pub(super) fn terminal_utf8_mouse_sequence(button_code: u8, column: u16, row: u16) -> Vec<u8> {
    let mut sequence = b"\x1b[M".to_vec();
    sequence.extend(terminal_utf8_mouse_codepoint(button_code as u32 + 32));
    sequence.extend(terminal_utf8_mouse_codepoint(column.max(1) as u32 + 32));
    sequence.extend(terminal_utf8_mouse_codepoint(row.max(1) as u32 + 32));
    sequence
}

#[cfg(test)]
pub(super) fn terminal_utf8_mouse_codepoint(value: u32) -> Vec<u8> {
    char::from_u32(value)
        .unwrap_or('\u{fffd}')
        .to_string()
        .into_bytes()
}

#[cfg(test)]
pub(super) fn terminal_urxvt_mouse_sequence(button_code: u8, column: u16, row: u16) -> Vec<u8> {
    format!(
        "\x1b[{};{};{}M",
        button_code.saturating_add(32),
        column.max(1),
        row.max(1)
    )
    .into_bytes()
}

#[cfg(test)]
pub(super) fn mouse_button_code(button: MouseButton) -> Option<u8> {
    match button {
        MouseButton::Left => Some(0),
        MouseButton::Middle => Some(1),
        MouseButton::Right => Some(2),
        _ => None,
    }
}
