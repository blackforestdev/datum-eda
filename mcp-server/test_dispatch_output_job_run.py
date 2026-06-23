#!/usr/bin/env python3
"""OutputJob run MCP dispatch tests."""

from __future__ import annotations

import subprocess
import unittest
from unittest.mock import patch

from server_runtime import EngineDaemonClient, StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchOutputJobRun(unittest.TestCase):
    def test_tools_call_dispatches_run_output_job(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 93,
                "method": "tools/call",
                "params": {
                    "name": "run_output_job",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "output_job": "11111111-1111-1111-1111-111111111111",
                        "output_dir": "/tmp/native-project/fab",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "run_output_job",
                    {
                        "path": "/tmp/native-project",
                        "output_job": "11111111-1111-1111-1111-111111111111",
                        "output_dir": "/tmp/native-project/fab",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "output_job_run_v1")
        self.assertEqual(payload["action"], "run_output_job")

    @patch("server_runtime.subprocess.run")
    def test_runs_output_job_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"output_job_run_v1","action":"run_output_job"}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.run_output_job(
            "/tmp/native-project",
            "11111111-1111-1111-1111-111111111111",
            "/tmp/native-project/fab",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "artifact",
                "generate",
                "/tmp/native-project",
                "--output-job",
                "11111111-1111-1111-1111-111111111111",
                "--output-dir",
                "/tmp/native-project/fab",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "output_job_run_v1")

    @patch("server_runtime.subprocess.run")
    def test_returns_failed_output_job_json_from_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=1,
            stdout='{"contract":"output_job_run_v1","action":"run_output_job","status":"failed"}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.run_output_job(
            "/tmp/native-project",
            "11111111-1111-1111-1111-111111111111",
        )
        self.assertEqual(response.result["contract"], "output_job_run_v1")
        self.assertEqual(response.result["status"], "failed")

    def test_tools_call_dispatches_start_and_cancel_output_job_run(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        start = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 94,
                "method": "tools/call",
                "params": {
                    "name": "start_output_job_run",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "output_job": "11111111-1111-1111-1111-111111111111",
                    },
                },
            }
        )
        cancel = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 95,
                "method": "tools/call",
                "params": {
                    "name": "cancel_output_job_run",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "run": "22222222-2222-2222-2222-222222222222",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "start_output_job_run",
                    {
                        "path": "/tmp/native-project",
                        "output_job": "11111111-1111-1111-1111-111111111111",
                    },
                ),
                (
                    "cancel_output_job_run",
                    {
                        "path": "/tmp/native-project",
                        "run": "22222222-2222-2222-2222-222222222222",
                    },
                ),
            ],
        )
        self.assertEqual(
            start["result"]["content"][0]["json"]["output_job_run"]["status"], "running"
        )
        self.assertEqual(
            cancel["result"]["content"][0]["json"]["output_job_run"]["status"], "canceled"
        )

    @patch("server_runtime.subprocess.run")
    def test_starts_and_cancels_output_job_run_via_cli(self, run_mock) -> None:
        run_mock.side_effect = [
            subprocess.CompletedProcess(
                args=[],
                returncode=0,
                stdout='{"contract":"output_job_run_lifecycle_v1","action":"start_output_job_run"}',
                stderr="",
            ),
            subprocess.CompletedProcess(
                args=[],
                returncode=0,
                stdout='{"contract":"output_job_run_lifecycle_v1","action":"cancel_output_job_run"}',
                stderr="",
            ),
        ]
        client = EngineDaemonClient()
        start = client.start_output_job_run(
            "/tmp/native-project", "11111111-1111-1111-1111-111111111111"
        )
        cancel = client.cancel_output_job_run(
            "/tmp/native-project", "22222222-2222-2222-2222-222222222222"
        )
        self.assertEqual(start.result["action"], "start_output_job_run")
        self.assertEqual(cancel.result["action"], "cancel_output_job_run")
        self.assertEqual(run_mock.call_args_list[0].args[0][3], "artifact")
        self.assertEqual(run_mock.call_args_list[0].args[0][4], "start-output-job-run")
        self.assertEqual(run_mock.call_args_list[1].args[0][3], "artifact")
        self.assertEqual(run_mock.call_args_list[1].args[0][4], "cancel-output-job-run")


if __name__ == "__main__":
    unittest.main()
