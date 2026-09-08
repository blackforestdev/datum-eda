"""Actual coverage-refusal entrypoints on synthetic inputs, not rollout proof."""

import os
import json
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_cases import run_case
from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_capture_hook import run_hook_case
from workflow_delivery_coverage_input import COVERAGE_INPUT_CASES
from workflow_delivery_coverage_shapes import coverage_shape
from workflow_delivery_io import DeliveryInputError


class CoverageCaptureTest(unittest.TestCase):
    def test_missing_and_duplicate_classifications_refuse_and_recover(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            baseline = store / "baseline-packet"
            prepare_fixture(store / "baseline", baseline)
            runtime = Path(__file__).resolve().parents[1]
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            identities = set()
            for variant in COVERAGE_INPUT_CASES:
                packet = store / (variant + "-packet")
                recipe = prepare_fixture(store / (variant + "-source"), packet, fixture_variant=variant)
                self.assertEqual("malformed-coverage-input", recipe["fixture_family"])
                if variant.endswith(".duplicate-classification"):
                    # Establish the precise malformed input separately. Actual
                    # entrypoints reject invalid authority before coverage runs.
                    policy = json.loads((store / (variant + "-source") /
                                         "specs/workflow_delivery_policy.json").read_bytes())
                    with self.assertRaises(DeliveryInputError) as error:
                        coverage_shape(policy["coverage"])
                    self.assertEqual("WDQ-COVERAGE", error.exception.code)
                    self.assertIn("coverage.rows: duplicate identity", str(error.exception))
                for surface in ("C", "R", "S-check", "S-details", "H"):
                    with self.subTest(case=variant, surface=surface):
                        for phase, source, case in (("refusal", packet, variant), ("recovery", baseline, "INFRA-S01-01")):
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
            self.assertEqual(20, len(identities))


if __name__ == "__main__":
    unittest.main()
