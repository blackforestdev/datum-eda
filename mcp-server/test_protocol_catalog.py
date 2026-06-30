#!/usr/bin/env python3
"""MCP protocol envelope and tools/list catalog tests."""

from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient
from tool_dispatch import registered_tool_names
from tools_catalog_data import (
    COMPATIBILITY_TOOL_SPECS,
    NON_JOURNALED_DAEMON_WRITE_METHODS,
    TOOL_BY_NAME,
    TOOLS,
)
from tools_catalog_retirement import (
    DEFAULT_HIDDEN_ALIAS_RETIREMENT_CRITERIA,
    HIDDEN_COMPATIBILITY_RETIREMENT_OVERRIDES,
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
                "create_pool_footprint",
                "set_pool_footprint_pad",
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
                "create_pool_pin_pad_map",
                "set_pool_pin_pad_map",
                "set_pool_library_object",
                "open_project",
                "close_project",
                "save",
                "validate_project",
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

    def test_private_writer_bypass_aliases_are_retired(self) -> None:
        names = {tool["name"] for tool in TOOLS}
        for retired in (
            "delete_track",
            "delete_via",
            "delete_component",
            "move_component",
            "rotate_component",
            "flip_component",
            "set_value",
            "assign_part",
            "set_package",
            "set_package_with_part",
            "set_reference",
            "set_net_class",
            "set_design_rule",
            "datum.board.delete_track",
            "datum.board.delete_via",
            "datum.board.delete_component",
            "datum.board.move_component",
            "datum.board.rotate_component",
            "datum.board.flip_component",
            "datum.board.set_component_value",
            "datum.board.assign_component_part",
            "datum.board.set_component_package",
            "datum.board.set_component_package_with_part",
            "datum.board.set_component_reference",
            "datum.board.set_net_class",
            "datum.board.set_design_rule",
            "replace_component",
            "replace_components",
            "apply_component_replacement_plan",
            "apply_component_replacement_policy",
            "apply_scoped_component_replacement_policy",
            "apply_scoped_component_replacement_plan",
            "datum.board.replace_component",
            "datum.board.replace_components",
            "datum.replacement.apply_plan",
            "datum.replacement.apply_policy",
            "datum.replacement.apply_scoped_policy",
            "datum.replacement.apply_scoped_plan",
        ):
            self.assertNotIn(retired, names)
            self.assertNotIn(retired, TOOL_BY_NAME)

    def test_replacement_apply_aliases_are_retired_but_read_planning_stays_public(self) -> None:
        names = {tool["name"] for tool in TOOLS}
        for retired in (
            "datum.replacement.apply_plan",
            "datum.replacement.apply_policy",
            "datum.replacement.apply_scoped_policy",
            "datum.replacement.apply_scoped_plan",
        ):
            self.assertNotIn(retired, names)
            self.assertNotIn(retired, TOOL_BY_NAME)
        for public in (
            "datum.replacement.get_plan",
            "datum.replacement.get_scoped_plan",
            "datum.replacement.edit_scoped_plan",
            "datum.replacement.package_candidates",
            "datum.replacement.part_candidates",
        ):
            self.assertIn(public, names)

    def test_hidden_compatibility_aliases_have_retirement_metadata(self) -> None:
        public_names = {tool["name"] for tool in TOOLS}
        for spec in COMPATIBILITY_TOOL_SPECS:
            self.assertEqual(spec.get("x_compatibility_visibility"), "hidden")
            self.assertIn(
                spec.get("x_retirement_status"),
                {
                    "retained_until_migration_plan",
                    "deprecated",
                    "scheduled_for_removal",
                },
            )
            self.assertTrue(spec.get("x_retirement_criteria"))
            replacements = spec.get("x_canonical_replacements")
            self.assertIsInstance(replacements, list)
            self.assertTrue(replacements)
            for replacement in replacements:
                self.assertIsInstance(replacement, str)
                self.assertTrue(
                    replacement in public_names or replacement.startswith("pending:"),
                    f"{spec['name']} replacement {replacement} is neither public nor pending",
                )

    def test_deprecated_hidden_aliases_have_explicit_retirement_plan(self) -> None:
        compatibility_by_name = {spec["name"]: spec for spec in COMPATIBILITY_TOOL_SPECS}
        deprecated_names = {
            spec["name"]
            for spec in COMPATIBILITY_TOOL_SPECS
            if spec.get("x_retirement_status") == "deprecated"
        }
        self.assertEqual(
            deprecated_names,
            set(HIDDEN_COMPATIBILITY_RETIREMENT_OVERRIDES),
        )
        for name, override in sorted(HIDDEN_COMPATIBILITY_RETIREMENT_OVERRIDES.items()):
            with self.subTest(name=name):
                spec = compatibility_by_name[name]
                replacements = spec.get("x_canonical_replacements", [])
                self.assertEqual(spec.get("x_retirement_status"), "deprecated")
                self.assertEqual(spec.get("x_retirement_criteria"), override["criteria"])
                self.assertNotEqual(
                    spec.get("x_retirement_criteria"),
                    DEFAULT_HIDDEN_ALIAS_RETIREMENT_CRITERIA,
                )
                self.assertIsInstance(replacements, list)
                self.assertTrue(replacements)

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

    def test_route_apply_tools_have_internal_write_surface_classification(self) -> None:
        expected = {
            "datum.route.apply": "journaled_route_apply",
            "datum.route.apply_selected": "journaled_route_apply",
            "datum.route.apply_proposal_artifact": "proposal_artifact_apply",
        }
        for name, expected_class in expected.items():
            self.assertEqual(
                TOOL_BY_NAME[name].get("x_public_write_surface_class"),
                expected_class,
            )
            self.assertTrue(TOOL_BY_NAME[name].get("x_write_surface_evidence"))
            description = TOOL_BY_NAME[name].get("description", "")
            self.assertIn("proposal", description)
            self.assertIn("journal", description)
            self.assertNotIn("directly", description.lower())

    def test_proposal_write_tools_have_public_write_surface_classification(self) -> None:
        expected = {
            "datum.proposal.create": "proposal_metadata_write",
            "datum.proposal.create_draw_wire": "proposal_metadata_write",
            "datum.proposal.create_place_label": "proposal_metadata_write",
            "datum.proposal.create_place_symbol": "proposal_metadata_write",
            "datum.proposal.create_board_component_replacement": "proposal_metadata_write",
            "datum.proposal.create_board_component_replacements": "proposal_metadata_write",
            "datum.proposal.create_board_component_replacement_plan": "proposal_metadata_write",
            "datum.proposal.create_panel_projection": "proposal_metadata_write",
            "datum.proposal.update_panel_projection": "proposal_metadata_write",
            "datum.proposal.delete_panel_projection": "proposal_metadata_write",
            "datum.proposal.create_manufacturing_plan": "proposal_metadata_write",
            "datum.proposal.update_manufacturing_plan": "proposal_metadata_write",
            "datum.proposal.delete_manufacturing_plan": "proposal_metadata_write",
            "datum.proposal.create_output_job": "proposal_metadata_write",
            "datum.proposal.update_output_job": "proposal_metadata_write",
            "datum.proposal.delete_output_job": "proposal_metadata_write",
            "datum.proposal.review": "proposal_review_state_write",
            "datum.proposal.defer": "proposal_review_state_write",
            "datum.proposal.reject": "proposal_review_state_write",
            "datum.proposal.accept_apply": "proposal_gateway_apply",
            "datum.proposal.apply": "proposal_gateway_apply",
        }
        read_tools = {
            "datum.proposal.list",
            "datum.proposal.show",
            "datum.proposal.preview",
            "datum.proposal.validate",
        }
        for name, expected_class in expected.items():
            self.assertEqual(
                TOOL_BY_NAME[name].get("x_public_write_surface_class"),
                expected_class,
            )
            self.assertTrue(TOOL_BY_NAME[name].get("x_write_surface_evidence"))
            description = TOOL_BY_NAME[name].get("description", "")
            self.assertIn("proposal", description)
            self.assertNotIn("directly", description.lower())
        for name in read_tools:
            self.assertIsNone(TOOL_BY_NAME[name].get("x_public_write_surface_class"))

    def test_public_production_authoring_aliases_are_proposal_mediated(self) -> None:
        expected = {
            "datum.manufacturing.create_panel_projection": "create_panel_projection_proposal",
            "datum.manufacturing.update_panel_projection": "update_panel_projection_proposal",
            "datum.manufacturing.delete_panel_projection": "delete_panel_projection_proposal",
            "datum.manufacturing.create_plan": "create_manufacturing_plan_proposal",
            "datum.manufacturing.update_plan": "update_manufacturing_plan_proposal",
            "datum.manufacturing.delete_plan": "delete_manufacturing_plan_proposal",
            "datum.output_job.create_gerber_set": "create_output_job_proposal",
            "datum.output_job.create": "create_output_job_proposal",
            "datum.output_job.update": "update_output_job_proposal",
            "datum.output_job.delete": "delete_output_job_proposal",
        }
        for name, dispatch_method in expected.items():
            self.assertEqual(TOOL_BY_NAME[name].get("x_dispatch_method"), dispatch_method)
            self.assertEqual(
                TOOL_BY_NAME[name].get("x_public_write_surface_class"),
                "proposal_metadata_write",
            )
            self.assertTrue(TOOL_BY_NAME[name].get("x_write_surface_evidence"))
            description = TOOL_BY_NAME[name].get("description", "")
            self.assertIn("proposal", description)
            self.assertNotIn("directly", description.lower())
        self.assertIsNone(TOOL_BY_NAME["datum.output_job.run"].get("x_public_write_surface_class"))

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
