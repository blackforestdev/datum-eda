#!/usr/bin/env python3
"""Canonical artifact generation dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchArtifactGenerate(unittest.TestCase):
    def test_tools_call_dispatches_artifact_generate_output_job_selector(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 501,
                "method": "tools/call",
                "params": {
                    "name": "datum.artifact.generate",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "output_job": "job-test",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "generate_artifacts",
                    {
                        "path": "/tmp/native-project",
                        "output_dir": None,
                        "include": None,
                        "prefix": None,
                        "output_job": "job-test",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["ok"], True)
        self.assertEqual(payload["schema"]["name"], "datum.artifact.generate")

    def test_tools_call_normalizes_artifact_generate_metadata(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 502,
                "method": "tools/call",
                "params": {
                    "name": "datum.artifact.generate",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "output_dir": "/tmp/fab",
                        "include": "gerber-set",
                        "prefix": "doa2526",
                    },
                },
            }
        )

        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["ok"], True)
        result = payload["result"]
        self.assertEqual(result["generated_count"], 1)
        generated = result["generated"][0]
        artifact = generated["artifact"]
        self.assertEqual(artifact["artifact_id"], "artifact-test")
        self.assertEqual(artifact["project_id"], "project-test")
        self.assertEqual(artifact["model_revision"], "model-test")
        self.assertEqual(artifact["output_job"], "output-job-test")
        self.assertEqual(artifact["variant"], "variant-test")
        self.assertEqual(artifact["generator_version"], "datum-test")
        self.assertEqual(artifact["validation_state"], "not_validated")
        self.assertEqual(artifact["file_count"], 1)
