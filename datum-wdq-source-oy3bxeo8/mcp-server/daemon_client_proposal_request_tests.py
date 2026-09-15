#!/usr/bin/env python3
"""EngineDaemonClient CLI argv tests for proposal tools."""

from __future__ import annotations

import subprocess
import unittest
from unittest.mock import patch

from server_runtime import EngineDaemonClient


class TestDaemonClientProposalRequests(unittest.TestCase):
    @patch("server_runtime.subprocess.run")
    def test_lists_proposals_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"proposals_query_v1","proposal_count":1}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.get_proposals("/tmp/native-project")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "proposal",
                "list",
                "/tmp/native-project",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "proposals_query_v1")

    @patch("server_runtime.subprocess.run")
    def test_rejects_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"review_proposal","status":"rejected"}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.reject_proposal("/tmp/native-project", "proposal-test")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "proposal",
                "reject",
                "/tmp/native-project",
                "--proposal",
                "proposal-test",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["status"], "rejected")

    @patch("server_runtime.subprocess.run")
    def test_reviews_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"review_proposal","status":"accepted"}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.review_proposal(
            "/tmp/native-project",
            "proposal-test",
            "accepted",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "proposal",
                "review",
                "/tmp/native-project",
                "--proposal",
                "proposal-test",
                "--status",
                "accepted",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["status"], "accepted")

    @patch("server_runtime.subprocess.run")
    def test_accept_applies_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"apply_proposal","status":"applied"}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.accept_apply_proposal("/tmp/native-project", "proposal-test")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "proposal",
                "accept-apply",
                "/tmp/native-project",
                "--proposal",
                "proposal-test",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["status"], "applied")


if __name__ == "__main__":
    unittest.main()
