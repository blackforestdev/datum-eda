#!/usr/bin/env python3
"""Schematic/query/check MCP tools/call dispatch tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchQueries(unittest.TestCase):
    def test_tools_call_dispatches_labels(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 70,
                "method": "tools/call",
                "params": {
                    "name": "get_labels",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_labels", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["name"], "SCL")

    def test_tools_call_dispatches_ports(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 71,
                "method": "tools/call",
                "params": {
                    "name": "get_ports",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_ports", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["name"], "SUB_IN")

    def test_tools_call_dispatches_symbols(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 74,
                "method": "tools/call",
                "params": {
                    "name": "get_symbols",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_symbols", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["reference"], "R1")

    def test_tools_call_dispatches_buses(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 72,
                "method": "tools/call",
                "params": {
                    "name": "get_buses",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_buses", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["name"], "DATA")

    def test_tools_call_dispatches_hierarchy(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 73,
                "method": "tools/call",
                "params": {
                    "name": "get_hierarchy",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_hierarchy", None)])
        self.assertEqual(response["result"]["content"][0]["json"]["instances"][0]["name"], "child")

    def test_tools_call_dispatches_noconnects(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 75,
                "method": "tools/call",
                "params": {
                    "name": "get_noconnects",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_noconnects", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["pin"], "2")

    def test_tools_call_dispatches_run_erc(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 76,
                "method": "tools/call",
                "params": {
                    "name": "run_erc",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("run_erc", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["code"], "undriven_power_net")

    def test_tools_call_dispatches_run_drc(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 77,
                "method": "tools/call",
                "params": {
                    "name": "run_drc",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("run_drc", None)])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["passed"], False)
        self.assertEqual(payload["violations"][0]["code"], "connectivity_unrouted_net")

    def test_tools_call_dispatches_explain_violation(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 78,
                "method": "tools/call",
                "params": {
                    "name": "explain_violation",
                    "arguments": {"domain": "drc", "index": 0},
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("explain_violation", {"domain": "drc", "index": 0})],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["rule_detail"], "drc connectivity_unrouted_net")

    def test_tools_call_dispatches_authored_copper_graph_policy_proposal_export(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 79,
                "method": "tools/call",
                "params": {
                    "name": "export_route_path_candidate_authored_copper_graph_proposal",
                    "arguments": {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                        "policy": "zone_obstacle_topology_layer_balance_aware",
                        "out": "/tmp/demo.route-proposal.json",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "export_route_path_candidate_authored_copper_graph_proposal",
                    {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                        "policy": "zone_obstacle_topology_layer_balance_aware",
                        "out": "/tmp/demo.route-proposal.json",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(
            payload["action"],
            "export_route_path_candidate_authored_copper_graph_proposal",
        )
        self.assertEqual(
            payload["contract"], "m5_route_path_candidate_authored_copper_graph_policy_v1"
        )
        self.assertEqual(payload["policy"], "zone_obstacle_topology_layer_balance_aware")

    def test_tools_call_dispatches_generic_route_path_proposal_export(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 82,
                "method": "tools/call",
                "params": {
                    "name": "export_route_path_proposal",
                    "arguments": {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                        "candidate": "route-path-candidate",
                        "out": "/tmp/demo.route-proposal.json",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "export_route_path_proposal",
                    {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                        "candidate": "route-path-candidate",
                        "policy": None,
                        "out": "/tmp/demo.route-proposal.json",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "export_route_path_proposal")
        self.assertEqual(payload["candidate"], "route-path-candidate")
        self.assertEqual(payload["artifact_kind"], "native_route_proposal_artifact")

    def test_tools_call_dispatches_route_apply(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 83,
                "method": "tools/call",
                "params": {
                    "name": "route_apply",
                    "arguments": {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                        "candidate": "authored-copper-graph",
                        "policy": "plain",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "route_apply",
                    {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                        "candidate": "authored-copper-graph",
                        "policy": "plain",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "route_apply")
        self.assertEqual(payload["candidate"], "authored-copper-graph")
        self.assertEqual(
            payload["contract"], "m5_route_path_candidate_authored_copper_graph_policy_v1"
        )

    def test_tools_call_dispatches_route_proposal_artifact_inspection(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 80,
                "method": "tools/call",
                "params": {
                    "name": "inspect_route_proposal_artifact",
                    "arguments": {
                        "artifact": "/tmp/demo.route-proposal.json",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("inspect_route_proposal_artifact", "/tmp/demo.route-proposal.json")],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "inspect_route_proposal_artifact")
        self.assertEqual(payload["artifact_kind"], "native_route_proposal_artifact")

    def test_tools_call_dispatches_route_proposal_artifact_apply(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 81,
                "method": "tools/call",
                "params": {
                    "name": "apply_route_proposal_artifact",
                    "arguments": {
                        "path": "/tmp/demo",
                        "artifact": "/tmp/demo.route-proposal.json",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "apply_route_proposal_artifact",
                    {
                        "path": "/tmp/demo",
                        "artifact": "/tmp/demo.route-proposal.json",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "apply_route_proposal_artifact")
        self.assertEqual(payload["artifact_actions"], 2)
        self.assertEqual(payload["applied_actions"], 0)
