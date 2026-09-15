//! Legacy and native terminal input contract tests.

use super::*;
use winit::event::MouseButton;

#[test]
fn conventional_terminal_font_zoom_shortcuts_are_chrome_owned() {
    for (key, modifiers, action) in [
        (
            KeyCode::Equal,
            ModifiersState::CONTROL,
            TerminalKeyAction::FontZoomIn,
        ),
        (
            KeyCode::Equal,
            ModifiersState::CONTROL | ModifiersState::SHIFT,
            TerminalKeyAction::FontZoomIn,
        ),
        (
            KeyCode::Minus,
            ModifiersState::CONTROL,
            TerminalKeyAction::FontZoomOut,
        ),
        (
            KeyCode::Digit0,
            ModifiersState::CONTROL,
            TerminalKeyAction::FontZoomReset,
        ),
    ] {
        assert_eq!(
            terminal_font_zoom_shortcut(
                ElementState::Pressed,
                false,
                PhysicalKey::Code(key),
                modifiers,
            ),
            Some(action),
        );
    }
    assert_eq!(
        terminal_font_zoom_shortcut(
            ElementState::Pressed,
            false,
            PhysicalKey::Code(KeyCode::Equal),
            ModifiersState::CONTROL | ModifiersState::ALT,
        ),
        None,
    );
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
