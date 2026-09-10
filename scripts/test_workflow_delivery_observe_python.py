"""Real subprocess observer tests on synthetic scripts, not rollout evidence."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

OBSERVER = Path(__file__).with_name("workflow_delivery_observe_python.py")


class ObservePythonTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.script = self.root / "fixture.py"
        self.output = self.root / "events.jsonl"

    def run_script(self, source, *, controlled=True):
        self.script.write_text(source)
        command = [sys.executable, *(["-I", "-S", "-B"] if controlled else []), str(OBSERVER),
                   "--script", str(self.script), "--source-root", str(self.root),
                   "--output", str(self.output), "--function", "main", "--", "fixture-argument"]
        result = subprocess.run(command, capture_output=True, timeout=15)
        rows = [json.loads(line) for line in self.output.read_text().splitlines()] if self.output.exists() else []
        return result, rows

    def test_records_actual_call_output_and_runtime_identities(self):
        source = 'import sys\ndef main():\n    print(sys.argv[1])\nmain()\n'
        result, rows = self.run_script(source)
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual(b"fixture-argument\n", result.stdout)
        self.assertEqual(["start", "call", "finish"], [row["event"] for row in rows])
        self.assertEqual(hashlib.sha256(source.encode()).hexdigest(), rows[1]["source"]["sha256"])
        self.assertEqual("main", rows[1]["function"])
        self.assertEqual(3, len({row["event_id"] for row in rows}))
        self.assertEqual(1, len({row["invocation_id"] for row in rows}))
        self.assertTrue(rows[-1]["profiler_still_installed"])
        self.assertTrue(rows[-1]["runtime"]["modules"])
        self.assertTrue(any("cache_candidate" in row for row in rows[-1]["runtime"]["modules"]))
        self.assertTrue(rows[-1]["runtime"]["mapped_files"])
        self.assertFalse(rows[-1]["acceptance_asserted"])
        self.assertFalse((self.root / "__pycache__").exists())

    def test_early_refusal_does_not_invent_handler_calls(self):
        result, rows = self.run_script('raise SystemExit(7)\n')
        self.assertEqual(7, result.returncode)
        self.assertEqual(["start", "finish"], [row["event"] for row in rows])
        self.assertEqual({"kind": "system_exit", "exit_code": 7}, rows[-1]["outcome"])

    def test_exception_is_preserved_not_converted_to_success(self):
        result, rows = self.run_script('raise ValueError("fixture failure")\n')
        self.assertNotEqual(0, result.returncode)
        self.assertIn(b"fixture failure", result.stderr)
        self.assertEqual("ValueError", rows[-1]["outcome"]["exception_type"])

    def test_startup_injection_flags_are_required_before_script_execution(self):
        result, rows = self.run_script('print("must not run")\n', controlled=False)
        self.assertNotEqual(0, result.returncode)
        self.assertEqual(b"", result.stdout)
        self.assertEqual([], rows)

    def test_existing_output_is_never_overwritten(self):
        self.output.write_text('previous attempt\n')
        self.script.write_text('print("must not run")\n')
        result = subprocess.run([sys.executable, "-I", "-S", "-B", str(OBSERVER),
            "--script", str(self.script), "--source-root", str(self.root),
            "--output", str(self.output), "--function", "main"], capture_output=True, timeout=15)
        self.assertNotEqual(0, result.returncode)
        self.assertEqual(b"", result.stdout)
        self.assertEqual('previous attempt\n', self.output.read_text())

    def test_removed_instrumentation_is_visible(self):
        result, rows = self.run_script('import sys\nsys.setprofile(None)\n')
        self.assertEqual(0, result.returncode)
        self.assertFalse(rows[-1]["profiler_still_installed"])

    def test_abrupt_exit_retains_start_without_fabricated_finish(self):
        result, rows = self.run_script('import os\nos._exit(9)\n')
        self.assertEqual(9, result.returncode)
        self.assertEqual(["start"], [row["event"] for row in rows])


if __name__ == "__main__":
    unittest.main()
