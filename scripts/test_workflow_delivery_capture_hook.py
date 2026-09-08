"""Actual hook success/refusal/recovery captures; synthetic input, not native proof."""

import os
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_capture_hook import evaluate_hook, run_hook_case
from workflow_delivery_capture_mutations import CASES
from workflow_delivery_environment_cases import PREPARED_VARIANTS


class HookCaptureTest(unittest.TestCase):
    def test_all_implemented_cases_and_fresh_recoveries_use_real_hook(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet = store / "packet"
            prepare_fixture(store / "baseline", packet)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            identities = set()
            ordinary_cases = {key: value for key, value in CASES.items() if key not in PREPARED_VARIANTS}
            for case_id, (_, code) in ordinary_cases.items():
                with self.subTest(case_id=case_id):
                    output = store / case_id
                    result = run_hook_case(packet, output, environment, case_id=case_id)
                    self.assertTrue(result["observation_matches_expected"])
                    self.assertTrue(result["validator_entrypoint_observed"])
                    for key in ("handler_trace_complete", "child_tool_closure_complete",
                                "complete_matrix", "acceptance_asserted"):
                        self.assertFalse(result[key])
                    self.assertNotIn(result["invocation_id"], identities)
                    identities.add(result["invocation_id"])
                    if code:
                        recovery = run_hook_case(packet, store / (case_id + "-recovery"), environment,
                                                 case_id="INFRA-S01-01")
                        self.assertNotIn(recovery["invocation_id"], identities)
                        identities.add(recovery["invocation_id"])
                    stdout = output / "capture/stdout.bin"
                    stdout.write_bytes(stdout.read_bytes() + b"changed capture\n")
                    with self.assertRaisesRegex(ValueError, "capture hash mismatch"):
                        evaluate_hook(output / "capture", case_id=case_id, hook="unused")
            self.assertEqual(2 * len(ordinary_cases) - 1, len(identities))


if __name__ == "__main__":
    unittest.main()
