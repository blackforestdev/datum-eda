"""Actual alternate-index selection without authority or worktree fallback."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_test_support import Fixture
from workflow_delivery_tree import Tree


class IndexEnvironmentTest(unittest.TestCase):
    def test_selected_index_is_captured_and_normal_index_is_preserved(self):
        f = Fixture()
        self.addCleanup(f.close)
        original = Tree(f.root, staged=True).read("src/read.py")
        index_before = (f.root / ".git/index").read_bytes()
        alternate = f.root / ".git/alternate-index"
        f.write(".git/alternate-index", index_before)
        changed = b"changed alternate input\n"
        f.write("src/read.py", changed)
        environment = dict(os.environ, GIT_INDEX_FILE=str(alternate))
        subprocess.run(["git", "add", "--", "src/read.py"], cwd=f.root, env=environment, check=True)
        for selected_path in (str(alternate), ".git/alternate-index"):
            with patch.dict(os.environ, {"GIT_INDEX_FILE": selected_path}):
                captured = Tree(f.root, staged=True)
                self.assertEqual(changed, captured.read("src/read.py"))
                self.assertEqual(original, Tree(f.root, revision=f.head).read("src/read.py"))
        alternate.write_bytes(index_before)
        self.assertEqual(changed, captured.read("src/read.py"))
        self.assertEqual(original, Tree(f.root, staged=True).read("src/read.py"))
        self.assertEqual(index_before, (f.root / ".git/index").read_bytes())

    def test_empty_or_missing_index_never_falls_back_to_normal_entries(self):
        f = Fixture()
        self.addCleanup(f.close)
        with patch.dict(os.environ, {"GIT_INDEX_FILE": ""}):
            with self.assertRaisesRegex(ValueError, "WDQ-INDEX"):
                Tree(f.root, staged=True)
            self.assertTrue(Tree(f.root, revision=f.head).entries)
        with patch.dict(os.environ, {"GIT_INDEX_FILE": str(f.root / ".git/missing-index")}):
            self.assertEqual({}, Tree(f.root, staged=True).entries)
        self.assertTrue(Tree(f.root, staged=True).entries)

    def test_actual_cli_checks_alternate_index_not_the_clean_normal_index(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            root = store / "fixture"
            recipe = prepare_fixture(root, store / "packet")
            alternate = root / ".git/alternate-index"
            alternate.write_bytes((root / ".git/index").read_bytes())
            (root / "src/unscoped.py").write_text("# unpermitted alternate-index input\n")
            environment = {"PATH": os.defpath, "HOME": temporary, "GIT_CONFIG_NOSYSTEM": "1",
                           "GIT_CONFIG_GLOBAL": os.devnull, "PYTHONDONTWRITEBYTECODE": "1"}
            selected = dict(environment, GIT_INDEX_FILE=str(alternate))
            subprocess.run(["git", "add", "--", "src/unscoped.py"], cwd=root, env=selected, check=True)
            command = [sys.executable, "-B", str(Path(__file__).with_name("check_workflow_delivery.py")),
                "--root", str(root), "--enforce", "--staged", "--authority-ref", recipe["head"],
                "--base-ref", recipe["head"], "--environment-path", "requested-environment.json"]
            normal = subprocess.run(command, env=environment, capture_output=True, timeout=30)
            self.assertEqual(0, normal.returncode, normal.stderr)
            refused = subprocess.run(command, env=selected, capture_output=True, timeout=30)
            self.assertEqual(1, refused.returncode, refused.stderr)
            finding = json.loads(refused.stdout)["findings"][0]
            self.assertEqual("WDQ-COVERAGE", finding["code"])
            self.assertEqual("src/unscoped.py", finding["path"])


if __name__ == "__main__":
    unittest.main()
