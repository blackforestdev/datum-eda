use super::*;

const PROOF_SEED: u64 = 0xd7c0_0021_5eed_cafe;

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReplayOutcome {
    snapshot: RenderSnapshot,
    title: Option<String>,
    working_directory: Option<String>,
    replies: Vec<TerminalReply>,
    events: Vec<CoreEvent>,
    damage: Vec<Damage>,
    errors: Vec<String>,
}

#[derive(Clone, Copy)]
struct NormativeCase {
    id: &'static str,
    authority: &'static str,
    bytes: &'static [u8],
}

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
        hyperlink_bytes: value,
        input_bytes: value,
        keyboard_stack: value,
        notification_bytes: value,
        reply_bytes: value,
        pending_events: value,
        pending_damage: value,
        history_lines: value,
        history_bytes: value.saturating_mul(64),
        graphic_objects: value,
        graphic_pixels: value.saturating_mul(64),
        graphic_decoded_bytes: value.saturating_mul(256),
        graphic_frames: value,
        compression_ratio: value,
        parser_work: value.saturating_mul(256),
        search_work: value.saturating_mul(256),
        reflow_work: value.saturating_mul(256),
        screen_cells: value.saturating_mul(256),
        snapshot_cells: value.saturating_mul(256),
    }
}

fn limits(value: usize) -> CoreLimits {
    CoreLimits::try_from(raw_limits(value)).unwrap()
}

fn replay(bytes: &[u8], chunk_pattern: &[usize], limits: CoreLimits) -> ReplayOutcome {
    let mut parser = StreamingParser::new(limits);
    let mut core = TerminalCore::new(limits, TerminalSize::new(16, 4, 160, 80).unwrap()).unwrap();
    let mut replies = Vec::new();
    let mut events = Vec::new();
    let mut damage = Vec::new();
    let mut errors = Vec::new();
    let mut offset = 0;
    let mut pattern = 0;
    while offset < bytes.len() {
        let requested = chunk_pattern
            .get(pattern % chunk_pattern.len())
            .copied()
            .unwrap_or(bytes.len())
            .max(1);
        let end = offset.saturating_add(requested).min(bytes.len());
        apply_parser_bytes(
            &mut parser,
            &mut core,
            &bytes[offset..end],
            &mut replies,
            &mut events,
            &mut damage,
            &mut errors,
        );
        offset = end;
        pattern += 1;
    }
    let mut final_actions = Vec::new();
    parser.finish(|action| final_actions.push(action));
    apply_actions(
        &mut core,
        final_actions,
        &mut replies,
        &mut events,
        &mut damage,
        &mut errors,
    );
    ReplayOutcome {
        snapshot: core.render_snapshot().unwrap(),
        title: core.state().title().map(|title| title.as_str().to_owned()),
        working_directory: core
            .state()
            .working_directory()
            .map(|cwd| cwd.as_str().to_owned()),
        replies,
        events,
        damage,
        errors,
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_parser_bytes(
    parser: &mut StreamingParser,
    core: &mut TerminalCore,
    mut bytes: &[u8],
    replies: &mut Vec<TerminalReply>,
    events: &mut Vec<CoreEvent>,
    damage: &mut Vec<Damage>,
    errors: &mut Vec<String>,
) {
    while !bytes.is_empty() {
        let mut actions = Vec::new();
        let report = parser.feed(bytes, |action| actions.push(action));
        assert!(
            report.consumed > 0,
            "proof parser must always make progress"
        );
        apply_actions(core, actions, replies, events, damage, errors);
        bytes = &bytes[report.consumed..];
    }
}

fn apply_actions(
    core: &mut TerminalCore,
    actions: Vec<Action>,
    replies: &mut Vec<TerminalReply>,
    events: &mut Vec<CoreEvent>,
    damage: &mut Vec<Damage>,
    errors: &mut Vec<String>,
) {
    for action in actions {
        match core.apply(action) {
            Ok(update) => {
                replies.extend_from_slice(update.replies());
                events.extend_from_slice(update.events());
                damage.extend(update.damage().iter());
            }
            Err(error) => errors.push(error.to_string()),
        }
    }
}

fn screen_text(snapshot: &RenderSnapshot) -> String {
    snapshot
        .rows()
        .filter(|row| matches!(row.source(), RenderRowSource::Screen { .. }))
        .map(|row| {
            row.cells()
                .iter()
                .map(|cell| match &cell.content {
                    CellContent::Cluster(cluster) => cluster.text(),
                    CellContent::Empty => " ",
                    CellContent::Continuation { .. } => "",
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn irregular_chunks(bytes: &[u8], seed: &mut u64) -> Vec<usize> {
    let mut remaining = bytes.len();
    let mut chunks = Vec::new();
    while remaining != 0 {
        let length = (next_random(seed) as usize % 23 + 1).min(remaining);
        chunks.push(length);
        remaining -= length;
    }
    chunks
}

fn next_random(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn normative_cases() -> [NormativeCase; 5] {
    [
        NormativeCase {
            id: "DTC-R01-R03-ECMA48-001",
            authority: "ECMA-48 cursor movement, printable cells, and DSR",
            bytes: b"abc\x1b[2DZ\x1b[6n",
        },
        NormativeCase {
            id: "DTC-R02-DECSTATE-001",
            authority: "VT220 alternate screen, save/restore, and SGR",
            bytes: b"main\x1b7\x1b[31;48;2;1;2;3m!\x1b[?1049halt\x1b[?1049l\x1b8?",
        },
        NormativeCase {
            id: "DTC-R04-R07-UNICODE-HISTORY-001",
            authority: "Unicode 17 UAX 29/11 clusters and logical primary history",
            bytes: "A界e\u{301}\r\none\r\ntwo\r\nthree\r\nfour".as_bytes(),
        },
        NormativeCase {
            id: "DTC-R06-METADATA-001",
            authority: "xterm OSC title, cwd, palette, OSC 8, and OSC 133",
            bytes: b"\x1b]2;agent\x07\x1b]7;file:///tmp/project\x1b\\\x1b]4;2;#112233\x1b\\\x1b]8;;https://example.test\x1b\\link\x1b]8;;\x1b\\\x1b]133;A\x1b\\",
        },
        NormativeCase {
            id: "DTC-R08-GRAPHICS-001",
            authority: "DEC sixel and Kitty graphics protocol owner specifications",
            bytes: b"\x1bPq#1;2;100;0;0@\x1b\\\x1b_Ga=T,f=32,s=1,v=1,i=7,p=1;/wAA/w==\x1b\\",
        },
    ]
}

#[test]
fn normative_corpus_matches_expected_state_and_every_chunk_partition() {
    let limits = limits(1_024);
    let mut seed = PROOF_SEED;
    for case in normative_cases() {
        assert!(case.id.starts_with("DTC-R"));
        assert!(!case.authority.is_empty());
        let whole = replay(case.bytes, &[case.bytes.len().max(1)], limits);
        let bytewise = replay(case.bytes, &[1], limits);
        let arbitrary = replay(case.bytes, &irregular_chunks(case.bytes, &mut seed), limits);
        assert_eq!(bytewise, whole, "{} bytewise replay", case.id);
        assert_eq!(arbitrary, whole, "{} arbitrary replay", case.id);
        assert!(whole.errors.is_empty(), "{}: {:?}", case.id, whole.errors);

        match case.id {
            "DTC-R01-R03-ECMA48-001" => {
                assert!(screen_text(&whole.snapshot).starts_with("aZc"));
                assert_eq!(whole.replies[0].bytes(), b"\x1b[1;3R");
            }
            "DTC-R02-DECSTATE-001" => {
                assert_eq!(whole.snapshot.active_buffer(), ScreenBuffer::Primary);
                assert!(screen_text(&whole.snapshot).starts_with("main?"));
            }
            "DTC-R04-R07-UNICODE-HISTORY-001" => {
                assert!(whole.snapshot.history_row_count() > 0);
                assert!(whole.snapshot.rows().any(|row| {
                    row.cells().iter().any(|cell| {
                    matches!(&cell.content, CellContent::Cluster(cluster) if cluster.text() == "界")
                })
                }));
            }
            "DTC-R06-METADATA-001" => {
                assert_eq!(whole.title.as_deref(), Some("agent"));
                assert_eq!(
                    whole.working_directory.as_deref(),
                    Some("file:///tmp/project")
                );
                assert_eq!(
                    whole.snapshot.palette().color(2),
                    Color::Rgb(Rgb {
                        red: 0x11,
                        green: 0x22,
                        blue: 0x33,
                    })
                );
                assert!(whole.events.iter().any(|event| matches!(
                    event,
                    CoreEvent::TitleChanged(title) if title.as_str() == "agent"
                )));
            }
            "DTC-R08-GRAPHICS-001" => assert_eq!(whole.snapshot.graphics().len(), 2),
            _ => unreachable!(),
        }
    }
}

fn mutate(base: &[u8], seed: &mut u64, generation: usize) -> Vec<u8> {
    let mut bytes = base.to_vec();
    for _ in 0..=generation % 7 {
        let operation = next_random(seed) % 3;
        let index = next_random(seed) as usize % bytes.len().max(1);
        match operation {
            0 if !bytes.is_empty() => bytes[index] ^= next_random(seed) as u8,
            1 if bytes.len() < 512 => bytes.insert(index.min(bytes.len()), next_random(seed) as u8),
            2 if !bytes.is_empty() => {
                bytes.remove(index);
            }
            _ => {}
        }
    }
    bytes
}

fn minimize_replay(mut bytes: Vec<u8>, fails: impl Fn(&[u8]) -> bool) -> Vec<u8> {
    let mut width = bytes.len() / 2;
    while width != 0 {
        let mut offset = 0;
        let mut reduced = false;
        while offset < bytes.len() {
            let end = offset.saturating_add(width).min(bytes.len());
            let mut candidate = bytes.clone();
            candidate.drain(offset..end);
            if fails(&candidate) {
                bytes = candidate;
                reduced = true;
                break;
            }
            offset = end;
        }
        if !reduced {
            width /= 2;
        }
    }
    bytes
}

#[test]
fn seeded_generational_mutation_replays_and_shrinks_deterministically() {
    let base = b"text\x1b[31mred\x1b[0m\r\n\xf0\x9f\x91\xa9\xe2\x80\x8d\xf0\x9f\x92\xbb\x1b]2;title\x07\x1b[6n";
    let limits = limits(1_024);
    let mut first_seed = PROOF_SEED;
    let mut second_seed = PROOF_SEED;
    for generation in 0..256 {
        let first = mutate(base, &mut first_seed, generation);
        let second = mutate(base, &mut second_seed, generation);
        assert_eq!(first, second, "generation {generation} is reproducible");
        let whole = replay(&first, &[first.len().max(1)], limits);
        let mut chunk_seed = PROOF_SEED ^ generation as u64;
        let chunks = irregular_chunks(&first, &mut chunk_seed);
        let partitioned = replay(&first, &chunks, limits);
        if whole != partitioned {
            let minimized = minimize_replay(first, |candidate| {
                replay(candidate, &[candidate.len().max(1)], limits)
                    != replay(candidate, &[1], limits)
            });
            panic!("generation {generation} replay diverged: {minimized:02x?}");
        }
    }
    assert_eq!(
        minimize_replay(vec![1, 2, 0xff, 3], |bytes| bytes.contains(&0xff)),
        vec![0xff]
    );
}

#[test]
fn hostile_streams_exhaust_limits_then_reset_to_bounded_initial_state() {
    let mut raw = raw_limits(32);
    raw.control_string_bytes = 16;
    raw.parameter_count = 4;
    raw.parameter_digits = 3;
    raw.graphic_pixels = 8;
    raw.graphic_decoded_bytes = 32;
    raw.history_lines = 2;
    raw.history_bytes = 128;
    let limits = CoreLimits::try_from(raw).unwrap();
    let mut bytes = b"\x1b]2;".to_vec();
    bytes.extend(std::iter::repeat_n(b'x', 128));
    bytes.extend_from_slice(b"\x07\x1b[1;2;3;4;5;9999m\xff\xfe");
    bytes.extend_from_slice(b"\x1bPq!999999~\x1b\\");
    let outcome = replay(&bytes, &[1, 7, 3, 19], limits);
    assert!(
        outcome
            .events
            .iter()
            .any(|event| matches!(event, CoreEvent::LimitReached(_)))
    );
    assert_eq!(outcome.snapshot.rows().len(), 4);
    assert!(outcome.snapshot.graphics().len() <= 32);

    let reset = replay(b"dirty\r\nrows\x1b]2;title\x07\x1bc", &[1, 4, 2], limits);
    assert_eq!(reset.snapshot.active_buffer(), ScreenBuffer::Primary);
    assert_eq!(reset.snapshot.history_row_count(), 0);
    assert_eq!(reset.snapshot.graphics().len(), 0);
    assert_eq!(reset.snapshot.cursor().position.row.get(), 0);
    assert_eq!(reset.snapshot.cursor().position.column.get(), 0);
    assert!(screen_text(&reset.snapshot).trim().is_empty());
    assert_eq!(reset.title, None);
    assert_eq!(reset.working_directory, None);
}

#[test]
fn repeated_generations_remain_within_history_graphics_and_snapshot_resources() {
    let mut raw = raw_limits(64);
    raw.history_lines = 3;
    raw.history_bytes = 192;
    raw.graphic_objects = 2;
    raw.graphic_pixels = 2;
    raw.graphic_decoded_bytes = 8;
    raw.graphic_frames = 2;
    raw.snapshot_cells = 112;
    let limits = CoreLimits::try_from(raw).unwrap();
    let mut stream = Vec::new();
    for generation in 0..128 {
        stream.extend_from_slice(format!("generation-{generation:03}\r\n").as_bytes());
    }
    let outcome = replay(&stream, &[3, 5, 8, 13], limits);
    assert!(outcome.snapshot.history_row_count() <= 3);
    assert!(outcome.snapshot.rows().len() <= 7);
    assert!(outcome.snapshot.graphics().len() <= 2);
    assert!(outcome.snapshot.history_trimmed_rows() > 0);
}
