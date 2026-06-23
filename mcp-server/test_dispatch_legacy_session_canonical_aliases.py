#!/usr/bin/env python3
"""Canonical session/board/replacement/pool/query MCP alias dispatch tests.

Mirrors the established canonical-alias dispatch pattern (see
test_dispatch_output_job_canonical_aliases.py): each canonical datum.<group>.<verb>
alias must dispatch to the same daemon method as its compatibility-only flat tool
and return the canonical schema envelope under the alias name.
"""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


SCOPE = {"reference_prefix": "R"}
PLAN = {"scope": SCOPE, "policy": "best_compatible_package", "replacements": []}


class TestDispatchLegacySessionCanonicalAliases(unittest.TestCase):
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

    def test_dispatches_session_and_pool_query_aliases(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        cases = [
            ("datum.session.open", "open_project", {"path": "/tmp/native-project"}),
            ("datum.session.close", "close_project", {}),
            ("datum.session.save", "save", {}),
            ("datum.session.validate", "validate_project", {"path": "/tmp/native-project"}),
            ("datum.pool.search", "search_pool", {"query": "resistor"}),
            ("datum.pool.get_part", "get_part", {"uuid": "part-1"}),
            ("datum.pool.get_package", "get_package", {"uuid": "package-1"}),
            ("datum.query.net_info", "get_net_info", {}),
            ("datum.query.unrouted", "get_unrouted", {}),
            ("datum.query.imported_hierarchy", "get_hierarchy", {}),
        ]
        for index, (tool_name, method_name, arguments) in enumerate(cases, start=900):
            self._assert_alias(host, daemon, index, tool_name, method_name, arguments)

    def test_dispatches_board_mutation_aliases(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        cases = [
            ("datum.board.delete_track", "delete_track", {"uuid": "track-1"}),
            ("datum.board.delete_component", "delete_component", {"uuid": "comp-1"}),
            ("datum.board.move_component", "move_component", {"uuid": "comp-1", "x_mm": 1.0, "y_mm": 2.0}),
            ("datum.board.rotate_component", "rotate_component", {"uuid": "comp-1", "rotation_deg": 90.0}),
            ("datum.board.flip_component", "flip_component", {"uuid": "comp-1", "layer": 31}),
            ("datum.board.set_component_value", "set_value", {"uuid": "comp-1", "value": "10k"}),
            ("datum.board.assign_component_part", "assign_part", {"uuid": "comp-1", "part_uuid": "part-1"}),
            ("datum.board.set_component_package", "set_package", {"uuid": "comp-1", "package_uuid": "pkg-1"}),
            ("datum.board.set_component_package_with_part", "set_package_with_part", {"uuid": "comp-1", "package_uuid": "pkg-1", "part_uuid": "part-1"}),
            ("datum.board.replace_component", "replace_component", {"uuid": "comp-1", "package_uuid": "pkg-1", "part_uuid": "part-1"}),
            ("datum.board.replace_components", "replace_components", {"replacements": []}),
            ("datum.board.set_component_reference", "set_reference", {"uuid": "comp-1", "reference": "R5"}),
            ("datum.board.set_net_class", "set_net_class", {"net_uuid": "net-1", "class_name": "power", "clearance": 200000, "track_width": 250000, "via_drill": 300000, "via_diameter": 600000}),
            ("datum.board.delete_via", "delete_via", {"uuid": "via-1"}),
            ("datum.board.set_design_rule", "set_design_rule", {"rule_type": "clearance", "scope": {}, "parameters": {}, "priority": 1}),
        ]
        for index, (tool_name, method_name, arguments) in enumerate(cases, start=930):
            self._assert_alias(host, daemon, index, tool_name, method_name, arguments)

    def test_dispatches_replacement_planning_aliases(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)
        cases = [
            ("datum.replacement.apply_plan", "apply_component_replacement_plan", {"replacements": []}),
            ("datum.replacement.apply_policy", "apply_component_replacement_policy", {"replacements": []}),
            ("datum.replacement.apply_scoped_policy", "apply_scoped_component_replacement_policy", {"scope": SCOPE, "policy": "best_compatible_package"}),
            ("datum.replacement.apply_scoped_plan", "apply_scoped_component_replacement_plan", {"plan": PLAN}),
            ("datum.replacement.get_plan", "get_component_replacement_plan", {"uuid": "comp-1"}),
            ("datum.replacement.get_scoped_plan", "get_scoped_component_replacement_plan", {"scope": SCOPE, "policy": "best_compatible_part"}),
            ("datum.replacement.edit_scoped_plan", "edit_scoped_component_replacement_plan", {"plan": PLAN}),
            ("datum.replacement.package_candidates", "get_package_change_candidates", {"uuid": "comp-1"}),
            ("datum.replacement.part_candidates", "get_part_change_candidates", {"uuid": "comp-1"}),
        ]
        for index, (tool_name, method_name, arguments) in enumerate(cases, start=960):
            self._assert_alias(host, daemon, index, tool_name, method_name, arguments)


if __name__ == "__main__":
    unittest.main()
