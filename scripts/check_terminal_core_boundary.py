#!/usr/bin/env python3
"""Enforce the std-only Datum TerminalCore foundation boundary."""

from __future__ import annotations

import sys
import hashlib
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRATE = Path("crates/terminal-core")

REQUIRED_MODULES = {
    "cell.rs": ("pub struct Cluster", "pub enum CellContent", "pub struct CellStyle"),
    "charset.rs": ("pub enum CharacterSet", "pub struct CharacterSetState", "pub fn map("),
    "color.rs": ("pub enum Color",),
    "control_string.rs": ("fn apply_control_string", "fn apply_osc", "fn apply_dcs"),
    "coordinates.rs": ("pub struct TerminalSize", "pub struct LogicalPoint"),
    "csi.rs": ("fn apply_csi", "fn set_private_mode", "fn report_mode"),
    "damage.rs": ("pub enum Damage", "pub struct DamageSet"),
    "event.rs": ("pub enum CoreEvent", "pub struct TerminalReply"),
    "grid.rs": ("struct GridBuffer", "fn repair_row", "fn clear_cluster_at"),
    "limits.rs": ("pub enum LimitKind", "pub struct CoreLimits", "pub struct CoreLimitValues"),
    "mode.rs": ("pub struct CursorState", "pub struct Margins", "pub struct ModeState"),
    "parser.rs": ("pub struct StreamingParser", "pub fn feed(", "pub fn finish("),
    "parser_action.rs": ("pub enum Action", "pub struct CsiSequence", "pub enum ParseError"),
    "reducer.rs": ("pub enum ScreenError", "pub struct Reduction", "pub fn reduce("),
    "reducer_action.rs": ("pub enum ScreenAction", "pub enum EraseLine", "pub enum FoundationMode"),
    "screen.rs": ("pub struct ScreenState", "pub struct TerminalCore"),
    "semantics.rs": ("pub enum CoreError", "pub struct CoreUpdate", "pub fn apply("),
    "sgr.rs": ("fn apply_sgr", "fn extended_color"),
    "snapshot.rs": ("pub struct TerminalSnapshot", "fn validate_continuations"),
    "unicode.rs": (
        'pub const UNICODE_VERSION: &str = "17.0.0"',
        "pub fn grapheme_break_before(",
        "pub fn grapheme_indices(",
        "pub fn terminal_cluster_width(",
        "BidirectionalTextPolicy::LogicalOrder",
        "pub struct ShapingCluster",
    ),
    "unicode_grapheme_tables.rs": ("GRAPHEME_BREAK_RANGES", "INCB_RANGES"),
    "unicode_width_tables.rs": (
        "EAST_ASIAN_WIDTH_RANGES",
        "EXTENDED_PICTOGRAPHIC_RANGES",
        "EMOJI_PRESENTATION_RANGES",
    ),
}

UNICODE_INPUTS = {
    "17.0.0/DerivedCoreProperties.txt": "24c7fed1195c482faaefd5c1e7eb821c5ee1fb6de07ecdbaa64b56a99da22c08",
    "17.0.0/EastAsianWidth.txt": "ea7ce50f3444a050333448dffef1cadd9325af55cbb764b4a2280faf52170a33",
    "17.0.0/GraphemeBreakProperty.txt": "d6b51d1d2ae5c33b451b7ed994b48f1f4dc62b2272a5831e7fd418514a6bae89",
    "17.0.0/GraphemeBreakTest.txt": "e2d134d2c52919bace503ebb6a551c1855fe1a1faec18478c78fff254a1793ec",
    "17.0.0/emoji-data.txt": "2cb2bb9455cda83e8481541ecf5b6dfda66a3bb89efa3fa7c5297eccf607b72b",
    "17.0.0/emoji-sequences.txt": "12cc8267dc33cbd11ed32bcf6fc5dc2ad9c7a77bae1bdfba2f41b1b9b3ead8dd",
    "17.0.0/emoji-variation-sequences.txt": "bb3d09ef03f206012c7532dd52dc0a21c9efddba0135ea4cf0d9201b8b9bba7e",
    "17.0.0/emoji-zwj-sequences.txt": "5b25441daed2322b068c5e70cda522946a4f0274df864445a1965a92e5fc5cad",
    "LICENSE.txt": "e7a93b009565cfce55919a381437ac4db883e9da2126fa28b91d12732bc53d96",
}

REQUIRED_LIMITS = (
    "ParameterCount", "ParameterDigits", "ParameterValue", "SubparameterCount",
    "IntermediateBytes", "ControlStringBytes",
    "ClusterBytes", "TitleBytes", "WorkingDirectoryBytes", "ClipboardBytes",
    "NotificationBytes", "ReplyBytes", "PendingEvents", "PendingDamage",
    "HistoryLines", "HistoryBytes", "GraphicObjects", "GraphicPixels",
    "GraphicDecodedBytes", "GraphicFrames", "CompressionRatio", "ParserWork",
    "SearchWork", "ReflowWork", "ScreenCells", "SnapshotCells",
)

FORBIDDEN_SOURCE = (
    "unsafe {", "unsafe fn", "std::fs", "std::process", "std::net", "std::os",
    "winit", "wgpu", "glyphon", "gui_app", "gui_protocol", "gui_render",
    "DesignModel", "Operation", "commit(", "journal", "libc::", "include!",
    "extern crate", "ghostty", "alacritty", "portable_pty", "vte::",
    "unicode_segmentation::", "unicode_width::", "unicode_bidi::", "icu_", "std::env",
    "from_utf8_lossy",
)

REQUIRED_PARSER_PROOFS = (
    "utf8_and_ecma48_actions_are_invariant_across_every_byte_boundary",
    "cancellation_aborts_sequences_and_recovers_at_ground",
    "malformed_utf8_replacement_and_reprocessing_are_chunk_invariant",
    "oversized_sequences_emit_one_error_discard_and_recover",
    "parser_work_cap_returns_a_resumable_consumed_prefix",
    "end_of_stream_reports_and_resets_incomplete_input",
    "seeded_malformed_streams_replay_identically_under_arbitrary_chunking",
    "malformed_csi_parameter_after_intermediate_discards_until_final",
)

REQUIRED_REDUCER_PROOFS = (
    "both_screen_buffers_are_admitted_as_one_checked_resource",
    "delayed_wrap_scroll_and_hard_soft_line_identity_are_deterministic",
    "primary_and_alternate_buffers_are_isolated_and_reset_is_total",
    "wide_clusters_remain_atomic_across_overwrite_insert_delete_and_erase",
    "selective_erase_preserves_protected_clusters_without_orphans",
    "margins_confine_line_insertion_deletion_and_scrolling",
    "cell_edits_outside_horizontal_margins_use_the_full_screen_without_underflow",
    "save_restore_modes_style_cursor_and_damage_are_closed",
    "seeded_edit_sequences_never_create_orphan_continuations",
)

REQUIRED_SEMANTIC_PROOFS = (
    "parser_actions_drive_controls_tabs_and_dec_special_graphics",
    "sgr_semicolon_and_colon_forms_preserve_complete_cell_style",
    "csi_origin_margins_protected_erase_and_tab_controls_reach_the_reducer",
    "save_restore_includes_designated_character_sets_and_protection",
    "private_modes_alternate_screen_cursor_style_and_mode_queries_are_exact",
    "device_cursor_window_and_status_string_reports_are_byte_exact",
    "osc_metadata_palette_and_default_color_state_are_bounded_and_queryable",
    "synchronized_output_defers_all_damage_until_one_final_flush",
    "complete_semantics_are_invariant_across_arbitrary_parser_chunks",
    "metadata_limits_and_unsupported_queries_fail_closed",
    "semantic_events_and_replies_share_one_checked_pending_limit",
)

REQUIRED_UNICODE_PROOFS = (
    "unicode_17_grapheme_break_corpus_matches_every_normative_boundary",
    "every_rgi_emoji_sequence_is_one_two_cell_cluster",
    "unicode_width_policy_covers_ascii_ambiguous_cjk_and_emoji",
    "terminal_core_combines_marks_and_emoji_without_orphan_cells",
    "variation_selector_width_expansion_wraps_atomically_at_the_right_edge",
    "unicode_screen_state_is_invariant_across_every_utf8_chunk_boundary",
    "bidirectional_text_policy_preserves_logical_cell_order",
    "shaping_boundary_exposes_original_cluster_text_and_fixed_cell_ownership",
)


def check(root: Path) -> list[str]:
    failures: list[str] = []
    crate = root / CRATE
    manifest_path = crate / "Cargo.toml"
    if not manifest_path.is_file():
        return [f"Datum TerminalCore manifest is missing: {CRATE / 'Cargo.toml'}"]

    manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest.get("package", {}).get("name") != "datum-terminal-core":
        failures.append("TerminalCore package must be named datum-terminal-core")
    for table in ("dependencies", "dev-dependencies", "build-dependencies"):
        if manifest.get(table):
            failures.append(f"TerminalCore must remain std-only; {table} is not empty")

    workspace = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    if "crates/terminal-core" not in workspace.get("workspace", {}).get("members", []):
        failures.append("TerminalCore must be an explicit workspace member")

    sources: dict[str, str] = {}
    for path in sorted((crate / "src").glob("*.rs")):
        if path.name == "tests.rs" or path.name.endswith("_tests.rs"):
            continue
        sources[path.name] = path.read_text(encoding="utf-8")

    for relative, markers in REQUIRED_MODULES.items():
        text = sources.get(relative)
        if text is None:
            failures.append(f"TerminalCore owned module is missing: {relative}")
            continue
        for marker in markers:
            if marker not in text:
                failures.append(f"TerminalCore module {relative} lacks owned marker: {marker}")

    joined = "\n".join(sources.values())
    for marker in FORBIDDEN_SOURCE:
        if marker in joined:
            failures.append(f"TerminalCore contains forbidden authority or dependency marker: {marker}")

    limits = sources.get("limits.rs", "")
    for name in REQUIRED_LIMITS:
        if name not in limits:
            failures.append(f"TerminalCore checked limit family is missing: {name}")
    if "impl Default for CoreLimits" in limits or "impl Default for CoreLimitValues" in limits:
        failures.append("TerminalCore numeric resource policy must remain owner-supplied")
    if "const MAX_" in limits:
        failures.append("DTC-P07 must not invent owner-owned numeric resource ceilings")

    lib = sources.get("lib.rs", "")
    if "pub use screen::{ScreenState, TerminalCore}" not in lib:
        failures.append("TerminalCore root must expose the owned state authority")
    if "pub use parser::{FeedReport, StreamingParser}" not in lib:
        failures.append("TerminalCore root must expose the DTC-P08 streaming parser")
    if "pub use reducer::{Reduction, ScreenError}" not in lib:
        failures.append("TerminalCore root must expose the DTC-P09 reducer result boundary")
    if "pub use reducer_action::{" not in lib or "ScreenAction" not in lib:
        failures.append("TerminalCore root must expose the DTC-P09 typed screen actions")
    if "pub use semantics::{CoreError, CoreUpdate}" not in lib:
        failures.append("TerminalCore root must expose the DTC-P10 semantic update boundary")
    parser = sources.get("parser.rs", "")
    action = sources.get("parser_action.rs", "")
    if "emit: impl FnMut(Action)" not in parser:
        failures.append("DTC-P08 parser must stream typed actions to a caller-owned sink")
    if "Vec<Action>" in parser or "VecDeque<Action>" in parser:
        failures.append("DTC-P08 parser must not retain an unbounded action queue")
    for marker in (
        "ControlStringKind::Osc", "ControlStringKind::Dcs", "ControlStringKind::Apc",
        "ControlStringKind::Pm", "ControlStringKind::Sos", "DiscardState",
        "MalformedUtf8", "work_exhausted",
    ):
        if marker not in parser:
            failures.append(f"DTC-P08 streaming parser behavior marker is missing: {marker}")
    for marker in ("Cancelled", "CsiParameter", "StringTerminator", "LimitExceeded"):
        if marker not in action:
            failures.append(f"DTC-P08 typed action marker is missing: {marker}")

    parser_tests = crate / "src" / "parser_tests.rs"
    proof_text = parser_tests.read_text(encoding="utf-8") if parser_tests.is_file() else ""
    for marker in REQUIRED_PARSER_PROOFS:
        if marker not in proof_text:
            failures.append(f"DTC-P08 deterministic parser proof is missing: {marker}")

    screen = sources.get("screen.rs", "")
    reducer = sources.get("reducer.rs", "")
    grid = sources.get("grid.rs", "")
    if "primary: GridBuffer" not in screen or "alternate: GridBuffer" not in screen:
        failures.append("DTC-P09 screen must own distinct primary and alternate buffers")
    if "impl TerminalCore" not in reducer or "self.apply_action(action)" not in reducer:
        failures.append("DTC-P09 must mutate screen state through the TerminalCore reducer")
    if "repair_row(row)" not in reducer or "CellContent::Continuation" not in grid:
        failures.append("DTC-P09 edits must repair wide-cell continuation invariants")
    if "VecDeque<ScreenAction>" in joined or "Vec<ScreenAction>" in joined:
        failures.append("DTC-P09 reducer must not retain an unbounded action queue")

    reducer_tests = crate / "src" / "reducer_tests.rs"
    reducer_proof_text = (
        reducer_tests.read_text(encoding="utf-8") if reducer_tests.is_file() else ""
    )
    for marker in REQUIRED_REDUCER_PROOFS:
        if marker not in reducer_proof_text:
            failures.append(f"DTC-P09 deterministic reducer proof is missing: {marker}")

    semantics = sources.get("semantics.rs", "")
    charset = sources.get("charset.rs", "")
    csi = sources.get("csi.rs", "")
    control_string = sources.get("control_string.rs", "")
    if "self.state.charsets.map(character)" not in semantics:
        failures.append("DTC-P10 printable bytes must pass through owned character-set mapping")
    if "CharacterSet::DecSpecialGraphics" not in charset:
        failures.append("DTC-P10 must retain the owned DEC special-graphics mapping")
    if "2026 => self.end_synchronized_output(update)?" not in csi:
        failures.append("DTC-P10 synchronized-output disable must release deferred damage")
    if "self.state.synchronized_dirty = true" not in semantics:
        failures.append("DTC-P10 synchronized output must retain deferred-damage state")
    if "ReplyKind::ModeReport" not in csi or "ReplyKind::DeviceAttributes" not in semantics:
        failures.append("DTC-P10 must retain exact mode and device report ownership")
    for marker in ("self.state.palette[index as usize] = color", "TitleText::new", "WorkingDirectoryText::new"):
        if marker not in control_string:
            failures.append(f"DTC-P10 bounded OSC metadata owner is missing: {marker}")
    if "Vec<CoreUpdate>" in joined or "VecDeque<CoreUpdate>" in joined:
        failures.append("DTC-P10 semantics must not retain an unbounded update queue")

    semantic_tests = crate / "src" / "semantic_tests.rs"
    semantic_proof_text = (
        semantic_tests.read_text(encoding="utf-8") if semantic_tests.is_file() else ""
    )
    for marker in REQUIRED_SEMANTIC_PROOFS:
        if marker not in semantic_proof_text:
            failures.append(f"DTC-P10 deterministic semantic proof is missing: {marker}")

    unicode_root = crate / "unicode"
    for relative, expected in UNICODE_INPUTS.items():
        path = unicode_root / relative
        if not path.is_file():
            failures.append(f"DTC-P11 Unicode input is missing: {relative}")
            continue
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != expected:
            failures.append(f"DTC-P11 Unicode input checksum drifted: {relative}")
    generator = root / "scripts/generate_terminal_unicode.py"
    generator_text = generator.read_text(encoding="utf-8") if generator.is_file() else ""
    for marker in ("EXPECTED_SHA256", "--check", "verify_inputs", "generated_files"):
        if marker not in generator_text:
            failures.append(f"DTC-P11 offline Unicode generator marker is missing: {marker}")
    if "urlopen" in generator_text or "requests" in generator_text or "curl" in generator_text:
        failures.append("DTC-P11 Unicode generation must remain offline")
    if "ScreenAction::AppendCluster" not in sources.get("semantics.rs", ""):
        failures.append("DTC-P11 grapheme extension must enter the sole screen reducer")
    if "grapheme_anchor" not in sources.get("screen.rs", ""):
        failures.append("DTC-P11 screen state must retain a bounded grapheme anchor")
    unicode_tests = crate / "src" / "unicode_tests.rs"
    unicode_proof_text = unicode_tests.read_text(encoding="utf-8") if unicode_tests.is_file() else ""
    for marker in REQUIRED_UNICODE_PROOFS:
        if marker not in unicode_proof_text:
            failures.append(f"DTC-P11 deterministic Unicode proof is missing: {marker}")
    return failures


def main() -> int:
    failures = check(ROOT)
    if failures:
        print("Datum TerminalCore boundary check FAILED:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1
    print("Datum TerminalCore boundary check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
