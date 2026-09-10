"""Real entry-point mixed environment routing over synthetic evidence inputs."""

import json
import os
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_cases import run_case
from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_capture_hook import run_hook_case
from workflow_delivery_environment_cases import MIXED_VARIANT, SWAPPED_VARIANT


class MixedCaptureTest(unittest.TestCase):
    def test_legacy_acceptance_and_headless_verification_reach_actual_entrypoints(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet = store / "packet"
            prepare_fixture(store / "source", packet, fixture_variant=MIXED_VARIANT)
            swapped = store / "swapped-packet"
            prepare_fixture(store / "swapped-source", swapped, fixture_variant=SWAPPED_VARIANT)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            runtime = Path(__file__).resolve().parents[1]
            for surface in ("C", "R", "S-check", "S-details", "H"):
                with self.subTest(surface=surface):
                    for phase, source, case_id in (("valid", packet, MIXED_VARIANT),
                                                   ("swapped", swapped, SWAPPED_VARIANT),
                                                   ("recovery", packet, MIXED_VARIANT)):
                        destination = store / (surface + "-" + phase)
                        if surface == "H":
                            result = run_hook_case(source, destination, environment, case_id=case_id)
                        else:
                            result = run_case(source, destination, runtime, environment,
                                              case_id=case_id, surface=surface)
                        self.assertTrue(result["observation_matches_expected"])
                        self.assertFalse(result["complete_matrix"])
                        if phase != "swapped" and surface in ("C", "R", "H"):
                            report = json.loads((destination / "capture/stdout.bin").read_bytes().splitlines()[-1])
                            self.assertEqual(["TASK: accept", "INFRA: verify"], report["checks"])


if __name__ == "__main__":
    unittest.main()
