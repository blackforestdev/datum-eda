"""Protected-state capture on owned synthetic repositories, not native proof."""

import os
import unittest
from unittest.mock import patch

from workflow_delivery_capture_state import protected_state
from workflow_delivery_test_support import Fixture


class CaptureStateTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)

    def capture(self):
        return protected_state(self.f.root, ["src"])

    def test_read_only_repeat_has_identical_files_index_refs_and_trust(self):
        before = self.f.snapshot()
        first = self.capture()
        self.assertEqual(first, self.capture())
        self.assertEqual(before, self.f.snapshot())
        self.assertIn(".beads/issues.jsonl", first["files"])
        self.assertEqual("file", first["index"]["kind"])

    def test_tracks_content_mode_deletion_and_relevant_untracked_files(self):
        before = self.capture()
        self.f.write("src/read.py", b"changed\n")
        os.chmod(self.f.root / "src/read.py", 0o755)
        self.f.write("src/new.py", b"untracked\n")
        self.f.write("unrelated.log", b"not a reviewed input\n")
        after = self.capture()
        self.assertNotEqual(before["files"]["src/read.py"], after["files"]["src/read.py"])
        self.assertEqual(0o755, after["files"]["src/read.py"]["mode"])
        self.assertIn("src/new.py", after["files"])
        self.assertNotIn("unrelated.log", after["files"])
        (self.f.root / "src/read.py").unlink()
        self.assertEqual({"kind": "missing"}, self.capture()["files"]["src/read.py"])

    def test_index_ref_and_selected_trust_changes_are_visible(self):
        before = self.capture()
        self.f.write("src/read.py", b"staged change\n")
        self.f.stage()
        self.f.git("update-ref", "refs/fixture/check", self.f.head)
        self.f.git("config", "datum.workflowDeliveryAuthorityRef", self.f.head)
        after = self.capture()
        for field in ("index", "index_entries_hex", "refs", "local_trust"):
            self.assertNotEqual(before[field], after[field], field)

    def test_unrelated_configuration_values_are_not_disclosed(self):
        self.f.git("config", "credential.helper", "fixture secret sentinel")
        self.assertNotIn("fixture secret sentinel", str(self.capture()))

    def test_local_exclusions_are_captured_without_hiding_production_input(self):
        before = self.capture()
        self.f.write(".git/info/exclude", b"/src/ignored.py\n")
        self.f.write("src/ignored.py", b"ignored production input\n")
        after = self.capture()
        self.assertNotEqual(before["git_info_exclude"], after["git_info_exclude"])
        self.assertEqual("file", after["files"]["src/ignored.py"]["kind"])

    def test_inherited_git_repository_overrides_are_not_used(self):
        before = self.capture()
        with patch.dict(os.environ, {"GIT_DIR": "/nonexistent-foreign-repository",
                                     "GIT_INDEX_FILE": "/nonexistent-foreign-index"}):
            self.assertEqual(before, self.capture())

    def test_symlinks_record_targets_without_reading_target_contents(self):
        (self.f.root / "src/link").symlink_to("/outside-fixture-missing")
        record = self.capture()["files"]["src/link"]
        self.assertEqual("symlink", record["kind"])
        self.assertNotIn("sha256", record)

    def test_escaping_or_metadata_roots_refuse(self):
        for root in ("../outside", "/absolute", ".git", "src/.git"):
            with self.assertRaises(ValueError):
                protected_state(self.f.root, [root])

    def test_symlink_ancestor_and_nested_repository_refuse(self):
        (self.f.root / "src/alias").symlink_to(self.f.root / "research", target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "directory symlink"):
            protected_state(self.f.root, ["src/alias/source.md"])
        self.f.write("src/nested/.git", b"fixture metadata\n")
        with self.assertRaisesRegex(ValueError, "nested Git"):
            self.capture()


if __name__ == "__main__":
    unittest.main()
