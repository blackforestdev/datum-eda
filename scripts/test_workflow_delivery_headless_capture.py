"""Exercise headless shape/identity/dimension refusals through actual entry points."""

import os
import json
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_cases import run_case
from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_capture_hook import run_hook_case
from workflow_delivery_environment_cases import HEADLESS_VARIANTS, MIXED_VARIANT, PRODUCT_HEADLESS_VARIANT


class HeadlessCaptureTest(unittest.TestCase):
    def test_product_headless_refuses_without_corrupting_infrastructure_selection(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            valid, invalid = store / "valid-packet", store / "invalid-packet"
            prepare_fixture(store / "valid", valid, fixture_variant=MIXED_VARIANT)
            prepare_fixture(store / "invalid", invalid, fixture_variant=PRODUCT_HEADLESS_VARIANT)
            selected = json.loads((store / "invalid/requested-environment.json").read_bytes())["environments"]
            self.assertEqual(selected[0]["environment"], selected[1]["environment"])
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            runtime = Path(__file__).resolve().parents[1]
            identities = set()
            for surface in ("C", "R", "S-check", "S-details", "H"):
                for phase, packet, case in (("refusal", invalid, PRODUCT_HEADLESS_VARIANT),
                                           ("recovery", valid, MIXED_VARIANT)):
                    with self.subTest(surface=surface, phase=phase):
                        output = store / (surface + phase)
                        result = (run_hook_case(packet, output, environment, case_id=case) if surface == "H" else
                                  run_case(packet, output, runtime, environment, case_id=case, surface=surface))
                        self.assertTrue(result["observation_matches_expected"])
                        self.assertFalse(result["complete_matrix"])
                        self.assertFalse(result["acceptance_asserted"])
                        self.assertNotIn(result["invocation_id"], identities)
                        identities.add(result["invocation_id"])
            self.assertEqual(10, len(identities))

    def test_malformed_environments_refuse_and_fresh_mixed_input_recovers(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            baseline = store / "baseline-packet"
            prepare_fixture(store / "baseline", baseline, fixture_variant=MIXED_VARIANT)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            runtime = Path(__file__).resolve().parents[1]
            identities = set()
            for variant in HEADLESS_VARIANTS:
                packet = store / (variant + "-packet")
                prepare_fixture(store / (variant + "-source"), packet, fixture_variant=variant)
                for surface in ("C", "R", "S-check", "S-details", "H"):
                    with self.subTest(variant=variant, surface=surface):
                        for phase, source, case in (("refusal", packet, variant),
                                                   ("recovery", baseline, MIXED_VARIANT)):
                            destination = store / (variant + "-" + surface + "-" + phase)
                            if surface == "H":
                                result = run_hook_case(source, destination, environment, case_id=case)
                            else:
                                result = run_case(source, destination, runtime, environment,
                                                  case_id=case, surface=surface)
                            self.assertTrue(result["observation_matches_expected"])
                            self.assertFalse(result["complete_matrix"])
                            self.assertFalse(result["acceptance_asserted"])
                            self.assertNotIn(result["invocation_id"], identities)
                            identities.add(result["invocation_id"])
            self.assertEqual(70, len(identities))


if __name__ == "__main__":
    unittest.main()
