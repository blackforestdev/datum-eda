"""Retained malformed-authority cases through real entry points and recovery."""

import os
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_cases import run_case
from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_capture_hook import run_hook_case
from workflow_delivery_environment_cases import FIXTURE_VARIANTS, case_uses_prepared_input


class EnvironmentCaptureTest(unittest.TestCase):
    def test_fixture_identity_cannot_be_silently_relabelled(self):
        key = next(iter(FIXTURE_VARIANTS))
        with self.assertRaisesRegex(ValueError, "does not match"):
            case_uses_prepared_input(key, {})
        with self.assertRaisesRegex(ValueError, "does not match"):
            case_uses_prepared_input("INFRA-S01-01", {"fixture_variant": key})

    def test_six_malformed_selections_refuse_and_fresh_baselines_recover(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            valid = store / "valid-packet"
            prepare_fixture(store / "valid-fixture", valid)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            runtime = Path(__file__).resolve().parents[1]
            identities = set()
            for key in FIXTURE_VARIANTS:
                packet = store / (key + "-packet")
                recipe = prepare_fixture(store / (key + "-fixture"), packet, fixture_variant=key)
                self.assertEqual(key, recipe["fixture_variant"])
                self.assertEqual(2, recipe["schema_version"])
                for surface in ("C", "R", "S-check", "S-details", "H"):
                    with self.subTest(case=key, surface=surface):
                        for phase, source, case_id in (("refusal", packet, key),
                                                       ("recovery", valid, "INFRA-S01-01")):
                            output = store / (key + "-" + surface + "-" + phase)
                            if surface == "H":
                                result = run_hook_case(source, output, environment, case_id=case_id)
                            else:
                                result = run_case(source, output, runtime, environment,
                                                  case_id=case_id, surface=surface)
                            self.assertTrue(result["observation_matches_expected"])
                            self.assertFalse(result["complete_matrix"])
                            self.assertFalse(result["acceptance_asserted"])
                            self.assertNotIn(result["invocation_id"], identities)
                            identities.add(result["invocation_id"])
            self.assertEqual(60, len(identities))


if __name__ == "__main__":
    unittest.main()
