use crate::{
    Action, BIDIRECTIONAL_TEXT_POLICY, BidirectionalTextPolicy, CellContent, CellWidth,
    CoreLimitValues, CoreLimits, ShapingCluster, StreamingParser, TerminalCore, TerminalSize,
    UNICODE_VERSION, grapheme_indices, terminal_cluster_width,
};

#[test]
fn unicode_17_grapheme_break_corpus_matches_every_normative_boundary() {
    let data = include_str!("../unicode/17.0.0/GraphemeBreakTest.txt");
    let mut cases = 0usize;
    for line in data.lines() {
        let body = line.split('#').next().unwrap_or("").trim();
        if body.is_empty() {
            continue;
        }
        let (text, expected) = parse_grapheme_case(body);
        let mut actual = grapheme_indices(&text)
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        actual.push(text.len());
        assert_eq!(actual, expected, "grapheme case: {body}");
        cases += 1;
    }
    assert!(cases > 700, "the full Unicode 17 corpus must be exercised");
}

#[test]
fn unicode_width_policy_covers_ascii_ambiguous_cjk_and_emoji() {
    assert_eq!(UNICODE_VERSION, "17.0.0");
    assert_eq!(terminal_cluster_width("A"), CellWidth::One);
    assert_eq!(terminal_cluster_width("Ω"), CellWidth::One);
    assert_eq!(terminal_cluster_width("界"), CellWidth::Two);
    assert_eq!(terminal_cluster_width("⌚"), CellWidth::Two);
    assert_eq!(terminal_cluster_width("❤\u{fe0e}"), CellWidth::One);
    assert_eq!(terminal_cluster_width("❤\u{fe0f}"), CellWidth::Two);
    assert_eq!(terminal_cluster_width("🇺🇸"), CellWidth::Two);
    assert_eq!(terminal_cluster_width("👩‍💻"), CellWidth::Two);
}

#[test]
fn every_rgi_emoji_sequence_is_one_two_cell_cluster() {
    for data in [
        include_str!("../unicode/17.0.0/emoji-sequences.txt"),
        include_str!("../unicode/17.0.0/emoji-zwj-sequences.txt"),
    ] {
        for line in data.lines() {
            let body = line.split('#').next().unwrap_or("").trim();
            if body.is_empty() {
                continue;
            }
            let fields = body.split(';').map(str::trim).collect::<Vec<_>>();
            if fields.len() < 2 || fields[0].contains("..") {
                continue;
            }
            let text = fields[0]
                .split_whitespace()
                .map(|code| char::from_u32(u32::from_str_radix(code, 16).unwrap()).unwrap())
                .collect::<String>();
            assert_eq!(
                grapheme_indices(&text).count(),
                1,
                "emoji sequence must be one grapheme: {}",
                fields[0]
            );
            assert_eq!(
                terminal_cluster_width(&text),
                CellWidth::Two,
                "emoji sequence must own two cells: {}",
                fields[0]
            );
        }
    }
}

#[test]
fn bidirectional_text_policy_preserves_logical_cell_order() {
    assert_eq!(
        BIDIRECTIONAL_TEXT_POLICY,
        BidirectionalTextPolicy::LogicalOrder
    );
    let text = "abc אבג 123";
    assert_eq!(
        grapheme_indices(text)
            .map(|(_, cluster)| cluster)
            .collect::<String>(),
        text
    );
}

#[test]
fn shaping_boundary_exposes_original_cluster_text_and_fixed_cell_ownership() {
    let mut core = core(8, 2, 256);
    for character in "👩‍💻".chars() {
        core.apply(Action::Print(character)).unwrap();
    }
    let CellContent::Cluster(cluster) = &core.state().cell(0, 0).unwrap().content else {
        panic!("emoji cluster is missing");
    };
    let shaping = ShapingCluster::from_cluster(cluster);
    assert_eq!(shaping.text(), "👩‍💻");
    assert_eq!(shaping.cell_width(), CellWidth::Two);
}

#[test]
fn terminal_core_combines_marks_and_emoji_without_orphan_cells() {
    let mut core = core(12, 3, 256);
    for character in "e\u{301}👩‍💻אב".chars() {
        core.apply(Action::Print(character)).unwrap();
    }
    assert_cluster(&core, 0, 0, "e\u{301}", CellWidth::One);
    assert_cluster(&core, 0, 1, "👩‍💻", CellWidth::Two);
    assert!(matches!(
        core.state().cell(0, 2).unwrap().content,
        CellContent::Continuation { .. }
    ));
    assert_cluster(&core, 0, 3, "א", CellWidth::One);
    assert_cluster(&core, 0, 4, "ב", CellWidth::One);
    assert_eq!(core.state().cursor().position.column.get(), 5);
    core.snapshot().unwrap();
}

#[test]
fn variation_selector_width_expansion_wraps_atomically_at_the_right_edge() {
    let mut core = core(4, 2, 256);
    for character in "abc❤\u{fe0f}".chars() {
        core.apply(Action::Print(character)).unwrap();
    }
    assert!(matches!(
        core.state().cell(0, 3).unwrap().content,
        CellContent::Empty
    ));
    assert_cluster(&core, 1, 0, "❤\u{fe0f}", CellWidth::Two);
    assert!(matches!(
        core.state().cell(1, 1).unwrap().content,
        CellContent::Continuation { .. }
    ));
    assert!(
        core.snapshot()
            .unwrap()
            .rows()
            .next()
            .unwrap()
            .soft_wrapped()
    );
}

#[test]
fn unicode_screen_state_is_invariant_across_every_utf8_chunk_boundary() {
    let bytes = "A界e\u{301}👩‍💻Ωאב".as_bytes();
    let expected = apply_chunks(bytes, &[bytes.len()]);
    for split in 1..bytes.len() {
        assert_eq!(apply_chunks(bytes, &[split, bytes.len() - split]), expected);
    }
    assert_eq!(apply_chunks(bytes, &vec![1; bytes.len()]), expected);
}

#[test]
fn grapheme_extension_respects_cluster_byte_limit_without_partial_mutation() {
    let mut core = core_with_cluster_limit(4, 2, 256, 1);
    core.apply(Action::Print('e')).unwrap();

    assert_eq!(
        core.apply(Action::Print('\u{301}')),
        Err(crate::CoreError::InvalidPrintable)
    );
    assert_cluster(&core, 0, 0, "e", CellWidth::One);
    assert_eq!(core.state().cursor().position.column.get(), 1);
}

fn core(columns: u16, rows: u16, limit: usize) -> TerminalCore {
    core_with_cluster_limit(columns, rows, limit, limit)
}

fn core_with_cluster_limit(
    columns: u16,
    rows: u16,
    limit: usize,
    cluster_bytes: usize,
) -> TerminalCore {
    let values = CoreLimitValues {
        parameter_count: limit,
        parameter_digits: limit,
        parameter_value: limit,
        subparameter_count: limit,
        intermediate_bytes: limit,
        control_string_bytes: limit,
        cluster_bytes,
        title_bytes: limit,
        working_directory_bytes: limit,
        clipboard_bytes: limit,
        hyperlink_bytes: limit,
        input_bytes: limit,
        keyboard_stack: limit,
        notification_bytes: limit,
        reply_bytes: limit,
        pending_events: limit,
        pending_damage: limit,
        history_lines: limit,
        history_bytes: limit,
        graphic_objects: limit,
        graphic_pixels: limit,
        graphic_decoded_bytes: limit,
        graphic_frames: limit,
        compression_ratio: limit,
        parser_work: limit,
        search_work: limit,
        reflow_work: limit,
        screen_cells: limit,
        snapshot_cells: limit,
    };
    let limits = CoreLimits::try_from(values).unwrap();
    TerminalCore::new(limits, TerminalSize::new(columns, rows, 0, 0).unwrap()).unwrap()
}

fn apply_chunks(bytes: &[u8], chunks: &[usize]) -> crate::TerminalSnapshot {
    let mut core = core(32, 4, 4_096);
    let mut parser = StreamingParser::new(*core.limits());
    let mut offset = 0usize;
    for chunk in chunks {
        let end = offset.saturating_add(*chunk).min(bytes.len());
        let mut actions = Vec::new();
        let report = parser.feed(&bytes[offset..end], |action| actions.push(action));
        assert_eq!(report.consumed, end - offset);
        for action in actions {
            core.apply(action).unwrap();
        }
        offset = end;
    }
    assert_eq!(offset, bytes.len());
    core.snapshot().unwrap()
}

fn assert_cluster(core: &TerminalCore, row: u16, column: u16, text: &str, width: CellWidth) {
    let CellContent::Cluster(cluster) = &core.state().cell(row, column).unwrap().content else {
        panic!("expected cluster at {row},{column}");
    };
    assert_eq!(cluster.text(), text);
    assert_eq!(cluster.width(), width);
}

fn parse_grapheme_case(body: &str) -> (String, Vec<usize>) {
    let mut text = String::new();
    let mut boundaries = Vec::new();
    let mut boundary = true;
    for token in body.split_whitespace() {
        match token {
            "÷" => boundary = true,
            "×" => boundary = false,
            code => {
                if boundary {
                    boundaries.push(text.len());
                }
                let character = char::from_u32(u32::from_str_radix(code, 16).unwrap()).unwrap();
                text.push(character);
                boundary = false;
            }
        }
    }
    if body.ends_with('÷') {
        boundaries.push(text.len());
    }
    (text, boundaries)
}
