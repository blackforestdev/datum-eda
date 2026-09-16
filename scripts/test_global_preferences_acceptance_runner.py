"""Capture failures, enforce mandatory compilation and reject legacy subset modes."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from global_preferences_acceptance.capture import Capture, executed_tests
from global_preferences_acceptance.gates import commands
from global_preferences_acceptance.inputs import Bundle


class RunnerTests(unittest.TestCase):
    def test_workspace_clippy_and_renderer_proof_are_mandatory(self):
        inventory = commands(Path(__file__).resolve().parents[1])
        self.assertIn('workspace-tests', inventory)
        self.assertIn('clippy', inventory)
        for name in ('workspace-tests', 'clippy'):
            argv = inventory[name][0]
            self.assertIn('scripts/run_cargo_guarded.py', argv)
            self.assertIn('--all-targets', argv)
            self.assertIn('--locked', argv)
            self.assertIn('--offline', argv)
        self.assertIn('-D', inventory['clippy'][0])

    def test_command_failure_retains_exact_outputs_and_status(self):
        with tempfile.TemporaryDirectory() as root:
            capture = Capture(root)
            row = capture.command([sys.executable, '-c', 'import sys; print("out"); print("err",file=sys.stderr); sys.exit(7)'],
                                  cwd=root, candidate='a' * 40, inputs='b' * 64)
            self.assertEqual(row['exit_code'], 7)
            bundle = Bundle(root, list(capture.inventory.values()))
            raw = bundle.json(row['log'])
            self.assertEqual(bundle.read(raw['stdout']), b'out\n')
            self.assertEqual(bundle.read(raw['stderr']), b'err\n')
            self.assertEqual(raw['executed_tests'], 0)

    def test_timeout_is_failure_and_retained(self):
        with tempfile.TemporaryDirectory() as root:
            capture = Capture(root)
            row = capture.command([sys.executable, '-c', 'import time; time.sleep(10)'], cwd=root,
                                  candidate='a' * 40, inputs='b' * 64, timeout=0.05)
            self.assertEqual(row['exit_code'], 124)
            bundle = Bundle(root, list(capture.inventory.values()))
            self.assertIn(b'timed out', bundle.read(bundle.json(row['log'])['stderr']))

    def test_test_discovery_does_not_count_as_execution(self):
        self.assertEqual(executed_tests(b'test_name: test\n100 tests listed\n'), 0)
        self.assertEqual(executed_tests(b'test result: ok. 0 passed;\n'), 0)
        self.assertEqual(executed_tests(b'test result: ok. 3 passed;\ntest result: ok. 2 passed;\n'), 5)
        self.assertEqual(executed_tests(b'Ran 11 tests in 0.1s\n\nOK\n'), 11)

    def test_optional_subset_flags_cannot_report_readiness(self):
        script = Path(__file__).parent / 'run_global_preferences_production_acceptance.py'
        for options in ([], ['--measure'], ['--full-workspace']):
            result = subprocess.run([sys.executable, str(script), *options], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertNotIn('PASS', result.stdout)
            self.assertNotIn('candidate_ready', result.stdout)


if __name__ == '__main__':
    unittest.main()
