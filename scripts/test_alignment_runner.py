#!/usr/bin/env python3
"""Verify companion build ordering and fail-closed workspace test setup."""
import sys
import os
import subprocess
import tempfile
from pathlib import Path
import unittest
from unittest.mock import patch

import check_alignment as alignment


class AlignmentRunnerTests(unittest.TestCase):
    def test_milestone_mode_runs_mcp_self_test_once(self):
        with patch.object(sys, "argv", ["check_alignment.py", "--run-gates"]), \
             patch.object(alignment, "run_command", return_value=(0, "")) as run:
            self.assertEqual(alignment.main(), 0)
        self.assertEqual(sum(call.args[0] == ["python3", "mcp-server/server.py", "--self-test"]
                             for call in run.call_args_list), 1)

    def test_focused_mode_has_no_cargo_and_default_retains_full_schedule(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            executable = root / "python3"
            executable.write_text('#!/bin/sh\nprintf "%s\\n" "$*" >> "$DATUM_TEST_COMMAND_LOG"\n')
            executable.chmod(0o755)
            log = root / "commands"
            environment = dict(os.environ, PATH=f"{root}:{os.environ['PATH']}",
                               DATUM_TEST_COMMAND_LOG=str(log))
            def schedule(*args):
                log.write_text("")
                result = subprocess.run(["bash", "scripts/run_drift_gates.sh", *args],
                                        cwd=alignment.ROOT, env=environment,
                                        capture_output=True, text=True, timeout=5)
                self.assertEqual(result.returncode, 0, result.stderr)
                return log.read_text().splitlines()
            focused = schedule("--focused")
            self.assertFalse(any("run_cargo_guarded.py" in line for line in focused))
            self.assertIn("scripts/check_source_health.py", focused)
            full = schedule("--full")
            self.assertEqual(full, schedule())
            self.assertIn("scripts/check_alignment.py --run-gates", full)
            self.assertNotIn("mcp-server/server.py --self-test", full)
            self.assertTrue(any("cargo clippy --workspace --all-targets" in line for line in full))
            self.assertIn("scripts/run_terminal_core_proof.py", full)
            self.assertTrue(any("cargo test -p datum-eda-cli" in line for line in full))

    def test_companions_finish_before_workspace_tests(self):
        with patch.object(sys, "argv", ["check_alignment.py", "--run-health"]), \
             patch.object(alignment, "run_command", return_value=(0, "")) as run:
            self.assertEqual(alignment.main(), 0)
        self.assertEqual(run.call_args_list[0].args[0],
                         ["cargo", "build", "-q", "-p", "datum-eda-cli", "-p", "eda-engine-daemon"])
        self.assertEqual(run.call_args_list[1].args[0], ["cargo", "test", "-q"])

    def test_companion_failure_stops_workspace_tests(self):
        with patch.object(sys, "argv", ["check_alignment.py", "--run-health"]), \
             patch.object(alignment, "run_command", return_value=(75, "refused")) as run:
            self.assertEqual(alignment.main(), 1)
        self.assertEqual(run.call_count, 1)


if __name__ == "__main__":
    unittest.main()
