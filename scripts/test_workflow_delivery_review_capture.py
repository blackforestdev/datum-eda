"""Review and acceptance-boundary refusals, not independent review proof."""

import os
import json
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_cases import run_case
from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_capture_hook import run_hook_case
from workflow_delivery_review_input import REVIEW_INPUT_CASES


class ReviewCaptureTest(unittest.TestCase):
    def test_wrong_and_candidate_only_receipts_refuse_and_recover(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            baseline, wrong = store / "baseline-packet", store / "wrong-packet"
            prepare_fixture(store / "baseline", baseline)
            prepare_fixture(store / "wrong", wrong, fixture_variant="INFRA-S04-08.wrong-receipt")
            runtime = Path(__file__).resolve().parents[1]
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            identities = set()
            for variant, packet in (("INFRA-S04-08.wrong-receipt", wrong),
                                    ("INFRA-S04-08.candidate-only-receipt", baseline)):
                for surface in ("C", "R", "S-check", "S-details", "H"):
                    with self.subTest(case=variant, surface=surface):
                        for phase, source, case in (("refusal", packet, variant),
                                                   ("recovery", baseline, "INFRA-S01-01")):
                            destination = store / (variant + surface + phase)
                            result = (run_hook_case(source, destination, environment, case_id=case)
                                      if surface == "H" else run_case(source, destination, runtime, environment,
                                                                     case_id=case, surface=surface))
                            self.assertTrue(result["observation_matches_expected"])
                            self.assertFalse(result["acceptance_asserted"])
                            self.assertFalse(result["complete_matrix"])
                            self.assertNotIn(result["invocation_id"], identities)
                            identities.add(result["invocation_id"])
            self.assertEqual(20, len(identities))

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
                if variant.endswith(".authorized-nonblocking-deferral"):
                    source = store / (variant + "-source")
                    review = json.loads((source / "docs/reviews/review.json").read_bytes())
                    self.assertEqual(["dat-next"], [row["issue_id"] for row in review["findings"]])
                    self.assertEqual("nonblocking", review["findings"][0]["severity"])
                    contract = json.loads((source / "contract.json").read_bytes())
                    proof = json.loads((source / contract["proof_path"]).read_bytes())
                    self.assertEqual(["dat-next"], proof["results"][0]["defects"])
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
                            if phase == "refusal" and variant.endswith(".authorized-nonblocking-deferral"):
                                retained = json.loads((destination / "fixture/docs/reviews/review.json").read_bytes())
                                self.assertEqual(review["findings"], retained["findings"])
            self.assertEqual(230, len(identities))


if __name__ == "__main__":
    unittest.main()
