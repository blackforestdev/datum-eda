#!/usr/bin/env python3
"""Hermetic mutation tests for the Datum Kitty graphics boundary guard."""

from __future__ import annotations

import importlib.util
import shutil
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("check_terminal_kitty_boundary.py")
SPEC = importlib.util.spec_from_file_location("terminal_kitty_guard", MODULE_PATH)
assert SPEC and SPEC.loader
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class TerminalKittyBoundaryTest(unittest.TestCase):
    def fixture(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        root = Path(temporary.name)
        crate = root / guard.CRATE
        crate.mkdir(parents=True)
        shutil.copy2(guard.ROOT / guard.CRATE / "Cargo.toml", crate / "Cargo.toml")
        shutil.copytree(guard.ROOT / guard.CRATE / "src", crate / "src")
        return temporary, root

    def test_valid_owned_kitty_implementation_passes(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        self.assertEqual(guard.check(root), [])

    def test_missing_module_and_external_io_authority_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        (root / guard.CRATE / "src/kitty_pixels.rs").unlink()
        protocol = root / guard.CRATE / "src/kitty_protocol.rs"
        protocol.write_text(protocol.read_text() + "\nstd::fs::read(path);\n", encoding="utf-8")
        failures = guard.check(root)
        self.assertTrue(any("kitty_pixels.rs" in failure for failure in failures))
        self.assertTrue(any("std::fs" in failure for failure in failures))

    def test_chunk_limit_and_cross_protocol_accounting_removal_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        graphics = root / guard.CRATE / "src/kitty_graphics.rs"
        graphics.write_text(graphics.read_text().replace("combined_length", "unchecked_length"))
        store = root / guard.CRATE / "src/kitty_store.rs"
        store.write_text(store.read_text().replace("other_frames", "ignored_frames"))
        failures = guard.check(root)
        self.assertTrue(any("combined_length" in failure for failure in failures))
        self.assertTrue(any("other_frames" in failure for failure in failures))

    def test_apc_lifecycle_and_proof_removal_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        control = root / guard.CRATE / "src/control_string.rs"
        control.write_text(control.read_text().replace("self.apply_kitty_graphics", "self.ignore_apc"))
        graphics = root / guard.CRATE / "src/graphics.rs"
        graphics.write_text(
            graphics.read_text().replace("fn prune_missing_kitty_parents(", "fn retain_orphans(")
        )
        proofs = root / guard.CRATE / "src/kitty_graphics_tests.rs"
        proofs.write_text(
            proofs.read_text().replace(
                "aggregate_frame_limit_is_atomic_across_sixel_and_kitty_graphics", "removed"
            )
        )
        failures = guard.check(root)
        self.assertTrue(any("APC input" in failure for failure in failures))
        self.assertTrue(any("prune_missing_kitty_parents" in failure for failure in failures))
        self.assertTrue(any("aggregate_frame_limit" in failure for failure in failures))


if __name__ == "__main__":
    unittest.main()
