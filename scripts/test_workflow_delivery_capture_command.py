"""Exercise actual capture subprocesses in owned synthetic fixtures."""

import json
import os
from pathlib import Path
import sys
import tempfile
import unittest

from workflow_delivery_capture_command import capture_command
from workflow_delivery_io import sha256
from workflow_delivery_test_support import Fixture


class CaptureCommandTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.output = Path(self.temp.name) / "capture"
        self.f.write(".git/datum-wdq-capture-owner", b"fixture-owned-nonce\n")
        self.environment = {"PATH": os.defpath, "HOME": self.temp.name,
            "XDG_CONFIG_HOME": self.temp.name, "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C", "PYTHONDONTWRITEBYTECODE": "1"}

    def run_command(self, command, **kwargs):
        return capture_command(self.f.root, self.output, command, self.environment,
            fixture_nonce="fixture-owned-nonce", case_id="fixture-only", surface="C",
            untracked_roots=["src"], timeout=kwargs.get("timeout", 10),
            interrupt_on_call=kwargs.get("interrupt_on_call"))

    def test_exit_output_and_exact_unchanged_state_are_retained(self):
        result = self.run_command([sys.executable, "-I", "-S", "-B", "-c",
                                  'import sys; print("actual"); print("error", file=sys.stderr); sys.exit(7)'])
        self.assertEqual("exited", result["kind"])
        self.assertEqual(7, result["returncode"])
        self.assertEqual(b"actual\n", (self.output / "stdout.bin").read_bytes())
        self.assertEqual(b"error\n", (self.output / "stderr.bin").read_bytes())
        self.assertEqual([], result["changed_state_fields"])
        self.assertFalse(result["acceptance_asserted"])
        self.assertEqual(result, json.loads((self.output / "result.json").read_text()))

    def test_mutation_is_recorded_not_repaired(self):
        result = self.run_command([sys.executable, "-I", "-S", "-B", "-c",
                                  'from pathlib import Path; Path("src/read.py").write_text("changed")'])
        self.assertEqual(["src/read.py"], result["changed_files"])
        self.assertEqual("changed", (self.f.root / "src/read.py").read_text())

    def test_observer_calls_raw_output_and_state_share_the_actual_child(self):
        self.f.write("src/entry.py", b'def main():\n    print("observed handler")\nmain()\n')
        observer = Path(__file__).with_name("workflow_delivery_observe_python.py")
        events = self.output / "python-events.jsonl"
        result = self.run_command([sys.executable, "-I", "-S", "-B", str(observer),
            "--script", str(self.f.root / "src/entry.py"), "--source-root", str(self.f.root),
            "--output", str(events), "--function", "main"])
        rows = [json.loads(line) for line in events.read_text().splitlines()]
        self.assertEqual(0, result["returncode"])
        self.assertEqual(["start", "call", "finish"], [row["event"] for row in rows])
        self.assertEqual({result["pid"]}, {row["pid"] for row in rows})
        self.assertEqual(b"observed handler\n", (self.output / "stdout.bin").read_bytes())
        self.assertIn({"path": events.name, "sha256": sha256(events.read_bytes())}, result["artifacts"])
        self.assertEqual([], result["changed_state_fields"])

    def test_timeout_is_not_an_expected_refusal(self):
        result = self.run_command([sys.executable, "-I", "-S", "-B", "-c",
                                  'import time; print("started", flush=True); time.sleep(60)'], timeout=0.2)
        self.assertEqual("timeout", result["kind"])
        self.assertLess(result["returncode"], 0)
        self.assertEqual([], result["changed_state_fields"])

    def test_missing_executable_is_retained_as_launch_error(self):
        result = self.run_command([str(Path(self.temp.name) / "missing-executable")])
        self.assertEqual("launch_or_process_error", result["kind"])
        self.assertIsNone(result["returncode"])
        self.assertIsNone(result["pid"])

    def test_missing_call_event_timeout_is_not_interruption_proof(self):
        result = self.run_command([sys.executable, "-I", "-S", "-B", "-c",
                                  'import time; time.sleep(60)'], timeout=0.2,
            interrupt_on_call={"trace": "missing.jsonl", "source": "/not-observed.py", "function": "main"})
        self.assertEqual("timeout", result["kind"])
        self.assertFalse((self.output / "interruption.json").exists())
        self.assertEqual([], result["changed_state_fields"])

    def test_exit_before_call_event_is_not_interruption_proof(self):
        result = self.run_command([sys.executable, "-I", "-S", "-B", "-c", 'print("ordinary exit")'],
            interrupt_on_call={"trace": "missing.jsonl", "source": "/not-observed.py", "function": "main"})
        self.assertEqual("exited", result["kind"])
        self.assertEqual(0, result["returncode"])
        self.assertFalse((self.output / "interruption.json").exists())

    def test_no_ownership_or_existing_output_refuses_without_launch(self):
        marker = self.f.root / ".git/datum-wdq-capture-owner"
        marker.write_text("different-owner")
        with self.assertRaises(ValueError):
            self.run_command([sys.executable, "-c", 'print("must not run")'])
        self.assertFalse(self.output.exists())
        marker.write_text("fixture-owned-nonce")
        self.output.mkdir()
        with self.assertRaises(FileExistsError):
            self.run_command([sys.executable, "-c", 'print("must not run")'])

    def test_unknown_environment_fields_are_not_written_or_executed(self):
        self.environment["SECRET_TOKEN"] = "not evidence"
        with self.assertRaises(ValueError):
            self.run_command([sys.executable, "-c", 'print("must not run")'])
        self.assertFalse(self.output.exists())


if __name__ == "__main__":
    unittest.main()
