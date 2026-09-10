"""Actual review-only boundary captures, not an independent review of this code."""

import json
import os
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_cases import run_case
from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_capture_hook import run_hook_case
from workflow_delivery_review_phase_input import REVIEW_PHASE_CASES


class ReviewPhaseCaptureTest(unittest.TestCase):
    def test_review_without_acceptance_on_every_entrypoint(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            runtime = Path(__file__).resolve().parents[1]
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            identities = set()
            recovery_case = "INFRA-S04-07.owner-pending"
            recovery_packet = store / "recovery-packet"
            prepare_fixture(store / "recovery-source", recovery_packet, fixture_variant=recovery_case)
            for variant in REVIEW_PHASE_CASES:
                source, packet = store / (variant + "-source"), store / (variant + "-packet")
                recipe = prepare_fixture(source, packet, fixture_variant=variant)
                self.assertEqual("synthetic-review-only", recipe["fixture_family"])
                if variant.endswith(".missing-record"):
                    self.assertFalse((source / "docs/reviews/review.json").exists())
                else:
                    review = json.loads((source / "docs/reviews/review.json").read_bytes())
                    self.assertIsNone(review["owner_receipt"])
                for surface in ("C", "R", "S-check", "S-details", "H"):
                    with self.subTest(case=variant, surface=surface):
                        destination = store / (variant + surface)
                        if surface == "H":
                            result = run_hook_case(packet, destination, environment, case_id=variant)
                        else:
                            result = run_case(packet, destination, runtime, environment, case_id=variant, surface=surface)
                        self.assertTrue(result["observation_matches_expected"])
                        self.assertFalse(result["complete_matrix"])
                        self.assertFalse(result["acceptance_asserted"])
                        self.assertNotIn(result["invocation_id"], identities)
                        identities.add(result["invocation_id"])
                        if surface in ("C", "R", "H") and REVIEW_PHASE_CASES[variant][1] is None:
                            raw = (destination / "capture/stdout.bin").read_bytes()
                            report = json.loads(raw.splitlines()[-1] if surface == "H" else raw)
                            self.assertEqual(["TASK: review"], report["checks"])
                            self.assertFalse(report["acceptance_asserted"])
                        if REVIEW_PHASE_CASES[variant][1] is not None:
                            destination = store / (variant + surface + "-recovery")
                            recovery = (run_hook_case(recovery_packet, destination, environment, case_id=recovery_case)
                                        if surface == "H" else run_case(recovery_packet, destination, runtime,
                                            environment, case_id=recovery_case, surface=surface))
                            self.assertTrue(recovery["observation_matches_expected"])
                            self.assertFalse(recovery["acceptance_asserted"])
                            self.assertFalse(recovery["complete_matrix"])
                            self.assertNotIn(recovery["invocation_id"], identities)
                            identities.add(recovery["invocation_id"])
            self.assertEqual(50, len(identities))


if __name__ == "__main__":
    unittest.main()
