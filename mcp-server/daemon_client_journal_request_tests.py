#!/usr/bin/env python3
"""EngineDaemonClient CLI argv tests for native project journal tools."""

from __future__ import annotations

import subprocess
import unittest
from unittest.mock import patch

from server_runtime import EngineDaemonClient


class TestDaemonClientJournalRequests(unittest.TestCase):
    @patch("server_runtime.subprocess.run")
    def test_gets_journal_list_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"project_transaction_journal_list_v1","count":1}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.get_journal_list("/tmp/native-project")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "journal",
                "list",
                "/tmp/native-project",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "project_transaction_journal_list_v1")
        self.assertEqual(response.result["count"], 1)

    @patch("server_runtime.subprocess.run")
    def test_gets_journal_transaction_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"project_transaction_journal_record_v1"}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.get_journal_transaction("/tmp/native-project", "txn-test")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "journal",
                "show",
                "/tmp/native-project",
                "--transaction",
                "txn-test",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "project_transaction_journal_record_v1")

    @patch("server_runtime.subprocess.run")
    def test_applies_journal_undo_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"project_transaction_journal_mutation_v1","action":"undo"}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.journal_undo("/tmp/native-project")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "journal",
                "undo",
                "/tmp/native-project",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "undo")

    @patch("server_runtime.subprocess.run")
    def test_applies_guarded_journal_undo_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"project_transaction_journal_mutation_v1","action":"undo"}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.journal_undo(
            "/tmp/native-project",
            expected_model_revision="model-rev",
            expected_tip_transaction="txn-tip",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "journal",
                "undo",
                "/tmp/native-project",
                "--expected-model-revision",
                "model-rev",
                "--expected-tip-transaction",
                "txn-tip",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "undo")

    @patch("server_runtime.subprocess.run")
    def test_applies_journal_redo_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"project_transaction_journal_mutation_v1","action":"redo"}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.journal_redo("/tmp/native-project")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "journal",
                "redo",
                "/tmp/native-project",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "redo")

    @patch("server_runtime.subprocess.run")
    def test_applies_guarded_journal_redo_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"project_transaction_journal_mutation_v1","action":"redo"}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.journal_redo(
            "/tmp/native-project",
            expected_model_revision="model-rev",
            expected_tip_transaction="txn-tip",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "journal",
                "redo",
                "/tmp/native-project",
                "--expected-model-revision",
                "model-rev",
                "--expected-tip-transaction",
                "txn-tip",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "redo")
