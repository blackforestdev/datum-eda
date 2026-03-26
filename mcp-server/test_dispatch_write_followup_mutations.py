#!/usr/bin/env python3
"""Additional stateful MCP write-mutation follow-up behavior tests."""

from __future__ import annotations

import unittest

from server_runtime import JsonRpcResponse, StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchWriteFollowupMutations(unittest.TestCase):
    def test_tools_call_set_package_changes_followup_net_info_response(self) -> None:
        class StatefulDaemon(FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.pin_count = 2

            def set_package(self, uuid: str, package_uuid: str) -> JsonRpcResponse:
                response = super().set_package(uuid, package_uuid)
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
                "id": 2431,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        before_nets = before["result"]["content"][0]["json"]
        self.assertEqual(len(before_nets[0]["pins"]), 2)

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2432,
                "method": "tools/call",
                "params": {
                    "name": "set_package",
                    "arguments": {"uuid": "comp-1", "package_uuid": "package-1"},
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2433,
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
                ("set_package", "comp-1", "package-1"),
                ("get_net_info", None),
            ],
        )

    def test_tools_call_set_net_class_changes_followup_net_info_response(self) -> None:
        class StatefulDaemon(FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.net_class = "Default"

            def set_net_class(
                self,
                net_uuid: str,
                class_name: str,
                clearance: int,
                track_width: int,
                via_drill: int,
                via_diameter: int,
                diffpair_width: int = 0,
                diffpair_gap: int = 0,
            ) -> JsonRpcResponse:
                response = super().set_net_class(
                    net_uuid,
                    class_name,
                    clearance,
                    track_width,
                    via_drill,
                    via_diameter,
                    diffpair_width,
                    diffpair_gap,
                )
                self.net_class = class_name
                return response

            def get_net_info(self) -> JsonRpcResponse:
                self.calls.append(("get_net_info", None))
                return JsonRpcResponse(
                    "2.0",
                    29,
                    [
                        {
                            "uuid": "net-1",
                            "name": "GND",
                            "class": self.net_class,
                            "pins": [],
                            "tracks": 1,
                            "vias": 1,
                            "zones": 0,
                            "routed_length_nm": 1000000,
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
                "id": 246,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        before_nets = before["result"]["content"][0]["json"]
        self.assertEqual(before_nets[0]["class"], "Default")

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 247,
                "method": "tools/call",
                "params": {
                    "name": "set_net_class",
                    "arguments": {
                        "net_uuid": "net-1",
                        "class_name": "power",
                        "clearance": 125000,
                        "track_width": 250000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                    },
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 248,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        after_nets = after["result"]["content"][0]["json"]
        self.assertEqual(after_nets[0]["class"], "power")
        self.assertEqual(
            daemon.calls,
            [
                ("get_net_info", None),
                (
                    "set_net_class",
                    "net-1",
                    "power",
                    125000,
                    250000,
                    300000,
                    600000,
                    0,
                    0,
                ),
                ("get_net_info", None),
            ],
        )

    def test_tools_call_set_reference_changes_followup_components_response(self) -> None:
        class StatefulDaemon(FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.reference = "R1"

            def set_reference(self, uuid: str, reference: str) -> JsonRpcResponse:
                response = super().set_reference(uuid, reference)
                self.reference = reference
                return response

            def get_components(self) -> JsonRpcResponse:
                self.calls.append(("get_components", None))
                return JsonRpcResponse(
                    "2.0",
                    25,
                    [
                        {
                            "uuid": "comp-1",
                            "reference": self.reference,
                            "value": "10k",
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
                "id": 227,
                "method": "tools/call",
                "params": {"name": "get_components", "arguments": {}},
            }
        )
        before_components = before["result"]["content"][0]["json"]
        self.assertEqual(before_components[0]["reference"], "R1")

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 228,
                "method": "tools/call",
                "params": {
                    "name": "set_reference",
                    "arguments": {"uuid": "comp-1", "reference": "R10"},
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 229,
                "method": "tools/call",
                "params": {"name": "get_components", "arguments": {}},
            }
        )
        after_components = after["result"]["content"][0]["json"]
        self.assertEqual(after_components[0]["reference"], "R10")
        self.assertEqual(
            daemon.calls,
            [
                ("get_components", None),
                ("set_reference", "comp-1", "R10"),
                ("get_components", None),
            ],
        )
