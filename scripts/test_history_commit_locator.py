"""Historical IDs remain evidence facts; only unique existing targets locate."""

import json
from pathlib import Path
import subprocess
import tempfile
import unittest

from history_commit_locator import locate_commit


class HistoryLocatorTests(unittest.TestCase):
    def test_mapping_requires_unique_valid_existing_target(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            def git(*args):
                return subprocess.check_output(["git", *args], cwd=root, stderr=subprocess.DEVNULL).decode().strip()
            git("init")
            git("-c", "user.name=Test", "-c", "user.email=test@example.invalid",
                "commit", "--allow-empty", "-m", "retained source")
            target = git("rev-parse", "HEAD")
            (root / "specs").mkdir()
            path = root / "specs/history_commit_map.json"
            old = "abcdef0" + "1" * 33
            def mapping(value):
                path.write_text(json.dumps(value))
            mapping({"schema_version": 1, "commits": {old: target}})
            self.assertEqual(locate_commit(root, old[:7]), target)
            self.assertEqual(locate_commit(root, target), target)
            mapping({"schema_version": 1, "commits": {old: target, "abcdef0" + "2" * 33: target}})
            self.assertIsNone(locate_commit(root, old[:7]))
            self.assertEqual(locate_commit(root, old), target)
            mapping({"schema_version": 1, "commits": {old: "3" * 40}})
            self.assertIsNone(locate_commit(root, old))
            for malformed in ([], {"schema_version": 2, "commits": {old: target}},
                              {"schema_version": 1, "commits": {old: "HEAD"}}):
                mapping(malformed)
                self.assertIsNone(locate_commit(root, old))
            self.assertIsNone(locate_commit(root, "HEAD~999"))


if __name__ == "__main__":
    unittest.main()
