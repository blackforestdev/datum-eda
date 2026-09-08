"""Actual clause-accounting entrypoints; synthetic inputs are not authored work."""

import os
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_cases import run_case
from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_capture_hook import run_hook_case
from workflow_delivery_specification_input import SPECIFICATION_INPUT_CASES, SPEC_VALID, SPEC_PENDING


class SpecificationCaptureTest(unittest.TestCase):
    def test_clause_accounting_and_fresh_recovery_on_every_surface(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packets = {}
            for variant in SPECIFICATION_INPUT_CASES:
                packet = store / (variant + "-packet")
                recipe = prepare_fixture(store / (variant + "-source"), packet, fixture_variant=variant)
                self.assertEqual("synthetic-specification-output", recipe["fixture_family"])
                self.assertFalse((store / (variant + "-source") / "missing-future-proof.json").exists())
                if variant == SPEC_PENDING:
                    import json
                    source = store / (variant + "-source")
                    self.assertFalse((source / "docs/specification-matrix.json").exists())
                    manifest = json.loads((source / "specs/active_frontier.json").read_bytes())
                    self.assertTrue(all(step["status"] == "pending" for step in
                                        manifest["frontier"][1]["completion"]["steps"]))
                packets[variant] = packet
            runtime = Path(__file__).resolve().parents[1]
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            identities = set()
            for variant, packet in packets.items():
                for surface in ("C", "R", "S-check", "S-details", "H"):
                    with self.subTest(case=variant, surface=surface):
                        runs = [("case", packet, variant)]
                        if SPECIFICATION_INPUT_CASES[variant][1] is not None:
                            valid = "INFRA-S03-09.pending-owner-valid" if variant.startswith(("INFRA-S03-09.", "INFRA-S03-08.")) else SPEC_VALID
                            if variant.startswith("INFRA-S02-03."):
                                valid = SPEC_PENDING
                            runs.append(("recovery", packets[valid], valid))
                        for phase, source, case in runs:
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
            self.assertEqual(115, len(identities))


if __name__ == "__main__":
    unittest.main()
