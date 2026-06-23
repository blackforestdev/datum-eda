#!/usr/bin/env python3
"""Relationship and variant MCP tools/call dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchRelationships(unittest.TestCase):
    def test_tools_call_dispatches_get_component_instances(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 200,
                "method": "tools/call",
                "params": {
                    "name": "get_component_instances",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_component_instances", "/tmp/native-project")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "component_instances_query_v1")
        self.assertEqual(payload["component_instance_count"], 1)

    def test_tools_call_dispatches_component_instance_writes(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        for tool_name, arguments, expected_call in [
            (
                "bind_component_instance",
                {
                    "path": "/tmp/native-project",
                    "symbol": "sym-test",
                    "package": "pkg-test",
                    "component_instance": "ci-test",
                },
                ("bind_component_instance", "/tmp/native-project", "sym-test", "pkg-test", "ci-test"),
            ),
            (
                "set_component_instance",
                {
                    "path": "/tmp/native-project",
                    "component_instance": "ci-test",
                    "symbol": "sym-next",
                    "package": "pkg-next",
                },
                ("set_component_instance", "/tmp/native-project", "ci-test", "sym-next", "pkg-next"),
            ),
            (
                "delete_component_instance",
                {"path": "/tmp/native-project", "component_instance": "ci-test"},
                ("delete_component_instance", "/tmp/native-project", "ci-test"),
            ),
        ]:
            host.handle_message(
                {
                    "jsonrpc": "2.0",
                    "id": 203,
                    "method": "tools/call",
                    "params": {"name": tool_name, "arguments": arguments},
                }
            )
            self.assertEqual(daemon.calls[-1], expected_call)

    def test_tools_call_dispatches_canonical_component_instance_aliases(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        for tool_name, arguments, expected_method in [
            ("datum.query.component_instances", {"path": "/tmp/native-project"}, "get_component_instances"),
            (
                "datum.component_instance.bind",
                {"path": "/tmp/native-project", "symbol": "sym-test", "package": "pkg-test"},
                "bind_component_instance",
            ),
            (
                "datum.component_instance.set",
                {
                    "path": "/tmp/native-project",
                    "component_instance": "ci-test",
                    "symbol": "sym-next",
                    "package": "pkg-next",
                },
                "set_component_instance",
            ),
            (
                "datum.component_instance.delete",
                {"path": "/tmp/native-project", "component_instance": "ci-test"},
                "delete_component_instance",
            ),
        ]:
            host.handle_message(
                {
                    "jsonrpc": "2.0",
                    "id": 204,
                    "method": "tools/call",
                    "params": {"name": tool_name, "arguments": arguments},
                }
            )
            self.assertEqual(daemon.calls[-1][0], expected_method)

    def test_tools_call_dispatches_get_relationships(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 201,
                "method": "tools/call",
                "params": {
                    "name": "get_relationships",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_relationships", "/tmp/native-project")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "relationships_query_v1")
        self.assertEqual(payload["relationship_count"], 1)

    def test_tools_call_dispatches_get_variants(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 202,
                "method": "tools/call",
                "params": {
                    "name": "get_variants",
                    "arguments": {"path": "/tmp/native-project"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_variants", "/tmp/native-project")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["contract"], "variants_query_v1")
        self.assertEqual(payload["variant_count"], 1)
