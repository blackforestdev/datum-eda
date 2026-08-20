use crate::*;

#[test]
fn legacy_keys_cover_text_control_meta_cursor_keypad_function_and_modifiers() {
    let limits = limits(4_096, 16);
    let mut core = core(limits);
    assert_bytes(&core, key(KeyCode::Text("x".into())), b"x");
    assert_bytes(
        &core,
        key_with(KeyCode::Text("c".into()), modifiers(false, false, true)),
        &[3],
    );
    assert_bytes(
        &core,
        key_with(KeyCode::Text("x".into()), modifiers(false, true, false)),
        b"\x1bx",
    );
    assert_bytes(&core, key(KeyCode::Up), b"\x1b[A");
    assert_bytes(&core, key(KeyCode::Keypad(KeypadKey::Home)), b"\x1b[H");
    apply(&mut core, limits, b"\x1b[?1h");
    assert_bytes(&core, key(KeyCode::Up), b"\x1bOA");
    assert_bytes(
        &core,
        key_with(KeyCode::Left, modifiers(true, false, true)),
        b"\x1b[1;6D",
    );
    apply(&mut core, limits, b"\x1b=");
    assert_bytes(&core, key(KeyCode::Keypad(KeypadKey::Digit(1))), b"\x1bOq");
    assert_bytes(&core, key(KeyCode::Function(5)), b"\x1b[15~");
    assert_bytes(&core, key(KeyCode::Function(35)), b"\x1b[56~");
    assert_eq!(
        core.encode_key(&KeyInput {
            kind: KeyEventKind::Release,
            ..key(KeyCode::Up)
        })
        .unwrap(),
        InputDisposition::Ignored
    );
}

#[test]
fn kitty_keyboard_negotiation_is_chunk_invariant_bounded_and_queryable() {
    let limits = limits(4_096, 2);
    let mut whole = core(limits);
    let mut bytewise = core(limits);
    let stream = b"\x1b[>3u\x1b[>8u\x1b[<1u\x1b[=16;2u\x1b[?u";
    let whole_updates = apply_chunks(&mut whole, limits, stream, &[stream.len()]);
    let byte_updates = apply_chunks(&mut bytewise, limits, stream, &[1; 64]);
    assert_eq!(whole.state().kitty_keyboard().flags(), 19);
    assert_eq!(whole.state().kitty_keyboard().stack_depth(), 1);
    assert_eq!(
        whole.state().kitty_keyboard(),
        bytewise.state().kitty_keyboard()
    );
    assert_eq!(reply_bytes(&whole_updates), reply_bytes(&byte_updates));
    assert!(reply_bytes(&whole_updates).contains(&b"\x1b[?19u".to_vec()));

    whole
        .apply(Action::Csi(CsiSequence {
            private_markers: vec![b'>'],
            parameters: vec![],
            intermediates: vec![],
            final_byte: b'u',
        }))
        .unwrap();
    let error = whole
        .apply(Action::Csi(CsiSequence {
            private_markers: vec![b'>'],
            parameters: vec![],
            intermediates: vec![],
            final_byte: b'u',
        }))
        .unwrap_err();
    assert!(matches!(
        error,
        CoreError::Limit(LimitError::Exceeded {
            kind: LimitKind::KeyboardStack,
            ..
        })
    ));
}

#[test]
fn kitty_encoding_reports_events_and_text_while_flag_one_keeps_plain_tab() {
    let limits = limits(4_096, 16);
    let mut core = core(limits);
    apply(&mut core, limits, b"\x1b[=1;1u");
    assert_bytes(&core, key(KeyCode::Tab), b"\t");
    assert_bytes(
        &core,
        key_with(KeyCode::Up, modifiers(true, false, false)),
        b"\x1b[57352;2:1u",
    );

    apply(&mut core, limits, b"\x1b[=30;1u");
    assert_bytes(
        &core,
        KeyInput {
            code: KeyCode::Text("é".into()),
            shifted_key: Some('É' as u32),
            base_layout_key: Some('e' as u32),
            modifiers: KeyModifiers::default(),
            kind: KeyEventKind::Repeat,
        },
        b"\x1b[233:201:101;1:2;233u",
    );
    assert_bytes(
        &core,
        KeyInput {
            code: KeyCode::Up,
            shifted_key: None,
            base_layout_key: None,
            modifiers: KeyModifiers::default(),
            kind: KeyEventKind::Release,
        },
        b"\x1b[57352;1:3u",
    );
    assert_bytes(
        &core,
        key(KeyCode::Keypad(KeypadKey::Digit(1))),
        b"\x1b[57400;1:1u",
    );
    assert_bytes(
        &core,
        key(KeyCode::Media(MediaKey::Mute)),
        b"\x1b[57440;1:1u",
    );
    assert_bytes(
        &core,
        key(KeyCode::Modifier(ModifierKey::IsoLevel5Shift)),
        b"\x1b[57454;1:1u",
    );
}

#[test]
fn focus_paste_and_ime_commit_have_distinct_transmission_contracts() {
    let limits = limits(4_096, 16);
    let mut core = core(limits);
    assert_eq!(
        core.encode_focus(FocusInput::Gained).unwrap(),
        InputDisposition::Ignored
    );
    apply(&mut core, limits, b"\x1b[?1004h\x1b[?2004h");
    assert_eq!(
        core.encode_focus(FocusInput::Gained).unwrap().bytes(),
        Some(b"\x1b[I".as_slice())
    );
    assert_eq!(
        core.encode_focus(FocusInput::Lost).unwrap().bytes(),
        Some(b"\x1b[O".as_slice())
    );
    assert_eq!(
        core.encode_paste("a\nb").unwrap().bytes(),
        Some(b"\x1b[200~a\nb\x1b[201~".as_slice())
    );
    assert_eq!(
        core.encode_ime(&ImeInput::Preedit("draft".into())).unwrap(),
        InputDisposition::LocalOnly
    );
    assert_eq!(
        core.encode_ime(&ImeInput::Commit("文".into()))
            .unwrap()
            .bytes(),
        Some("文".as_bytes())
    );
}

#[test]
fn every_mouse_family_clips_coordinates_and_preserves_release_identity() {
    let limits = limits(4_096, 16);
    let mut core = core(limits);
    apply(&mut core, limits, b"\x1b[?1003h\x1b[?1006h");
    let press = mouse(MouseAction::Press(MouseButton::Left), 3, 4);
    assert_eq!(
        core.encode_mouse(press).unwrap().bytes(),
        Some(b"\x1b[<0;4;5M".as_slice())
    );
    assert_eq!(
        core.encode_mouse(MouseInput {
            action: MouseAction::Release(MouseButton::Left),
            position: MousePosition {
                column: 9_999,
                row: -20,
                ..MousePosition::default()
            },
            modifiers: KeyModifiers::default(),
            local_override: false,
        })
        .unwrap()
        .bytes(),
        Some(b"\x1b[<0;20;1m".as_slice())
    );
    apply(&mut core, limits, b"\x1b[?1006l\x1b[?1015h");
    assert_eq!(
        core.encode_mouse(press).unwrap().bytes(),
        Some(b"\x1b[32;4;5M".as_slice())
    );
    assert_eq!(
        core.encode_mouse(MouseInput {
            action: MouseAction::Release(MouseButton::Left),
            ..press
        })
        .unwrap()
        .bytes(),
        Some(b"\x1b[35;4;5M".as_slice())
    );
    apply(&mut core, limits, b"\x1b[?1015l\x1b[?1005h");
    assert!(
        core.encode_mouse(press)
            .unwrap()
            .bytes()
            .unwrap()
            .starts_with(b"\x1b[M")
    );
    apply(&mut core, limits, b"\x1b[?1005l");
    assert_eq!(
        core.encode_mouse(press).unwrap().bytes(),
        Some(&[0x1b, b'[', b'M', 32, 36, 37][..])
    );
}

#[test]
fn pixel_mouse_and_local_override_are_explicit_and_never_escape_bounds() {
    let limits = limits(4_096, 16);
    let mut core = core(limits);
    apply(&mut core, limits, b"\x1b[?1003h\x1b[?1016h");
    let input = MouseInput {
        action: MouseAction::Move(None),
        position: MousePosition {
            pixel_x: 9_999,
            pixel_y: -1,
            ..MousePosition::default()
        },
        modifiers: KeyModifiers::default(),
        local_override: false,
    };
    assert_eq!(
        core.encode_mouse(input).unwrap().bytes(),
        Some(b"\x1b[<35;200;1M".as_slice())
    );
    assert_eq!(
        core.encode_mouse(MouseInput {
            local_override: true,
            ..input
        })
        .unwrap(),
        InputDisposition::LocalOnly
    );
}

#[test]
fn tracking_modes_filter_motion_release_and_press_without_crossing_local_policy() {
    let limits = limits(4_096, 16);
    let mut core = core(limits);
    apply(&mut core, limits, b"\x1b[?9h");
    assert!(
        core.encode_mouse(mouse(MouseAction::Press(MouseButton::Left), 0, 0))
            .unwrap()
            .bytes()
            .is_some()
    );
    assert_eq!(
        core.encode_mouse(mouse(MouseAction::Release(MouseButton::Left), 0, 0))
            .unwrap(),
        InputDisposition::Ignored
    );
    apply(&mut core, limits, b"\x1b[?9l\x1b[?1002h");
    assert_eq!(
        core.encode_mouse(mouse(MouseAction::Move(None), 0, 0))
            .unwrap(),
        InputDisposition::Ignored
    );
    assert!(
        core.encode_mouse(mouse(MouseAction::Move(Some(MouseButton::Left)), 0, 0))
            .unwrap()
            .bytes()
            .is_some()
    );
}

#[test]
fn input_byte_limit_rejects_complete_key_paste_and_commit_without_prefixes() {
    let limits = limits(4, 16);
    let core = core(limits);
    for result in [
        core.encode_paste("12345"),
        core.encode_ime(&ImeInput::Commit("12345".into())),
        core.encode_key(&key(KeyCode::Text("12345".into()))),
    ] {
        assert!(matches!(
            result,
            Err(InputError::Limit(LimitError::Exceeded {
                kind: LimitKind::InputBytes,
                ..
            }))
        ));
    }
}

#[test]
fn reset_clears_negotiated_input_protocol_state() {
    let limits = limits(4_096, 16);
    let mut core = core(limits);
    apply(
        &mut core,
        limits,
        b"\x1b[?1h\x1b[?66h\x1b[?1003h\x1b[?1006h\x1b[=31;1u\x1bc",
    );
    assert!(!core.state().modes().application_cursor);
    assert!(!core.state().modes().application_keypad);
    assert_eq!(core.state().mouse_tracking(), MouseTracking::Off);
    assert_eq!(core.state().mouse_encoding(), MouseEncoding::Default);
    assert_eq!(core.state().kitty_keyboard().flags(), 0);
}

fn limits(input_bytes: usize, keyboard_stack: usize) -> CoreLimits {
    CoreLimits::try_from(CoreLimitValues {
        parameter_count: 4_096,
        parameter_digits: 4_096,
        parameter_value: 4_096,
        subparameter_count: 4_096,
        intermediate_bytes: 4_096,
        control_string_bytes: 4_096,
        cluster_bytes: 4_096,
        title_bytes: 4_096,
        working_directory_bytes: 4_096,
        clipboard_bytes: 4_096,
        input_bytes,
        keyboard_stack,
        notification_bytes: 4_096,
        reply_bytes: 4_096,
        pending_events: 4_096,
        pending_damage: 4_096,
        history_lines: 4_096,
        history_bytes: 4_096,
        graphic_objects: 4_096,
        graphic_pixels: 4_096,
        graphic_decoded_bytes: 4_096,
        graphic_frames: 4_096,
        compression_ratio: 4_096,
        parser_work: 4_096,
        search_work: 4_096,
        reflow_work: 4_096,
        screen_cells: 4_096,
        snapshot_cells: 4_096,
    })
    .unwrap()
}

fn core(limits: CoreLimits) -> TerminalCore {
    TerminalCore::new(limits, TerminalSize::new(20, 6, 200, 120).unwrap()).unwrap()
}

fn apply(core: &mut TerminalCore, limits: CoreLimits, bytes: &[u8]) {
    apply_chunks(core, limits, bytes, &[bytes.len()]);
}

fn apply_chunks(
    core: &mut TerminalCore,
    limits: CoreLimits,
    bytes: &[u8],
    chunks: &[usize],
) -> Vec<CoreUpdate> {
    let mut parser = StreamingParser::new(limits);
    let mut updates = Vec::new();
    let mut offset = 0;
    let mut chunk = 0;
    while offset < bytes.len() {
        let length = chunks
            .get(chunk)
            .copied()
            .unwrap_or(bytes.len() - offset)
            .min(bytes.len() - offset);
        let mut actions = Vec::new();
        parser.feed(&bytes[offset..offset + length], |action| {
            actions.push(action)
        });
        for action in actions {
            updates.push(core.apply(action).unwrap());
        }
        offset += length;
        chunk += 1;
    }
    updates
}

fn reply_bytes(updates: &[CoreUpdate]) -> Vec<Vec<u8>> {
    updates
        .iter()
        .flat_map(CoreUpdate::replies)
        .map(|reply| reply.bytes().to_vec())
        .collect()
}

fn key(code: KeyCode) -> KeyInput {
    KeyInput {
        code,
        shifted_key: None,
        base_layout_key: None,
        modifiers: KeyModifiers::default(),
        kind: KeyEventKind::Press,
    }
}

fn modifiers(shift: bool, alt: bool, control: bool) -> KeyModifiers {
    KeyModifiers {
        shift,
        alt,
        control,
        ..KeyModifiers::default()
    }
}

fn key_with(code: KeyCode, modifiers: KeyModifiers) -> KeyInput {
    KeyInput {
        code,
        shifted_key: None,
        base_layout_key: None,
        modifiers,
        kind: KeyEventKind::Press,
    }
}

fn mouse(action: MouseAction, column: i64, row: i64) -> MouseInput {
    MouseInput {
        action,
        position: MousePosition {
            column,
            row,
            ..MousePosition::default()
        },
        modifiers: KeyModifiers::default(),
        local_override: false,
    }
}

fn assert_bytes(core: &TerminalCore, input: KeyInput, expected: &[u8]) {
    assert_eq!(core.encode_key(&input).unwrap().bytes(), Some(expected));
}
