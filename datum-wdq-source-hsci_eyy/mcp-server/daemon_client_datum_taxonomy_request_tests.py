#!/usr/bin/env python3
"""EngineDaemonClient request tests for canonical datum.* MCP tools."""

from __future__ import annotations

import subprocess
import unittest
from unittest.mock import patch

from server_runtime import EngineDaemonClient


class TestDaemonClientDatumTaxonomyRequests(unittest.TestCase):
    @patch("server_runtime.subprocess.run")
    def test_datum_context_get_uses_cli_context_surface(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"contract":"datum_terminal_context_v1","actor_type":"ExternalAgent",'
                '"visible_artifact_ids":[],"visible_output_job_ids":[],'
                '"visible_artifact_file_paths":[],"latest_output_job_id":null,'
                '"active_context_commands":{"check_waive_finding":'
                '"datum-eda check waive /tmp/native-project --fingerprint '
                "'sha256:selected-finding' --rationale '<rationale>'"
                '","check_accept_deviation":'
                '"datum-eda check accept-deviation /tmp/native-project --fingerprint '
                "'sha256:selected-finding' --rationale '<rationale>'"
                '"}}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.datum_context_get(
            session="session-test",
            path="/tmp/context.json",
            project_root="/tmp/native-project",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "context",
                "get",
                "--session",
                "session-test",
                "--path",
                "/tmp/context.json",
                "--project-root",
                "/tmp/native-project",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "datum_terminal_context_v1")
        self.assertEqual(response.result["actor_type"], "ExternalAgent")
        self.assertEqual(response.result["visible_artifact_ids"], [])
        self.assertEqual(response.result["visible_output_job_ids"], [])
        self.assertEqual(response.result["visible_artifact_file_paths"], [])
        self.assertIsNone(response.result["latest_output_job_id"])
        self.assertEqual(
            response.result["active_context_commands"]["check_waive_finding"],
            "datum-eda check waive /tmp/native-project "
            "--fingerprint 'sha256:selected-finding' --rationale '<rationale>'",
        )

    @patch("server_runtime.subprocess.run")
    def test_datum_context_refresh_uses_cli_context_surface(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"datum_terminal_context_v1","actor_type":"ExternalAgent"}',
            stderr="",
        )
        client = EngineDaemonClient()
        client.datum_context_refresh(project_root="/tmp/native-project")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "context",
                "refresh",
                "--project-root",
                "/tmp/native-project",
            ],
            capture_output=True,
            text=True,
            check=False,
        )

    @patch("server_runtime.subprocess.run")
    def test_datum_context_session_events_uses_cli_context_surface(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"datum_tool_session_events_v1","event_count":0,"events":[]}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.datum_context_session_events(
            session="session-test",
            path="/tmp/context.json",
            project_root="/tmp/native-project",
            event_kind="terminal_command_handoff",
            origin="production_terminal_command",
            command_id="datum.artifact.generate",
            execution_id="exec-test",
            limit=1,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "context",
                "session-events",
                "--session",
                "session-test",
                "--path",
                "/tmp/context.json",
                "--project-root",
                "/tmp/native-project",
                "--event-kind",
                "terminal_command_handoff",
                "--origin",
                "production_terminal_command",
                "--command-id",
                "datum.artifact.generate",
                "--execution-id",
                "exec-test",
                "--limit",
                "1",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "datum_tool_session_events_v1")

    @patch("server_runtime.subprocess.run")
    def test_datum_context_session_activity_uses_cli_context_surface(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"datum_tool_session_activity_summary_v1","activity_event_count":0}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.datum_context_session_activity(
            session="session-test",
            path="/tmp/context.json",
            project_root="/tmp/native-project",
            origin="production_terminal_command",
            execution_id="exec-test",
            limit=1,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "context",
                "session-activity",
                "--session",
                "session-test",
                "--path",
                "/tmp/context.json",
                "--project-root",
                "/tmp/native-project",
                "--origin",
                "production_terminal_command",
                "--execution-id",
                "exec-test",
                "--limit",
                "1",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "datum_tool_session_activity_summary_v1")
        self.assertEqual(response.result["activity_event_count"], 0)


if __name__ == "__main__":
    unittest.main()
