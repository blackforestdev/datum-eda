#!/usr/bin/env python3
"""EngineDaemonClient artifact generation CLI bridge tests."""

from __future__ import annotations

import subprocess
import unittest
from unittest.mock import patch

from server_runtime import EngineDaemonClient


class TestDaemonClientArtifactGenerateRequests(unittest.TestCase):
    @patch("server_runtime.subprocess.run")
    def test_generates_artifacts_from_output_job_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"output_job_run_v1","action":"run_output_job"}',
            stderr="",
        )
        response = EngineDaemonClient().generate_artifacts(
            "/tmp/native-project",
            output_job="job-test",
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
                "job-test",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "output_job_run_v1")
        self.assertEqual(response.result["action"], "run_output_job")
