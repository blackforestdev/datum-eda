#!/usr/bin/env python3
"""Hermetic mutations for the DTC-P25 shadow-comparison guard."""

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("check_terminal_shadow_boundary.py")
SPEC = importlib.util.spec_from_file_location("terminal_shadow_guard", MODULE_PATH)
assert SPEC and SPEC.loader
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class TerminalShadowBoundaryTest(unittest.TestCase):
    def fixture(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        root = Path(temporary.name)
        source = root / guard.SRC
        source.mkdir(parents=True)
        (root / guard.ADAPTER).write_text(
            '#[cfg(test)]\n#[path = "terminal_shadow.rs"]\nmod shadow;\n',
            encoding="utf-8",
        )
        shadow = "\n".join(
            [
                *guard.PINNED_BOUNDS,
                "struct DeclaredOverlap;",
                "fn replay(){ TerminalScreen::default(); TerminalCoreSessionAdapter::new(); }",
                "fn compare(){",
                "assert_recorded_boundaries_match(recording);",
                'assert_recording_matches(recording, "whole", chunks);',
                'assert_recording_matches(recording, "recorded PTY chunks", chunks);',
                'assert_recording_matches(recording, "bytewise", chunks);',
                'assert_recording_matches(recording, "seeded irregular", chunks);',
                "}",
                *(f"fn {proof}() {{}}" for proof in guard.REQUIRED_PROOFS),
            ]
        )
        (root / guard.SHADOW).write_text(shadow, encoding="utf-8")
        for relative in guard.PRODUCTION_OWNERS:
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("fn production() {}\n", encoding="utf-8")
        return temporary, root

    def assert_mutation_fails(self, relative: Path, old: str, new: str) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        self.assertEqual(guard.check(root), [])
        path = root / relative
        path.write_text(path.read_text(encoding="utf-8").replace(old, new), encoding="utf-8")
        self.assertTrue(guard.check(root))

    def test_valid_test_only_shadow_passes(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        self.assertEqual(guard.check(root), [])

    def test_cfg_broadening_and_production_import_fail(self) -> None:
        self.assert_mutation_fails(guard.ADAPTER, "cfg(test)", "cfg(debug_assertions)")
        self.assert_mutation_fails(
            guard.PRODUCTION_OWNERS[2],
            "fn production() {}",
            "use crate::terminal_shadow; fn production() {}",
        )

    def test_missing_comparison_path_or_normative_proof_fails(self) -> None:
        self.assert_mutation_fails(
            guard.SHADOW,
            "assert_recorded_boundaries_match(recording);",
            "",
        )
        self.assert_mutation_fails(
            guard.SHADOW,
            "fn dtc_p25_non_overlap_uses_terminal_core_normative_unicode_and_link_proof() {}",
            "",
        )

    def test_unbounded_replay_or_external_io_fails(self) -> None:
        self.assert_mutation_fails(
            guard.SHADOW,
            "const MAX_SHADOW_REPLAY_CHUNKS: usize = 4_096;",
            "const MAX_SHADOW_REPLAY_CHUNKS: usize = usize::MAX;",
        )
        self.assert_mutation_fails(
            guard.SHADOW,
            "fn replay(){",
            "fn replay(){ std::process::Command::new(\"sh\");",
        )


if __name__ == "__main__":
    unittest.main()
