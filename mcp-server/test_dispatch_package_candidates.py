#!/usr/bin/env python3
"""Focused MCP package-compatibility behavior tests."""

from __future__ import annotations

import unittest

from server_runtime import JsonRpcResponse, StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchPackageCandidates(unittest.TestCase):
    def test_tools_call_set_package_preserves_logical_nets_across_known_part_remap_response(
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
                return response

            def set_package(self, uuid: str, package_uuid: str) -> JsonRpcResponse:
                response = super().set_package(uuid, package_uuid)
                if package_uuid == "altamp-package":
                    self.pin_count = 2
                return response

            def get_net_info(self) -> JsonRpcResponse:
                self.calls.append(("get_net_info", None))
                return JsonRpcResponse(
                    "2.0",
                    39,
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
                "id": 2434,
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
                "id": 2435,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        intermediate_nets = intermediate["result"]["content"][0]["json"]
        self.assertEqual(len(intermediate_nets[0]["pins"]), 2)

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2436,
                "method": "tools/call",
                "params": {
                    "name": "set_package",
                    "arguments": {"uuid": "comp-1", "package_uuid": "altamp-package"},
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2437,
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
                ("set_package", "comp-1", "altamp-package"),
                ("get_net_info", None),
            ],
        )

    def test_tools_call_set_package_with_part_preserves_logical_nets_for_explicit_candidate(
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
                return response

            def set_package_with_part(
                self, uuid: str, package_uuid: str, part_uuid: str
            ) -> JsonRpcResponse:
                response = super().set_package_with_part(uuid, package_uuid, part_uuid)
                if package_uuid == "altamp-package" and part_uuid == "altamp-part":
                    self.pin_count = 2
                return response

            def get_net_info(self) -> JsonRpcResponse:
                self.calls.append(("get_net_info", None))
                return JsonRpcResponse(
                    "2.0",
                    40,
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
                "id": 2534,
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
                "id": 2535,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        intermediate_nets = intermediate["result"]["content"][0]["json"]
        self.assertEqual(len(intermediate_nets[0]["pins"]), 2)

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2536,
                "method": "tools/call",
                "params": {
                    "name": "set_package_with_part",
                    "arguments": {
                        "uuid": "comp-1",
                        "package_uuid": "altamp-package",
                        "part_uuid": "altamp-part",
                    },
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2537,
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
                (
                    "set_package_with_part",
                    "comp-1",
                    "altamp-package",
                    "altamp-part",
                ),
                ("get_net_info", None),
            ],
        )

    def test_tools_call_replace_component_preserves_logical_nets_for_explicit_candidate(
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
                return response

            def replace_component(
                self, uuid: str, package_uuid: str, part_uuid: str
            ) -> JsonRpcResponse:
                response = super().replace_component(uuid, package_uuid, part_uuid)
                if package_uuid == "altamp-package" and part_uuid == "altamp-part":
                    self.pin_count = 2
                return response

            def get_net_info(self) -> JsonRpcResponse:
                self.calls.append(("get_net_info", None))
                return JsonRpcResponse(
                    "2.0",
                    41,
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
                "id": 2634,
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
                "id": 2635,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        intermediate_nets = intermediate["result"]["content"][0]["json"]
        self.assertEqual(len(intermediate_nets[0]["pins"]), 2)

        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2636,
                "method": "tools/call",
                "params": {
                    "name": "replace_component",
                    "arguments": {
                        "uuid": "comp-1",
                        "package_uuid": "altamp-package",
                        "part_uuid": "altamp-part",
                    },
                },
            }
        )
        self.assertEqual(
            response["result"]["content"][0]["json"]["description"],
            "replace_component comp-1",
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2637,
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
                ("replace_component", "comp-1", "altamp-package", "altamp-part"),
                ("get_net_info", None),
            ],
        )

    def test_tools_call_replace_components_batches_into_one_undo_step(self) -> None:
        class StatefulDaemon(FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.values = {"comp-1": "10k", "comp-2": "10k"}

            def replace_components(self, replacements: list[dict[str, str]]) -> JsonRpcResponse:
                response = super().replace_components(replacements)
                for replacement in replacements:
                    self.values[replacement["uuid"]] = "ALTAMP"
                return response

            def undo(self) -> JsonRpcResponse:
                response = super().undo()
                self.values = {"comp-1": "10k", "comp-2": "10k"}
                response.result["description"] = "undo replace_components 2"
                return response

            def get_components(self) -> JsonRpcResponse:
                self.calls.append(("get_components", None))
                return JsonRpcResponse(
                    "2.0",
                    42,
                    [
                        {"uuid": uuid, "reference": ref, "value": value}
                        for uuid, ref, value in (
                            ("comp-1", "R1", self.values["comp-1"]),
                            ("comp-2", "R2", self.values["comp-2"]),
                        )
                    ],
                    None,
                )

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2638,
                "method": "tools/call",
                "params": {
                    "name": "replace_components",
                    "arguments": {
                        "replacements": [
                            {
                                "uuid": "comp-1",
                                "package_uuid": "altamp-package",
                                "part_uuid": "altamp-part",
                            },
                            {
                                "uuid": "comp-2",
                                "package_uuid": "altamp-package",
                                "part_uuid": "altamp-part",
                            },
                        ]
                    },
                },
            }
        )
        self.assertEqual(
            response["result"]["content"][0]["json"]["description"],
            "replace_components 2",
        )
        undo = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2639,
                "method": "tools/call",
                "params": {"name": "undo", "arguments": {}},
            }
        )
        self.assertEqual(
            undo["result"]["content"][0]["json"]["description"],
            "undo replace_components 2",
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2640,
                "method": "tools/call",
                "params": {"name": "get_components", "arguments": {}},
            }
        )
        values = [component["value"] for component in after["result"]["content"][0]["json"]]
        self.assertEqual(values, ["10k", "10k"])
        self.assertEqual(
            daemon.calls,
            [
                (
                    "replace_components",
                    [
                        {
                            "uuid": "comp-1",
                            "package_uuid": "altamp-package",
                            "part_uuid": "altamp-part",
                        },
                        {
                            "uuid": "comp-2",
                            "package_uuid": "altamp-package",
                            "part_uuid": "altamp-part",
                        },
                    ],
                ),
                ("undo", None),
                ("get_components", None),
            ],
        )
