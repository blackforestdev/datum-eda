"""Real staged/candidate/selector refusal and recovery on synthetic inputs."""

import os
import json
from pathlib import Path
import tempfile
import unittest

from workflow_delivery_capture_cases import CASES, run_case
from workflow_delivery_capture_assertions import evaluate_capture
from workflow_delivery_capture_fixture import prepare_fixture
from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_environment_cases import PREPARED_VARIANTS
from workflow_delivery_capture_history import HISTORY_CASES
from workflow_delivery_capture_snapshots import SNAPSHOT_CASES
from workflow_delivery_interruption_case import INTERRUPTION_CASE


class CaptureCasesTest(unittest.TestCase):
    def test_explicit_cases_refuse_for_intended_reason_and_fresh_baseline_recovers(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet = store / "packet"
            prepare_fixture(store / "baseline", packet)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull,
                "PYTHONDONTWRITEBYTECODE": "1", "LANG": "C"}
            runtime = Path(__file__).resolve().parents[1]
            invocations = set()
            ordinary_cases = {key: value for key, value in CASES.items()
                              if key not in PREPARED_VARIANTS | HISTORY_CASES.keys() | SNAPSHOT_CASES.keys() | {INTERRUPTION_CASE}}
            for case_id in ordinary_cases:
                for surface in ("C", "R", "S-check", "S-details"):
                    with self.subTest(case_id=case_id, surface=surface):
                        output = store / (case_id + "-" + surface)
                        result = run_case(packet, output, runtime, environment,
                                          case_id=case_id, surface=surface)
                        self.assertTrue(result["observation_matches_expected"])
                        self.assertFalse(result["complete_matrix"])
                        self.assertNotIn(result["invocation_id"], invocations)
                        invocations.add(result["invocation_id"])
                        script = runtime / "scripts" / ("project_status.py" if surface.startswith("S-")
                                                        else "check_workflow_delivery.py")
                        if CASES[case_id][1] is not None:
                            with self.assertRaisesRegex(ValueError, "precise diagnostic"):
                                evaluate_capture(output / "capture", case_id=case_id,
                                    surface="S" if surface.startswith("S-") else surface,
                                    expected_code="WDQ-WRONG-FAILURE", script=script)
                            for extra, message in (({"expected_path": "wrong-input.json"}, "input path"),
                                                   ({"expected_detail": "wrong cause"}, "diagnostic cause")):
                                with self.assertRaisesRegex(ValueError, message):
                                    evaluate_capture(output / "capture", case_id=case_id,
                                        surface="S" if surface.startswith("S-") else surface,
                                        expected_code=CASES[case_id][1], script=script, **extra)
                        if CASES[case_id][1] is not None:
                            recovery = run_case(packet, store / (output.name + "-recovery"), runtime,
                                environment, case_id="INFRA-S01-01", surface=surface)
                            self.assertTrue(recovery["observation_matches_expected"])
                            self.assertNotIn(recovery["invocation_id"], invocations)
                            invocations.add(recovery["invocation_id"])
            self.assertEqual(4 + 8 * (len(ordinary_cases) - 1), len(invocations))

    def test_capture_tampering_and_false_unchanged_summary_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            store = Path(temporary)
            packet = store / "packet"
            prepare_fixture(store / "baseline", packet)
            environment = {"PATH": os.defpath, "HOME": str(store), "XDG_CONFIG_HOME": str(store),
                "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
            runtime = Path(__file__).resolve().parents[1]
            output = store / "case"
            run_case(packet, output, runtime, environment, case_id="INFRA-S01-01", surface="C")
            capture = output / "capture"
            def evaluate():
                return evaluate_capture(capture, case_id="INFRA-S01-01", surface="C",
                    expected_code=None, script=runtime / "scripts/check_workflow_delivery.py")
            original = (capture / "stdout.bin").read_bytes()
            (capture / "stdout.bin").write_bytes(original + b"\n")
            with self.assertRaisesRegex(ValueError, "hash mismatch"):
                evaluate()
            (capture / "stdout.bin").write_bytes(original)
            result_path = capture / "result.json"
            result = json.loads(result_path.read_text())
            result["kind"] = "timeout"
            result_path.write_bytes(canonical_json(result))
            with self.assertRaisesRegex(ValueError, "not a refusal"):
                evaluate()
            result["kind"] = "exited"
            after_path = capture / "after.json"
            after = json.loads(after_path.read_text())
            after["files"]["src/read.py"]["sha256"] = "0" * 64
            after_path.write_bytes(canonical_json(after))
            for artifact in result["artifacts"]:
                if artifact["path"] == "after.json":
                    artifact["sha256"] = sha256(after_path.read_bytes())
            result_path.write_bytes(canonical_json(result))
            with self.assertRaisesRegex(ValueError, "protected state changed"):
                evaluate()


if __name__ == "__main__":
    unittest.main()
