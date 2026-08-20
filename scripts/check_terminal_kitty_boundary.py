#!/usr/bin/env python3
"""Enforce the Datum-owned, bounded Kitty graphics protocol boundary."""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRATE = Path("crates/terminal-core")

REQUIRED_MODULES = {
    "kitty_protocol.rs": (
        "pub(crate) fn parse_control(",
        "pub(crate) fn decode_pixels(",
        "KittyMedium::Direct",
        "KittyGraphicsError::UnsupportedMedium",
        "z_set",
    ),
    "kitty_store.rs": (
        "pub(crate) struct PendingKittyTransfer",
        "pub(crate) fn store_image(",
        "pub(crate) fn add_frame(",
        "pub(crate) fn advance(",
        "other_frames",
    ),
    "kitty_pixels.rs": (
        "pub(crate) fn composite_block(",
        "pub(crate) fn copy_rectangle(",
        "pub(crate) fn valid_continuation_header(",
    ),
    "kitty_graphics.rs": (
        "pub(crate) fn apply_kitty_graphics(",
        "fn collect_kitty_transfer(",
        "combined_length",
        "MAX_RELATIVE_DEPTH",
        "pub fn advance_kitty_animations(",
    ),
    "kitty_commands.rs": (
        "pub(crate) fn kitty_animate(",
        "pub(crate) fn kitty_compose(",
        "pub(crate) fn kitty_delete(",
        "pub(crate) fn kitty_success(",
        "pub(crate) fn kitty_failure(",
    ),
    "kitty_placeholder.rs": (
        "pub struct KittyPlaceholder",
        "pub fn kitty_placeholder(",
        "const KITTY_DIACRITICS: [u32; 297]",
    ),
}

REQUIRED_PROOFS = (
    "direct_rgb_rgba_query_and_image_number_replies_are_exact",
    "chunked_transfer_is_atomic_and_uses_final_cursor_position",
    "metadata_order_is_irrelevant_and_put_replaces_named_placement",
    "placement_crop_virtual_relative_cycle_and_parent_lifetime_are_bounded",
    "animation_frames_composition_and_deterministic_tick_update_placements",
    "animation_defaults_missing_frame_gap_to_forty_milliseconds_and_rejects_bad_indices",
    "soft_and_hard_delete_obey_placement_and_image_lifetimes",
    "unsupported_external_transfers_and_quiet_modes_are_safe",
    "unicode_placeholders_resolve_color_ids_diacritics_and_inheritance",
    "malformed_interleaving_and_limits_leave_prior_state_unchanged",
    "aggregate_frame_limit_is_atomic_across_sixel_and_kitty_graphics",
    "graphics_survive_history_reflow_and_buffer_teardown",
    "zlib_and_png_transfer_use_datum_owned_codecs",
)

FORBIDDEN = (
    "unsafe {",
    "unsafe fn",
    "std::fs",
    "std::process",
    "std::net",
    "std::os",
    "libc::",
    "include!",
    "ghostty",
    "alacritty",
    "portable_pty",
    "vte::",
    "from_utf8_lossy",
)


def check(root: Path) -> list[str]:
    failures: list[str] = []
    crate = root / CRATE
    manifest_path = crate / "Cargo.toml"
    if not manifest_path.is_file():
        return [f"Datum TerminalCore manifest is missing: {manifest_path}"]
    manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    for table in ("dependencies", "dev-dependencies", "build-dependencies"):
        if manifest.get(table):
            failures.append(f"DTC-P19 Kitty implementation must remain std-only; {table} is not empty")

    sources: dict[str, str] = {}
    for name, markers in REQUIRED_MODULES.items():
        path = crate / "src" / name
        if not path.is_file():
            failures.append(f"DTC-P19 owned Kitty module is missing: {name}")
            continue
        text = path.read_text(encoding="utf-8")
        sources[name] = text
        for marker in markers:
            if marker not in text:
                failures.append(f"DTC-P19 module {name} lacks owned marker: {marker}")

    joined = "\n".join(sources.values())
    for marker in FORBIDDEN:
        if marker in joined:
            failures.append(f"DTC-P19 contains forbidden authority or dependency marker: {marker}")

    control = (crate / "src/control_string.rs").read_text(encoding="utf-8")
    if "ControlStringKind::Apc => self.apply_kitty_graphics(&string.bytes, update)" not in control:
        failures.append("DTC-P19 APC input must enter the owned Kitty semantic reducer")
    graphics = (crate / "src/graphics.rs").read_text(encoding="utf-8")
    for marker in (
        "pub(crate) fn insert_kitty(",
        "fn prune_missing_kitty_parents(",
        "checked_add(self.kitty.total_frames())",
        "self.kitty.clear()",
    ):
        if marker not in graphics:
            failures.append(f"DTC-P19 graphics lifecycle/accounting marker is missing: {marker}")
    library = (crate / "src/lib.rs").read_text(encoding="utf-8")
    for marker in (
        "pub use kitty_placeholder::KittyPlaceholder",
        "pub use kitty_protocol::KittyGraphicsError",
        "pub use kitty_store::{KittyAnimationState, KittyFrame, KittyImage}",
    ):
        if marker not in library:
            failures.append(f"DTC-P19 public data contract is missing: {marker}")

    tests = crate / "src/kitty_graphics_tests.rs"
    proof_text = tests.read_text(encoding="utf-8") if tests.is_file() else ""
    for marker in REQUIRED_PROOFS:
        if marker not in proof_text:
            failures.append(f"DTC-P19 deterministic Kitty proof is missing: {marker}")
    return failures


def main() -> int:
    failures = check(ROOT)
    if failures:
        print("Datum TerminalCore Kitty boundary check FAILED:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1
    print("Datum TerminalCore Kitty boundary check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
