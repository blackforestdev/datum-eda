"""Real Git consumer tests; not installed activation or product acceptance."""

import hashlib
import importlib.util
import py_compile
import stat
import unittest

from workflow_delivery_activation_preflight import clean_inputs, preflight
from workflow_delivery_capture_state import protected_state
from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_publication_delta import publication_delta
from workflow_delivery_test_support import Fixture
from workflow_delivery_transaction import changed_production_paths
from workflow_delivery_tree import Tree
from workflow_delivery_workspace import WorkspaceInputs


class WorkspaceConsumersTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.f.git("branch", "-M", "main")
        self.tree = Tree(self.f.root)
        self.policy = {"schema_version": 1, "kind": "datum.workflow-delivery.workspace-inputs",
                       "local_files": [], "python_caches": "verified-source-v1",
                       "beads_runtime": "standard-v1"}
        self.f.write(".git/info/exclude", b"src/__pycache__/\nsrc/local.sh\n.beads/beads.db\n")
        self.f.write("src/local.sh", b"#!/bin/sh\nexit 0\n")
        local = self.f.root / "src/local.sh"
        local.chmod(0o775)
        self.policy["local_files"].append({"path": "src/local.sh", "category": "owner_local",
            "sha256": hashlib.sha256(local.read_bytes()).hexdigest(),
            "size": local.stat().st_size, "mode": stat.S_IMODE(local.stat().st_mode)})
        self.f.write(".beads/beads.db", b"fixture runtime")
        source = self.f.root / "src/read.py"
        py_compile.compile(str(source), doraise=True)
        self.cache = importlib.util.cache_from_source(str(source))
        self.workspace = WorkspaceInputs(self.f.root, self.policy, self.tree.entries, cache_isolated=True)

    def changes(self, tree=None, roots=None):
        return changed_production_paths(tree or self.tree, roots or ["src", ".beads"],
                                        base_ref=self.f.head, workspace=self.workspace)

    def inspect(self, **overrides):
        tree_id = self.f.git("rev-parse", "HEAD^{tree}").decode().strip()
        candidate = self.f.git("commit-tree", tree_id, "-p", self.f.head,
                               "-m", "fixture publication").decode().strip()
        delta = publication_delta(self.f.root, base=self.f.head, candidate=candidate)
        roots = ["src", ".beads"]
        options = dict(base=self.f.head, candidate=candidate, input_roots=roots,
            expected_local_trust=protected_state(self.f.root, roots)["local_trust"],
            publication_review={"delta_sha256": sha256(canonical_json(delta)), "paths": delta["touched_paths"]},
            workspace=self.workspace)
        options.update(overrides)
        return preflight(self.f.root, **options)

    def test_shared_classification_accepts_preserved_workspace_and_retains_capture(self):
        before = protected_state(self.f.root, ["src", ".beads"])
        self.assertEqual([], self.changes())
        self.assertEqual(Tree(self.f.root, revision=self.f.head).manifest(["src", ".beads"]),
                         self.tree.manifest(["src", ".beads"], workspace=self.workspace))
        result = self.inspect()
        self.assertEqual(before, result["protected_state"])
        self.assertIn(".beads/beads.db", result["protected_state"]["files"])
        self.assertIn("src/local.sh", result["protected_state"]["files"])
        self.assertFalse(result["publication_authorized"])
        self.assertEqual(before, protected_state(self.f.root, ["src", ".beads"]))

    def test_absent_policy_does_not_silently_enable_exemptions(self):
        self.assertIn("src/local.sh", changed_production_paths(self.tree, ["src"], base_ref=self.f.head))
        self.assertIn("src/local.sh", {row["path"] for row in self.tree.manifest(["src"])})
        with self.assertRaisesRegex(ValueError, "untracked, ignored"):
            self.inspect(workspace=None)

    def test_explicit_file_inputs_are_never_filtered(self):
        self.assertEqual(["src/local.sh"], self.changes(roots=["src/local.sh"]))
        self.assertEqual(["src/local.sh"], [row["path"] for row in
            self.tree.manifest(["src/local.sh"], workspace=self.workspace)])
        with self.assertRaisesRegex(ValueError, "untracked, ignored"):
            self.inspect(input_roots=["src", ".beads", "src/local.sh"])

    def test_changed_owner_file_and_new_ignored_source_still_refuse(self):
        self.f.write("src/local.sh", b"#!/bin/sh\nexit 1\n")
        self.f.write("src/__pycache__/hidden.py", b"value = 1\n")
        self.assertEqual(["src/__pycache__/hidden.py", "src/local.sh"], self.changes())
        with self.assertRaisesRegex(ValueError, "untracked, ignored"):
            self.inspect()

    def test_tracked_and_index_changes_cannot_use_local_exemptions(self):
        self.f.write(".beads/issues.jsonl", b"changed canonical tracker\n")
        self.assertIn(".beads/issues.jsonl", self.changes())
        self.f.git("add", ".beads/issues.jsonl")
        self.assertEqual([".beads/issues.jsonl"], self.changes(Tree(self.f.root, staged=True)))
        with self.assertRaisesRegex(ValueError, "dirty"):
            self.inspect()

    def test_classifier_missing_tracked_paths_is_rejected_by_consumers(self):
        self.workspace = WorkspaceInputs(self.f.root, self.policy, [], cache_isolated=True)
        for call in (self.changes, lambda: self.tree.manifest(["src"], workspace=self.workspace), self.inspect):
            with self.assertRaisesRegex(ValueError, "every tracked input"):
                call()

    def test_mutating_runtime_during_preflight_still_refuses(self):
        from unittest.mock import patch

        def change_runtime(*args, **kwargs):
            result = publication_delta(*args, **kwargs)
            self.f.write(".beads/beads.db", b"concurrent writer")
            return result

        with patch("workflow_delivery_activation_preflight.publication_delta", side_effect=change_runtime):
            with self.assertRaisesRegex(ValueError, "state changed"):
                self.inspect()

    def test_captured_bad_local_file_cannot_borrow_live_good_identity(self):
        path = self.f.root / "src/local.sh"
        original = path.read_bytes()
        path.write_bytes(b"bad local file")
        captured = protected_state(self.f.root, ["src", ".beads"])
        path.write_bytes(original)
        with self.assertRaisesRegex(ValueError, "untracked, ignored"):
            clean_inputs(self.f.root, captured, self.f.head, workspace=self.workspace)

    def test_captured_forged_cache_cannot_borrow_live_derived_bytes(self):
        from pathlib import Path
        path = Path(self.cache)
        original = path.read_bytes()
        path.write_bytes(original[:16] + b"forged payload")
        captured = protected_state(self.f.root, ["src", ".beads"])
        path.write_bytes(original)
        with self.assertRaisesRegex(ValueError, "untracked, ignored"):
            clean_inputs(self.f.root, captured, self.f.head, workspace=self.workspace)


if __name__ == "__main__":
    unittest.main()
