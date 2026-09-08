"""Review and acceptance-boundary refusals, not independent review proof."""

import os
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_cases import run_case
from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_capture_hook import run_hook_case
from workflow_delivery_review_input import REVIEW_INPUT_CASES


class ReviewCaptureTest(unittest.TestCase):
    def test_review_and_acceptance_refusals_on_every_entrypoint(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            baseline = store / "baseline-packet"
            prepare_fixture(store / "baseline", baseline)
            runtime = Path(__file__).resolve().parents[1]
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            identities = set()
            for variant in REVIEW_INPUT_CASES:
                packet = store / (variant + "-packet")
                recipe = prepare_fixture(store / (variant + "-source"), packet, fixture_variant=variant)
                self.assertEqual("accepted-history-and-review", recipe["fixture_family"])
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
            self.assertEqual(210, len(identities))


if __name__ == "__main__":
    unittest.main()
