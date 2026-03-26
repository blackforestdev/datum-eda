#!/usr/bin/env python3
"""Read-surface MCP dispatch tests for check and query endpoints."""

from __future__ import annotations

import unittest

from server_runtime import JsonRpcResponse, StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchReadSurfaceQueries(unittest.TestCase):
    def test_tools_call_dispatches_check_report(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "get_check_report",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_check_report", None)])
        self.assertEqual(response["result"]["content"][0]["json"]["domain"], "board")
        self.assertEqual(response["result"]["content"][0]["json"]["summary"]["status"], "warning")

    def test_tools_call_dispatches_check_report_with_input_without_explicit_driver(
        self,
    ) -> None:
        class AnalogCheckDaemon(FakeDaemonClient):
            def get_check_report(self) -> JsonRpcResponse:
                self.calls.append(("get_check_report", None))
                return JsonRpcResponse(
                    "2.0",
                    2,
                    {
                        "domain": "schematic",
                        "summary": {
                            "status": "info",
                            "errors": 0,
                            "warnings": 0,
                            "infos": 1,
                            "waived": 0,
                            "by_code": [
                                {"code": "input_without_explicit_driver", "count": 1},
                            ],
                        },
                        "diagnostics": [],
                        "erc": [
                            {
                                "code": "input_without_explicit_driver",
                                "severity": "Info",
                                "net_name": "IN_P",
                            }
                        ],
                    },
                    None,
                )

        daemon = AnalogCheckDaemon()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 31,
                "method": "tools/call",
                "params": {
                    "name": "get_check_report",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_check_report", None)])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["domain"], "schematic")
        self.assertEqual(payload["summary"]["status"], "info")
        self.assertEqual(
            payload["summary"]["by_code"][0]["code"], "input_without_explicit_driver"
        )
        self.assertEqual(payload["erc"][0]["code"], "input_without_explicit_driver")

    def test_tools_call_dispatches_board_summary(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {
                    "name": "get_board_summary",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_board_summary", None)])
        self.assertEqual(response["result"]["content"][0]["json"]["name"], "simple-demo")

    def test_tools_call_dispatches_schematic_summary(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 5,
                "method": "tools/call",
                "params": {
                    "name": "get_schematic_summary",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_schematic_summary", None)])
        self.assertEqual(response["result"]["content"][0]["json"]["sheet_count"], 1)

    def test_tools_call_dispatches_sheets(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 51,
                "method": "tools/call",
                "params": {
                    "name": "get_sheets",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_sheets", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["name"], "Root")

    def test_tools_call_dispatches_components(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 50,
                "method": "tools/call",
                "params": {
                    "name": "get_components",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_components", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["reference"], "R1")

    def test_tools_call_dispatches_symbol_fields(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 53,
                "method": "tools/call",
                "params": {
                    "name": "get_symbol_fields",
                    "arguments": {"symbol_uuid": "abcd"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_symbol_fields", "abcd")])
        self.assertEqual(response["result"]["content"][0]["json"][0]["key"], "Reference")

    def test_tools_call_dispatches_netlist(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 52,
                "method": "tools/call",
                "params": {
                    "name": "get_netlist",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_netlist", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["name"], "GND")

    def test_tools_call_dispatches_bus_entries(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 54,
                "method": "tools/call",
                "params": {
                    "name": "get_bus_entries",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_bus_entries", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["uuid"], "be1")

    def test_tools_call_dispatches_board_net_info(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 6,
                "method": "tools/call",
                "params": {
                    "name": "get_net_info",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_net_info", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["name"], "GND")

    def test_tools_call_dispatches_schematic_net_info(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 7,
                "method": "tools/call",
                "params": {
                    "name": "get_schematic_net_info",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_schematic_net_info", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["name"], "SCL")

    def test_tools_call_dispatches_unrouted(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 8,
                "method": "tools/call",
                "params": {
                    "name": "get_unrouted",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_unrouted", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["net_name"], "SIG")

    def test_tools_call_dispatches_connectivity_diagnostics(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 76,
                "method": "tools/call",
                "params": {
                    "name": "get_connectivity_diagnostics",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_connectivity_diagnostics", None)])
        diagnostics = response["result"]["content"][0]["json"]
        self.assertEqual(len(diagnostics), 2)
        self.assertTrue(
            any(diagnostic["kind"] == "partially_routed_net" for diagnostic in diagnostics)
        )

    def test_tools_call_dispatches_design_rules(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 202,
                "method": "tools/call",
                "params": {
                    "name": "get_design_rules",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_design_rules", None)])
        self.assertEqual(response["result"]["content"][0]["json"], [])
