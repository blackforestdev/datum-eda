#!/usr/bin/env python3
"""Enforce DTC-P20 immutable render snapshots and complete damage projection."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRATE = Path("crates/terminal-core")

REQUIRED_MODULES = {
    "damage.rs": (
        "pub enum Damage",
        "Cell(CellPoint)",
        "Row(Row)",
        "Rows {",
        "Scroll {",
        "Cursor",
        "Palette(PaletteIndex)",
        "History",
        "Graphics",
        "pub(crate) fn push_coalesced(",
        "fn visible_damage(",
    ),
    "reducer_damage.rs": (
        "pub(crate) struct DamagePlan",
        "core.state.history.fingerprint()",
        "Damage::Cell(at)",
        "Damage::Row(cursor.row)",
        "Damage::Rows {",
        "Damage::Scroll {",
        "Damage::Cursor",
        "Damage::History",
        "Damage::Graphics",
    ),
    "snapshot.rs": (
        "pub const RENDER_SNAPSHOT_SCHEMA_VERSION: u16 = 1",
        "pub struct RenderRow",
        "pub struct RenderPalette",
        "pub struct RenderGraphic",
        "pub struct RenderSnapshot",
        "cell_limit.check(cells)",
        "graphics.sort_by_key(",
    ),
    "screen.rs": (
        "pub fn render_snapshot(",
        "RenderRowSource::History",
        "RenderRowSource::Screen",
        "self.resolve_graphic(placement.id())",
        "limits.snapshot_cells",
    ),
    "semantics.rs": ("update.damage.push_coalesced(damage)",),
    "history.rs": ("pub(crate) fn fingerprint(",),
}

REQUIRED_PROOFS = (
    "render_snapshot_is_stable_versioned_and_orders_history_before_screen",
    "alternate_render_snapshot_excludes_primary_history_and_preserves_selection",
    "render_snapshot_accounts_history_cells_before_cloning",
    "render_graphics_are_resolved_and_sorted_by_z_then_identity",
    "damage_reports_cell_rows_scroll_cursor_history_palette_and_graphics",
    "wide_cell_prints_dirty_every_cell_they_create_or_clear",
    "damage_overflow_degrades_deterministically_to_full",
    "full_visible_damage_retains_independent_history_invalidation",
)

FORBIDDEN = (
    "wgpu",
    "glyphon",
    "winit",
    "gui_render",
    "gui_protocol",
    "std::fs",
    "std::process",
    "unsafe {",
    "unsafe fn",
    "include!",
)


def check(root: Path) -> list[str]:
    failures: list[str] = []
    source = root / CRATE / "src"
    joined = []
    for name, markers in REQUIRED_MODULES.items():
        path = source / name
        if not path.is_file():
            failures.append(f"DTC-P20 owned projection module is missing: {name}")
            continue
        text = path.read_text(encoding="utf-8")
        joined.append(text)
        for marker in markers:
            if marker not in text:
                failures.append(f"DTC-P20 module {name} lacks owned marker: {marker}")

    combined = "\n".join(joined)
    for marker in FORBIDDEN:
        if marker in combined:
            failures.append(f"DTC-P20 snapshot/damage boundary contains forbidden authority: {marker}")

    library = (source / "lib.rs").read_text(encoding="utf-8")
    for marker in (
        "RENDER_SNAPSHOT_SCHEMA_VERSION",
        "RenderGraphic",
        "RenderPalette",
        "RenderRowSource",
        "RenderSnapshot",
    ):
        if marker not in library:
            failures.append(f"DTC-P20 public immutable projection is missing: {marker}")

    proofs = source / "render_snapshot_tests.rs"
    proof_text = proofs.read_text(encoding="utf-8") if proofs.is_file() else ""
    for marker in REQUIRED_PROOFS:
        if marker not in proof_text:
            failures.append(f"DTC-P20 deterministic projection proof is missing: {marker}")
    return failures


def main() -> int:
    failures = check(ROOT)
    if failures:
        print("Datum TerminalCore snapshot boundary check FAILED:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1
    print("Datum TerminalCore snapshot boundary check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
