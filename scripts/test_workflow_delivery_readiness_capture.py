"""Actual readiness refusals with synthetic input, not completed rollout readiness."""

import os
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_cases import run_case
from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_capture_hook import run_hook_case
from workflow_delivery_readiness_input import READINESS_CASES, READY_VALID


class ReadinessCaptureTest(unittest.TestCase):
    def test_readiness_cases_and_fresh_recovery_through_all_entrypoints(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            baseline = store / "baseline-packet"
            recipe = prepare_fixture(store / "baseline", baseline, fixture_variant=READY_VALID)
            self.assertEqual("accepted-history-and-readiness", recipe["fixture_family"])
            runtime = Path(__file__).resolve().parents[1]
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            identities = set()
            for variant in READINESS_CASES:
                packet = baseline
                if variant != READY_VALID:
                    packet = store / (variant + "-packet")
                    prepare_fixture(store / (variant + "-source"), packet, fixture_variant=variant)
                for surface in ("C", "R", "S-check", "S-details", "H"):
                    with self.subTest(case=variant, surface=surface):
                        phases = [("case", packet, variant)]
                        if variant != READY_VALID:
                            phases.append(("recovery", baseline, READY_VALID))
                        for phase, source, case in phases:
                            destination = store / (variant + surface + phase)
                            if surface == "H":
                                result = run_hook_case(source, destination, environment, case_id=case)
                            else:
                                result = run_case(source, destination, runtime, environment, case_id=case, surface=surface)
                            self.assertTrue(result["observation_matches_expected"])
                            self.assertFalse(result["complete_matrix"])
                            self.assertFalse(result["acceptance_asserted"])
                            self.assertNotIn(result["invocation_id"], identities)
                            identities.add(result["invocation_id"])
                            self.assertFalse((destination / "fixture/missing-future-proof.json").exists())
                            self.assertFalse((destination / "fixture/missing-future-review.json").exists())
            self.assertEqual(45, len(identities))


if __name__ == "__main__":
    unittest.main()
