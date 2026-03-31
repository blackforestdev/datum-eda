#!/usr/bin/env python3
"""MCP protocol envelope and tools/list catalog tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient
from tool_dispatch import registered_tool_names
from tools_catalog_data import TOOLS


class TestProtocolCatalog(unittest.TestCase):
    def test_tools_list_returns_registered_tools(self) -> None:
        host = StdioToolHost(FakeDaemonClient())
        response = host.handle_message({"jsonrpc": "2.0", "id": 1, "method": "tools/list"})
        self.assertIn("result", response)
        tools = response["result"]["tools"]
        self.assertEqual([tool["name"] for tool in tools], [tool["name"] for tool in TOOLS])

    def test_catalog_and_dispatch_share_the_same_registered_names(self) -> None:
        self.assertEqual(registered_tool_names(), [tool["name"] for tool in TOOLS])

    def test_initialize_returns_server_info_and_capabilities(self) -> None:
        host = StdioToolHost(FakeDaemonClient())
        response = host.handle_message({"jsonrpc": "2.0", "id": 1, "method": "initialize"})
        assert isinstance(response, dict)
        self.assertEqual(response["jsonrpc"], "2.0")
        self.assertEqual(response["id"], 1)
        self.assertEqual(response["result"]["protocolVersion"], "2024-11-05")
        self.assertEqual(response["result"]["serverInfo"]["name"], "datum-eda")
        self.assertIn("tools", response["result"]["capabilities"])

    def test_ping_returns_empty_result(self) -> None:
        host = StdioToolHost(FakeDaemonClient())
        response = host.handle_message({"jsonrpc": "2.0", "id": 7, "method": "ping"})
        assert isinstance(response, dict)
        self.assertEqual(response, {"jsonrpc": "2.0", "id": 7, "result": {}})

    def test_initialized_notification_returns_no_response(self) -> None:
        host = StdioToolHost(FakeDaemonClient())
        response = host.handle_message(
            {"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}}
        )
        self.assertIsNone(response)
