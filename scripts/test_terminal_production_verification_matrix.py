#!/usr/bin/env python3
"""Hermetic mutation tests for the T4V-01 production matrix."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("check_terminal_production_verification_matrix.py")
SPEC = importlib.util.spec_from_file_location("terminal_production_matrix_guard", SCRIPT)
assert SPEC and SPEC.loader
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class TerminalProductionMatrixTests(unittest.TestCase):
    def fixture(self) -> Path:
        root = Path(tempfile.mkdtemp(prefix="datum-t4v-matrix-"))
        matrix_target = root / guard.MATRIX.relative_to(guard.ROOT)
        matrix_target.parent.mkdir(parents=True)
        shutil.copy2(guard.MATRIX, matrix_target)
        matrix = json.loads(matrix_target.read_text(encoding="utf-8"))
        for entry in matrix["checked_in_evidence"]:
            source = guard.ROOT / entry["path"]
            target = root / entry["path"]
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, target)
        self.addCleanup(shutil.rmtree, root)
        return root

    def mutate(self, root: Path, callback) -> None:
        path = root / guard.MATRIX.relative_to(guard.ROOT)
        matrix = json.loads(path.read_text(encoding="utf-8"))
        callback(matrix)
        path.write_text(json.dumps(matrix), encoding="utf-8")

    def test_complete_matrix_passes(self) -> None:
        self.assertEqual(guard.failures(self.fixture()), [])

    def test_removing_a_completed_package_fails(self) -> None:
        root = self.fixture()
        self.mutate(root, lambda matrix: matrix["completed_packages"].pop())
        self.assertTrue(any("package inventory" in problem for problem in guard.failures(root)))

    def test_removing_a_live_run_fails(self) -> None:
        root = self.fixture()
        self.mutate(root, lambda matrix: matrix["t4v02_runs"].pop())
        self.assertTrue(any("run inventory" in problem for problem in guard.failures(root)))

    def test_erasing_exact_command_fails(self) -> None:
        root = self.fixture()
        self.mutate(root, lambda matrix: matrix["t4v02_runs"][0].update(command=""))
        self.assertTrue(any("exact command" in problem for problem in guard.failures(root)))

    def test_owner_acceptance_cannot_disappear(self) -> None:
        root = self.fixture()
        self.mutate(root, lambda matrix: matrix.pop("owner_acceptance"))
        self.assertTrue(any("hands-on checklist" in problem for problem in guard.failures(root)))


if __name__ == "__main__":
    unittest.main()
