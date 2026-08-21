#!/usr/bin/env python3
"""Hermetic mutation tests for the DTC-P28 compatibility matrix gate."""

from __future__ import annotations

import importlib.util
import json
import pathlib
import shutil
import tempfile
import unittest

SCRIPT = pathlib.Path(__file__).with_name("check_terminal_compatibility_matrix.py")
SPEC = importlib.util.spec_from_file_location("terminal_compatibility_guard", SCRIPT)
assert SPEC and SPEC.loader
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class CompatibilityMatrixGuardTests(unittest.TestCase):
    def fixture(self) -> pathlib.Path:
        root = pathlib.Path(tempfile.mkdtemp(prefix="datum-p28-matrix-"))
        for source in (guard.MATRIX, guard.TEST):
            target = root / source.relative_to(guard.ROOT)
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, target)
        matrix_path = root / guard.MATRIX.relative_to(guard.ROOT)
        matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
        artifact = pathlib.Path(matrix["result_artifact"])
        (root / artifact).parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(guard.ROOT / artifact, root / artifact)
        self.addCleanup(shutil.rmtree, root)
        return root

    def test_valid_matrix_and_production_witness_pass(self) -> None:
        self.assertEqual(guard.failures(self.fixture()), [])

    def test_missing_named_program_fails(self) -> None:
        root = self.fixture()
        path = root / guard.MATRIX.relative_to(guard.ROOT)
        payload = json.loads(path.read_text(encoding="utf-8"))
        payload["external_witnesses"] = [
            entry for entry in payload["external_witnesses"] if entry["name"] != "tmux"
        ]
        path.write_text(json.dumps(payload), encoding="utf-8")
        self.assertTrue(any("tmux" in problem for problem in guard.failures(root)))

    def test_bypassing_production_adapter_fails(self) -> None:
        root = self.fixture()
        path = root / guard.TEST.relative_to(guard.ROOT)
        text = path.read_text(encoding="utf-8").replace(
            "TerminalCoreSessionAdapter::new_with_profile",
            "fake_adapter",
        )
        path.write_text(text, encoding="utf-8")
        self.assertTrue(any("TerminalCoreSessionAdapter" in problem for problem in guard.failures(root)))

    def test_external_suite_cannot_be_claimed_as_normative_without_results(self) -> None:
        root = self.fixture()
        path = root / guard.MATRIX.relative_to(guard.ROOT)
        payload = json.loads(path.read_text(encoding="utf-8"))
        payload["optional_external_conformance"][0]["status"] = "passed"
        path.write_text(json.dumps(payload), encoding="utf-8")
        self.assertTrue(any("vttest" in problem for problem in guard.failures(root)))


if __name__ == "__main__":
    unittest.main()
