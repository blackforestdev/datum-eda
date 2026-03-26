#!/usr/bin/env python3
"""Core MCP tools/call dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchCore(unittest.TestCase):
    def test_tools_call_dispatches_open_project(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "open_project",
                    "arguments": {"path": "/tmp/demo.kicad_pcb"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("open_project", "/tmp/demo.kicad_pcb")])
        self.assertEqual(response["result"]["content"][0]["json"]["kind"], "kicad_board")

    def test_tools_call_dispatches_close_project(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 200,
                "method": "tools/call",
                "params": {
                    "name": "close_project",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("close_project", None)])
        self.assertEqual(response["result"]["content"][0]["json"]["closed"], True)

    def test_tools_call_dispatches_save(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 204,
                "method": "tools/call",
                "params": {
                    "name": "save",
                    "arguments": {"path": "/tmp/out.kicad_pcb"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("save", "/tmp/out.kicad_pcb")])
        self.assertEqual(response["result"]["content"][0]["json"]["path"], "/tmp/out.kicad_pcb")

    def test_tools_call_dispatches_delete_track(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 205,
                "method": "tools/call",
                "params": {
                    "name": "delete_track",
                    "arguments": {"uuid": "track-1"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("delete_track", "track-1")])
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["deleted"][0]["uuid"],
            "track-1",
        )

    def test_tools_call_dispatches_delete_component(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 206,
                "method": "tools/call",
                "params": {
                    "name": "delete_component",
                    "arguments": {"uuid": "comp-1"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("delete_component", "comp-1")])
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["deleted"][0]["uuid"],
            "comp-1",
        )

    def test_tools_call_dispatches_delete_via(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 208,
                "method": "tools/call",
                "params": {
                    "name": "delete_via",
                    "arguments": {"uuid": "via-1"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("delete_via", "via-1")])
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["deleted"][0]["uuid"],
            "via-1",
        )

    def test_tools_call_dispatches_set_design_rule(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 209,
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
        self.assertEqual(
            daemon.calls,
            [
                (
                    "set_design_rule",
                    "ClearanceCopper",
                    "All",
                    {"Clearance": {"min": 125000}},
                    10,
                    "default clearance",
                )
            ],
        )
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["created"][0]["object_type"],
            "rule",
        )

    def test_tools_call_dispatches_assign_part(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 20915,
                "method": "tools/call",
                "params": {
                    "name": "assign_part",
                    "arguments": {"uuid": "comp-1", "part_uuid": "part-1"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("assign_part", "comp-1", "part-1")])
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "component",
        )

    def test_tools_call_dispatches_set_package_with_part(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 20916,
                "method": "tools/call",
                "params": {
                    "name": "set_package_with_part",
                    "arguments": {
                        "uuid": "comp-1",
                        "package_uuid": "package-1",
                        "part_uuid": "part-1",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("set_package_with_part", "comp-1", "package-1", "part-1")],
        )
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "component",
        )

    def test_tools_call_dispatches_set_package(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2093,
                "method": "tools/call",
                "params": {
                    "name": "set_package",
                    "arguments": {"uuid": "comp-1", "package_uuid": "package-1"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("set_package", "comp-1", "package-1")])
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "component",
        )

    def test_tools_call_dispatches_set_net_class(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 20916,
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
        self.assertEqual(
            daemon.calls,
            [
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
                )
            ],
        )
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "net",
        )

    def test_tools_call_dispatches_set_reference(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2092,
                "method": "tools/call",
                "params": {
                    "name": "set_reference",
                    "arguments": {"uuid": "comp-1", "reference": "R10"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("set_reference", "comp-1", "R10")])
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "component",
        )
