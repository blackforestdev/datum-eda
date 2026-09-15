#!/usr/bin/env python3
"""Hermetic mutation tests for the DTC-P23 renderer guard."""

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("check_terminal_core_renderer_boundary.py")
SPEC = importlib.util.spec_from_file_location("terminal_core_renderer_guard", MODULE_PATH)
assert SPEC and SPEC.loader
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class TerminalCoreRendererBoundaryTest(unittest.TestCase):
    def fixture(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        root = Path(temporary.name)
        paths = (
            guard.RENDER / "src/render",
            guard.APP,
        )
        for path in paths:
            (root / path).mkdir(parents=True, exist_ok=True)
        (root / guard.RENDER / "Cargo.toml").write_text(
            'datum-terminal-core = { path = "../terminal-core" }\n', encoding="utf-8"
        )
        (root / guard.CORE_RENDER).write_text(
            "RenderSnapshot render_row( CellContent::Continuation CellWidth::Two "
            "CellAttribute::Bold CellAttribute::Italic CellAttribute::Hidden "
            "CellAttribute::Inverse UnderlineStyle::Double UnderlineStyle::Curly "
            "UnderlineStyle::Dotted UnderlineStyle::Dashed underline_color "
            "snapshot.selection() snapshot.cursor() clip_bounds prepare_terminal_graphics "
            "GraphicAnchorResolution::History GraphicAnchorResolution::Screen\n",
            encoding="utf-8",
        )
        (root / guard.CACHE).write_text(
            "damage: &[Damage] row_is_damaged Damage::Cursor Damage::Graphics "
            "active_session_id scroll_offset rebuilt_rows\n",
            encoding="utf-8",
        )
        (root / guard.GRAPHICS).write_text(
            "Rgba8UnormSrgb BlendState::ALPHA_BLENDING queue.write_texture(\n",
            encoding="utf-8",
        )
        (root / guard.GPU).write_text(
            "encode_terminal_graphics(&mut encoder, &msaa_view, target, false)\n"
            "encode_terminal_graphics(&mut encoder, &msaa_view, target, true)\n",
            encoding="utf-8",
        )
        (root / guard.SCENE).write_text(
            "terminal_panes: &[crate::TerminalPaneRenderState]\n",
            encoding="utf-8",
        )
        (root / guard.PANE_RENDER).write_text(
            "pub struct TerminalPaneRenderState { pub session_id: String, "
            "pub snapshot: datum_terminal_core::RenderSnapshot, "
            "pub damage: Vec<datum_terminal_core::Damage> }\n",
            encoding="utf-8",
        )
        (root / guard.APP / "main.rs").write_text(
            "mod runtime_terminal_render;\n", encoding="utf-8"
        )
        (root / guard.APP / "runtime_terminal_render.rs").write_text(
            "take_active_tab_render_states( from_workspace_with_terminal_renderer( "
            "Some(&mut self.terminal_render_cache)\n",
            encoding="utf-8",
        )
        (root / guard.APP / "terminal_core_adapter.rs").write_text(
            "pixel_width: u32 pixel_height: u32 take_render_state( pending_render_damage\n",
            encoding="utf-8",
        )
        (root / guard.APP / "terminal_session.rs").write_text("", encoding="utf-8")
        (root / guard.APP / "terminal_session_render.rs").write_text(
            "geometry.screen.width.round() as u32\n", encoding="utf-8"
        )
        renderer_proofs = "\n".join(
            f"fn {proof}() {{}}" for proof in guard.REQUIRED_PROOFS[:4]
        )
        (root / guard.TESTS).write_text(renderer_proofs, encoding="utf-8")
        (root / guard.VISUAL).write_text(
            "fn offscreen_terminal_sixel_uses_the_production_texture_pipeline() {}\n",
            encoding="utf-8",
        )
        (root / guard.APP / "terminal_core_adapter_tests.rs").write_text(
            "fn render_state_preserves_surface_pixels_and_consumes_damage_once() {}\n",
            encoding="utf-8",
        )
        return temporary, root

    def mutate(self, relative: Path, old: str, new: str) -> list[str]:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        self.assertEqual(guard.check(root), [])
        path = root / relative
        path.write_text(path.read_text(encoding="utf-8").replace(old, new), encoding="utf-8")
        return guard.check(root)

    def test_valid_renderer_boundary_passes(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        self.assertEqual(guard.check(root), [])

    def test_snapshot_damage_and_provisional_authority_mutations_fail(self) -> None:
        self.assertTrue(self.mutate(guard.CORE_RENDER, "RenderSnapshot", "LaneSnapshot"))
        self.assertTrue(self.mutate(guard.CACHE, "row_is_damaged", "rebuild_every_row"))
        self.assertTrue(self.mutate(guard.CORE_RENDER, "\n", "\nuse TerminalScreen;\n"))

    def test_image_dpi_and_runtime_wiring_mutations_fail(self) -> None:
        self.assertTrue(self.mutate(guard.GRAPHICS, "queue.write_texture(", ""))
        self.assertTrue(
            self.mutate(
                guard.GPU,
                "encode_terminal_graphics(&mut encoder, &msaa_view, target, true)",
                "",
            )
        )
        self.assertTrue(
            self.mutate(
                guard.APP / "runtime_terminal_render.rs", "take_active_tab_render_states(", ""
            )
        )
        self.assertTrue(
            self.mutate(
                guard.APP / "terminal_session_render.rs",
                "geometry.screen.width.round() as u32",
                "0",
            )
        )

    def test_visual_or_incremental_proof_removal_fails(self) -> None:
        self.assertTrue(
            self.mutate(
                guard.VISUAL,
                "fn offscreen_terminal_sixel_uses_the_production_texture_pipeline() {}",
                "",
            )
        )
        self.assertTrue(
            self.mutate(
                guard.TESTS,
                "fn retained_rows_rebuild_only_for_declared_damage_or_geometry_change() {}",
                "",
            )
        )


if __name__ == "__main__":
    unittest.main()
