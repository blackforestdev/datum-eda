"""Real selector sees unstaged/ignored production inputs; staged checks exclude them."""

import json
import os
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_cases import run_case
from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_capture_hook import run_hook_case
from workflow_delivery_capture_snapshots import UNTRACKED_CASES
from workflow_delivery_capture_state import git


class UntrackedCaptureTest(unittest.TestCase):
    def test_ignored_and_untracked_inputs_are_judged_in_the_requested_snapshot(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet = store / "packet"
            prepare_fixture(store / "baseline", packet)
            runtime = Path(__file__).resolve().parents[1]
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            identities = set()
            for case_id in sorted(UNTRACKED_CASES):
                for surface in ("C", "H", "S-check", "S-details"):
                    with self.subTest(case=case_id, surface=surface):
                        for phase, case in (("input", case_id), ("recovery", "INFRA-S01-01")):
                            destination = store / (case_id + surface + phase)
                            if surface == "H":
                                result = run_hook_case(packet, destination, environment, case_id=case)
                            else:
                                result = run_case(packet, destination, runtime, environment, case_id=case, surface=surface)
                            self.assertTrue(result["observation_matches_expected"])
                            self.assertFalse(result["complete_matrix"])
                            self.assertFalse(result["acceptance_asserted"])
                            self.assertNotIn(result["invocation_id"], identities)
                            identities.add(result["invocation_id"])
                            if phase == "input":
                                root = destination / "fixture"
                                mutation = json.loads((destination / "mutation.json").read_bytes())
                                target = mutation["path"]
                                self.assertEqual(b"", git(root, "ls-files", "--", target))
                                self.assertEqual(b"", git(root, "diff", "--cached", "--name-only"))
                                self.assertEqual(mutation["worktree_bytes_hex"], (root / target).read_bytes().hex())
                                ignored = git(root, "check-ignore", "--", target, allowed=(0, 1))
                                self.assertEqual(case_id.endswith(".ignored"), bool(ignored))
                                self.assertEqual(mutation["exclude_after_hex"], (root / ".git/info/exclude").read_bytes().hex())
                                before = json.loads((destination / "capture/before.json").read_bytes())
                                after = json.loads((destination / "capture/after.json").read_bytes())
                                self.assertEqual(before["git_info_exclude"], after["git_info_exclude"])
            self.assertEqual(16, len(identities))


if __name__ == "__main__":
    unittest.main()
