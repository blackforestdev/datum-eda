#!/usr/bin/env python3
"""Canonical hierarchy query MCP dispatch tests."""

from __future__ import annotations

import subprocess
import unittest
from unittest.mock import patch

from server_runtime import EngineDaemonClient, JsonRpcResponse, StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchQueryHierarchy(unittest.TestCase):
    def test_datum_query_hierarchy_uses_native_project_path(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)

        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 910,
                "method": "tools/call",
                "params": {
                    "name": "datum.query.hierarchy",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )

        self.assertEqual(daemon.calls, [("get_project_hierarchy", "/tmp/native-project")])
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(payload["schema"], {"name": "datum.query.hierarchy", "version": 1})
        self.assertEqual(payload["result"]["instances"][0]["name"], "Main Instance")

    def test_datum_query_hierarchy_without_path_preserves_legacy_session_query(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)

        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 911,
                "method": "tools/call",
                "params": {"name": "datum.query.hierarchy", "arguments": {}},
            }
        )

        self.assertEqual(daemon.calls, [("get_project_hierarchy", None)])
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(payload["result"]["instances"][0]["name"], "Main Instance")

    def test_runtime_project_hierarchy_without_path_calls_legacy_query(self) -> None:
        client = EngineDaemonClient()
        calls = []

        def legacy_hierarchy() -> JsonRpcResponse:
            calls.append("legacy")
            return JsonRpcResponse("2.0", 1, {"instances": [{"name": "child"}]}, None)

        client.get_hierarchy = legacy_hierarchy
        response = client.get_project_hierarchy()

        self.assertEqual(calls, ["legacy"])
        self.assertEqual(response.result["instances"][0]["name"], "child")

    @patch("server_runtime.subprocess.run")
    def test_runtime_project_hierarchy_uses_project_query_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"instances":[{"name":"Main Instance"}],"links":[]}',
            stderr="",
        )

        response = EngineDaemonClient().get_project_hierarchy("/tmp/native-project")

        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "query",
                "/tmp/native-project",
                "hierarchy",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["instances"][0]["name"], "Main Instance")


if __name__ == "__main__":
    unittest.main()
