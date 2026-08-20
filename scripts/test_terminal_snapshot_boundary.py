#!/usr/bin/env python3
"""Hermetic mutations for the DTC-P20 snapshot and damage guard."""

from __future__ import annotations

import importlib.util
import shutil
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("check_terminal_snapshot_boundary.py")
SPEC = importlib.util.spec_from_file_location("terminal_snapshot_guard", MODULE_PATH)
assert SPEC and SPEC.loader
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class TerminalSnapshotBoundaryTest(unittest.TestCase):
    def fixture(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        root = Path(temporary.name)
        crate = root / guard.CRATE
        crate.mkdir(parents=True)
        shutil.copytree(guard.ROOT / guard.CRATE / "src", crate / "src")
        return temporary, root

    def test_valid_projection_passes(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        self.assertEqual(guard.check(root), [])

    def test_renderer_authority_and_snapshot_limit_removal_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        snapshot = root / guard.CRATE / "src/snapshot.rs"
        snapshot.write_text(
            snapshot.read_text().replace("cell_limit.check(cells)", "Ok(())")
            + "\nuse wgpu::Texture;\n",
            encoding="utf-8",
        )
        failures = guard.check(root)
        self.assertTrue(any("cell_limit.check" in failure for failure in failures))
        self.assertTrue(any("wgpu" in failure for failure in failures))

    def test_history_graphics_and_damage_coalescing_removal_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        screen = root / guard.CRATE / "src/screen.rs"
        screen.write_text(
            screen.read_text().replace("RenderRowSource::History", "RenderRowSource::Screen"),
            encoding="utf-8",
        )
        semantics = root / guard.CRATE / "src/semantics.rs"
        semantics.write_text(
            semantics.read_text().replace("update.damage.push_coalesced(damage)", "let _ = damage"),
            encoding="utf-8",
        )
        reducer_damage = root / guard.CRATE / "src/reducer_damage.rs"
        reducer_damage.write_text(
            reducer_damage.read_text().replace(
                "core.state.history.fingerprint()", "core.state.history.snapshot()"
            ),
            encoding="utf-8",
        )
        failures = guard.check(root)
        self.assertTrue(any("RenderRowSource::History" in failure for failure in failures))
        self.assertTrue(any("push_coalesced" in failure for failure in failures))
        self.assertTrue(any("history.fingerprint" in failure for failure in failures))

    def test_deterministic_order_and_named_proof_removal_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        snapshot = root / guard.CRATE / "src/snapshot.rs"
        snapshot.write_text(
            snapshot.read_text().replace("graphics.sort_by_key(", "graphics.iter().map("),
            encoding="utf-8",
        )
        proofs = root / guard.CRATE / "src/render_snapshot_tests.rs"
        proofs.write_text(
            proofs.read_text().replace(
                "damage_reports_cell_rows_scroll_cursor_history_palette_and_graphics", "removed"
            ),
            encoding="utf-8",
        )
        failures = guard.check(root)
        self.assertTrue(any("graphics.sort_by_key" in failure for failure in failures))
        self.assertTrue(any("damage_reports_cell" in failure for failure in failures))


if __name__ == "__main__":
    unittest.main()
