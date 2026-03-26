#!/usr/bin/env python3
"""Replacement and planning MCP dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import JsonRpcResponse, StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchCoreReplacements(unittest.TestCase):
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

    def test_tools_call_dispatches_replace_component(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 20917,
                "method": "tools/call",
                "params": {
                    "name": "replace_component",
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
            [("replace_component", "comp-1", "package-1", "part-1")],
        )
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "component",
        )

    def test_tools_call_dispatches_replace_components(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 20918,
                "method": "tools/call",
                "params": {
                    "name": "replace_components",
                    "arguments": {
                        "replacements": [
                            {
                                "uuid": "comp-1",
                                "package_uuid": "package-1",
                                "part_uuid": "part-1",
                            },
                            {
                                "uuid": "comp-2",
                                "package_uuid": "package-2",
                                "part_uuid": "part-2",
                            },
                        ]
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "replace_components",
                    [
                        {
                            "uuid": "comp-1",
                            "package_uuid": "package-1",
                            "part_uuid": "part-1",
                        },
                        {
                            "uuid": "comp-2",
                            "package_uuid": "package-2",
                            "part_uuid": "part-2",
                        },
                    ],
                )
            ],
        )
        self.assertEqual(
            len(response["result"]["content"][0]["json"]["diff"]["modified"]),
            2,
        )

    def test_tools_call_dispatches_apply_component_replacement_plan(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 20919,
                "method": "tools/call",
                "params": {
                    "name": "apply_component_replacement_plan",
                    "arguments": {
                        "replacements": [
                            {
                                "uuid": "comp-1",
                                "package_uuid": "package-1",
                                "part_uuid": None,
                            },
                            {
                                "uuid": "comp-2",
                                "package_uuid": None,
                                "part_uuid": "part-2",
                            },
                        ]
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "apply_component_replacement_plan",
                    [
                        {
                            "uuid": "comp-1",
                            "package_uuid": "package-1",
                            "part_uuid": None,
                        },
                        {
                            "uuid": "comp-2",
                            "package_uuid": None,
                            "part_uuid": "part-2",
                        },
                    ],
                )
            ],
        )
        self.assertEqual(
            len(response["result"]["content"][0]["json"]["diff"]["modified"]),
            2,
        )

    def test_tools_call_dispatches_apply_component_replacement_policy(self) -> None:
        class PolicyDaemon(FakeDaemonClient):
            def apply_component_replacement_policy(
                self, replacements: list[dict[str, str]]
            ) -> JsonRpcResponse:
                self.calls.append(("apply_component_replacement_policy", replacements))
                return JsonRpcResponse(
                    "2.0",
                    1215,
                    {
                        "diff": {
                            "created": [],
                            "modified": [
                                {"object_type": "component", "uuid": item["uuid"]}
                                for item in replacements
                            ],
                            "deleted": [],
                        },
                        "description": f"replace_components {len(replacements)}",
                    },
                    None,
                )

        daemon = PolicyDaemon()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 20920,
                "method": "tools/call",
                "params": {
                    "name": "apply_component_replacement_policy",
                    "arguments": {
                        "replacements": [
                            {"uuid": "comp-1", "policy": "best_compatible_package"},
                            {"uuid": "comp-2", "policy": "best_compatible_part"},
                        ]
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "apply_component_replacement_policy",
                    [
                        {"uuid": "comp-1", "policy": "best_compatible_package"},
                        {"uuid": "comp-2", "policy": "best_compatible_part"},
                    ],
                )
            ],
        )
        self.assertEqual(
            len(response["result"]["content"][0]["json"]["diff"]["modified"]),
            2,
        )

    def test_tools_call_dispatches_apply_scoped_component_replacement_policy(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 20921,
                "method": "tools/call",
                "params": {
                    "name": "apply_scoped_component_replacement_policy",
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
                    "apply_scoped_component_replacement_policy",
                    {"reference_prefix": "R", "value_equals": "LMV321"},
                    "best_compatible_package",
                )
            ],
        )
        self.assertEqual(
            len(response["result"]["content"][0]["json"]["diff"]["modified"]),
            2,
        )

    def test_tools_call_dispatches_apply_scoped_component_replacement_plan(self) -> None:
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
                }
            ],
        }
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 20922,
                "method": "tools/call",
                "params": {
                    "name": "apply_scoped_component_replacement_plan",
                    "arguments": {"plan": plan},
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("apply_scoped_component_replacement_plan", plan)],
        )
        self.assertEqual(
            len(response["result"]["content"][0]["json"]["diff"]["modified"]),
            2,
        )
