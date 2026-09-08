"""Pre-publication refusal tests on isolated owned repositories only."""

import os
import unittest
from unittest.mock import patch

from workflow_delivery_activation_preflight import preflight
from workflow_delivery_capture_state import protected_state
from workflow_delivery_publication_delta import publication_delta
from workflow_delivery_test_support import Fixture


class ActivationPreflightTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.base = self.f.head
        tree = self.f.git("rev-parse", "HEAD^{tree}").decode().strip()
        self.candidate = self.f.git("commit-tree", tree, "-p", self.base,
                                   "-m", "synthetic candidate").decode().strip()
        self.trust = protected_state(self.f.root, ["src"])["local_trust"]

    def run_preflight(self, **overrides):
        options = dict(base=self.base, candidate=self.candidate, input_roots=["src"],
                       expected_local_trust=self.trust)
        options.update(overrides)
        return preflight(self.f.root, **options)

    def test_clean_preparation_is_nonmutating_and_not_permission(self):
        before = self.f.snapshot()
        result = self.run_preflight()
        self.assertFalse(result["activation_asserted"])
        self.assertFalse(result["publication_authorized"])
        self.assertEqual(before, self.f.snapshot())

    def test_stale_base_and_prior_trust_refuse(self):
        with self.assertRaisesRegex(ValueError, "HEAD differs"):
            self.run_preflight(base=self.candidate)
        self.f.git("config", "--local", "datum.workflowDeliveryAuthorityRef", self.base)
        with self.assertRaisesRegex(ValueError, "local trust differs"):
            self.run_preflight()

    def test_dirty_tracked_index_and_worktree_refuse(self):
        self.f.write("src/read.py", b"uncommitted\n")
        with self.assertRaisesRegex(ValueError, "dirty"):
            self.run_preflight()
        self.f.stage()
        with self.assertRaisesRegex(ValueError, "dirty"):
            self.run_preflight()

    def test_untracked_file_outside_reviewed_roots_still_refuses(self):
        self.f.write("unexpected-note.txt", b"untracked\n")
        with self.assertRaisesRegex(ValueError, "dirty"):
            self.run_preflight()

    def test_filemode_setting_cannot_hide_executable_change(self):
        self.f.git("config", "core.filemode", "false")
        (self.f.root / "src/read.py").chmod(0o755)
        with self.assertRaisesRegex(ValueError, "executable mode changed"):
            self.run_preflight()

    def test_assume_unchanged_cannot_hide_tracked_byte_changes(self):
        self.f.git("update-index", "--assume-unchanged", "src/read.py")
        self.f.write("src/read.py", b"hidden change\n")
        with self.assertRaisesRegex(ValueError, "tracked bytes differ"):
            self.run_preflight()

    def test_ignored_source_input_refuses(self):
        self.f.write(".git/info/exclude", b"src/ignored.py\n")
        self.f.write("src/ignored.py", b"ignored build input\n")
        with self.assertRaisesRegex(ValueError, "ignored or missing"):
            self.run_preflight()

    def test_missing_reviewed_input_and_alternate_index_refuse(self):
        with self.assertRaisesRegex(ValueError, "missing reviewed input"):
            self.run_preflight(input_roots=["src", "absent-input"])
        with patch.dict(os.environ, {"GIT_INDEX_FILE": "/nonexistent-alternate-index"}):
            with self.assertRaisesRegex(ValueError, "overrides"):
                self.run_preflight()

    def test_changes_during_preparation_are_not_silently_accepted(self):
        def changed(*args, **kwargs):
            result = publication_delta(*args, **kwargs)
            self.f.write("src/read.py", b"concurrent input\n")
            return result
        with patch("workflow_delivery_activation_preflight.publication_delta", side_effect=changed):
            with self.assertRaisesRegex(ValueError, "state changed"):
                self.run_preflight()


if __name__ == "__main__":
    unittest.main()
