#!/usr/bin/env python3
"""Import map MCP tools/call dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient, decode_tool_payload


class TestDispatchImportMap(unittest.TestCase):
    def test_tools_call_dispatches_get_import_map(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 203,
                "method": "tools/call",
                "params": {
                    "name": "get_import_map",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_import_map", "/tmp/native-project")])
        payload = decode_tool_payload(response)
        self.assertEqual(payload["contract"], "import_map_query_v1")
        self.assertEqual(payload["import_map_count"], 1)
