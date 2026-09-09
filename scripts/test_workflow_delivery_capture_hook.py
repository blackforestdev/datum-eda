"""Actual hook success/refusal/recovery captures; synthetic input, not native proof."""

import os
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_capture_hook import HOOK_TRUST_CASES, evaluate_hook, run_hook_case
from workflow_delivery_capture_mutations import CASES, CLI_ERROR_CASES
from workflow_delivery_capture_mutations import TEXT_CASES, RECOVERY_CASE
from workflow_delivery_capture_mutations import TRUST_CASES
from workflow_delivery_environment_cases import PREPARED_VARIANTS
from workflow_delivery_capture_history import BASE_CASES, CURRENT_HISTORY_CASE, HISTORY_CASES
from workflow_delivery_capture_snapshots import SNAPSHOT_CASES
from workflow_delivery_interruption_case import INTERRUPTION_CASE
from workflow_delivery_review_input import REPORT_CASES


class HookCaptureTest(unittest.TestCase):
    def test_selected_support_refusals_preserve_invalid_input_and_fresh_recovery(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet = store / "packet"
            prepare_fixture(store / "baseline", packet)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            identities = set()
            for case in HOOK_TRUST_CASES:
                with self.subTest(case=case):
                    output = store / case
                    refused = run_hook_case(packet, output, environment, case_id=case)
                    self.assertFalse(refused["validator_entrypoint_observed"])
                    self.assertEqual((output / "support-input-state.json").read_bytes(),
                                     (output / "support-output-state.json").read_bytes())
                    self.assertEqual(refused["support_input_sha256"], refused["support_output_sha256"])
                    self.assertNotIn(b"UNTRUSTED SUPPORT EXECUTED", (output / "capture/stdout.bin").read_bytes())
                    recovered = run_hook_case(packet, store / (case + "-recovery"), environment,
                                              case_id="INFRA-S01-01")
                    for result in (refused, recovered):
                        self.assertTrue(result["observation_matches_expected"])
                        self.assertFalse(result["complete_matrix"])
                        self.assertFalse(result["acceptance_asserted"])
                        self.assertNotIn(result["invocation_id"], identities)
                        identities.add(result["invocation_id"])
            self.assertEqual(8, len(identities))

    def test_all_implemented_cases_and_fresh_recoveries_use_real_hook(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet = store / "packet"
            prepare_fixture(store / "baseline", packet)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            identities = set()
            ordinary_cases = {key: value for key, value in CASES.items()
                              if key not in PREPARED_VARIANTS | HISTORY_CASES.keys() | BASE_CASES.keys() | SNAPSHOT_CASES.keys() | CLI_ERROR_CASES | REPORT_CASES | TRUST_CASES.keys() | {INTERRUPTION_CASE, CURRENT_HISTORY_CASE}}
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
                    if case_id in TEXT_CASES:
                        self.assertTrue(result["text_evidence"]["noninteractive_descriptors_observed"])
                        self.assertTrue(result["text_evidence"]["no_color_escapes"])
                        self.assertFalse(result["text_evidence"]["gui_accessibility_asserted"])
                    if case_id == RECOVERY_CASE:
                        self.assertFalse(result["recovery"]["authority_refreshed"])
                        self.assertNotEqual(result["invocation_id"], result["recovery"]["prior_invocation_id"])
                        self.assertTrue((output / result["recovery"]["prior_result_path"]).is_file())
                    if code:
                        recovery = run_hook_case(packet, store / (case_id + "-recovery"), environment,
                                                 case_id="INFRA-S01-01")
                        self.assertNotIn(recovery["invocation_id"], identities)
                        identities.add(recovery["invocation_id"])
                    stdout = output / "capture/stdout.bin"
                    stdout.write_bytes(stdout.read_bytes() + b"changed capture\n")
                    with self.assertRaisesRegex(ValueError, "capture hash mismatch"):
                        evaluate_hook(output / "capture", case_id=case_id, hook="unused")
            self.assertEqual(sum(1 + (code is not None) for _, code in ordinary_cases.values()), len(identities))


if __name__ == "__main__":
    unittest.main()
