#!/usr/bin/env python3
"""Enforce the std-only Datum TerminalCore foundation boundary."""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRATE = Path("crates/terminal-core")

REQUIRED_MODULES = {
    "cell.rs": ("pub struct Cluster", "pub enum CellContent", "pub struct CellStyle"),
    "color.rs": ("pub enum Color",),
    "coordinates.rs": ("pub struct TerminalSize", "pub struct LogicalPoint"),
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
    "snapshot.rs": ("pub struct TerminalSnapshot", "fn validate_continuations"),
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
        if path.name == "tests.rs":
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
