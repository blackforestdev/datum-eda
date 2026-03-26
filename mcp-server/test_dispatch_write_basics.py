#!/usr/bin/env python3
"""Stateful MCP tools/call behavior tests."""

from __future__ import annotations

from typing import Any
import unittest

from server_runtime import JsonRpcResponse, StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchWriteBasics(unittest.TestCase):
    def test_tools_call_dispatches_move_component(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 210,
                "method": "tools/call",
                "params": {
                    "name": "move_component",
                    "arguments": {
                        "uuid": "comp-1",
                        "x_mm": 15.0,
                        "y_mm": 12.0,
                        "rotation_deg": 90.0,
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("move_component", "comp-1", 15.0, 12.0, 90.0)],
        )
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "component",
        )

    def test_tools_call_dispatches_rotate_component(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 221,
                "method": "tools/call",
                "params": {
                    "name": "rotate_component",
                    "arguments": {
                        "uuid": "comp-1",
                        "rotation_deg": 180.0,
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("rotate_component", "comp-1", 180.0)],
        )
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "component",
        )

    def test_tools_call_dispatches_set_value(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 223,
                "method": "tools/call",
                "params": {
                    "name": "set_value",
                    "arguments": {
                        "uuid": "comp-1",
                        "value": "22k",
                    },
                },
            }
        )
        self.assertEqual(daemon.calls, [("set_value", "comp-1", "22k")])
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "component",
        )

    def test_tools_call_move_component_changes_followup_unrouted_response(self) -> None:
        class StatefulDaemon(FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.distance_nm = 20_000_000

            def move_component(
                self, uuid: str, x_mm: float, y_mm: float, rotation_deg: float | None
            ) -> JsonRpcResponse:
                response = super().move_component(uuid, x_mm, y_mm, rotation_deg)
                self.distance_nm = 16_278_820
                return response

            def get_unrouted(self) -> JsonRpcResponse:
                self.calls.append(("get_unrouted", None))
                return JsonRpcResponse(
                    "2.0",
                    31,
                    [
                        {
                            "net_name": "SIG",
                            "from": {"component": "R1", "pin": "1"},
                            "to": {"component": "R2", "pin": "1"},
                            "distance_nm": self.distance_nm,
                        }
                    ],
                    None,
                )

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        before = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 211,
                "method": "tools/call",
                "params": {
                    "name": "get_unrouted",
                    "arguments": {},
                },
            }
        )
        before_distance = before["result"]["content"][0]["json"][0]["distance_nm"]
        self.assertEqual(before_distance, 20_000_000)

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 212,
                "method": "tools/call",
                "params": {
                    "name": "move_component",
                    "arguments": {
                        "uuid": "comp-1",
                        "x_mm": 15.0,
                        "y_mm": 12.0,
                        "rotation_deg": 90.0,
                    },
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 213,
                "method": "tools/call",
                "params": {
                    "name": "get_unrouted",
                    "arguments": {},
                },
            }
        )
        after_distance = after["result"]["content"][0]["json"][0]["distance_nm"]
        self.assertNotEqual(after_distance, before_distance)
        self.assertEqual(
            daemon.calls,
            [
                ("get_unrouted", None),
                ("move_component", "comp-1", 15.0, 12.0, 90.0),
                ("get_unrouted", None),
            ],
        )

    def test_tools_call_rotate_component_changes_followup_components_response(self) -> None:
        class StatefulDaemon(FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.rotation = 0

            def rotate_component(self, uuid: str, rotation_deg: float) -> JsonRpcResponse:
                response = super().rotate_component(uuid, rotation_deg)
                self.rotation = int(rotation_deg)
                return response

            def get_components(self) -> JsonRpcResponse:
                self.calls.append(("get_components", None))
                return JsonRpcResponse(
                    "2.0",
                    27,
                    [
                        {
                            "uuid": "comp-1",
                            "reference": "R1",
                            "value": "10k",
                            "position": {"x": 10_000_000, "y": 10_000_000},
                            "rotation": self.rotation,
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
                "id": 240,
                "method": "tools/call",
                "params": {"name": "get_components", "arguments": {}},
            }
        )
        before_components = before["result"]["content"][0]["json"]
        self.assertEqual(before_components[0]["rotation"], 0)

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 241,
                "method": "tools/call",
                "params": {
                    "name": "rotate_component",
                    "arguments": {"uuid": "comp-1", "rotation_deg": 180.0},
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 242,
                "method": "tools/call",
                "params": {"name": "get_components", "arguments": {}},
            }
        )
        after_components = after["result"]["content"][0]["json"]
        self.assertEqual(after_components[0]["rotation"], 180)
        self.assertEqual(
            daemon.calls,
            [
                ("get_components", None),
                ("rotate_component", "comp-1", 180.0),
                ("get_components", None),
            ],
        )

    def test_tools_call_delete_track_changes_followup_check_report(self) -> None:
        class StatefulDaemon(FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.deleted = False

            def delete_track(self, uuid: str) -> JsonRpcResponse:
                response = super().delete_track(uuid)
                self.deleted = True
                return response

            def get_check_report(self) -> JsonRpcResponse:
                self.calls.append(("get_check_report", None))
                return JsonRpcResponse(
                    "2.0",
                    2,
                    {
                        "domain": "board",
                        "summary": {
                            "status": "info" if self.deleted else "warning",
                            "errors": 0,
                            "warnings": 0 if self.deleted else 1,
                            "infos": 1 if self.deleted else 0,
                            "waived": 0,
                            "by_code": [
                                {
                                    "code": "net_without_copper" if self.deleted else "partially_routed_net",
                                    "count": 1,
                                }
                            ],
                        },
                        "diagnostics": [
                            {
                                "kind": "net_without_copper" if self.deleted else "partially_routed_net",
                                "severity": "info" if self.deleted else "warning",
                            }
                        ],
                    },
                    None,
                )

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        before = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 214,
                "method": "tools/call",
                "params": {"name": "get_check_report", "arguments": {}},
            }
        )
        before_kinds = [d["kind"] for d in before["result"]["content"][0]["json"]["diagnostics"]]
        self.assertEqual(before_kinds, ["partially_routed_net"])

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 215,
                "method": "tools/call",
                "params": {"name": "delete_track", "arguments": {"uuid": "track-1"}},
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 216,
                "method": "tools/call",
                "params": {"name": "get_check_report", "arguments": {}},
            }
        )
        after_kinds = [d["kind"] for d in after["result"]["content"][0]["json"]["diagnostics"]]
        self.assertEqual(after_kinds, ["net_without_copper"])
        self.assertEqual(
            daemon.calls,
            [
                ("get_check_report", None),
                ("delete_track", "track-1"),
                ("get_check_report", None),
            ],
        )

    def test_tools_call_delete_component_changes_followup_components_response(self) -> None:
        class StatefulDaemon(FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.present = True

            def delete_component(self, uuid: str) -> JsonRpcResponse:
                response = super().delete_component(uuid)
                self.present = False
                return response

            def get_components(self) -> JsonRpcResponse:
                self.calls.append(("get_components", None))
                components: list[dict[str, Any]] = []
                if self.present:
                    components.append(
                        {
                            "uuid": "comp-1",
                            "reference": "R1",
                            "value": "10k",
                            "position": {"x": 10_000_000, "y": 10_000_000},
                            "rotation": 0,
                            "layer": 0,
                            "locked": False,
                        }
                    )
                return JsonRpcResponse("2.0", 26, components, None)

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        before = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 211,
                "method": "tools/call",
                "params": {"name": "get_components", "arguments": {}},
            }
        )
        before_components = before["result"]["content"][0]["json"]
        self.assertEqual(len(before_components), 1)

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 212,
                "method": "tools/call",
                "params": {
                    "name": "delete_component",
                    "arguments": {"uuid": "comp-1"},
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 213,
                "method": "tools/call",
                "params": {"name": "get_components", "arguments": {}},
            }
        )
        after_components = after["result"]["content"][0]["json"]
        self.assertEqual(after_components, [])
        self.assertEqual(
            daemon.calls,
            [
                ("get_components", None),
                ("delete_component", "comp-1"),
                ("get_components", None),
            ],
        )

    def test_tools_call_delete_via_changes_followup_net_info_response(self) -> None:
        class StatefulDaemon(FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.vias = 1

            def delete_via(self, uuid: str) -> JsonRpcResponse:
                response = super().delete_via(uuid)
                self.vias = 0
                return response

            def get_net_info(self) -> JsonRpcResponse:
                self.calls.append(("get_net_info", None))
                return JsonRpcResponse(
                    "2.0",
                    22,
                    [
                        {"name": "GND", "tracks": 1, "vias": self.vias, "zones": 0},
                        {"name": "VCC", "tracks": 0, "vias": 0, "zones": 0},
                    ],
                    None,
                )

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        before = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 220,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        before_gnd = before["result"]["content"][0]["json"][0]
        self.assertEqual(before_gnd["name"], "GND")
        self.assertEqual(before_gnd["vias"], 1)

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 221,
                "method": "tools/call",
                "params": {"name": "delete_via", "arguments": {"uuid": "via-1"}},
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 222,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        after_gnd = after["result"]["content"][0]["json"][0]
        self.assertEqual(after_gnd["vias"], 0)
        self.assertEqual(
            daemon.calls,
            [
                ("get_net_info", None),
                ("delete_via", "via-1"),
                ("get_net_info", None),
            ],
        )

    def test_tools_call_dispatches_undo_and_redo(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        undo = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 206,
                "method": "tools/call",
                "params": {"name": "undo", "arguments": {}},
            }
        )
        redo = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 207,
                "method": "tools/call",
                "params": {"name": "redo", "arguments": {}},
            }
        )
        self.assertEqual(daemon.calls, [("undo", None), ("redo", None)])
        self.assertEqual(
            undo["result"]["content"][0]["json"]["diff"]["created"][0]["uuid"],
            "track-1",
        )
        self.assertEqual(
            redo["result"]["content"][0]["json"]["diff"]["deleted"][0]["uuid"],
            "track-1",
        )
