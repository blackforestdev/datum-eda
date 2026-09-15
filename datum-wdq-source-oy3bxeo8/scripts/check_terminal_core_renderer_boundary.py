#!/usr/bin/env python3
"""Enforce DTC-P23's immutable TerminalCore-to-GPU renderer boundary."""

from __future__ import annotations

import sys
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RENDER = Path("crates/gui-render")
APP = Path("crates/gui-app/src")
CORE_RENDER = RENDER / "src/terminal_core_render.rs"
CACHE = RENDER / "src/terminal_render_cache.rs"
GRAPHICS = RENDER / "src/render/terminal_graphics.rs"
GPU = RENDER / "src/render/gpu.rs"
SCENE = RENDER / "src/render/scene.rs"
TESTS = RENDER / "src/terminal_core_render_tests.rs"
VISUAL = RENDER / "src/visual_capture.rs"
PANE_RENDER = RENDER / "src/terminal_pane_render.rs"

REQUIRED_PROOFS = (
    "immutable_core_snapshot_drives_complete_style_and_fixed_cluster_geometry",
    "cursor_uses_core_row_column_shape_and_palette_without_lane_projection",
    "retained_rows_rebuild_only_for_declared_damage_or_geometry_change",
    "sixel_snapshot_produces_clipped_gpu_image_placement",
    "offscreen_terminal_sixel_uses_the_production_texture_pipeline",
    "render_state_preserves_surface_pixels_and_consumes_damage_once",
)


def read(root: Path, relative: Path) -> str:
    path = root / relative
    return path.read_text(encoding="utf-8") if path.is_file() else ""


def check(root: Path) -> list[str]:
    failures: list[str] = []
    manifest = read(root, RENDER / "Cargo.toml")
    core = read(root, CORE_RENDER)
    cache = read(root, CACHE)
    graphics = read(root, GRAPHICS)
    gpu = read(root, GPU)
    scene = read(root, SCENE)
    tests = read(root, TESTS)
    visual = read(root, VISUAL)
    pane_render = read(root, PANE_RENDER)
    main = read(root, APP / "main.rs")
    runtime_render = read(root, APP / "runtime_terminal_render.rs")
    adapter = read(root, APP / "terminal_core_adapter.rs")
    session = read(root, APP / "terminal_session.rs")
    session_render = read(root, APP / "terminal_session_render.rs")

    if 'datum-terminal-core = { path = "../terminal-core" }' not in manifest:
        failures.append("gui-render must consume the first-party TerminalCore path crate")
    for forbidden in ("alacritty_terminal", "portable-pty", "ghostty", "libghostty"):
        if forbidden in manifest:
            failures.append(f"renderer contains forbidden terminal dependency: {forbidden}")

    for marker in (
        "RenderSnapshot",
        "render_row(",
        "CellContent::Continuation",
        "CellWidth::Two",
        "CellAttribute::Bold",
        "CellAttribute::Italic",
        "CellAttribute::Hidden",
        "CellAttribute::Inverse",
        "UnderlineStyle::Double",
        "UnderlineStyle::Curly",
        "UnderlineStyle::Dotted",
        "UnderlineStyle::Dashed",
        "underline_color",
        "snapshot.selection()",
        "snapshot.cursor()",
        "clip_bounds",
        "prepare_terminal_graphics",
    ):
        if marker not in core:
            failures.append(f"immutable core renderer lacks behavior marker: {marker}")
    for forbidden in ("TerminalStyledLine", "pty_grid_mut", "grid_styled_lines"):
        if forbidden in core:
            failures.append(f"core renderer reads provisional authority: {forbidden}")
    if re.search(r"use\s+[^;]*\bTerminalScreen\b", core):
        failures.append("core renderer reads provisional authority: TerminalScreen")

    for marker in (
        "damage: &[Damage]",
        "row_is_damaged",
        "Damage::Cursor",
        "Damage::Graphics",
        "active_session_id",
        "scroll_offset",
        "rebuilt_rows",
    ):
        if marker not in cache:
            failures.append(f"damage cache lacks ownership marker: {marker}")
    if "cached.cells != row.cells()" in cache:
        failures.append("row rebuilds must be driven by declared damage, not full cell comparison")

    for marker in (
        "Rgba8UnormSrgb",
        "BlendState::ALPHA_BLENDING",
        "queue.write_texture(",
        "GraphicAnchorResolution::History",
        "GraphicAnchorResolution::Screen",
        "encode_terminal_graphics(&mut encoder, &msaa_view, target, false)",
        "encode_terminal_graphics(&mut encoder, &msaa_view, target, true)",
    ):
        corpus = core + graphics + gpu
        if marker not in corpus:
            failures.append(f"GPU image path lacks marker: {marker}")

    for marker in (
        "take_active_tab_render_states(",
        "from_workspace_with_terminal_renderer(",
        "Some(&mut self.terminal_render_cache)",
    ):
        if marker not in main + runtime_render:
            failures.append(f"production runtime bypasses retained core rendering: {marker}")
    if "terminal_panes: &[crate::TerminalPaneRenderState]" not in scene:
        failures.append("prepared scene does not accept immutable per-pane TerminalCore snapshots")
    for marker in (
        "pub struct TerminalPaneRenderState",
        "pub session_id: String",
        "pub snapshot: datum_terminal_core::RenderSnapshot",
        "pub damage: Vec<datum_terminal_core::Damage>",
    ):
        if marker not in pane_render:
            failures.append(f"per-pane immutable renderer input lacks marker: {marker}")
    for marker in (
        "pixel_width: u32",
        "pixel_height: u32",
        "take_render_state(",
        "pending_render_damage",
    ):
        if marker not in adapter + session + session_render:
            failures.append(f"DPI/damage adapter lacks marker: {marker}")
    if "geometry.screen.width.round() as u32" not in session_render:
        failures.append("each split PTY/core resize does not receive its rendered pixel width")

    proof_corpus = tests + visual + read(root, APP / "terminal_core_adapter_tests.rs")
    for proof in REQUIRED_PROOFS:
        if proof not in proof_corpus:
            failures.append(f"DTC-P23 lacks governed proof: {proof}")
    return failures


def main() -> int:
    failures = check(ROOT)
    if failures:
        print("terminal core renderer boundary: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1
    print("terminal core renderer boundary: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
