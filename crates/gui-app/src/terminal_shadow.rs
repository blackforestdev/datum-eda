//! DTC-P25 test-only comparison between the retired provisional parser and
//! TerminalCore. This module is never compiled into a production Datum binary.
//!
//! The overlap is deliberately closed: ASCII cell text, projected SGR styles,
//! cursor position/visibility, title, working directory, bell count, geometry,
//! application cursor/keypad modes, focus reporting, mouse mode/encoding,
//! bracketed paste, and exact standard replies. Unicode grapheme/width rules,
//! hyperlinks, modern keyboard protocols, history/reflow, graphics, damage,
//! accessibility, and security events have no trustworthy provisional oracle;
//! their existing TerminalCore normative proofs remain authoritative.

use super::{TerminalCoreAdapterUpdate, TerminalCoreSessionAdapter};
use crate::terminal_screen::TerminalScreen;
use datum_gui_protocol::{TerminalLaneState, TerminalStyledLine};

const MAX_SHADOW_RECORDING_BYTES: usize = 64 * 1024;
const MAX_SHADOW_REPLAY_CHUNKS: usize = 4_096;
const MAX_SHADOW_RECORDINGS: usize = 16;

#[derive(Clone, Copy)]
struct ShadowRecording {
    name: &'static str,
    source: &'static str,
    columns: u16,
    rows: u16,
    chunks: &'static [&'static [u8]],
}

// Datum-authored, deterministic recordings of the byte sequences emitted by
// ordinary shell and TUI operations. Chunk boundaries model separate PTY reads;
// every test also replays the same bytes whole, bytewise, and with a seeded
// irregular partition.
const SHELL_RECORDING: ShadowRecording = ShadowRecording {
    name: "bash-prompt-and-command",
    source: "bash --noprofile --norc: colored prompt, printf, pwd, bell",
    columns: 48,
    rows: 8,
    chunks: &[
        b"\x1b]2;Datum shadow shell\x07",
        b"\x1b]7;file:///tmp/datum-shadow\x07",
        b"\x1b[32;1muser@host\x1b[0m:\x1b[34m/tmp\x1b[0m$ ",
        b"printf alpha\\n\r\nalpha\r\n",
        b"\x07user@host:/tmp$ ",
    ],
};

const EDIT_RECORDING: ShadowRecording = ShadowRecording {
    name: "line-edit-and-erase",
    source: "readline-style cursor addressing and erase operations",
    columns: 32,
    rows: 6,
    chunks: &[
        b"abcdef",
        b"\x1b[3DXYZ",
        b"\r\nsecond line",
        b"\x1b[5D\x1b[Ktail",
        b"\r\nthird\x1b[1A\x1b[2K\rreplacement",
    ],
};

const TUI_RECORDING: ShadowRecording = ShadowRecording {
    name: "alternate-screen-modes",
    source: "minimal full-screen TUI enter, draw, mode negotiation, and exit",
    columns: 40,
    rows: 10,
    chunks: &[
        b"before",
        b"\x1b[?1049h\x1b[2J\x1b[H",
        b"\x1b[1;36mTITLE\x1b[0m\x1b[2;1Hrow two",
        b"\x1b[?25l\x1b[?1h\x1b=",
        b"\x1b[?1002h\x1b[?1006h\x1b[?1004h\x1b[?2004h",
        b"\x1b[?25h\x1b[?1002l\x1b[?1006l\x1b[?1004l\x1b[?2004l",
        b"\x1b[?1049l\x1b[?1l\x1b>",
    ],
};

const REPLY_RECORDING: ShadowRecording = ShadowRecording {
    name: "standard-status-replies",
    source: "application DSR and CPR queries over a positioned cursor",
    columns: 30,
    rows: 6,
    chunks: &[b"ready", b"\x1b[3;7H", b"\x1b[5n", b"\x1b[6n"],
};

const STYLE_RECORDING: ShadowRecording = ShadowRecording {
    name: "sgr-style-ranges",
    source: "colored shell listing with ANSI, indexed, RGB, and decoration SGR",
    columns: 64,
    rows: 8,
    chunks: &[
        b"\x1b[1;31;44mERR\x1b[0m ",
        b"\x1b[38;5;196mindexed\x1b[39m ",
        b"\x1b[38;2;12;34;56mtruecolor\x1b[0m\r\n",
        b"\x1b[3;4;9;53mdecorated\x1b[23;24;29;55m plain",
    ],
};

const ERASE_RECORDING: ShadowRecording = ShadowRecording {
    name: "bounded-screen-editing",
    source: "fixed-grid erase, insert, delete, and cursor-addressing operations",
    columns: 12,
    rows: 5,
    chunks: &[
        b"abcdefghijkl\r\nsecond line\r\nthird line",
        b"\x1b[2;4H\x1b[3X",
        b"XYZ\x1b[2D\x1b[@Q\x1b[P",
        b"\x1b[3;3H\x1b[Ktail",
    ],
};

const TAB_RECORDING: ShadowRecording = ShadowRecording {
    name: "terminal-tab-stops",
    source: "shell horizontal tabs with custom and cleared stops",
    columns: 40,
    rows: 5,
    chunks: &[
        b"x\ty\r\n",
        b"\x1b[6G\x1bH\rx\ty\r\n",
        b"\x1b[10G\x1bH\rx\ty",
    ],
};

const C1_RECORDING: ShadowRecording = ShadowRecording {
    name: "eight-bit-c1-controls",
    source: "8-bit CSI, OSC, index, next-line, and reverse-index aliases",
    columns: 30,
    rows: 6,
    chunks: &[
        b"abcdef\x9b3DZ\r\n",
        b"a\x9d0;C1 title\x9cb\x84Z",
        b"\x85next\x9b2;2H\x8dR",
    ],
};

const RECORDINGS: &[ShadowRecording] = &[
    SHELL_RECORDING,
    EDIT_RECORDING,
    TUI_RECORDING,
    REPLY_RECORDING,
    STYLE_RECORDING,
    ERASE_RECORDING,
    TAB_RECORDING,
    C1_RECORDING,
];

#[derive(Debug, PartialEq, Eq)]
struct DeclaredOverlap {
    lines: Vec<String>,
    styled_lines: Vec<TerminalStyledLine>,
    title: Option<String>,
    current_working_directory: Option<String>,
    bell_count: usize,
    columns: u16,
    rows: u16,
    cursor_row: usize,
    cursor_column: usize,
    cursor_visible: bool,
    application_cursor_keys: bool,
    application_keypad: bool,
    focus_reporting: bool,
    mouse_reporting_mode: Option<String>,
    mouse_coordinate_encoding: Option<String>,
    scroll_offset: usize,
    bracketed_paste: bool,
    replies: Vec<Vec<u8>>,
}

impl DeclaredOverlap {
    fn capture(lane: &TerminalLaneState, bracketed_paste: bool, replies: Vec<Vec<u8>>) -> Self {
        let mut lines = lane
            .grid_lines()
            .iter()
            .map(|line| line.trim_end_matches(' ').to_owned())
            .collect::<Vec<_>>();
        let mut styled_lines = lane
            .grid_styled_lines()
            .iter()
            .cloned()
            .map(normalize_styled_line)
            .collect::<Vec<_>>();
        let retained_rows = usize::min(
            lines
                .iter()
                .rposition(|line| !line.is_empty())
                .map_or(0, |index| index + 1)
                .max(lane.screen_cursor_row + 1),
            lines.len(),
        );
        lines.truncate(retained_rows);
        styled_lines.truncate(retained_rows);
        Self {
            lines,
            styled_lines,
            title: lane.title.clone(),
            current_working_directory: lane
                .current_working_directory
                .as_deref()
                .map(normalize_working_directory),
            bell_count: lane.bell_count,
            columns: lane.columns,
            rows: lane.rows,
            cursor_row: lane.screen_cursor_row,
            cursor_column: lane.screen_cursor_col,
            cursor_visible: lane.screen_cursor_visible,
            application_cursor_keys: lane.application_cursor_keys,
            application_keypad: lane.application_keypad,
            focus_reporting: lane.focus_event_reporting,
            mouse_reporting_mode: lane.mouse_reporting_mode.clone(),
            mouse_coordinate_encoding: lane.mouse_coordinate_encoding.clone(),
            scroll_offset: lane.scroll_offset,
            bracketed_paste,
            replies,
        }
    }
}

fn normalize_styled_line(mut line: TerminalStyledLine) -> TerminalStyledLine {
    line.text = line.text.trim_end_matches(' ').to_owned();
    let text_bytes = line.text.len();
    for span in &mut line.spans {
        span.fg = span.fg.as_deref().map(normalize_color);
        span.bg = span.bg.as_deref().map(normalize_color);
        span.end = span.end.min(text_bytes);
    }
    line.spans.retain(|span| span.start < span.end);
    line
}

fn normalize_color(color: &str) -> String {
    let index = match color {
        "black" => Some(0),
        "red" => Some(1),
        "green" => Some(2),
        "yellow" => Some(3),
        "blue" => Some(4),
        "magenta" => Some(5),
        "cyan" => Some(6),
        "white" => Some(7),
        "bright-black" => Some(8),
        "bright-red" => Some(9),
        "bright-green" => Some(10),
        "bright-yellow" => Some(11),
        "bright-blue" => Some(12),
        "bright-magenta" => Some(13),
        "bright-cyan" => Some(14),
        "bright-white" => Some(15),
        _ => None,
    };
    index.map_or_else(|| color.to_owned(), |index| format!("ansi256:{index}"))
}

fn normalize_working_directory(directory: &str) -> String {
    directory
        .strip_prefix("file://")
        .unwrap_or(directory)
        .to_owned()
}

fn flatten(recording: ShadowRecording) -> Vec<u8> {
    let byte_count = recording
        .chunks
        .iter()
        .map(|chunk| chunk.len())
        .sum::<usize>();
    assert!(
        byte_count <= MAX_SHADOW_RECORDING_BYTES,
        "shadow recording {} exceeds its test-only byte bound",
        recording.name
    );
    recording.chunks.concat()
}

fn seeded_chunks(bytes: &[u8]) -> Vec<&[u8]> {
    let mut chunks = Vec::new();
    let mut offset = 0;
    let mut state = 0x005e_ed25_u32;
    while offset < bytes.len() {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let count = (usize::try_from(state % 23).unwrap() + 1).min(bytes.len() - offset);
        chunks.push(&bytes[offset..offset + count]);
        offset += count;
    }
    chunks
}

fn replay(recording: ShadowRecording, chunks: &[&[u8]]) -> (DeclaredOverlap, DeclaredOverlap) {
    assert!(chunks.len() <= MAX_SHADOW_REPLAY_CHUNKS);
    let mut provisional = TerminalScreen::default();
    provisional.resize_grid(recording.columns, recording.rows);
    let mut provisional_lane = TerminalLaneState::default();
    provisional_lane.columns = recording.columns;
    provisional_lane.rows = recording.rows;
    let mut provisional_replies = Vec::new();

    let mut core = TerminalCoreSessionAdapter::new(
        format!("shadow-{}", recording.name),
        format!("shadow-context-{}", recording.name),
        recording.columns,
        recording.rows,
    )
    .expect("bounded shadow core should construct");
    let mut core_lane = TerminalLaneState::default();
    let mut core_replies = Vec::new();

    for chunk in chunks {
        provisional_replies
            .extend(provisional.apply_bytes_with_responses(&mut provisional_lane, chunk));
        let TerminalCoreAdapterUpdate { replies, .. } = core
            .apply_output(&mut core_lane, chunk)
            .expect("bounded shadow recording should be accepted by TerminalCore");
        core_replies.extend(replies);
    }

    (
        DeclaredOverlap::capture(
            &provisional_lane,
            provisional.bracketed_paste_enabled(),
            provisional_replies,
        ),
        DeclaredOverlap::capture(&core_lane, core.bracketed_paste_enabled(), core_replies),
    )
}

fn assert_recording_matches(recording: ShadowRecording, label: &str, chunks: &[&[u8]]) {
    let (provisional, core) = replay(recording, chunks);
    assert_eq!(
        provisional, core,
        "DTC-P25 declared overlap diverged for {} ({label}); source: {}",
        recording.name, recording.source
    );
}

fn assert_recorded_boundaries_match(recording: ShadowRecording) {
    let mut provisional = TerminalScreen::default();
    provisional.resize_grid(recording.columns, recording.rows);
    let mut provisional_lane = TerminalLaneState::default();
    provisional_lane.columns = recording.columns;
    provisional_lane.rows = recording.rows;
    let mut provisional_replies = Vec::new();

    let mut core = TerminalCoreSessionAdapter::new(
        format!("shadow-boundary-{}", recording.name),
        format!("shadow-boundary-context-{}", recording.name),
        recording.columns,
        recording.rows,
    )
    .expect("bounded shadow core should construct");
    let mut core_lane = TerminalLaneState::default();
    let mut core_replies = Vec::new();

    for (index, chunk) in recording.chunks.iter().enumerate() {
        provisional_replies
            .extend(provisional.apply_bytes_with_responses(&mut provisional_lane, chunk));
        let update = core
            .apply_output(&mut core_lane, chunk)
            .expect("recorded semantic boundary should apply");
        core_replies.extend(update.replies);

        assert_eq!(
            DeclaredOverlap::capture(
                &provisional_lane,
                provisional.bracketed_paste_enabled(),
                provisional_replies.clone(),
            ),
            DeclaredOverlap::capture(
                &core_lane,
                core.bracketed_paste_enabled(),
                core_replies.clone(),
            ),
            "DTC-P25 declared overlap diverged for {} after recorded boundary {index}; source: {}",
            recording.name,
            recording.source,
        );
    }
}

#[test]
fn dtc_p25_recorded_overlap_matches_whole_recorded_and_arbitrary_chunks() {
    assert!(RECORDINGS.len() <= MAX_SHADOW_RECORDINGS);
    for &recording in RECORDINGS {
        assert_recorded_boundaries_match(recording);
        let bytes = flatten(recording);
        let whole = [bytes.as_slice()];
        assert_recording_matches(recording, "whole", &whole);
        assert_recording_matches(recording, "recorded PTY chunks", recording.chunks);
        let bytewise = bytes.iter().map(std::slice::from_ref).collect::<Vec<_>>();
        assert_recording_matches(recording, "bytewise", &bytewise);
        let irregular = seeded_chunks(&bytes);
        assert_recording_matches(recording, "seeded irregular", &irregular);
    }
}

#[test]
fn dtc_p25_non_overlap_uses_terminal_core_normative_unicode_and_link_proof() {
    // The provisional parser is not an oracle for grapheme width or OSC 8.
    // Prove these behaviors directly against the governed TerminalCore result
    // and keep them outside `DeclaredOverlap`.
    let mut core = TerminalCoreSessionAdapter::new("normative", "normative-context", 20, 4)
        .expect("normative core should construct");
    let mut lane = TerminalLaneState::default();
    core.apply_output(
        &mut lane,
        "A界e\u{301}\x1b]8;;https://example.com\x07link\x1b]8;;\x07".as_bytes(),
    )
    .expect("normative Unicode/link stream should apply");

    assert_eq!(lane.grid_lines()[0], "A界e\u{301}link");
    assert_eq!(lane.screen_cursor_col, 8);
    let accessibility = core
        .accessibility_snapshot(4, 0, true)
        .expect("normative accessibility snapshot should project");
    assert_eq!(accessibility.links.len(), 1);
    assert_eq!(accessibility.links[0].uri, "https://example.com");
    assert_eq!(accessibility.links[0].start, 4);
    assert_eq!(accessibility.links[0].end, 8);
}

#[test]
fn dtc_p25_shadow_is_bounded_and_has_no_production_selector() {
    let total = RECORDINGS
        .iter()
        .flat_map(|recording| recording.chunks)
        .map(|chunk| chunk.len())
        .sum::<usize>();
    assert!(total <= MAX_SHADOW_RECORDINGS * MAX_SHADOW_RECORDING_BYTES);
    assert!(
        RECORDINGS
            .iter()
            .all(|recording| !recording.name.is_empty())
    );
}
