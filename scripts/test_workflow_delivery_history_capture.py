"""Actual candidate CLI refuses net-zero history; current snapshots are distinct."""

import json
import os
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_cases import run_case
from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_capture_history import BASE_CASES, CURRENT_HISTORY_CASE, HISTORY_CASES
from workflow_delivery_capture_hook import run_hook_case
from workflow_delivery_capture_state import git
from workflow_delivery_publication_delta import verify_publication_delta
from workflow_delivery_io import sha256


class HistoryCaptureTest(unittest.TestCase):
    def test_each_net_zero_history_refuses_and_fresh_candidate_recovers(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet = store / "packet"
            prepare_fixture(store / "baseline", packet)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            runtime = Path(__file__).resolve().parents[1]
            identities = set()
            for case_id, (target, _) in HISTORY_CASES.items():
                with self.subTest(case=case_id):
                    destination = store / case_id
                    result = run_case(packet, destination, runtime, environment,
                                      case_id=case_id, surface="R")
                    mutation = json.loads((destination / "mutation.json").read_bytes())
                    delta = mutation["publication_delta"]
                    root = destination / "fixture"
                    verify_publication_delta(root, delta, base=delta["base"], candidate=delta["candidate"])
                    archive = destination / mutation["history_bundle"]["path"]
                    self.assertEqual(sha256(archive.read_bytes()), mutation["history_bundle"]["sha256"])
                    restored = destination / "restored-history"
                    git(root, "clone", "--quiet", str(archive), str(restored))
                    verify_publication_delta(restored, delta, base=delta["base"], candidate=delta["candidate"])
                    self.assertEqual([], delta["net_changes"])
                    self.assertEqual([target], delta["touched_paths"])
                    self.assertEqual(b"", git(root, "status", "--porcelain"))
                    self.assertEqual(4 if "merge-parent" in case_id else 2, len(delta["commits"]))
                    if "merge-parent" in case_id:
                        self.assertEqual(2, len(delta["commits"][-1]["parents"]))
                    recovery = run_case(packet, store / (case_id + "-recovery"), runtime,
                                        environment, case_id="INFRA-S01-01", surface="R")
                    for observed in (result, recovery):
                        self.assertTrue(observed["observation_matches_expected"])
                        self.assertFalse(observed["complete_matrix"])
                        self.assertFalse(observed["acceptance_asserted"])
                        self.assertNotIn(observed["invocation_id"], identities)
                        identities.add(observed["invocation_id"])
            self.assertEqual(6, len(identities))

    def test_history_cases_cannot_be_relabelled_as_current_snapshot_proof(self):
        for case_id in HISTORY_CASES | BASE_CASES:
            for surface in ("C", "S-check", "S-details"):
                with self.assertRaisesRegex(ValueError, "surface R"):
                    run_case(None, None, None, {}, case_id=case_id, surface=surface)
            with self.assertRaisesRegex(ValueError, "explicit implemented hook"):
                run_hook_case(None, None, {}, case_id=case_id)

    def test_missing_and_nonancestor_bases_refuse_and_recover(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet = store / "packet"
            recipe = prepare_fixture(store / "baseline", packet)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            identities = set()
            for case in BASE_CASES:
                destination = store / case
                result = run_case(packet, destination, Path(__file__).resolve().parents[1],
                                  environment, case_id=case, surface="R")
                mutation = json.loads((destination / "mutation.json").read_bytes())
                self.assertEqual(recipe["head"], mutation["candidate_ref"])
                self.assertNotEqual(recipe["head"], mutation["base_ref"])
                archive = destination / mutation["history_bundle"]["path"]
                self.assertEqual(sha256(archive.read_bytes()), mutation["history_bundle"]["sha256"])
                if case.endswith(".nonancestor-base"):
                    root = destination / "fixture"
                    self.assertEqual(recipe["head"], git(root, "merge-base", mutation["base_ref"], recipe["head"]).decode().strip())
                recovery = run_case(packet, store / (case + "-recovery"), Path(__file__).resolve().parents[1],
                                    environment, case_id="INFRA-S01-01", surface="R")
                for observed in (result, recovery):
                    self.assertTrue(observed["observation_matches_expected"])
                    self.assertFalse(observed["complete_matrix"])
                    self.assertFalse(observed["acceptance_asserted"])
                    self.assertNotIn(observed["invocation_id"], identities)
                    identities.add(observed["invocation_id"])
            self.assertEqual(4, len(identities))

    def test_empty_current_transaction_does_not_certify_prior_history(self):
        with self.assertRaisesRegex(ValueError, "not history certification"):
            run_case(None, None, None, {}, case_id=CURRENT_HISTORY_CASE, surface="R")
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet = store / "packet"
            recipe = prepare_fixture(store / "baseline", packet)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            identities = set()
            for surface in ("C", "S-check", "S-details", "H"):
                destination = store / surface
                result = (run_hook_case(packet, destination, environment, case_id=CURRENT_HISTORY_CASE)
                          if surface == "H" else run_case(packet, destination, Path(__file__).resolve().parents[1],
                              environment, case_id=CURRENT_HISTORY_CASE, surface=surface))
                self.assertTrue(result["observation_matches_expected"])
                self.assertFalse(result["acceptance_asserted"])
                self.assertFalse(result["complete_matrix"])
                mutation = json.loads((destination / "mutation.json").read_bytes())
                delta = mutation["publication_delta"]
                self.assertFalse(mutation["history_certified"])
                self.assertEqual([], delta["net_changes"])
                self.assertEqual(["src/unscoped.py"], delta["touched_paths"])
                before = json.loads((destination / "capture/before.json").read_bytes())
                self.assertEqual(delta["candidate"], before["head"])
                self.assertNotEqual(recipe["head"], before["head"])
                self.assertEqual([recipe["head"]], before["local_trust"]["datum.workflowDeliveryBaseRef"])
                self.assertEqual(b"", git(destination / "fixture", "status", "--porcelain"))
                self.assertNotIn(result["invocation_id"], identities)
                identities.add(result["invocation_id"])
            self.assertEqual(4, len(identities))


if __name__ == "__main__":
    unittest.main()
