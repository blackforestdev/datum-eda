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

fn parse_in_chunks(bytes: &[u8], chunks: &[usize], limits: CoreLimits) -> Vec<Action> {
    let mut parser = StreamingParser::new(limits);
    let mut actions = Vec::new();
    let mut offset = 0;
    for &length in chunks {
        let end = (offset + length).min(bytes.len());
        while offset < end {
            let report = parser.feed(&bytes[offset..end], |action| actions.push(action));
            assert!(report.consumed > 0);
            offset += report.consumed;
        }
    }
    while offset < bytes.len() {
        let report = parser.feed(&bytes[offset..], |action| actions.push(action));
        assert!(report.consumed > 0);
        offset += report.consumed;
    }
    parser.finish(|action| actions.push(action));
    actions
}

#[test]
fn utf8_and_ecma48_actions_are_invariant_across_every_byte_boundary() {
    let bytes = b"A\xf0\x9f\xa6\x80\x1b[?25;1:2h\x1b]2;title\x07\x1bPqdata\x1b\\Z";
    let expected = parse_in_chunks(bytes, &[bytes.len()], limits(128));
    let one_byte = parse_in_chunks(bytes, &vec![1; bytes.len()], limits(128));
    let irregular = parse_in_chunks(bytes, &[2, 1, 7, 3, 5, 1, 11], limits(128));
    assert_eq!(one_byte, expected);
    assert_eq!(irregular, expected);

    assert!(expected.contains(&Action::Print('🦀')));
    assert!(expected.contains(&Action::Csi(CsiSequence {
        private_markers: vec![b'?'],
        parameters: vec![
            CsiParameter {
                subparameters: vec![Some(25)],
            },
            CsiParameter {
                subparameters: vec![Some(1), Some(2)],
            },
        ],
        intermediates: Vec::new(),
        final_byte: b'h',
    })));
    assert!(expected.contains(&Action::ControlString(ControlString {
        kind: ControlStringKind::Osc,
        bytes: b"2;title".to_vec(),
        terminator: StringTerminator::Bell,
    })));
    assert!(expected.contains(&Action::ControlString(ControlString {
        kind: ControlStringKind::Dcs,
        bytes: b"qdata".to_vec(),
        terminator: StringTerminator::StringTerminator,
    })));
}

#[test]
fn c0_c1_escape_and_all_control_string_families_are_typed() {
    let bytes = [
        0x00, 0x1b, b'7', 0x9b, b'm', 0x90, b'd', 0x9c, 0x98, b's', 0x9c, 0x9e, b'p', 0x9c, 0x9f,
        b'a', 0x9c,
    ];
    let actions = parse_in_chunks(&bytes, &[1; 18], limits(32));
    assert_eq!(actions[0], Action::Execute(ControlCode::new(0).unwrap()));
    assert!(actions.contains(&Action::Escape(EscapeSequence {
        intermediates: Vec::new(),
        final_byte: b'7',
    })));
    for (kind, data) in [
        (ControlStringKind::Dcs, b"d".as_slice()),
        (ControlStringKind::Sos, b"s".as_slice()),
        (ControlStringKind::Pm, b"p".as_slice()),
        (ControlStringKind::Apc, b"a".as_slice()),
    ] {
        assert!(actions.contains(&Action::ControlString(ControlString {
            kind,
            bytes: data.to_vec(),
            terminator: StringTerminator::StringTerminator,
        })));
    }
}

#[test]
fn cancellation_aborts_sequences_and_recovers_at_ground() {
    let actions = parse_in_chunks(b"\x1b[12\x18A\x1b]discard\x1aB", &[1; 19], limits(32));
    assert!(actions.contains(&Action::Cancelled {
        state: ParserStateKind::Csi,
        by: ControlCode::new(0x18).unwrap(),
    }));
    assert!(actions.contains(&Action::Cancelled {
        state: ParserStateKind::Osc,
        by: ControlCode::new(0x1a).unwrap(),
    }));
    assert!(actions.ends_with(&[Action::Print('B')]));
}

#[test]
fn malformed_utf8_replacement_and_reprocessing_are_chunk_invariant() {
    let bytes = b"x\xf0\x80\x80\x80y\xe2z\xff";
    let whole = parse_in_chunks(bytes, &[bytes.len()], limits(64));
    let split = parse_in_chunks(bytes, &[1; 10], limits(64));
    assert_eq!(whole, split);
    assert_eq!(
        whole
            .iter()
            .filter(|action| **action == Action::Error(ParseError::MalformedUtf8))
            .count(),
        3
    );
    assert!(whole.contains(&Action::Print('z')));
}

#[test]
fn oversized_sequences_emit_one_error_discard_and_recover() {
    let mut raw = raw_limits(32);
    raw.parameter_count = 2;
    raw.parameter_digits = 2;
    raw.parameter_value = 99;
    raw.subparameter_count = 2;
    raw.intermediate_bytes = 1;
    raw.control_string_bytes = 3;
    let limits = CoreLimits::try_from(raw).unwrap();

    let csi = parse_in_chunks(b"\x1b[123;4mA", &[2, 1, 4], limits);
    assert_eq!(
        csi,
        vec![
            Action::Error(ParseError::LimitExceeded(LimitKind::ParameterDigits)),
            Action::Print('A'),
        ]
    );
    let string = parse_in_chunks(b"\x1b]abcd-more\x07B", &[1; 14], limits);
    assert_eq!(
        string,
        vec![
            Action::Error(ParseError::LimitExceeded(LimitKind::ControlStringBytes)),
            Action::Print('B'),
        ]
    );

    let mut raw = raw_limits(32);
    raw.parameter_count = 1;
    let final_overflow = parse_in_chunks(b"\x1b[1;2mC", &[8], CoreLimits::try_from(raw).unwrap());
    assert_eq!(
        final_overflow,
        vec![
            Action::Error(ParseError::LimitExceeded(LimitKind::ParameterCount)),
            Action::Print('C'),
        ]
    );
}

#[test]
fn end_of_stream_reports_and_resets_incomplete_input() {
    for (bytes, state) in [
        (b"\x1b[".as_slice(), ParserStateKind::Csi),
        (b"\x1b]title".as_slice(), ParserStateKind::Osc),
    ] {
        let actions = parse_in_chunks(bytes, &[bytes.len()], limits(32));
        assert_eq!(
            actions.last(),
            Some(&Action::Error(ParseError::IncompleteSequence { state }))
        );
    }
}

#[test]
fn malformed_csi_parameter_after_intermediate_discards_until_final() {
    let actions = parse_in_chunks(b"\x1b[1 2;3mD", &[1; 10], limits(32));
    assert_eq!(
        actions,
        vec![
            Action::Error(ParseError::UnexpectedByte {
                state: ParserStateKind::Csi,
                byte: b'2',
            }),
            Action::Print('D'),
        ]
    );
}

#[test]
fn parser_work_cap_returns_a_resumable_consumed_prefix() {
    let mut raw = raw_limits(16);
    raw.parser_work = 2;
    let mut parser = StreamingParser::new(CoreLimits::try_from(raw).unwrap());
    let mut actions = Vec::new();
    let first = parser.feed(b"abcd", |action| actions.push(action));
    assert_eq!(first.consumed, 2);
    assert_eq!(first.actions, 2);
    assert!(first.work_exhausted);
    let second = parser.feed(&b"abcd"[first.consumed..], |action| actions.push(action));
    assert_eq!(second.consumed, 2);
    assert!(!second.work_exhausted);
    assert_eq!(actions, ['a', 'b', 'c', 'd'].map(Action::Print).to_vec());
}

#[test]
fn seeded_malformed_streams_replay_identically_under_arbitrary_chunking() {
    let mut seed = 0x5eed_d7c0_0000_0008_u64;
    for case in 0..256 {
        let length = 1 + usize::try_from(next_random(&mut seed) % 128).unwrap();
        let mut bytes = Vec::with_capacity(length);
        for _ in 0..length {
            bytes.push(next_random(&mut seed).to_le_bytes()[0]);
        }

        let whole = parse_in_chunks(&bytes, &[bytes.len()], limits(1_024));
        let bytewise = parse_in_chunks(&bytes, &vec![1; bytes.len()], limits(1_024));
        let mut chunks = Vec::new();
        let mut remaining = bytes.len();
        while remaining != 0 {
            let length = (1 + usize::try_from(next_random(&mut seed) % 17).unwrap()).min(remaining);
            chunks.push(length);
            remaining -= length;
        }
        let arbitrary = parse_in_chunks(&bytes, &chunks, limits(1_024));
        assert_eq!(bytewise, whole, "bytewise replay diverged for case {case}");
        assert_eq!(arbitrary, whole, "seeded replay diverged for case {case}");
    }
}

fn next_random(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}
