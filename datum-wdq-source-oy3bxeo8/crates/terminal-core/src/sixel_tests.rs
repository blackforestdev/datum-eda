use super::*;

fn limits(graphic_pixels: usize, parser_work: usize, history_lines: usize) -> CoreLimits {
    CoreLimits::try_from(CoreLimitValues {
        parameter_count: 64,
        parameter_digits: 32,
        parameter_value: usize::MAX,
        subparameter_count: 64,
        intermediate_bytes: 64,
        control_string_bytes: 1 << 20,
        cluster_bytes: 4_096,
        title_bytes: 4_096,
        working_directory_bytes: 4_096,
        clipboard_bytes: 1 << 20,
        hyperlink_bytes: 1 << 20,
        input_bytes: 1 << 20,
        keyboard_stack: 64,
        notification_bytes: 4_096,
        reply_bytes: 4_096,
        pending_events: 1_024,
        pending_damage: 1_024,
        history_lines,
        history_bytes: 1 << 20,
        graphic_objects: 16,
        graphic_pixels,
        graphic_decoded_bytes: graphic_pixels.saturating_mul(4),
        graphic_frames: 16,
        compression_ratio: 1_024,
        parser_work,
        search_work: 1 << 20,
        reflow_work: 1 << 20,
        screen_cells: 1 << 20,
        snapshot_cells: 1 << 20,
    })
    .unwrap()
}

fn decoder_limits(pixels: usize, work: usize) -> SixelLimits {
    SixelLimits {
        pixels: GraphicPixelsLimit::new(pixels).unwrap(),
        decoded_bytes: GraphicDecodedBytesLimit::new(pixels * 4).unwrap(),
        work: ParserWorkLimit::new(work).unwrap(),
    }
}

fn dcs(bytes: &[u8]) -> Action {
    Action::ControlString(ControlString {
        kind: ControlStringKind::Dcs,
        bytes: bytes.to_vec(),
        terminator: StringTerminator::StringTerminator,
    })
}

fn rgb(red: u8, green: u8, blue: u8) -> Rgba8 {
    Rgba8 {
        red,
        green,
        blue,
        alpha: 255,
    }
}

#[test]
fn sixel_raster_carriage_return_rgb_and_hls_are_exact() {
    let mut registers = SixelColorRegisters::default();
    let image = decode_sixel(
        b"\"1;1;2;2#1;2;100;0;0@@$#2;1;120;50;100AA",
        Some(rgb(0, 0, 0)),
        &mut registers,
        PixelAspect::new(2, 1).unwrap(),
        decoder_limits(64, 1_024),
    )
    .unwrap();

    assert_eq!((image.width, image.height), (2, 2));
    assert_eq!(image.pixel_aspect, PixelAspect::SQUARE);
    assert_eq!(image.pixels, vec![rgb(255, 0, 0); 4]);
    assert_eq!(registers.color(1), rgb(255, 0, 0));
    assert_eq!(registers.color(2), rgb(255, 0, 0));
}

#[test]
fn repeat_newline_and_transparent_background_preserve_sparse_pixels() {
    let mut registers = SixelColorRegisters::default();
    let image = decode_sixel(
        b"#6!3@-$#1!2~",
        None,
        &mut registers,
        PixelAspect::SQUARE,
        decoder_limits(128, 4_096),
    )
    .unwrap();

    assert_eq!((image.width, image.height), (3, 12));
    assert_eq!(&image.pixels[..3], &[rgb(255, 255, 0); 3]);
    assert!(image.pixels[3..18].iter().all(|pixel| pixel.alpha == 0));
    for row in 6..12usize {
        assert_eq!(image.pixels[row * 3], rgb(0, 0, 255));
        assert_eq!(image.pixels[row * 3 + 1], rgb(0, 0, 255));
        assert_eq!(image.pixels[row * 3 + 2].alpha, 0);
    }
}

#[test]
fn malformed_and_hostile_sixel_fail_before_unbounded_allocation() {
    let mut registers = SixelColorRegisters::default();
    let original = registers.clone();
    assert!(matches!(
        decode_sixel(
            b"!x",
            None,
            &mut registers,
            PixelAspect::SQUARE,
            decoder_limits(64, 64),
        ),
        Err(SixelError::Malformed { .. })
    ));
    assert!(matches!(
        decode_sixel(
            b"!999999~",
            None,
            &mut registers,
            PixelAspect::SQUARE,
            decoder_limits(64, 64),
        ),
        Err(SixelError::Limit(LimitError::Exceeded {
            kind: LimitKind::ParserWork,
            maximum: 64,
            ..
        }))
    ));
    assert!(matches!(
        decode_sixel(
            b"\"1;1;999999999999999999999999;1@",
            None,
            &mut registers,
            PixelAspect::SQUARE,
            decoder_limits(64, 1_024),
        ),
        Err(SixelError::Malformed { .. })
    ));
    assert_eq!(registers, original);
}

#[test]
fn dec_color_register_defaults_hls_wheel_and_macro_aspects_are_exact() {
    let registers = SixelColorRegisters::default();
    let expected = [
        rgb(0, 0, 0),
        rgb(0, 0, 255),
        rgb(255, 0, 0),
        rgb(0, 255, 0),
        rgb(255, 0, 255),
        rgb(0, 255, 255),
        rgb(255, 255, 0),
        rgb(128, 128, 128),
    ];
    for (index, color) in expected.into_iter().enumerate() {
        assert_eq!(registers.color(index as u8), color);
    }

    let limits = limits(1_024, 8_192, 16);
    for (parameter, aspect) in [
        (0, PixelAspect::new(2, 1).unwrap()),
        (2, PixelAspect::new(5, 1).unwrap()),
        (3, PixelAspect::new(3, 1).unwrap()),
        (7, PixelAspect::SQUARE),
    ] {
        let mut core = TerminalCore::new(limits, TerminalSize::new(8, 4, 80, 40).unwrap()).unwrap();
        core.state.modes.sixel_scrolling = false;
        core.apply(dcs(format!("{parameter};1q#2@").as_bytes()))
            .unwrap();
        assert_eq!(
            core.state().graphics().next().unwrap().pixel_aspect(),
            aspect
        );
    }

    let mut hls_registers = SixelColorRegisters::default();
    decode_sixel(
        b"#20;1;0;50;100#21;1;120;50;100#22;1;240;50;100",
        None,
        &mut hls_registers,
        PixelAspect::SQUARE,
        decoder_limits(64, 1_024),
    )
    .unwrap();
    assert_eq!(hls_registers.color(20), rgb(0, 0, 255));
    assert_eq!(hls_registers.color(21), rgb(255, 0, 0));
    assert_eq!(hls_registers.color(22), rgb(0, 255, 0));
}

#[test]
fn dcs_sixel_adds_one_typed_placement_and_damage_event() {
    let limits = limits(1_024, 8_192, 16);
    let mut core = TerminalCore::new(limits, TerminalSize::new(4, 3, 40, 30).unwrap()).unwrap();
    let update = core
        .apply(dcs(b"0;1;0q\"1;1;2;2#2;2;100;0;0@@$AA"))
        .unwrap();

    let placement = core.state().graphics().next().unwrap();
    assert_eq!(placement.protocol(), GraphicProtocol::Sixel);
    assert_eq!((placement.width(), placement.height()), (2, 2));
    assert_eq!(placement.pixels()[0], rgb(255, 0, 0));
    assert_eq!(
        core.state().resolve_graphic(placement.id()),
        GraphicAnchorResolution::Screen {
            row: 0,
            column: 0,
            visible_pixel_width: 2,
            visible_pixel_height: 2,
        }
    );
    assert!(
        update
            .damage()
            .iter()
            .any(|damage| matches!(damage, Damage::Graphics | Damage::Full))
    );
    assert!(
        update
            .events()
            .contains(&CoreEvent::GraphicAdded(placement.id()))
    );
}

#[test]
fn sixel_palette_persists_unless_private_color_mode_is_enabled() {
    let limits = limits(2_048, 16_384, 16);
    let mut core = TerminalCore::new(limits, TerminalSize::new(8, 4, 80, 40).unwrap()).unwrap();
    core.apply(dcs(b"0;1q#9;2;100;0;0@")).unwrap();
    core.apply(dcs(b"0;1q#9@")).unwrap();
    assert_eq!(
        core.state().graphics().last().unwrap().pixels()[0],
        rgb(255, 0, 0)
    );

    core.apply(Action::Csi(CsiSequence {
        private_markers: vec![b'?'],
        parameters: vec![CsiParameter {
            subparameters: vec![Some(1070)],
        }],
        intermediates: Vec::new(),
        final_byte: b'h',
    }))
    .unwrap();
    core.apply(dcs(b"0;1q#9;2;0;100;0@")).unwrap();
    core.apply(dcs(b"0;1q#9@")).unwrap();
    assert_eq!(
        core.state().graphics().last().unwrap().pixels()[0],
        rgb(0, 0, 128)
    );
}

#[test]
fn sixel_scrolls_into_history_and_history_trim_releases_pixels() {
    let limits = limits(4_096, 32_768, 1);
    let mut core = TerminalCore::new(limits, TerminalSize::new(4, 2, 40, 20).unwrap()).unwrap();
    core.apply(dcs(b"0;1q\"1;1;2;12#2!2~-!2~")).unwrap();
    let id = core.state().graphics().next().unwrap().id();
    assert!(matches!(
        core.state().resolve_graphic(id),
        GraphicAnchorResolution::History { .. }
    ));

    core.reduce(ScreenAction::LineFeed).unwrap();
    core.reduce(ScreenAction::LineFeed).unwrap();
    assert_eq!(core.state().graphics().len(), 0);
    assert_eq!(
        core.state().resolve_graphic(id),
        GraphicAnchorResolution::Unknown
    );
}

#[test]
fn sixel_logical_anchor_survives_primary_reflow() {
    let limits = limits(4_096, 32_768, 16);
    let mut core = TerminalCore::new(limits, TerminalSize::new(6, 2, 60, 20).unwrap()).unwrap();
    for character in "abcde".chars() {
        core.apply(Action::Print(character)).unwrap();
    }
    core.reduce(ScreenAction::SetCursor { row: 0, column: 2 })
        .unwrap();
    core.state.modes.sixel_scrolling = false;
    core.apply(dcs(b"0;1q\"1;1;2;2#2@@")).unwrap();
    let placement = core.state().graphics().next().unwrap();
    let id = placement.id();
    let anchor = placement.anchor();

    core.resize(TerminalSize::new(3, 3, 30, 30).unwrap())
        .unwrap();

    assert_eq!(core.state().graphics().next().unwrap().anchor(), anchor);
    assert_eq!(
        core.state().resolve_graphic(id),
        GraphicAnchorResolution::Screen {
            row: 0,
            column: 2,
            visible_pixel_width: 2,
            visible_pixel_height: 2,
        }
    );
}

#[test]
fn screen_edge_clipping_and_alternate_teardown_are_deterministic() {
    let limits = limits(4_096, 32_768, 16);
    let mut core = TerminalCore::new(limits, TerminalSize::new(4, 2, 40, 20).unwrap()).unwrap();
    core.reduce(ScreenAction::SetCursor { row: 1, column: 3 })
        .unwrap();
    core.state.modes.sixel_scrolling = false;
    core.apply(dcs(b"0;1q\"1;1;20;20#2@")).unwrap();
    let id = core.state().graphics().next().unwrap().id();
    assert_eq!(
        core.state().resolve_graphic(id),
        GraphicAnchorResolution::Screen {
            row: 1,
            column: 3,
            visible_pixel_width: 10,
            visible_pixel_height: 10,
        }
    );

    core.reduce(ScreenAction::SwitchBuffer {
        buffer: ScreenBuffer::Alternate,
        clear: true,
        home: true,
    })
    .unwrap();
    assert_eq!(
        core.state().resolve_graphic(id),
        GraphicAnchorResolution::InactiveBuffer
    );
    core.apply(dcs(b"0;1q#2@")).unwrap();
    assert_eq!(core.state().graphics().len(), 2);
    core.reduce(ScreenAction::SwitchBuffer {
        buffer: ScreenBuffer::Alternate,
        clear: true,
        home: true,
    })
    .unwrap();
    assert_eq!(core.state().graphics().len(), 1);
    core.reduce(ScreenAction::Reset).unwrap();
    assert_eq!(core.state().graphics().len(), 0);
}

#[test]
fn aggregate_object_and_pixel_limits_apply_across_graphics() {
    let mut configured = limits(8, 1_024, 16);
    configured.graphic_objects = GraphicObjectsLimit::new(1).unwrap();
    configured.graphic_frames = GraphicFramesLimit::new(1).unwrap();
    let mut core = TerminalCore::new(configured, TerminalSize::new(4, 2, 40, 20).unwrap()).unwrap();
    core.state.modes.sixel_scrolling = false;
    core.apply(dcs(b"0;1q#9;2;100;0;0@")).unwrap();
    assert_eq!(
        core.apply(dcs(b"0;1q#9;2;0;100;0@")),
        Err(CoreError::Limit(LimitError::Exceeded {
            kind: LimitKind::GraphicObjects,
            requested: 2,
            maximum: 1,
        }))
    );
    assert_eq!(core.state().graphics().len(), 1);
    assert_eq!(core.state.sixel_colors.color(9), rgb(255, 0, 0));
}

#[test]
fn streaming_dcs_boundaries_preserve_sixel_grammar_and_cursor_modes() {
    let limits = limits(4_096, 32_768, 16);
    let mut core = TerminalCore::new(limits, TerminalSize::new(8, 4, 80, 40).unwrap()).unwrap();
    let mut parser = StreamingParser::new(limits);
    let stream = b"\x1bP0;1;0q\"1;1;12;2#2;2;100;0;0!12@\x1b\\";
    for chunk in stream.chunks(2) {
        let mut actions = Vec::new();
        let report = parser.feed(chunk, |action| actions.push(action));
        assert_eq!(report.consumed, chunk.len());
        for action in actions {
            core.apply(action).unwrap();
        }
    }
    assert_eq!(core.state().graphics().len(), 1);
    assert_eq!(core.state().cursor().position.row.get(), 1);

    for (mode, enabled) in [(80, true), (8452, true)] {
        core.apply(Action::Csi(CsiSequence {
            private_markers: vec![b'?'],
            parameters: vec![CsiParameter {
                subparameters: vec![Some(mode)],
            }],
            intermediates: Vec::new(),
            final_byte: if enabled { b'h' } else { b'l' },
        }))
        .unwrap();
    }
    let before = core.state().cursor().position;
    core.apply(dcs(b"0;1q\"1;1;12;2#2!12@")).unwrap();
    assert_eq!(core.state().cursor().position.row, before.row);
    assert_eq!(
        core.state().cursor().position.column.get(),
        before.column.get() + 2
    );
}
