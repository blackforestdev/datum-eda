"""Workspace policy trust-boundary tests using real isolated Git snapshots."""

from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch

from workflow_delivery_test_support import Fixture
from workflow_delivery_tree import Tree
from workflow_delivery_workspace_authority import WORKSPACE_POLICY_PATH, load_workspace


class WorkspaceAuthorityTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.policy = {"schema_version": 1, "kind": "datum.workflow-delivery.workspace-inputs",
                       "local_files": [], "python_caches": "none", "beads_runtime": "standard-v1"}
        self.authority = Tree(self.f.root, revision=self.f.head)

    def promote_fixture(self):
        self.f.save(WORKSPACE_POLICY_PATH, self.policy)
        self.f.git("add", WORKSPACE_POLICY_PATH)
        self.f.git("commit", "-qm", "fixture workspace policy")
        self.authority = Tree(self.f.root, revision="HEAD")

    def load(self, candidate=None, **kwargs):
        return load_workspace(candidate or Tree(self.f.root), self.authority,
                              policy_schema=kwargs.get("policy_schema", 2))

    def test_absent_policy_preserves_strict_legacy_behavior(self):
        self.assertIsNone(self.load())

    def test_candidate_only_policy_refuses_untracked_staged_and_committed(self):
        self.f.save(WORKSPACE_POLICY_PATH, self.policy)
        with self.assertRaisesRegex(ValueError, "candidate-only"):
            self.load()
        self.f.git("add", WORKSPACE_POLICY_PATH)
        with self.assertRaisesRegex(ValueError, "candidate-only"):
            self.load(Tree(self.f.root, staged=True))
        self.f.git("commit", "-qm", "fixture unauthorized policy")
        with self.assertRaisesRegex(ValueError, "candidate-only"):
            self.load(Tree(self.f.root, revision="HEAD"))

    def test_exact_promoted_policy_loads_without_mutation(self):
        self.promote_fixture()
        before = self.f.snapshot()
        workspace = self.load()
        self.assertEqual(self.policy, workspace.policy)
        self.assertEqual(before, self.f.snapshot())
        with self.assertRaisesRegex(ValueError, "schema-2"):
            self.load(policy_schema=1)

    def test_candidate_policy_change_cannot_expand_exemptions(self):
        self.promote_fixture()
        self.policy["python_caches"] = "verified-source-v1"
        self.f.save(WORKSPACE_POLICY_PATH, self.policy)
        with self.assertRaisesRegex(ValueError, "requires owner promotion"):
            self.load()

    def test_selected_index_never_borrows_repaired_worktree_policy(self):
        self.promote_fixture()
        original = (self.f.root / WORKSPACE_POLICY_PATH).read_bytes()
        self.policy["python_caches"] = "verified-source-v1"
        self.f.save(WORKSPACE_POLICY_PATH, self.policy)
        self.f.git("add", WORKSPACE_POLICY_PATH)
        selected = Tree(self.f.root, staged=True)
        self.f.write(WORKSPACE_POLICY_PATH, original)
        with self.assertRaisesRegex(ValueError, "requires owner promotion"):
            self.load(selected)

    def test_removal_symlink_and_executable_policy_refuse(self):
        self.promote_fixture()
        path = self.f.root / WORKSPACE_POLICY_PATH
        original = path.read_bytes()
        path.chmod(0o755)
        with self.assertRaisesRegex(ValueError, "executable mode"):
            self.load()
        path.unlink()
        with self.assertRaisesRegex(ValueError, "worktree redirect or executable mode"):
            self.load()
        self.f.write("same-policy.json", original)
        path.symlink_to(self.f.root / "same-policy.json")
        with self.assertRaisesRegex(ValueError, "redirect"):
            self.load()
        self.f.git("rm", "-f", WORKSPACE_POLICY_PATH)
        with self.assertRaisesRegex(ValueError, "remain tracked"):
            self.load(Tree(self.f.root, staged=True))

    def test_cache_policy_refuses_without_observed_source_only_import(self):
        self.policy["python_caches"] = "verified-source-v1"
        self.promote_fixture()
        with patch("workflow_delivery_workspace_authority.source_only_import", return_value=False):
            with self.assertRaisesRegex(ValueError, "verified source-only startup"):
                self.load()

    def test_fresh_process_observes_loader_not_a_supplied_boolean(self):
        scripts = Path(__file__).resolve().parent
        prefix = f"import sys; sys.path.insert(0, {str(scripts)!r}); "
        probe = "import workflow_delivery_workspace_authority as w; print(w.source_only_import())"
        plain = subprocess.run([sys.executable, "-I", "-B", "-c", prefix + probe],
                               capture_output=True, text=True, check=True)
        self.assertEqual("False\n", plain.stdout)
        bootstrap = scripts / "workflow_delivery_source_only.py"
        setup = (f"p={str(bootstrap)!r}; n={{'__name__':'_test_bootstrap'}}; "
                 "exec(compile(open(p, 'rb').read(), p, 'exec'), n); "
                 f"n['install']({str(scripts)!r}); ")
        isolated = subprocess.run([sys.executable, "-I", "-B", "-c", prefix + setup + probe],
                                  capture_output=True, text=True, check=True)
        self.assertEqual("True\n", isolated.stdout)


if __name__ == "__main__":
    unittest.main()
