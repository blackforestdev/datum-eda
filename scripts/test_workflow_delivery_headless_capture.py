"""Exercise headless shape/identity/dimension refusals through actual entry points."""

import os
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_cases import run_case
from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_capture_hook import run_hook_case
from workflow_delivery_environment_cases import HEADLESS_VARIANTS, MIXED_VARIANT


class HeadlessCaptureTest(unittest.TestCase):
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
