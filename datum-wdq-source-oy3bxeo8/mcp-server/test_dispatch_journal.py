#!/usr/bin/env python3
"""Native project journal MCP tools/call dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchJournal(unittest.TestCase):
    def test_tools_call_dispatches_get_journal_list(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 111,
                "method": "tools/call",
                "params": {
                    "name": "get_journal_list",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_journal_list", "/tmp/native-project")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "project_transaction_journal_list_v1")
        self.assertEqual(payload["count"], 1)

    def test_tools_call_dispatches_get_journal_transaction(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 112,
                "method": "tools/call",
                "params": {
                    "name": "get_journal_transaction",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "transaction": "txn-test",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("get_journal_transaction", "/tmp/native-project", "txn-test")],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "project_transaction_journal_record_v1")
        self.assertEqual(payload["transaction"]["transaction_id"], "txn-test")

    def test_tools_call_dispatches_journal_undo_and_redo(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        undo_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 113,
                "method": "tools/call",
                "params": {
                    "name": "journal_undo",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "expected_tip_transaction": "txn-tip",
                    },
                },
            }
        )
        redo_response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 114,
                "method": "tools/call",
                "params": {
                    "name": "journal_redo",
                    "arguments": {
                        "path": "/tmp/native-project",
                        "expected_model_revision": "model-rev",
                        "expected_tip_transaction": "txn-tip-redo",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                ("journal_undo", "/tmp/native-project", None, "txn-tip"),
                ("journal_redo", "/tmp/native-project", "model-rev", "txn-tip-redo"),
            ],
        )
        self.assertEqual(undo_response["result"]["content"][0]["json"]["action"], "undo")
        self.assertEqual(redo_response["result"]["content"][0]["json"]["action"], "redo")
        self.assertTrue(undo_response["result"]["content"][0]["json"]["guard"]["checked"])
        self.assertEqual(
            redo_response["result"]["content"][0]["json"]["guard"]["expected_model_revision"],
            "model-rev",
        )

    def test_tools_call_dispatches_unguarded_journal_undo(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 115,
                "method": "tools/call",
                "params": {
                    "name": "journal_undo",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("journal_undo", "/tmp/native-project", None, None)])
        self.assertEqual(response["result"]["content"][0]["json"]["action"], "undo")
