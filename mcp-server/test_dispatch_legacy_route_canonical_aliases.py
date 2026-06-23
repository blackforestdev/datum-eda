#!/usr/bin/env python3
"""Canonical routing MCP alias dispatch tests.

Mirrors the established canonical-alias dispatch pattern (see
test_dispatch_output_job_canonical_aliases.py): each canonical datum.route.<verb>
alias must dispatch to the same daemon method as its compatibility-only flat tool
and return the canonical schema envelope under the alias name.
"""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


ANCHORS = {
    "path": "/tmp/native-project",
    "net_uuid": "net-1",
    "from_anchor_pad_uuid": "pad-a",
    "to_anchor_pad_uuid": "pad-b",
}


class TestDispatchLegacyRouteCanonicalAliases(unittest.TestCase):
    def _assert_alias(self, host, daemon, index, tool_name, method_name, arguments):
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": index,
                "method": "tools/call",
                "params": {"name": tool_name, "arguments": arguments},
            }
        )
        self.assertEqual(daemon.calls[-1][0], method_name)
        payload = response["result"]["content"][0]["json"]
        self.assertTrue(payload["ok"])
        self.assertEqual(payload["schema"], {"name": tool_name, "version": 1})

    def test_dispatches_route_proposal_and_apply_aliases(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        cases = [
            ("datum.route.export_path_proposal", "export_route_path_proposal", {**ANCHORS, "candidate": "route-path-candidate", "out": "/tmp/out.json"}),
            ("datum.route.select_proposal", "route_proposal", {**ANCHORS}),
            ("datum.route.review_proposal", "review_route_proposal", {**ANCHORS}),
            ("datum.route.explain_proposal", "route_proposal_explain", {**ANCHORS}),
            ("datum.route.export_proposal", "export_route_proposal", {**ANCHORS, "out": "/tmp/out.json"}),
            ("datum.route.apply", "route_apply", {**ANCHORS, "candidate": "route-path-candidate"}),
            ("datum.route.apply_selected", "route_apply_selected", {**ANCHORS}),
            ("datum.route.inspect_proposal_artifact", "inspect_route_proposal_artifact", {"artifact": "/tmp/artifact.json"}),
            ("datum.route.revalidate_proposal_artifact", "revalidate_route_proposal_artifact", {"path": "/tmp/native-project", "artifact": "/tmp/artifact.json"}),
            ("datum.route.apply_proposal_artifact", "apply_route_proposal_artifact", {"path": "/tmp/native-project", "artifact": "/tmp/artifact.json"}),
        ]
        for index, (tool_name, method_name, arguments) in enumerate(cases, start=800):
            self._assert_alias(host, daemon, index, tool_name, method_name, arguments)

    def test_dispatches_route_strategy_aliases(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        cases = [
            ("datum.route.strategy_report", "route_strategy_report", {**ANCHORS}),
            ("datum.route.strategy_compare", "route_strategy_compare", {**ANCHORS}),
            ("datum.route.strategy_delta", "route_strategy_delta", {**ANCHORS}),
            ("datum.route.write_strategy_fixture_suite", "write_route_strategy_curated_fixture_suite", {"out_dir": "/tmp/suite"}),
            ("datum.route.capture_strategy_baseline", "capture_route_strategy_curated_baseline", {"out_dir": "/tmp/suite"}),
            ("datum.route.strategy_batch_evaluate", "route_strategy_batch_evaluate", {"requests": "/tmp/requests.json"}),
            ("datum.route.inspect_strategy_batch_result", "inspect_route_strategy_batch_result", {"artifact": "/tmp/result.json"}),
            ("datum.route.validate_strategy_batch_result", "validate_route_strategy_batch_result", {"artifact": "/tmp/result.json"}),
            ("datum.route.compare_strategy_batch_result", "compare_route_strategy_batch_result", {"before": "/tmp/a.json", "after": "/tmp/b.json"}),
            ("datum.route.gate_strategy_batch_result", "gate_route_strategy_batch_result", {"before": "/tmp/a.json", "after": "/tmp/b.json"}),
            ("datum.route.summarize_strategy_batch_results", "summarize_route_strategy_batch_results", {}),
        ]
        for index, (tool_name, method_name, arguments) in enumerate(cases, start=830):
            self._assert_alias(host, daemon, index, tool_name, method_name, arguments)


if __name__ == "__main__":
    unittest.main()
