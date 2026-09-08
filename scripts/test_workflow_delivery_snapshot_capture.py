"""Observe real staged, worktree and candidate isolation without shared edits."""

import json
import os
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_cases import run_case
from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_capture_hook import run_hook_case
from workflow_delivery_capture_snapshots import POLICY, SNAPSHOT_CASES, UNTRACKED_CASES
from workflow_delivery_capture_state import git
from workflow_delivery_io import sha256


class SnapshotCaptureTest(unittest.TestCase):
    def test_each_surface_judges_its_own_snapshot_and_preserves_the_others(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet = store / "packet"
            prepare_fixture(store / "baseline", packet)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            runtime = Path(__file__).resolve().parents[1]
            identities = set()
            for case_id in SNAPSHOT_CASES.keys() - UNTRACKED_CASES:
                surfaces = ("R",) if "candidate-isolation" in case_id else ("C", "H", "S-check", "S-details")
                for surface in surfaces:
                    with self.subTest(case=case_id, surface=surface):
                        destination = store / (case_id + surface)
                        if surface == "H":
                            result = run_hook_case(packet, destination, environment, case_id=case_id)
                        else:
                            result = run_case(packet, destination, runtime, environment,
                                              case_id=case_id, surface=surface)
                        root = destination / "fixture"
                        mutation = json.loads((destination / "mutation.json").read_bytes())
                        staged = git(root, "show", ":" + POLICY)
                        worktree = (root / POLICY).read_bytes()
                        committed = git(root, "show", mutation["head"] + ":" + POLICY)
                        self.assertEqual(staged.hex(), mutation["index_bytes_hex"])
                        self.assertEqual(worktree.hex(), mutation["worktree_bytes_hex"])
                        self.assertEqual(sha256(committed), mutation["head_bytes_sha256"])
                        self.assertNotEqual(staged, worktree)
                        if surface == "R":
                            self.assertNotIn(committed, (staged, worktree))
                        self.assertTrue(result["observation_matches_expected"])
                        self.assertFalse(result["complete_matrix"])
                        self.assertFalse(result["acceptance_asserted"])
                        self.assertNotIn(result["invocation_id"], identities)
                        identities.add(result["invocation_id"])
                        recovery_path = store / (case_id + surface + "-recovery")
                        if surface == "H":
                            recovered = run_hook_case(packet, recovery_path, environment, case_id="INFRA-S01-01")
                        else:
                            recovered = run_case(packet, recovery_path, runtime, environment,
                                                 case_id="INFRA-S01-01", surface=surface)
                        self.assertTrue(recovered["observation_matches_expected"])
                        self.assertNotIn(recovered["invocation_id"], identities)
                        identities.add(recovered["invocation_id"])
            self.assertEqual(18, len(identities))

    def test_wrong_snapshot_surface_is_rejected_before_fixture_creation(self):
        with self.assertRaisesRegex(ValueError, "divergence requires"):
            run_case(None, None, None, {}, case_id="INFRA-S05-01.staged-bad", surface="R")
        with self.assertRaisesRegex(ValueError, "isolation requires surface R"):
            run_hook_case(None, None, {}, case_id="INFRA-S05-03.candidate-isolation")


if __name__ == "__main__":
    unittest.main()
