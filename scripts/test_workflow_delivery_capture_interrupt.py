"""Actual entrypoint interruption; no timeout or copied event counts as proof."""

import json
import os
from pathlib import Path
import signal
import sys
import tempfile
import unittest

from workflow_delivery_capture_cases import run_case
from workflow_delivery_capture_command import capture_command
from workflow_delivery_capture_fixture import prepare_fixture, restore_fixture
from workflow_delivery_capture_interrupt import validate_trigger
from workflow_delivery_io import sha256


class CaptureInterruptTest(unittest.TestCase):
    def test_interrupt_observed_validator_then_fresh_invocation_recovers(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet = store / "packet"
            prepare_fixture(store / "baseline", packet)
            root, output = store / "fixture", store / "capture"
            recipe = restore_fixture(packet, root)
            runtime = Path(__file__).resolve().parents[1]
            script = runtime / "scripts/check_workflow_delivery.py"
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                           "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            command = [sys.executable, "-I", "-S", "-B",
                       str(runtime / "scripts/workflow_delivery_observe_python.py"),
                       "--script", str(script), "--source-root", str(runtime),
                       "--output", str(output / "calls.jsonl"), "--function", "main", "--",
                       "--root", str(root), "--enforce", "--staged",
                       "--authority-ref", recipe["head"], "--base-ref", recipe["head"],
                       "--environment-path", "requested-environment.json"]
            result = capture_command(root, output, command, environment,
                fixture_nonce=recipe["fixture_nonce"], case_id="INFRA-S06-06.validator", surface="C",
                untracked_roots=recipe["untracked_roots"], timeout=30,
                interrupt_on_call={"trace": "calls.jsonl", "source": str(script), "function": "main"})
            self.assertEqual("interrupted", result["kind"])
            self.assertEqual(-signal.SIGTERM, result["returncode"])
            self.assertEqual([], result["changed_state_fields"])
            self.assertFalse(result["acceptance_asserted"])
            interruption = json.loads((output / "interruption.json").read_bytes())
            self.assertEqual(result["pid"], interruption["pid"])
            self.assertEqual(result["pid"], interruption["event"]["pid"])
            line = bytes.fromhex(interruption["event_line_hex"])
            self.assertEqual(sha256(line), interruption["event_line_sha256"])
            self.assertIn(line, (output / "calls.jsonl").read_bytes())
            self.assertFalse(interruption["escalated"])
            recovery = run_case(packet, store / "recovery", runtime, environment,
                                case_id="INFRA-S01-01", surface="C")
            self.assertTrue(recovery["observation_matches_expected"])
            self.assertNotEqual(result["invocation_id"], recovery["invocation_id"])

    def test_trigger_cannot_select_an_external_trace_or_implicit_source(self):
        for value in ({}, {"trace": "../other.jsonl", "source": "/source.py", "function": "main"},
                      {"trace": "events.jsonl", "source": "source.py", "function": "main"}):
            with self.assertRaises(ValueError):
                validate_trigger(value)


if __name__ == "__main__":
    unittest.main()
