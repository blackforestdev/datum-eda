"""Real handler-triggered interruption and fresh recovery across all surfaces."""

import json
import os
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_cases import run_case
from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_capture_hook import run_hook_case
from workflow_delivery_interruption_case import INTERRUPTION_CASE, evaluate_interruption
from workflow_delivery_io import canonical_json, sha256


class InterruptionCaseTest(unittest.TestCase):
    def test_all_surfaces_interrupt_the_named_handler_and_recover_with_fresh_events(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet = store / "packet"
            prepare_fixture(store / "baseline", packet)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            runtime = Path(__file__).resolve().parents[1]
            identities = set()
            for surface in ("C", "R", "S-check", "S-details", "H"):
                with self.subTest(surface=surface):
                    for phase, case in (("interrupted", INTERRUPTION_CASE), ("recovery", "INFRA-S01-01")):
                        destination = store / (surface + phase)
                        if surface == "H":
                            result = run_hook_case(packet, destination, environment, case_id=case)
                        else:
                            result = run_case(packet, destination, runtime, environment, case_id=case, surface=surface)
                        self.assertTrue(result["observation_matches_expected"])
                        self.assertFalse(result["complete_matrix"])
                        self.assertFalse(result["acceptance_asserted"])
                        self.assertNotIn(result["invocation_id"], identities)
                        identities.add(result["invocation_id"])
                        if phase == "interrupted":
                            capture = destination / "capture"
                            invocation = json.loads((capture / "invocation.json").read_bytes())
                            trigger = invocation["interrupt_on_call"]
                            record_path = capture / "interruption.json"
                            record = json.loads(record_path.read_bytes())
                            record["pid"] += 1
                            record_path.write_bytes(canonical_json(record))
                            manifest_path = capture / "result.json"
                            manifest = json.loads(manifest_path.read_bytes())
                            for artifact in manifest["artifacts"]:
                                if artifact["path"] == "interruption.json":
                                    artifact["sha256"] = sha256(record_path.read_bytes())
                            manifest_path.write_bytes(canonical_json(manifest))
                            with self.assertRaisesRegex(ValueError, "wrong signal or child"):
                                evaluate_interruption(capture, case_id=case, surface=invocation["surface"], trigger=trigger)
            self.assertEqual(10, len(identities))


if __name__ == "__main__":
    unittest.main()
