#!/usr/bin/env python3
"""Stateful MCP tools/call behavior tests."""

from __future__ import annotations

import unittest

from server_runtime import JsonRpcResponse, StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchReadSurface(unittest.TestCase):
    def test_tools_call_dispatches_search_pool(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 201,
                "method": "tools/call",
                "params": {
                    "name": "search_pool",
                    "arguments": {"query": "sot23"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("search_pool", "sot23")])
        self.assertEqual(response["result"]["content"][0]["json"][0]["package"], "SOT23")

    def test_tools_call_dispatches_get_part(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 202,
                "method": "tools/call",
                "params": {
                    "name": "get_part",
                    "arguments": {"uuid": "part-uuid"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_part", "part-uuid")])
        self.assertEqual(response["result"]["content"][0]["json"]["mpn"], "MMBT3904")

    def test_tools_call_dispatches_get_package(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 203,
                "method": "tools/call",
                "params": {
                    "name": "get_package",
                    "arguments": {"uuid": "package-uuid"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_package", "package-uuid")])
        self.assertEqual(response["result"]["content"][0]["json"]["name"], "SOT23")

    def test_tools_call_dispatches_get_package_change_candidates(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2031,
                "method": "tools/call",
                "params": {
                    "name": "get_package_change_candidates",
                    "arguments": {"uuid": "comp-uuid"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_package_change_candidates", "comp-uuid")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["status"], "candidates_available")
        self.assertEqual(payload["candidates"][0]["package_name"], "ALT-3")

    def test_tools_call_dispatches_get_part_change_candidates(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2032,
                "method": "tools/call",
                "params": {
                    "name": "get_part_change_candidates",
                    "arguments": {"uuid": "comp-uuid"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_part_change_candidates", "comp-uuid")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["status"], "candidates_available")
        self.assertEqual(payload["candidates"][0]["value"], "ALTAMP")

    def test_tools_call_dispatches_get_component_replacement_plan(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2033,
                "method": "tools/call",
                "params": {
                    "name": "get_component_replacement_plan",
                    "arguments": {"uuid": "comp-uuid"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_component_replacement_plan", "comp-uuid")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["current_reference"], "R1")
        self.assertEqual(payload["package_change"]["status"], "candidates_available")
        self.assertEqual(payload["part_change"]["status"], "candidates_available")

    def test_tools_call_dispatches_get_scoped_component_replacement_plan(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2034,
                "method": "tools/call",
                "params": {
                    "name": "get_scoped_component_replacement_plan",
                    "arguments": {
                        "scope": {
                            "reference_prefix": "R",
                            "value_equals": "LMV321",
                        },
                        "policy": "best_compatible_package",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "get_scoped_component_replacement_plan",
                    {"reference_prefix": "R", "value_equals": "LMV321"},
                    "best_compatible_package",
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["policy"], "best_compatible_package")
        self.assertEqual(payload["replacements"][0]["current_reference"], "R1")
        self.assertEqual(payload["replacements"][0]["target_package_name"], "ALT-3")

    def test_tools_call_dispatches_edit_scoped_component_replacement_plan(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        plan = {
            "scope": {"reference_prefix": "R", "value_equals": "LMV321"},
            "policy": "best_compatible_package",
            "replacements": [
                {
                    "component_uuid": "comp-1",
                    "current_reference": "R1",
                    "current_value": "LMV321",
                    "current_part_uuid": "part-uuid",
                    "current_package_uuid": "package-uuid",
                    "target_part_uuid": "alt-part-uuid",
                    "target_package_uuid": "alt-package-uuid",
                    "target_value": "ALTAMP",
                    "target_package_name": "ALT-3",
                },
                {
                    "component_uuid": "comp-2",
                    "current_reference": "R2",
                    "current_value": "LMV321",
                    "current_part_uuid": "part-uuid",
                    "current_package_uuid": "package-uuid",
                    "target_part_uuid": "alt-part-uuid",
                    "target_package_uuid": "alt-package-uuid",
                    "target_value": "ALTAMP",
                    "target_package_name": "ALT-3",
                },
            ],
        }
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2035,
                "method": "tools/call",
                "params": {
                    "name": "edit_scoped_component_replacement_plan",
                    "arguments": {
                        "plan": plan,
                        "exclude_component_uuids": ["comp-2"],
                        "overrides": [
                            {
                                "component_uuid": "comp-1",
                                "target_package_uuid": "alt-package-uuid",
                                "target_part_uuid": "alt-part-uuid",
                            }
                        ],
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "edit_scoped_component_replacement_plan",
                    plan,
                    ["comp-2"],
                    [
                        {
                            "component_uuid": "comp-1",
                            "target_package_uuid": "alt-package-uuid",
                            "target_part_uuid": "alt-part-uuid",
                        }
                    ],
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(len(payload["replacements"]), 1)
        self.assertEqual(payload["replacements"][0]["component_uuid"], "comp-1")

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

    def test_tools_call_dispatches_check_report_with_input_without_explicit_driver(self) -> None:
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
        self.assertEqual(payload["summary"]["by_code"][0]["code"], "input_without_explicit_driver")
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
