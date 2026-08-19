#!/usr/bin/env python3
"""Hermetic mutation tests for the Datum TerminalCore boundary guard."""

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("check_terminal_core_boundary.py")
SPEC = importlib.util.spec_from_file_location("terminal_core_guard", MODULE_PATH)
assert SPEC and SPEC.loader
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class TerminalCoreBoundaryTest(unittest.TestCase):
    def fixture(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        root = Path(temporary.name)
        source = root / guard.CRATE / "src"
        source.mkdir(parents=True)
        (root / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["crates/terminal-core"]\n', encoding="utf-8"
        )
        (root / guard.CRATE / "Cargo.toml").write_text(
            '[package]\nname = "datum-terminal-core"\nversion = "0.1.0"\n'
            'edition = "2024"\n[dependencies]\n',
            encoding="utf-8",
        )
        for relative, markers in guard.REQUIRED_MODULES.items():
            text = "\n".join(markers)
            if relative == "limits.rs":
                text += "\n" + "\n".join(guard.REQUIRED_LIMITS)
            (source / relative).write_text(text, encoding="utf-8")
        (source / "lib.rs").write_text(
            "pub use screen::{ScreenState, TerminalCore};\n", encoding="utf-8"
        )
        return temporary, root

    def test_valid_std_only_foundation_passes(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        self.assertEqual(guard.check(root), [])

    def test_external_dependency_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        manifest = root / guard.CRATE / "Cargo.toml"
        manifest.write_text(manifest.read_text() + 'serde = "1"\n', encoding="utf-8")
        self.assertTrue(any("std-only" in item for item in guard.check(root)))

    def test_renderer_or_process_authority_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        screen = root / guard.CRATE / "src/screen.rs"
        screen.write_text(screen.read_text() + "\nuse wgpu::Texture;\nstd::process::Command;\n")
        failures = guard.check(root)
        self.assertTrue(any("wgpu" in item for item in failures))
        self.assertTrue(any("std::process" in item for item in failures))

    def test_missing_limit_family_and_numeric_default_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        limits = root / guard.CRATE / "src/limits.rs"
        text = limits.read_text().replace("SnapshotCells", "")
        limits.write_text(text + "\nimpl Default for CoreLimits {}\n")
        failures = guard.check(root)
        self.assertTrue(any("SnapshotCells" in item for item in failures))
        self.assertTrue(any("owner-supplied" in item for item in failures))

    def test_early_parser_or_missing_core_authority_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        screen = root / guard.CRATE / "src/screen.rs"
        screen.write_text("pub fn feed_pty() {}\n")
        failures = guard.check(root)
        self.assertTrue(any("owned marker" in item for item in failures))
        self.assertTrue(any("DTC-P08" in item for item in failures))


if __name__ == "__main__":
    unittest.main()
