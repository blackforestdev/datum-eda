#!/usr/bin/env python3
"""Hermetic mutations for the DTC-P21 integrated proof guard."""

from __future__ import annotations

import importlib.util
import shutil
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("check_terminal_core_proof.py")
SPEC = importlib.util.spec_from_file_location("terminal_core_proof_guard", MODULE_PATH)
assert SPEC and SPEC.loader
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class TerminalCoreProofTest(unittest.TestCase):
    def fixture(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        root = Path(temporary.name)
        crate = root / guard.CRATE
        (crate / "src").mkdir(parents=True)
        (crate / "examples").mkdir()
        (root / "scripts").mkdir()
        for relative in (
            "Cargo.toml",
            "src/lib.rs",
            "src/proof_tests.rs",
            "examples/dtc_p21_probe.rs",
        ):
            source = guard.ROOT / guard.CRATE / relative
            target = crate / relative
            shutil.copy2(source, target)
        for name in ("run_terminal_core_proof.py", "run_drift_gates.sh"):
            shutil.copy2(guard.ROOT / "scripts" / name, root / "scripts" / name)
        return temporary, root

    def test_valid_integrated_proof_passes(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        self.assertEqual(guard.check(root), [])

    def test_corpus_chunk_and_shrinker_removal_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        proof = root / guard.CRATE / "src/proof_tests.rs"
        proof.write_text(
            proof.read_text(encoding="utf-8")
            .replace("DTC-R08-GRAPHICS-001", "removed")
            .replace("report.consumed > 0", "true")
            .replace("minimize_replay", "removed_shrinker"),
            encoding="utf-8",
        )
        failures = guard.check(root)
        self.assertTrue(any("DTC-R08" in failure for failure in failures))
        self.assertTrue(any("report.consumed" in failure for failure in failures))
        self.assertTrue(any("minimize_replay" in failure for failure in failures))

    def test_external_authority_and_dependency_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        proof = root / guard.CRATE / "src/proof_tests.rs"
        proof.write_text(proof.read_text(encoding="utf-8") + "\nuse std::fs;\n", encoding="utf-8")
        manifest = root / guard.CRATE / "Cargo.toml"
        manifest.write_text(manifest.read_text(encoding="utf-8") + "serde = \"1\"\n", encoding="utf-8")
        failures = guard.check(root)
        self.assertTrue(any("std::fs" in failure for failure in failures))
        self.assertTrue(any("code dependency" in failure for failure in failures))

    def test_measurement_threshold_and_drift_disconnection_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        probe = root / guard.CRATE / "examples/dtc_p21_probe.rs"
        probe.write_text(
            probe.read_text(encoding="utf-8") + "\n// assert!(mib_per_second >= 20.0);\n",
            encoding="utf-8",
        )
        drift = root / "scripts/run_drift_gates.sh"
        drift.write_text(
            drift.read_text(encoding="utf-8").replace(
                "python3 scripts/run_terminal_core_proof.py", "# measurement removed"
            ),
            encoding="utf-8",
        )
        failures = guard.check(root)
        self.assertTrue(any("performance budget" in failure for failure in failures))
        self.assertTrue(any("run_terminal_core_proof.py" in failure for failure in failures))


if __name__ == "__main__":
    unittest.main()
