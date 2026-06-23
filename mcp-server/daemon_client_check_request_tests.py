#!/usr/bin/env python3
"""EngineDaemonClient CLI argv tests for check and standards-repair tools."""

from __future__ import annotations

import subprocess
import unittest
from unittest.mock import patch

from server_runtime import EngineDaemonClient


class TestDaemonClientCheckRequests(unittest.TestCase):
    @patch("server_runtime.subprocess.run")
    def test_gets_check_run_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"native_project_check_run","persisted":true}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.get_check_run("/tmp/native-project", "native-combined")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "check",
                "run",
                "/tmp/native-project",
                "--profile",
                "native-combined",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "native_project_check_run")
        self.assertEqual(response.result["persisted"], True)

    @patch("server_runtime.subprocess.run")
    def test_gets_check_runs_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"check_run_list_v1","check_run_count":1}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.get_check_runs("/tmp/native-project")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "check",
                "list",
                "/tmp/native-project",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "check_run_list_v1")
        self.assertEqual(response.result["check_run_count"], 1)

    @patch("server_runtime.subprocess.run")
    def test_shows_check_run_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"check_run_record_v1"}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.show_check_run("/tmp/native-project", "check-run-test")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "check",
                "show",
                "/tmp/native-project",
                "--check-run",
                "check-run-test",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "check_run_record_v1")

    @patch("server_runtime.subprocess.run")
    def test_gets_check_profiles_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"check_profiles_v1","default_profile_id":"native-combined"}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.get_check_profiles("/tmp/native-project")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "check",
                "profiles",
                "/tmp/native-project",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "check_profiles_v1")
        self.assertEqual(response.result["default_profile_id"], "native-combined")

    @patch("server_runtime.subprocess.run")
    def test_gets_zone_fills_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"zone_fills_query_v1","zone_fill_count":1}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.get_zone_fills("/tmp/native-project")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "query",
                "zone-fills",
                "/tmp/native-project",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "zone_fills_query_v1")
        self.assertEqual(response.result["zone_fill_count"], 1)

    @patch("server_runtime.subprocess.run")
    def test_fills_zones_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"zone_fill_generate_v1","zone_fill_count":1}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.fill_zones(
            "/tmp/native-project",
            zone="zone-test",
            net="net-test",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "check",
                "fill-zones",
                "/tmp/native-project",
                "--zone",
                "zone-test",
                "--net",
                "net-test",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "zone_fill_generate_v1")
        self.assertEqual(response.result["zone_fill_count"], 1)

    @patch("server_runtime.subprocess.run")
    def test_generates_standards_repair_proposals_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"generate_standards_repair_proposals","proposal_count":1}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.generate_standards_repair_proposals("/tmp/native-project")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "check",
                "repair-standards",
                "/tmp/native-project",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "generate_standards_repair_proposals")
        self.assertEqual(response.result["proposal_count"], 1)

    @patch("server_runtime.subprocess.run")
    def test_waives_finding_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"project_waive_finding_v1","status":"applied"}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.waive_finding(
            "/tmp/native-project",
            "sha256:finding",
            "Intentional design decision",
            "mcp-test",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "check",
                "waive",
                "/tmp/native-project",
                "--fingerprint",
                "sha256:finding",
                "--rationale",
                "Intentional design decision",
                "--created-by",
                "mcp-test",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "project_waive_finding_v1")
        self.assertEqual(response.result["status"], "applied")

    @patch("server_runtime.subprocess.run")
    def test_accepts_deviation_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"project_accept_deviation_v1","status":"applied"}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.accept_deviation(
            "/tmp/native-project",
            "sha256:finding",
            "Accepted design deviation",
            "mcp-test",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "check",
                "accept-deviation",
                "/tmp/native-project",
                "--fingerprint",
                "sha256:finding",
                "--rationale",
                "Accepted design deviation",
                "--accepted-by",
                "mcp-test",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "project_accept_deviation_v1")
        self.assertEqual(response.result["status"], "applied")
