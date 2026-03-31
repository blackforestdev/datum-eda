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

    def test_tools_call_dispatches_route_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 84,
                "method": "tools/call",
                "params": {
                    "name": "route_proposal",
                    "arguments": {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                        "profile": "authored-copper-priority",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "route_proposal",
                    {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                        "profile": "authored-copper-priority",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "route_proposal")
        self.assertEqual(payload["selection_profile"], "authored-copper-priority")
        self.assertEqual(payload["selected_candidate"], "route-path-candidate")

    def test_tools_call_dispatches_route_proposal_explain(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 85,
                "method": "tools/call",
                "params": {
                    "name": "route_proposal_explain",
                    "arguments": {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "route_proposal_explain",
                    {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                        "profile": None,
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "route_proposal_explain")
        self.assertEqual(payload["families"][0]["status"], "selected")

    def test_tools_call_dispatches_route_strategy_report(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 86,
                "method": "tools/call",
                "params": {
                    "name": "route_strategy_report",
                    "arguments": {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                        "objective": "authored-copper-priority",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "route_strategy_report",
                    {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                        "objective": "authored-copper-priority",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "route_strategy_report")
        self.assertEqual(payload["recommended_profile"], "authored-copper-priority")
        self.assertEqual(payload["selected_candidate"], "authored-copper-graph")

    def test_tools_call_dispatches_route_strategy_compare(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 87,
                "method": "tools/call",
                "params": {
                    "name": "route_strategy_compare",
                    "arguments": {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "route_strategy_compare",
                    {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "route_strategy_compare")
        self.assertEqual(payload["recommended_profile"], "default")
        self.assertEqual(len(payload["entries"]), 2)

    def test_tools_call_dispatches_route_strategy_delta(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 88,
                "method": "tools/call",
                "params": {
                    "name": "route_strategy_delta",
                    "arguments": {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "route_strategy_delta",
                    {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "route_strategy_delta")
        self.assertEqual(payload["delta_classification"], "different_candidate_family")
        self.assertEqual(len(payload["profiles"]), 2)

    def test_tools_call_dispatches_write_route_strategy_curated_fixture_suite(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 88,
                "method": "tools/call",
                "params": {
                    "name": "write_route_strategy_curated_fixture_suite",
                    "arguments": {
                        "out_dir": "/tmp/route-strategy-fixtures",
                        "manifest": "/tmp/route-strategy-fixtures/requests.json",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "write_route_strategy_curated_fixture_suite",
                    {
                        "out_dir": "/tmp/route-strategy-fixtures",
                        "manifest": "/tmp/route-strategy-fixtures/requests.json",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "write_route_strategy_curated_fixture_suite")
        self.assertEqual(payload["suite_id"], "m6_route_strategy_curated_fixture_suite_v1")
        self.assertEqual(payload["requests_manifest_kind"], "native_route_strategy_batch_requests")

    def test_tools_call_dispatches_capture_route_strategy_curated_baseline(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 881,
                "method": "tools/call",
                "params": {
                    "name": "capture_route_strategy_curated_baseline",
                    "arguments": {
                        "out_dir": "/tmp/route-strategy-fixtures",
                        "manifest": "/tmp/route-strategy-fixtures/requests.json",
                        "result": "/tmp/route-strategy-fixtures/result.json",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "capture_route_strategy_curated_baseline",
                    {
                        "out_dir": "/tmp/route-strategy-fixtures",
                        "manifest": "/tmp/route-strategy-fixtures/requests.json",
                        "result": "/tmp/route-strategy-fixtures/result.json",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "capture_route_strategy_curated_baseline")
        self.assertEqual(
            payload["result_kind"], "native_route_strategy_batch_result_artifact"
        )
        self.assertEqual(payload["total_requests"], 4)

    def test_tools_call_dispatches_route_strategy_batch_evaluate(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 89,
                "method": "tools/call",
                "params": {
                    "name": "route_strategy_batch_evaluate",
                    "arguments": {
                        "requests": "/tmp/route-strategy-batch.json",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "route_strategy_batch_evaluate",
                    {
                        "requests": "/tmp/route-strategy-batch.json",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "route_strategy_batch_evaluate")
        self.assertEqual(payload["kind"], "native_route_strategy_batch_result_artifact")
        self.assertEqual(payload["summary"]["total_evaluated_requests"], 2)
        self.assertEqual(len(payload["results"]), 2)

    def test_tools_call_dispatches_route_strategy_batch_result_inspection(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 97,
                "method": "tools/call",
                "params": {
                    "name": "inspect_route_strategy_batch_result",
                    "arguments": {
                        "artifact": "/tmp/route-strategy-batch-result.json",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("inspect_route_strategy_batch_result", "/tmp/route-strategy-batch-result.json")],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "inspect_route_strategy_batch_result")
        self.assertEqual(payload["kind"], "native_route_strategy_batch_result_artifact")
        self.assertEqual(payload["summary"]["total_evaluated_requests"], 2)

    def test_tools_call_dispatches_native_project_validation(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 97,
                "method": "tools/call",
                "params": {
                    "name": "validate_project",
                    "arguments": {
                        "path": "/tmp/native-project",
                    },
                },
            }
        )
        self.assertEqual(daemon.calls, [("validate_project", "/tmp/native-project")])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "validate_project")
        self.assertEqual(payload["project_root"], "/tmp/native-project")
        self.assertEqual(payload["valid"], True)

    def test_tools_call_dispatches_route_strategy_batch_result_validation(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 98,
                "method": "tools/call",
                "params": {
                    "name": "validate_route_strategy_batch_result",
                    "arguments": {
                        "artifact": "/tmp/route-strategy-batch-result.json",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("validate_route_strategy_batch_result", "/tmp/route-strategy-batch-result.json")],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "validate_route_strategy_batch_result")
        self.assertEqual(payload["structurally_valid"], True)

    def test_tools_call_dispatches_route_strategy_batch_result_comparison(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 99,
                "method": "tools/call",
                "params": {
                    "name": "compare_route_strategy_batch_result",
                    "arguments": {
                        "before": "/tmp/before.route-strategy-batch.json",
                        "after": "/tmp/after.route-strategy-batch.json",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "compare_route_strategy_batch_result",
                    {
                        "before": "/tmp/before.route-strategy-batch.json",
                        "after": "/tmp/after.route-strategy-batch.json",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "compare_route_strategy_batch_result")
        self.assertEqual(
            payload["comparison_classification"], "per_request_outcomes_changed"
        )

    def test_tools_call_dispatches_route_strategy_batch_result_gate(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 100,
                "method": "tools/call",
                "params": {
                    "name": "gate_route_strategy_batch_result",
                    "arguments": {
                        "before": "/tmp/before.route-strategy-batch.json",
                        "after": "/tmp/after.route-strategy-batch.json",
                        "policy": "strict_identical",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "gate_route_strategy_batch_result",
                    {
                        "before": "/tmp/before.route-strategy-batch.json",
                        "after": "/tmp/after.route-strategy-batch.json",
                        "policy": "strict_identical",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "gate_route_strategy_batch_result")
        self.assertEqual(payload["selected_gate_policy"], "strict_identical")
        self.assertEqual(payload["passed"], False)

    def test_tools_call_dispatches_route_strategy_batch_results_summary(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 101,
                "method": "tools/call",
                "params": {
                    "name": "summarize_route_strategy_batch_results",
                    "arguments": {
                        "artifacts": ["/tmp/run-a.json", "/tmp/run-b.json"],
                        "baseline": "/tmp/run-a.json",
                        "policy": "strict_identical",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "summarize_route_strategy_batch_results",
                    {
                        "dir": None,
                        "artifacts": ["/tmp/run-a.json", "/tmp/run-b.json"],
                        "baseline": "/tmp/run-a.json",
                        "policy": "strict_identical",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "summarize_route_strategy_batch_results")
        self.assertEqual(payload["summary"]["total_artifacts"], 2)

    def test_tools_call_dispatches_export_route_proposal(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 90,
                "method": "tools/call",
                "params": {
                    "name": "export_route_proposal",
                    "arguments": {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                        "out": "/tmp/demo.route-proposal.json",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "export_route_proposal",
                    {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                        "profile": None,
                        "out": "/tmp/demo.route-proposal.json",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "export_route_proposal")
        self.assertEqual(payload["artifact_kind"], "native_route_proposal_artifact")

    def test_tools_call_dispatches_route_apply_selected(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 87,
                "method": "tools/call",
                "params": {
                    "name": "route_apply_selected",
                    "arguments": {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "route_apply_selected",
                    {
                        "path": "/tmp/demo",
                        "net_uuid": "11111111-1111-1111-1111-111111111111",
                        "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                        "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                        "profile": None,
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "route_apply_selected")
        self.assertEqual(payload["applied_actions"], 1)

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

    def test_tools_call_dispatches_route_proposal_artifact_revalidation(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 88,
                "method": "tools/call",
                "params": {
                    "name": "revalidate_route_proposal_artifact",
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
                    "revalidate_route_proposal_artifact",
                    {
                        "path": "/tmp/demo",
                        "artifact": "/tmp/demo.route-proposal.json",
                    },
                )
            ],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["action"], "revalidate_route_proposal_artifact")
        self.assertEqual(payload["matches_live"], True)

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
