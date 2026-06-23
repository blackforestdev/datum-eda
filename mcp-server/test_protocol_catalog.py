#!/usr/bin/env python3
"""MCP protocol envelope and tools/list catalog tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient
from tool_dispatch import registered_tool_names
from tools_catalog_data import (
    NON_JOURNALED_DAEMON_WRITE_METHODS,
    TOOL_BY_NAME,
    TOOLS,
)


class TestProtocolCatalog(unittest.TestCase):
    def test_tools_list_returns_registered_tools(self) -> None:
        host = StdioToolHost(FakeDaemonClient())
        response = host.handle_message({"jsonrpc": "2.0", "id": 1, "method": "tools/list"})
        self.assertIn("result", response)
        tools = response["result"]["tools"]
        self.assertEqual([tool["name"] for tool in tools], [tool["name"] for tool in TOOLS])

    def test_catalog_and_dispatch_share_the_same_registered_names(self) -> None:
        self.assertLessEqual(
            set(tool["name"] for tool in TOOLS),
            set(registered_tool_names()),
        )

    def test_tools_list_hides_legacy_check_and_journal_aliases(self) -> None:
        names = {tool["name"] for tool in TOOLS}
        self.assertFalse(
            {
                "get_check_run",
                "get_check_runs",
                "show_check_run",
                "get_check_profiles",
                "get_check_report",
                "run_erc",
                "run_drc",
                "explain_violation",
                "fill_zones",
                "generate_standards_repair_proposals",
                "get_zone_fills",
                "waive_finding",
                "accept_deviation",
                "get_journal_list",
                "get_journal_transaction",
                "journal_undo",
                "journal_redo",
                "undo",
                "redo",
                "generate_artifacts",
                "get_artifacts",
                "show_artifact",
                "get_artifact_files",
                "preview_artifact_file",
                "compare_artifacts",
                "validate_artifact",
                "start_output_job_run",
                "cancel_output_job_run",
                "export_manufacturing_set",
                "validate_manufacturing_set",
                "get_panel_projections",
                "get_manufacturing_plans",
                "get_output_jobs",
                "create_panel_projection_proposal",
                "update_panel_projection_proposal",
                "delete_panel_projection_proposal",
                "create_manufacturing_plan_proposal",
                "update_manufacturing_plan_proposal",
                "delete_manufacturing_plan_proposal",
                "create_output_job_proposal",
                "update_output_job_proposal",
                "delete_output_job_proposal",
                "create_panel_projection",
                "update_panel_projection",
                "delete_panel_projection",
                "create_manufacturing_plan",
                "update_manufacturing_plan",
                "delete_manufacturing_plan",
                "create_gerber_output_job",
                "create_output_job",
                "update_output_job",
                "run_output_job",
                "delete_output_job",
                "get_component_instances",
                "bind_component_instance",
                "set_component_instance",
                "delete_component_instance",
                "get_relationships",
                "get_variants",
                "get_import_map",
                "get_components",
                "get_netlist",
                "get_schematic_net_info",
                "get_board_summary",
                "get_schematic_summary",
                "get_sheets",
                "get_labels",
                "get_symbols",
                "get_symbol_fields",
                "get_ports",
                "get_buses",
                "get_bus_entries",
                "get_noconnects",
                "get_connectivity_diagnostics",
                "get_design_rules",
                "create_proposal",
                "create_draw_wire_proposal",
                "create_place_label_proposal",
                "create_place_symbol_proposal",
                "get_proposals",
                "show_proposal",
                "preview_proposal",
                "validate_proposal",
                "defer_proposal",
                "reject_proposal",
                "review_proposal",
                "accept_apply_proposal",
                "apply_proposal",
                "get_pool_library_objects",
                "show_pool_library_object",
                "get_pool_model_blobs",
                "gc_pool_model_blobs",
                "create_pool_library_object",
                "delete_pool_library_object",
                "create_pool_unit",
                "set_pool_unit_pin",
                "create_pool_symbol",
                "add_pool_symbol_line",
                "add_pool_symbol_rect",
                "add_pool_symbol_circle",
                "add_pool_symbol_arc",
                "add_pool_symbol_polygon",
                "add_pool_symbol_text",
                "set_pool_symbol_pin_anchor",
                "create_pool_entity",
                "create_pool_padstack",
                "create_pool_package",
                "set_pool_package_pad",
                "set_pool_package_courtyard_rect",
                "set_pool_package_courtyard_polygon",
                "add_pool_package_silkscreen_line",
                "add_pool_package_silkscreen_rect",
                "add_pool_package_silkscreen_polygon",
                "add_pool_package_silkscreen_circle",
                "add_pool_package_silkscreen_arc",
                "add_pool_package_silkscreen_text",
                "add_pool_package_model_3d",
                "set_pool_package_body_heights",
                "create_pool_part",
                "set_pool_part_metadata",
                "set_pool_part_parametric",
                "set_pool_part_orderable_mpns",
                "set_pool_part_tags",
                "set_pool_part_packaging_options",
                "set_pool_part_supply_chain",
                "set_pool_part_behavioural_models",
                "attach_pool_part_model",
                "detach_pool_part_model",
                "set_pool_part_thermal",
                "set_pool_part_pad_map_entry",
                "set_pool_part_pad_map",
                "set_pool_library_object",
                "open_project",
                "close_project",
                "save",
                "validate_project",
                "delete_track",
                "delete_component",
                "move_component",
                "rotate_component",
                "flip_component",
                "set_value",
                "assign_part",
                "set_package",
                "set_package_with_part",
                "replace_component",
                "replace_components",
                "set_reference",
                "set_net_class",
                "delete_via",
                "set_design_rule",
                "apply_component_replacement_plan",
                "apply_component_replacement_policy",
                "apply_scoped_component_replacement_policy",
                "apply_scoped_component_replacement_plan",
                "get_component_replacement_plan",
                "get_scoped_component_replacement_plan",
                "edit_scoped_component_replacement_plan",
                "get_package_change_candidates",
                "get_part_change_candidates",
                "search_pool",
                "get_part",
                "get_package",
                "get_net_info",
                "get_unrouted",
                "get_hierarchy",
                "export_route_path_proposal",
                "route_proposal",
                "review_route_proposal",
                "route_strategy_report",
                "route_strategy_compare",
                "route_strategy_delta",
                "write_route_strategy_curated_fixture_suite",
                "capture_route_strategy_curated_baseline",
                "route_strategy_batch_evaluate",
                "inspect_route_strategy_batch_result",
                "validate_route_strategy_batch_result",
                "compare_route_strategy_batch_result",
                "gate_route_strategy_batch_result",
                "summarize_route_strategy_batch_results",
                "route_proposal_explain",
                "export_route_proposal",
                "route_apply",
                "route_apply_selected",
                "inspect_route_proposal_artifact",
                "revalidate_route_proposal_artifact",
                "apply_route_proposal_artifact",
            }
            & names
        )

    def test_no_public_write_tool_bypasses_the_journaled_commit_path(self) -> None:
        """Decision 004 (Private Mutation Ban): no publicly-listed MCP write tool may
        dispatch to a non-journaled daemon write arm. The journaled datum.pcb.*
        family is the only public board-write surface; the legacy flat tools and
        the bypassing datum.board.* aliases remain compatibility-only/hidden."""
        offenders = [
            tool["name"]
            for tool in TOOLS
            if TOOL_BY_NAME[tool["name"]].get("x_dispatch_method", tool["name"])
            in NON_JOURNALED_DAEMON_WRITE_METHODS
        ]
        self.assertEqual(
            offenders,
            [],
            f"public catalog exposes non-journaled write tools: {offenders}",
        )

    def test_bypassing_board_write_aliases_are_hidden_but_dispatchable(self) -> None:
        """The legacy board-write aliases stay reachable for backward compatibility
        but must not appear in the public tools/list."""
        names = {tool["name"] for tool in TOOLS}
        for hidden in ("datum.board.move_component", "datum.board.set_net_class", "move_component"):
            self.assertNotIn(hidden, names)
            self.assertIn(hidden, TOOL_BY_NAME)

    def test_catalog_names_are_unique(self) -> None:
        names = [tool["name"] for tool in TOOLS]
        self.assertEqual(len(names), len(set(names)))

    def test_tools_list_hides_internal_dispatch_metadata(self) -> None:
        host = StdioToolHost(FakeDaemonClient())
        response = host.handle_message({"jsonrpc": "2.0", "id": 1, "method": "tools/list"})
        for tool in response["result"]["tools"]:
            self.assertFalse(any(key.startswith("x_") for key in tool))

    def test_route_strategy_fixture_tools_expose_generated_fixture_boundary(self) -> None:
        tools = {tool["name"]: tool for tool in TOOLS}
        names = ["datum.route.write_strategy_fixture_suite", "datum.route.capture_strategy_baseline"]
        for name in names:
            self.assertEqual(tools[name]["authoring_boundary"], "generated_fixture_only")
            self.assertEqual(
                tools[name]["write_path_policy"],
                "direct project-shard writes are restricted to deterministic regression fixture generation",
            )

    def test_initialize_returns_server_info_and_capabilities(self) -> None:
        host = StdioToolHost(FakeDaemonClient())
        response = host.handle_message({"jsonrpc": "2.0", "id": 1, "method": "initialize"})
        assert isinstance(response, dict)
        self.assertEqual(response["jsonrpc"], "2.0")
        self.assertEqual(response["id"], 1)
        self.assertEqual(response["result"]["protocolVersion"], "2024-11-05")
        self.assertEqual(response["result"]["serverInfo"]["name"], "datum-eda")
        self.assertIn("tools", response["result"]["capabilities"])

    def test_ping_returns_empty_result(self) -> None:
        host = StdioToolHost(FakeDaemonClient())
        response = host.handle_message({"jsonrpc": "2.0", "id": 7, "method": "ping"})
        assert isinstance(response, dict)
        self.assertEqual(response, {"jsonrpc": "2.0", "id": 7, "result": {}})

    def test_initialized_notification_returns_no_response(self) -> None:
        host = StdioToolHost(FakeDaemonClient())
        response = host.handle_message(
            {"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}}
        )
        self.assertIsNone(response)
