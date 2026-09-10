"""Actual readiness refusals with synthetic input, not completed rollout readiness."""

import os
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_cases import run_case
from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_capture_hook import run_hook_case
from workflow_delivery_readiness_input import READINESS_CASES, READY_VALID, PENDING_CATEGORIES, ENROLLMENT_BASELINES
from workflow_delivery_product_readiness_input import PRODUCT_READINESS_CASES, PRODUCT_READY


class ReadinessCaptureTest(unittest.TestCase):
    def test_readiness_cases_and_fresh_recovery_through_all_entrypoints(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            baselines = {}
            for normal in (READY_VALID, PRODUCT_READY, *PENDING_CATEGORIES):
                baselines[normal] = store / (normal + "-packet")
                recipe = prepare_fixture(store / (normal + "-source"), baselines[normal], fixture_variant=normal)
                self.assertEqual("accepted-history-and-readiness", recipe["fixture_family"])
            runtime = Path(__file__).resolve().parents[1]
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            identities = set()
            for variant in READINESS_CASES:
                normal = PRODUCT_READY if variant in PRODUCT_READINESS_CASES else READY_VALID
                if variant in PENDING_CATEGORIES:
                    normal = variant
                elif variant in ENROLLMENT_BASELINES:
                    normal = ENROLLMENT_BASELINES[variant]
                baseline = baselines[normal]
                packet = baseline
                if variant != normal:
                    packet = store / (variant + "-packet")
                    prepare_fixture(store / (variant + "-source"), packet, fixture_variant=variant)
                for surface in ("C", "R", "S-check", "S-details", "H"):
                    with self.subTest(case=variant, surface=surface):
                        phases = [("case", packet, variant)]
                        if variant != normal:
                            phases.append(("recovery", baseline, normal))
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
            self.assertEqual(110, len(identities))


if __name__ == "__main__":
    unittest.main()
