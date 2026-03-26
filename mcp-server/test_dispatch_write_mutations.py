#!/usr/bin/env python3
"""Stateful MCP tools/call behavior tests."""

from __future__ import annotations

from typing import Any
import unittest

from server_runtime import JsonRpcResponse, StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchWriteMutations(unittest.TestCase):
    def test_tools_call_set_design_rule_changes_followup_design_rules_response(self) -> None:
        class StatefulDaemon(FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.rules: list[dict[str, Any]] = []

            def set_design_rule(
                self,
                rule_type: str,
                scope: dict[str, Any] | str,
                parameters: dict[str, Any],
                priority: int,
                name: str | None,
            ) -> JsonRpcResponse:
                response = super().set_design_rule(
                    rule_type, scope, parameters, priority, name
                )
                self.rules = [
                    {
                        "uuid": "rule-1",
                        "name": name or "default clearance",
                        "scope": scope,
                        "priority": priority,
                        "enabled": True,
                        "rule_type": rule_type,
                        "parameters": parameters,
                    }
                ]
                return response

            def get_design_rules(self) -> JsonRpcResponse:
                self.calls.append(("get_design_rules", None))
                return JsonRpcResponse("2.0", 102, self.rules, None)

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        before = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 217,
                "method": "tools/call",
                "params": {"name": "get_design_rules", "arguments": {}},
            }
        )
        self.assertEqual(before["result"]["content"][0]["json"], [])

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 218,
                "method": "tools/call",
                "params": {
                    "name": "set_design_rule",
                    "arguments": {
                        "rule_type": "ClearanceCopper",
                        "scope": "All",
                        "parameters": {"Clearance": {"min": 125000}},
                        "priority": 10,
                        "name": "default clearance",
                    },
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 219,
                "method": "tools/call",
                "params": {"name": "get_design_rules", "arguments": {}},
            }
        )
        rules = after["result"]["content"][0]["json"]
        self.assertEqual(len(rules), 1)
        self.assertEqual(rules[0]["name"], "default clearance")
        self.assertEqual(
            daemon.calls,
            [
                ("get_design_rules", None),
                (
                    "set_design_rule",
                    "ClearanceCopper",
                    "All",
                    {"Clearance": {"min": 125000}},
                    10,
                    "default clearance",
                ),
                ("get_design_rules", None),
            ],
        )

    def test_tools_call_set_value_changes_followup_components_response(self) -> None:
        class StatefulDaemon(FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.value = "10k"

            def set_value(self, uuid: str, value: str) -> JsonRpcResponse:
                response = super().set_value(uuid, value)
                self.value = value
                return response

            def get_components(self) -> JsonRpcResponse:
                self.calls.append(("get_components", None))
                return JsonRpcResponse(
                    "2.0",
                    24,
                    [
                        {
                            "uuid": "comp-1",
                            "reference": "R1",
                            "value": self.value,
                            "position": {"x": 10_000_000, "y": 10_000_000},
                            "rotation": 0,
                            "layer": 0,
                            "locked": False,
                        }
                    ],
                    None,
                )

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        before = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 224,
                "method": "tools/call",
                "params": {"name": "get_components", "arguments": {}},
            }
        )
        before_components = before["result"]["content"][0]["json"]
        self.assertEqual(before_components[0]["value"], "10k")

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 225,
                "method": "tools/call",
                "params": {
                    "name": "set_value",
                    "arguments": {"uuid": "comp-1", "value": "22k"},
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 226,
                "method": "tools/call",
                "params": {"name": "get_components", "arguments": {}},
            }
        )
        after_components = after["result"]["content"][0]["json"]
        self.assertEqual(after_components[0]["value"], "22k")
        self.assertEqual(
            daemon.calls,
            [
                ("get_components", None),
                ("set_value", "comp-1", "22k"),
                ("get_components", None),
            ],
        )

    def test_tools_call_assign_part_changes_followup_net_info_response(self) -> None:
        class StatefulDaemon(FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.pin_count = 2

            def assign_part(self, uuid: str, part_uuid: str) -> JsonRpcResponse:
                response = super().assign_part(uuid, part_uuid)
                self.pin_count = 1
                return response

            def get_net_info(self) -> JsonRpcResponse:
                self.calls.append(("get_net_info", None))
                return JsonRpcResponse(
                    "2.0",
                    28,
                    [
                        {
                            "uuid": "net-1",
                            "name": "SIG",
                            "class": "Default",
                            "pins": [
                                {"component": "R1", "pin": "1"}
                                for _ in range(self.pin_count)
                            ],
                            "tracks": 1,
                            "vias": 0,
                            "zones": 0,
                            "routed_length_nm": 11000000,
                            "routed_pct": 1.0,
                        }
                    ],
                    None,
                )

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        before = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 243,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        before_nets = before["result"]["content"][0]["json"]
        self.assertEqual(len(before_nets[0]["pins"]), 2)

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 244,
                "method": "tools/call",
                "params": {
                    "name": "assign_part",
                    "arguments": {"uuid": "comp-1", "part_uuid": "part-1"},
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 245,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        after_nets = after["result"]["content"][0]["json"]
        self.assertEqual(len(after_nets[0]["pins"]), 1)
        self.assertEqual(
            daemon.calls,
            [
                ("get_net_info", None),
                ("assign_part", "comp-1", "part-1"),
                ("get_net_info", None),
            ],
        )

    def test_tools_call_assign_part_preserves_logical_nets_across_known_part_remap_response(
        self,
    ) -> None:
        class StatefulDaemon(FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.pin_count = 0

            def assign_part(self, uuid: str, part_uuid: str) -> JsonRpcResponse:
                response = super().assign_part(uuid, part_uuid)
                if part_uuid == "lmv321-part":
                    self.pin_count = 2
                elif part_uuid == "altamp-part":
                    self.pin_count = 2
                return response

            def get_net_info(self) -> JsonRpcResponse:
                self.calls.append(("get_net_info", None))
                return JsonRpcResponse(
                    "2.0",
                    38,
                    [
                        {
                            "uuid": "net-1",
                            "name": "SIG",
                            "class": "Default",
                            "pins": [
                                {"component": "R1", "pin": "1"}
                                for _ in range(self.pin_count)
                            ],
                            "tracks": 1,
                            "vias": 0,
                            "zones": 0,
                            "routed_length_nm": 11000000,
                            "routed_pct": 1.0,
                        }
                    ],
                    None,
                )

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2451,
                "method": "tools/call",
                "params": {
                    "name": "assign_part",
                    "arguments": {"uuid": "comp-1", "part_uuid": "lmv321-part"},
                },
            }
        )
        intermediate = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2452,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        intermediate_nets = intermediate["result"]["content"][0]["json"]
        self.assertEqual(len(intermediate_nets[0]["pins"]), 2)

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2453,
                "method": "tools/call",
                "params": {
                    "name": "assign_part",
                    "arguments": {"uuid": "comp-1", "part_uuid": "altamp-part"},
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2454,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        after_nets = after["result"]["content"][0]["json"]
        self.assertEqual(len(after_nets[0]["pins"]), 2)
        self.assertEqual(
            daemon.calls,
            [
                ("assign_part", "comp-1", "lmv321-part"),
                ("get_net_info", None),
                ("assign_part", "comp-1", "altamp-part"),
                ("get_net_info", None),
            ],
        )
