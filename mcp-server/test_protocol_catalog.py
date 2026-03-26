#!/usr/bin/env python3
"""MCP protocol envelope and tools/list catalog tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestProtocolCatalog(unittest.TestCase):
    def test_tools_list_returns_registered_tools(self) -> None:
        host = StdioToolHost(FakeDaemonClient())
        response = host.handle_message({"jsonrpc": "2.0", "id": 1, "method": "tools/list"})
        self.assertIn("result", response)
        tools = response["result"]["tools"]
        self.assertEqual([tool["name"] for tool in tools], [
            "open_project",
            "close_project",
            "save",
            "delete_track",
            "delete_component",
            "move_component",
            "rotate_component",
            "set_value",
            "assign_part",
            "set_package",
            "set_reference",
            "set_net_class",
            "delete_via",
            "set_design_rule",
            "undo",
            "redo",
            "search_pool",
            "get_part",
            "get_package",
            "get_components",
            "get_netlist",
            "get_check_report",
            "get_net_info",
            "get_unrouted",
            "get_schematic_net_info",
            "get_board_summary",
            "get_schematic_summary",
            "get_sheets",
            "get_labels",
            "get_symbols",
            "get_symbol_fields",
            "get_ports",
            "get_buses",
            "get_bus_entries",
            "get_hierarchy",
            "get_noconnects",
            "get_connectivity_diagnostics",
            "get_design_rules",
            "run_erc",
            "run_drc",
            "explain_violation",
        ])

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

