#!/usr/bin/env python3
"""MCP explain_violation dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchExplainViolation(unittest.TestCase):
    def test_tools_call_dispatches_explain_violation_by_index(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 78,
                "method": "tools/call",
                "params": {
                    "name": "explain_violation",
                    "arguments": {"domain": "drc", "index": 0},
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("explain_violation", {"domain": "drc", "index": 0, "fingerprint": None})],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["rule_detail"], "drc connectivity_unrouted_net")

    def test_tools_call_dispatches_explain_violation_by_fingerprint(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 79,
                "method": "tools/call",
                "params": {
                    "name": "explain_violation",
                    "arguments": {"domain": "drc", "fingerprint": "sha256:finding"},
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "explain_violation",
                    {"domain": "drc", "index": None, "fingerprint": "sha256:finding"},
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["fingerprint"], "sha256:finding")

    def test_tools_call_dispatches_canonical_explain_violation_by_index(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 80,
                "method": "tools/call",
                "params": {
                    "name": "datum.check.explain_violation",
                    "arguments": {"domain": "drc", "index": 0},
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("explain_violation", {"domain": "drc", "index": 0, "fingerprint": None})],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(
            payload["schema"], {"name": "datum.check.explain_violation", "version": 1}
        )
        self.assertEqual(payload["result"]["rule_detail"], "drc connectivity_unrouted_net")

    def test_tools_call_dispatches_canonical_explain_violation_by_fingerprint(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 81,
                "method": "tools/call",
                "params": {
                    "name": "datum.check.explain_violation",
                    "arguments": {"domain": "drc", "fingerprint": "sha256:finding"},
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "explain_violation",
                    {"domain": "drc", "index": None, "fingerprint": "sha256:finding"},
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(payload["result"]["fingerprint"], "sha256:finding")


if __name__ == "__main__":
    unittest.main()
