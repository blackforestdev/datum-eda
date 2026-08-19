use super::*;

fn raw_limits(value: usize) -> CoreLimitValues {
    CoreLimitValues {
        parameter_count: value,
        parameter_digits: value,
        parameter_value: value,
        subparameter_count: value,
        intermediate_bytes: value,
        control_string_bytes: value,
        cluster_bytes: value,
        title_bytes: value,
        working_directory_bytes: value,
        clipboard_bytes: value,
        notification_bytes: value,
        reply_bytes: value,
        pending_events: value,
        pending_damage: value,
        history_lines: value,
        history_bytes: value,
        graphic_objects: value,
        graphic_pixels: value,
        graphic_decoded_bytes: value,
        graphic_frames: value,
        compression_ratio: value,
        parser_work: value,
        search_work: value,
        reflow_work: value,
        screen_cells: value,
        snapshot_cells: value,
    }
}

fn limits(value: usize) -> CoreLimits {
    CoreLimits::try_from(raw_limits(value)).unwrap()
}

fn core(limits: CoreLimits) -> TerminalCore {
    TerminalCore::new(limits, TerminalSize::new(20, 6, 200, 120).unwrap()).unwrap()
}

fn apply_bytes(
    core: &mut TerminalCore,
    parser: &mut StreamingParser,
    bytes: &[u8],
    chunks: &[usize],
) -> Result<Vec<CoreUpdate>, CoreError> {
    let mut updates = Vec::new();
    let mut offset = 0;
    let mut chunk_index = 0;
    while offset < bytes.len() {
        let length = chunks
            .get(chunk_index)
            .copied()
            .unwrap_or(bytes.len() - offset)
            .min(bytes.len() - offset);
        let mut actions = Vec::new();
        let report = parser.feed(&bytes[offset..offset + length], |action| {
            actions.push(action)
        });
        assert_eq!(report.consumed, length);
        for action in actions {
            updates.push(core.apply(action)?);
        }
        offset += length;
        chunk_index += 1;
    }
    Ok(updates)
}

fn terminal_text(core: &TerminalCore) -> String {
    core.snapshot()
        .unwrap()
        .rows()
        .flat_map(|row| row.cells())
        .filter_map(|cell| match &cell.content {
            CellContent::Cluster(cluster) => Some(cluster.text()),
            CellContent::Empty | CellContent::Continuation { .. } => None,
        })
        .collect()
}

fn replies(updates: &[CoreUpdate]) -> Vec<(ReplyKind, Vec<u8>)> {
    updates
        .iter()
        .flat_map(CoreUpdate::replies)
        .map(|reply| (reply.kind(), reply.bytes().to_vec()))
        .collect()
}

#[test]
fn parser_actions_drive_controls_tabs_and_dec_special_graphics() {
    let limits = limits(4_096);
    let mut core = core(limits);
    let mut parser = StreamingParser::new(limits);
    let updates =
        apply_bytes(&mut core, &mut parser, b"A\tB\x1b(0lqk\x1b(B\x07", &[1; 16]).unwrap();

    assert_eq!(
        core.state().cell(0, 0).unwrap().content,
        CellContent::Cluster(cluster("A"))
    );
    assert_eq!(
        core.state().cell(0, 8).unwrap().content,
        CellContent::Cluster(cluster("B"))
    );
    assert!(terminal_text(&core).contains("┌─┐"));
    assert_eq!(
        updates
            .iter()
            .flat_map(CoreUpdate::events)
            .filter(|event| **event == CoreEvent::Bell)
            .count(),
        1
    );
}

#[test]
fn sgr_semicolon_and_colon_forms_preserve_complete_cell_style() {
    let limits = limits(4_096);
    let mut core = core(limits);
    let mut parser = StreamingParser::new(limits);
    apply_bytes(
        &mut core,
        &mut parser,
        b"\x1b[1;3;4:3;38;5;196;48:2::1:2:3;58;2;4;5;6mX\x1b[0mY",
        &[3, 2, 1, 5, 7],
    )
    .unwrap();

    let x = core.state().cell(0, 0).unwrap();
    assert!(x.style.attributes.contains(CellAttribute::Bold));
    assert!(x.style.attributes.contains(CellAttribute::Italic));
    assert_eq!(x.style.underline, UnderlineStyle::Curly);
    assert_eq!(x.style.foreground, Color::Indexed(PaletteIndex::new(196)));
    assert_eq!(
        x.style.background,
        Color::Rgb(Rgb {
            red: 1,
            green: 2,
            blue: 3,
        })
    );
    assert_eq!(
        x.style.underline_color,
        Color::Rgb(Rgb {
            red: 4,
            green: 5,
            blue: 6,
        })
    );
    assert_eq!(core.state().cell(0, 1).unwrap().style, CellStyle::default());
}

#[test]
fn csi_origin_margins_protected_erase_and_tab_controls_reach_the_reducer() {
    let limits = limits(4_096);
    let mut core = core(limits);
    let mut parser = StreamingParser::new(limits);
    apply_bytes(
        &mut core,
        &mut parser,
        b"\x1b[2;5r\x1b[3;10s\x1b[?6h\x1b[1;1H\x1b[1\"qP\x1b[0\"qU\x1b[?2J\x1b[3g",
        &[1, 2, 5, 3, 7],
    )
    .unwrap();

    assert_eq!(core.state().cursor().position.row.get(), 1);
    assert_eq!(core.state().cursor().position.column.get(), 4);
    assert!(matches!(
        core.state().cell(1, 2).unwrap().content,
        CellContent::Cluster(_)
    ));
    assert!(core.state().cell(1, 2).unwrap().protected);
    assert!(matches!(
        core.state().cell(1, 3).unwrap().content,
        CellContent::Empty
    ));
    assert_eq!(core.state().tab_stops().iter().len(), 0);
}

#[test]
fn save_restore_includes_designated_character_sets_and_protection() {
    let limits = limits(4_096);
    let mut core = core(limits);
    let mut parser = StreamingParser::new(limits);
    apply_bytes(
        &mut core,
        &mut parser,
        b"\x1b(0\x1b[1\"q\x1b7\x1b(B\x1b[0\"q\x1b8q",
        &[2, 3, 1],
    )
    .unwrap();
    assert_eq!(terminal_text(&core), "─");
    assert!(core.state().cell(0, 0).unwrap().protected);
}

#[test]
fn private_modes_alternate_screen_cursor_style_and_mode_queries_are_exact() {
    let limits = limits(4_096);
    let mut core = core(limits);
    let mut parser = StreamingParser::new(limits);
    let updates = apply_bytes(
        &mut core,
        &mut parser,
        b"\x1b[?1;25;1004;2004h\x1b[5 qmain\x1b[?1049hALT\x1b[?25$p\x1b[?999$p\x1b[?1049l",
        &[2, 1, 4, 3],
    )
    .unwrap();

    assert_eq!(terminal_text(&core), "main");
    assert_eq!(core.state().active_buffer(), ScreenBuffer::Primary);
    assert!(core.state().modes().application_cursor);
    assert!(core.state().modes().focus_reporting);
    assert!(core.state().modes().bracketed_paste);
    assert_eq!(core.state().cursor().shape, CursorShape::Bar);
    assert!(core.state().cursor().blinking);
    let replies = replies(&updates);
    assert!(
        replies.contains(&(ReplyKind::ModeReport, b"\x1b[?25;1$y".to_vec())),
        "{replies:?}"
    );
    assert!(
        replies.contains(&(ReplyKind::ModeReport, b"\x1b[?999;0$y".to_vec())),
        "{replies:?}"
    );
}

#[test]
fn device_cursor_window_and_status_string_reports_are_byte_exact() {
    let limits = limits(4_096);
    let mut core = core(limits);
    let mut parser = StreamingParser::new(limits);
    let updates = apply_bytes(
        &mut core,
        &mut parser,
        b"\x1b[3;4H\x1b[5n\x1b[6n\x1b[?6n\x1b[c\x1b[>c\x1b[18t\x1bP$qm\x1b\\",
        &[1; 48],
    )
    .unwrap();
    let replies = replies(&updates);
    for expected in [
        b"\x1b[0n".as_slice(),
        b"\x1b[3;4R",
        b"\x1b[?3;4R",
        b"\x1b[?1;2c",
        b"\x1b[>0;1;0c",
        b"\x1b[8;6;20t",
        b"\x1bP1$r0m\x1b\\",
    ] {
        assert!(replies.iter().any(|(_, bytes)| bytes == expected));
    }
}

#[test]
fn osc_metadata_palette_and_default_color_state_are_bounded_and_queryable() {
    let limits = limits(4_096);
    let mut core = core(limits);
    let mut parser = StreamingParser::new(limits);
    let updates = apply_bytes(
        &mut core,
        &mut parser,
        b"\x1b]2;Datum EDA\x07\x1b]7;file:///tmp/project\x1b\\\x1b]4;3;#112233\x1b\\\x1b]4;3;?\x1b\\\x1b]10;rgb:aa/bb/cc\x1b\\\x1b]10;?\x1b\\",
        &[5, 2, 11, 1],
    )
    .unwrap();

    assert_eq!(core.state().title().unwrap().as_str(), "Datum EDA");
    assert_eq!(
        core.state().working_directory().unwrap().as_str(),
        "file:///tmp/project"
    );
    assert_eq!(
        core.state().palette_color(3),
        Color::Rgb(Rgb {
            red: 0x11,
            green: 0x22,
            blue: 0x33,
        })
    );
    assert_eq!(
        core.state().default_foreground(),
        Color::Rgb(Rgb {
            red: 0xaa,
            green: 0xbb,
            blue: 0xcc,
        })
    );
    let replies = replies(&updates);
    assert!(
        replies
            .iter()
            .any(|(_, bytes)| bytes == b"\x1b]4;3;rgb:1111/2222/3333\x1b\\")
    );
    assert!(
        replies
            .iter()
            .any(|(_, bytes)| bytes == b"\x1b]10;rgb:aaaa/bbbb/cccc\x1b\\")
    );
}

#[test]
fn synchronized_output_defers_all_damage_until_one_final_flush() {
    let limits = limits(4_096);
    let mut core = core(limits);
    let mut parser = StreamingParser::new(limits);
    let updates = apply_bytes(
        &mut core,
        &mut parser,
        b"\x1b[?2026hABC\x1b]4;1;#010203\x1b\\\x1b[?2026l",
        &[7, 1, 1, 1, 9],
    )
    .unwrap();
    let damaged = updates
        .iter()
        .filter(|update| update.damage().iter().len() != 0)
        .collect::<Vec<_>>();
    assert_eq!(damaged.len(), 1);
    assert_eq!(
        damaged[0].damage().iter().collect::<Vec<_>>(),
        vec![Damage::Full]
    );
    assert_eq!(terminal_text(&core), "ABC");
    assert!(!core.state().modes().synchronized_output);
}

#[test]
fn complete_semantics_are_invariant_across_arbitrary_parser_chunks() {
    let limits = limits(4_096);
    let bytes = b"\x1b]2;chunked\x07\x1b[31;48:2::1:2:3mA\x1b(0lqk\x1b(B\x1b[2;5H!";
    let mut whole = core(limits);
    let mut whole_parser = StreamingParser::new(limits);
    let whole_updates = apply_bytes(&mut whole, &mut whole_parser, bytes, &[bytes.len()]).unwrap();

    let mut split = core(limits);
    let mut split_parser = StreamingParser::new(limits);
    let split_updates = apply_bytes(&mut split, &mut split_parser, bytes, &[1; 64]).unwrap();
    assert_eq!(split.snapshot().unwrap(), whole.snapshot().unwrap());
    assert_eq!(split.state().title(), whole.state().title());
    assert_eq!(replies(&split_updates), replies(&whole_updates));
    assert_eq!(
        split_updates
            .iter()
            .flat_map(CoreUpdate::events)
            .collect::<Vec<_>>(),
        whole_updates
            .iter()
            .flat_map(CoreUpdate::events)
            .collect::<Vec<_>>()
    );
}

#[test]
fn metadata_limits_and_unsupported_queries_fail_closed() {
    let mut raw = raw_limits(512);
    raw.title_bytes = 4;
    let limits = CoreLimits::try_from(raw).unwrap();
    let mut core = core(limits);
    let mut parser = StreamingParser::new(limits);
    assert!(matches!(
        apply_bytes(&mut core, &mut parser, b"\x1b]2;too-long\x07", &[32]),
        Err(CoreError::Limit(LimitError::Exceeded {
            kind: LimitKind::TitleBytes,
            ..
        }))
    ));

    let update = core
        .apply(Action::Csi(CsiSequence {
            private_markers: vec![b'?'],
            parameters: vec![CsiParameter {
                subparameters: vec![Some(1000)],
            }],
            intermediates: Vec::new(),
            final_byte: b'h',
        }))
        .unwrap();
    assert!(!update.recognized());
    assert!(update.damage().iter().len() == 0 && update.events().is_empty());
}

#[test]
fn semantic_events_and_replies_share_one_checked_pending_limit() {
    let mut raw = raw_limits(512);
    raw.pending_events = 1;
    let limits = CoreLimits::try_from(raw).unwrap();
    let mut core = core(limits);
    let mut parser = StreamingParser::new(limits);
    assert!(matches!(
        apply_bytes(&mut core, &mut parser, b"\x1b]4;1;#010203;1;?\x1b\\", &[64]),
        Err(CoreError::Limit(LimitError::Exceeded {
            kind: LimitKind::PendingEvents,
            ..
        }))
    ));
}

fn cluster(text: &str) -> Cluster {
    Cluster::new(text, CellWidth::One, ClusterBytesLimit::new(64).unwrap()).unwrap()
}
