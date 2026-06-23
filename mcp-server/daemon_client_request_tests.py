#!/usr/bin/env python3
"""Engine daemon client request construction tests."""

from __future__ import annotations

import subprocess
import unittest
from unittest.mock import patch
import os

from server_runtime import DAEMON_CLIENT_METHOD_SPECS, EngineDaemonClient
from test_support import FakeDaemonClient
from tools_catalog_data import TOOL_BY_NAME, TOOLS


class TestDaemonClientRequests(unittest.TestCase):
    @patch.dict(os.environ, {}, clear=True)
    def test_cli_prefix_defaults_to_canonical_binary(self) -> None:
        client = EngineDaemonClient()
        self.assertEqual(client._cli_prefix(), ["datum-eda"])

    @patch.dict(os.environ, {"DATUM_CLI_BIN": "custom-datum --flag"}, clear=True)
    def test_cli_prefix_uses_canonical_environment(self) -> None:
        client = EngineDaemonClient()
        self.assertEqual(client._cli_prefix(), ["custom-datum", "--flag"])

    @patch.dict(os.environ, {"EDA_CLI_BIN": "legacy-eda"}, clear=True)
    def test_cli_prefix_accepts_legacy_environment(self) -> None:
        client = EngineDaemonClient()
        self.assertEqual(client._cli_prefix(), ["legacy-eda"])

    @patch.dict(
        os.environ,
        {"DATUM_CLI_BIN": "custom-datum", "EDA_CLI_BIN": "legacy-eda"},
        clear=True,
    )
    def test_cli_prefix_prefers_canonical_environment(self) -> None:
        client = EngineDaemonClient()
        self.assertEqual(client._cli_prefix(), ["custom-datum"])

    def test_registered_tools_have_runtime_and_fake_daemon_methods(self) -> None:
        for tool in TOOLS:
            name = tool["name"]
            method_name = TOOL_BY_NAME[name].get("x_dispatch_method", name)
            self.assertTrue(callable(getattr(EngineDaemonClient, method_name, None)), name)
            self.assertTrue(callable(getattr(FakeDaemonClient, method_name, None)), name)

    def test_daemon_client_method_specs_install_request_and_call_wrappers(self) -> None:
        client = EngineDaemonClient()
        for spec in DAEMON_CLIENT_METHOD_SPECS:
            name = spec["name"]
            request_method = getattr(client, f"{name}_request", None)
            call_method = getattr(client, name, None)
            self.assertTrue(callable(request_method), name)
            self.assertTrue(callable(call_method), name)

    def test_builds_get_check_report_request(self) -> None:
        client = EngineDaemonClient()
        request = client.get_check_report_request()
        self.assertEqual(request.jsonrpc, "2.0")
        self.assertEqual(request.id, 1)
        self.assertEqual(request.method, "get_check_report")
        self.assertEqual(request.params, {})

    def test_request_ids_increment_across_methods(self) -> None:
        client = EngineDaemonClient()
        first = client.get_check_report_request()
        second = client.get_connectivity_diagnostics_request()
        third = client.run_erc_request()
        fourth = client.run_drc_request()
        self.assertEqual((first.id, second.id, third.id, fourth.id), (1, 2, 3, 4))
        self.assertEqual(fourth.params, {})

    def test_builds_run_drc_request_with_selected_rules(self) -> None:
        client = EngineDaemonClient()
        request = client.run_drc_request(["TrackWidth"])
        self.assertEqual(request.method, "run_drc")
        self.assertEqual(request.params, {"rules": ["TrackWidth"]})

    def test_builds_open_project_request(self) -> None:
        client = EngineDaemonClient()
        request = client.open_project_request("/tmp/demo.kicad_pcb")
        self.assertEqual(request.method, "open_project")
        self.assertEqual(request.params, {"path": "/tmp/demo.kicad_pcb"})

    def test_generated_rotate_component_request_keeps_fixed_mm_fields(self) -> None:
        client = EngineDaemonClient()
        request = client.rotate_component_request("comp-uuid", 180.0)
        self.assertEqual(request.method, "rotate_component")
        self.assertEqual(
            request.params,
            {"uuid": "comp-uuid", "x_mm": 0.0, "y_mm": 0.0, "rotation_deg": 180.0},
        )

    def test_builds_validate_project_request(self) -> None:
        client = EngineDaemonClient()
        request = client.validate_project_request("/tmp/native-project")
        self.assertEqual(request.method, "validate_project")
        self.assertEqual(request.params, {"path": "/tmp/native-project"})

    def test_builds_close_project_request(self) -> None:
        client = EngineDaemonClient()
        request = client.close_project_request()
        self.assertEqual(request.method, "close_project")
        self.assertEqual(request.params, {})

    def test_builds_m3_write_requests(self) -> None:
        client = EngineDaemonClient()
        save = client.save_request("/tmp/out.kicad_pcb")
        delete = client.delete_track_request("track-uuid")
        delete_via = client.delete_via_request("via-uuid")
        delete_component = client.delete_component_request("comp-uuid")
        move_component = client.move_component_request("comp-uuid", 15.0, 12.0, 90.0)
        rotate_component = client.rotate_component_request("comp-uuid", 180.0)
        flip_component = client.flip_component_request("comp-uuid", 2)
        set_value = client.set_value_request("comp-uuid", "22k")
        assign_part = client.assign_part_request("comp-uuid", "part-uuid")
        set_package = client.set_package_request("comp-uuid", "package-uuid")
        set_package_with_part = client.set_package_with_part_request(
            "comp-uuid", "package-uuid", "part-uuid"
        )
        replace_component = client.replace_component_request(
            "comp-uuid", "package-uuid", "part-uuid"
        )
        replace_components = client.replace_components_request(
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
            ]
        )
        apply_replacement_plan = client.apply_component_replacement_plan_request(
            [
                {"uuid": "comp-1", "package_uuid": "package-1", "part_uuid": None},
                {"uuid": "comp-2", "package_uuid": None, "part_uuid": "part-2"},
            ]
        )
        apply_replacement_policy = client.apply_component_replacement_policy_request(
            [
                {"uuid": "comp-1", "policy": "best_compatible_package"},
                {"uuid": "comp-2", "policy": "best_compatible_part"},
            ]
        )
        apply_scoped_replacement_policy = (
            client.apply_scoped_component_replacement_policy_request(
                {"reference_prefix": "R", "value_equals": "LMV321"},
                "best_compatible_package",
            )
        )
        apply_scoped_replacement_plan = (
            client.apply_scoped_component_replacement_plan_request(
                {
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
            )
        )
        get_scoped_replacement_plan = (
            client.get_scoped_component_replacement_plan_request(
                {"reference_prefix": "R", "value_equals": "LMV321"},
                "best_compatible_package",
            )
        )
        edit_scoped_replacement_plan = (
            client.edit_scoped_component_replacement_plan_request(
                {
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
                },
                ["comp-2"],
                [
                    {
                        "component_uuid": "comp-1",
                        "target_package_uuid": "alt-package-uuid",
                        "target_part_uuid": "alt-part-uuid",
                    }
                ],
            )
        )
        set_net_class = client.set_net_class_request(
            "net-uuid", "power", 125000, 250000, 300000, 600000
        )
        set_reference = client.set_reference_request("comp-uuid", "R10")
        set_rule = client.set_design_rule_request(
            "ClearanceCopper",
            "All",
            {"Clearance": {"min": 125000}},
            10,
            "default clearance",
        )
        undo = client.undo_request()
        redo = client.redo_request()
        self.assertEqual(save.method, "save")
        self.assertEqual(save.params, {"path": "/tmp/out.kicad_pcb"})
        self.assertEqual(delete.method, "delete_track")
        self.assertEqual(delete.params, {"uuid": "track-uuid"})
        self.assertEqual(delete_via.method, "delete_via")
        self.assertEqual(delete_via.params, {"uuid": "via-uuid"})
        self.assertEqual(delete_component.method, "delete_component")
        self.assertEqual(delete_component.params, {"uuid": "comp-uuid"})
        self.assertEqual(move_component.method, "move_component")
        self.assertEqual(
            move_component.params,
            {"uuid": "comp-uuid", "x_mm": 15.0, "y_mm": 12.0, "rotation_deg": 90.0},
        )
        self.assertEqual(rotate_component.method, "rotate_component")
        self.assertEqual(
            rotate_component.params,
            {"uuid": "comp-uuid", "x_mm": 0.0, "y_mm": 0.0, "rotation_deg": 180.0},
        )
        self.assertEqual(flip_component.method, "flip_component")
        self.assertEqual(flip_component.params, {"uuid": "comp-uuid", "layer": 2})
        self.assertEqual(set_value.method, "set_value")
        self.assertEqual(set_value.params, {"uuid": "comp-uuid", "value": "22k"})
        self.assertEqual(assign_part.method, "assign_part")
        self.assertEqual(
            assign_part.params, {"uuid": "comp-uuid", "part_uuid": "part-uuid"}
        )
        self.assertEqual(set_package.method, "set_package")
        self.assertEqual(
            set_package.params, {"uuid": "comp-uuid", "package_uuid": "package-uuid"}
        )
        self.assertEqual(set_package_with_part.method, "set_package_with_part")
        self.assertEqual(
            set_package_with_part.params,
            {
                "uuid": "comp-uuid",
                "package_uuid": "package-uuid",
                "part_uuid": "part-uuid",
            },
        )
        self.assertEqual(replace_component.method, "replace_component")
        self.assertEqual(
            replace_component.params,
            {
                "uuid": "comp-uuid",
                "package_uuid": "package-uuid",
                "part_uuid": "part-uuid",
            },
        )
        self.assertEqual(replace_components.method, "replace_components")
        self.assertEqual(
            replace_components.params,
            {
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
        )
        self.assertEqual(
            apply_replacement_plan.method, "apply_component_replacement_plan"
        )
        self.assertEqual(
            apply_replacement_plan.params,
            {
                "replacements": [
                    {"uuid": "comp-1", "package_uuid": "package-1", "part_uuid": None},
                    {"uuid": "comp-2", "package_uuid": None, "part_uuid": "part-2"},
                ]
            },
        )
        self.assertEqual(
            apply_replacement_policy.method, "apply_component_replacement_policy"
        )
        self.assertEqual(
            apply_replacement_policy.params,
            {
                "replacements": [
                    {"uuid": "comp-1", "policy": "best_compatible_package"},
                    {"uuid": "comp-2", "policy": "best_compatible_part"},
                ]
            },
        )
        self.assertEqual(
            apply_scoped_replacement_policy.method,
            "apply_scoped_component_replacement_policy",
        )
        self.assertEqual(
            apply_scoped_replacement_policy.params,
            {
                "scope": {"reference_prefix": "R", "value_equals": "LMV321"},
                "policy": "best_compatible_package",
            },
        )
        self.assertEqual(
            apply_scoped_replacement_plan.method,
            "apply_scoped_component_replacement_plan",
        )
        self.assertEqual(
            apply_scoped_replacement_plan.params["plan"]["policy"],
            "best_compatible_package",
        )
        self.assertEqual(
            apply_scoped_replacement_plan.params["plan"]["replacements"][0][
                "target_package_name"
            ],
            "ALT-3",
        )
        self.assertEqual(
            get_scoped_replacement_plan.method,
            "get_scoped_component_replacement_plan",
        )
        self.assertEqual(
            get_scoped_replacement_plan.params,
            {
                "scope": {"reference_prefix": "R", "value_equals": "LMV321"},
                "policy": "best_compatible_package",
            },
        )
        self.assertEqual(
            edit_scoped_replacement_plan.method,
            "edit_scoped_component_replacement_plan",
        )
        self.assertEqual(
            edit_scoped_replacement_plan.params["exclude_component_uuids"],
            ["comp-2"],
        )
        self.assertEqual(
            edit_scoped_replacement_plan.params["overrides"][0]["component_uuid"],
            "comp-1",
        )
        self.assertEqual(set_net_class.method, "set_net_class")
        self.assertEqual(
            set_net_class.params,
            {
                "net_uuid": "net-uuid",
                "class_name": "power",
                "clearance": 125000,
                "track_width": 250000,
                "via_drill": 300000,
                "via_diameter": 600000,
                "diffpair_width": 0,
                "diffpair_gap": 0,
            },
        )
        self.assertEqual(set_reference.method, "set_reference")
        self.assertEqual(
            set_reference.params, {"uuid": "comp-uuid", "reference": "R10"}
        )
        self.assertEqual(set_rule.method, "set_design_rule")
        self.assertEqual(
            set_rule.params,
            {
                "rule_type": "ClearanceCopper",
                "scope": "All",
                "parameters": {"Clearance": {"min": 125000}},
                "priority": 10,
                "name": "default clearance",
            },
        )
        self.assertEqual(undo.method, "undo")
        self.assertEqual(undo.params, {})
        self.assertEqual(redo.method, "redo")
        self.assertEqual(redo.params, {})

    def test_builds_search_pool_request(self) -> None:
        client = EngineDaemonClient()
        request = client.search_pool_request("sot23")
        self.assertEqual(request.method, "search_pool")
        self.assertEqual(request.params, {"query": "sot23"})

    def test_builds_get_part_and_get_package_requests(self) -> None:
        client = EngineDaemonClient()
        part = client.get_part_request("11111111-1111-1111-1111-111111111111")
        package = client.get_package_request("22222222-2222-2222-2222-222222222222")
        package_candidates = client.get_package_change_candidates_request(
            "33333333-3333-3333-3333-333333333333"
        )
        part_candidates = client.get_part_change_candidates_request(
            "44444444-4444-4444-4444-444444444444"
        )
        replacement_plan = client.get_component_replacement_plan_request(
            "55555555-5555-5555-5555-555555555555"
        )
        self.assertEqual(part.method, "get_part")
        self.assertEqual(part.params, {"uuid": "11111111-1111-1111-1111-111111111111"})
        self.assertEqual(package.method, "get_package")
        self.assertEqual(
            package.params, {"uuid": "22222222-2222-2222-2222-222222222222"}
        )
        self.assertEqual(package_candidates.method, "get_package_change_candidates")
        self.assertEqual(
            package_candidates.params,
            {"uuid": "33333333-3333-3333-3333-333333333333"},
        )
        self.assertEqual(part_candidates.method, "get_part_change_candidates")
        self.assertEqual(
            part_candidates.params,
            {"uuid": "44444444-4444-4444-4444-444444444444"},
        )
        self.assertEqual(replacement_plan.method, "get_component_replacement_plan")
        self.assertEqual(
            replacement_plan.params,
            {"uuid": "55555555-5555-5555-5555-555555555555"},
        )

    def test_builds_explain_violation_request(self) -> None:
        client = EngineDaemonClient()
        request = client.explain_violation_request("drc", 3)
        self.assertEqual(request.method, "explain_violation")
        self.assertEqual(request.params, {"domain": "drc", "index": 3})

    def test_builds_summary_requests(self) -> None:
        client = EngineDaemonClient()
        board = client.get_board_summary_request()
        schematic = client.get_schematic_summary_request()
        self.assertEqual(board.method, "get_board_summary")
        self.assertEqual(board.params, {})
        self.assertEqual(schematic.method, "get_schematic_summary")
        self.assertEqual(schematic.params, {})

    def test_builds_net_info_requests(self) -> None:
        client = EngineDaemonClient()
        board = client.get_net_info_request()
        unrouted = client.get_unrouted_request()
        schematic = client.get_schematic_net_info_request()
        self.assertEqual(board.method, "get_net_info")
        self.assertEqual(board.params, {})
        self.assertEqual(unrouted.method, "get_unrouted")
        self.assertEqual(unrouted.params, {})
        self.assertEqual(schematic.method, "get_schematic_net_info")
        self.assertEqual(schematic.params, {})

    def test_builds_component_and_schematic_object_requests(self) -> None:
        client = EngineDaemonClient()
        components = client.get_components_request()
        netlist = client.get_netlist_request()
        labels = client.get_labels_request()
        symbols = client.get_symbols_request()
        symbol_fields = client.get_symbol_fields_request(
            "11111111-1111-1111-1111-111111111111"
        )
        ports = client.get_ports_request()
        buses = client.get_buses_request()
        bus_entries = client.get_bus_entries_request()
        noconnects = client.get_noconnects_request()
        hierarchy = client.get_hierarchy_request()
        self.assertEqual(components.method, "get_components")
        self.assertEqual(netlist.method, "get_netlist")
        self.assertEqual(labels.method, "get_labels")
        self.assertEqual(symbols.method, "get_symbols")
        self.assertEqual(symbol_fields.method, "get_symbol_fields")
        self.assertEqual(ports.method, "get_ports")
        self.assertEqual(buses.method, "get_buses")
        self.assertEqual(bus_entries.method, "get_bus_entries")
        self.assertEqual(noconnects.method, "get_noconnects")
        self.assertEqual(hierarchy.method, "get_hierarchy")
        self.assertEqual(components.params, {})
        self.assertEqual(netlist.params, {})
        self.assertEqual(labels.params, {})
        self.assertEqual(symbols.params, {})
        self.assertEqual(
            symbol_fields.params,
            {"symbol_uuid": "11111111-1111-1111-1111-111111111111"},
        )
        self.assertEqual(ports.params, {})
        self.assertEqual(buses.params, {})
        self.assertEqual(bus_entries.params, {})
        self.assertEqual(noconnects.params, {})
        self.assertEqual(hierarchy.params, {})

    @patch("server_runtime.subprocess.run")
    def test_exports_route_path_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"export_route_path_proposal",'
                '"contract":"m5_route_path_candidate_v2"}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.export_route_path_proposal(
            "/tmp/demo",
            "11111111-1111-1111-1111-111111111111",
            "22222222-2222-2222-2222-222222222222",
            "33333333-3333-3333-3333-333333333333",
            "route-path-candidate",
            None,
            "/tmp/demo.route-proposal.json",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "export-route-path-proposal",
                "/tmp/demo",
                "--net",
                "11111111-1111-1111-1111-111111111111",
                "--from-anchor",
                "22222222-2222-2222-2222-222222222222",
                "--to-anchor",
                "33333333-3333-3333-3333-333333333333",
                "--candidate",
                "route-path-candidate",
                "--out",
                "/tmp/demo.route-proposal.json",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "export_route_path_proposal")
        self.assertEqual(response.result["contract"], "m5_route_path_candidate_v2")

    @patch("server_runtime.subprocess.run")
    def test_applies_route_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=('{"action":"route_apply","proposal_actions":1,"applied_actions":1}'),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.route_apply(
            "/tmp/demo",
            "11111111-1111-1111-1111-111111111111",
            "22222222-2222-2222-2222-222222222222",
            "33333333-3333-3333-3333-333333333333",
            "route-path-candidate",
            None,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "route-apply",
                "/tmp/demo",
                "--net",
                "11111111-1111-1111-1111-111111111111",
                "--from-anchor",
                "22222222-2222-2222-2222-222222222222",
                "--to-anchor",
                "33333333-3333-3333-3333-333333333333",
                "--candidate",
                "route-path-candidate",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "route_apply")
        self.assertEqual(response.result["proposal_actions"], 1)
        self.assertEqual(response.result["applied_actions"], 1)

    @patch("server_runtime.subprocess.run")
    def test_selects_route_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"route_proposal","selected_candidate":"route-path-candidate",'
                '"selected_contract":"m5_route_path_candidate_v2"}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.route_proposal(
            "/tmp/demo",
            "11111111-1111-1111-1111-111111111111",
            "22222222-2222-2222-2222-222222222222",
            "33333333-3333-3333-3333-333333333333",
            "authored-copper-priority",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "route-proposal",
                "/tmp/demo",
                "--net",
                "11111111-1111-1111-1111-111111111111",
                "--from-anchor",
                "22222222-2222-2222-2222-222222222222",
                "--to-anchor",
                "33333333-3333-3333-3333-333333333333",
                "--profile",
                "authored-copper-priority",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "route_proposal")
        self.assertEqual(response.result["selected_candidate"], "route-path-candidate")

    @patch("server_runtime.subprocess.run")
    def test_reports_route_strategy_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"route_strategy_report","recommended_profile":"authored-copper-priority",'
                '"selected_candidate":"authored-copper-graph"}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.route_strategy_report(
            "/tmp/demo",
            "11111111-1111-1111-1111-111111111111",
            "22222222-2222-2222-2222-222222222222",
            "33333333-3333-3333-3333-333333333333",
            "authored-copper-priority",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "route-strategy-report",
                "/tmp/demo",
                "--net",
                "11111111-1111-1111-1111-111111111111",
                "--from-anchor",
                "22222222-2222-2222-2222-222222222222",
                "--to-anchor",
                "33333333-3333-3333-3333-333333333333",
                "--objective",
                "authored-copper-priority",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "route_strategy_report")
        self.assertEqual(
            response.result["recommended_profile"], "authored-copper-priority"
        )

    @patch("server_runtime.subprocess.run")
    def test_compares_route_strategy_profiles_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"route_strategy_compare","recommended_profile":"default",'
                '"entries":[{"profile":"default"},{"profile":"authored-copper-priority"}]}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.route_strategy_compare(
            "/tmp/demo",
            "11111111-1111-1111-1111-111111111111",
            "22222222-2222-2222-2222-222222222222",
            "33333333-3333-3333-3333-333333333333",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "route-strategy-compare",
                "/tmp/demo",
                "--net",
                "11111111-1111-1111-1111-111111111111",
                "--from-anchor",
                "22222222-2222-2222-2222-222222222222",
                "--to-anchor",
                "33333333-3333-3333-3333-333333333333",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "route_strategy_compare")
        self.assertEqual(response.result["recommended_profile"], "default")

    @patch("server_runtime.subprocess.run")
    def test_reports_route_strategy_delta_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"route_strategy_delta","delta_classification":"different_candidate_family",'
                '"recommended_profile":"default"}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.route_strategy_delta(
            "/tmp/demo",
            "11111111-1111-1111-1111-111111111111",
            "22222222-2222-2222-2222-222222222222",
            "33333333-3333-3333-3333-333333333333",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "route-strategy-delta",
                "/tmp/demo",
                "--net",
                "11111111-1111-1111-1111-111111111111",
                "--from-anchor",
                "22222222-2222-2222-2222-222222222222",
                "--to-anchor",
                "33333333-3333-3333-3333-333333333333",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "route_strategy_delta")
        self.assertEqual(
            response.result["delta_classification"], "different_candidate_family"
        )

    @patch("server_runtime.subprocess.run")
    def test_evaluates_route_strategy_batch_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"route_strategy_batch_evaluate","kind":"native_route_strategy_batch_result_artifact",'
                '"version":1,"summary":{"total_evaluated_requests":2}}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.route_strategy_batch_evaluate("/tmp/route-strategy-batch.json")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "route-strategy-batch-evaluate",
                "--requests",
                "/tmp/route-strategy-batch.json",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "route_strategy_batch_evaluate")
        self.assertEqual(
            response.result["kind"], "native_route_strategy_batch_result_artifact"
        )
        self.assertEqual(response.result["summary"]["total_evaluated_requests"], 2)

    @patch("server_runtime.subprocess.run")
    def test_writes_route_strategy_curated_fixture_suite_via_cli(
        self, run_mock
    ) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"write_route_strategy_curated_fixture_suite","suite_id":"m6_route_strategy_curated_fixture_suite_v1",'
                '"requests_manifest_kind":"native_route_strategy_batch_requests","total_requests":4}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.write_route_strategy_curated_fixture_suite(
            "/tmp/route-strategy-fixtures",
            "/tmp/route-strategy-fixtures/requests.json",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "write-route-strategy-curated-fixture-suite",
                "--out-dir",
                "/tmp/route-strategy-fixtures",
                "--manifest",
                "/tmp/route-strategy-fixtures/requests.json",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(
            response.result["action"], "write_route_strategy_curated_fixture_suite"
        )
        self.assertEqual(
            response.result["suite_id"], "m6_route_strategy_curated_fixture_suite_v1"
        )
        self.assertEqual(response.result["total_requests"], 4)

    @patch("server_runtime.subprocess.run")
    def test_captures_route_strategy_curated_baseline_via_cli(
        self, run_mock
    ) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"capture_route_strategy_curated_baseline","result_kind":"native_route_strategy_batch_result_artifact",'
                '"result_version":1,"total_requests":4}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.capture_route_strategy_curated_baseline(
            "/tmp/route-strategy-fixtures",
            "/tmp/route-strategy-fixtures/requests.json",
            "/tmp/route-strategy-fixtures/result.json",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "capture-route-strategy-curated-baseline",
                "--out-dir",
                "/tmp/route-strategy-fixtures",
                "--manifest",
                "/tmp/route-strategy-fixtures/requests.json",
                "--result",
                "/tmp/route-strategy-fixtures/result.json",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(
            response.result["action"], "capture_route_strategy_curated_baseline"
        )
        self.assertEqual(
            response.result["result_kind"], "native_route_strategy_batch_result_artifact"
        )
        self.assertEqual(response.result["total_requests"], 4)

    @patch("server_runtime.subprocess.run")
    def test_inspects_route_strategy_batch_result_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"inspect_route_strategy_batch_result","kind":"native_route_strategy_batch_result_artifact",'
                '"summary":{"total_evaluated_requests":2}}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.inspect_route_strategy_batch_result(
            "/tmp/route-strategy-batch-result.json"
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "inspect-route-strategy-batch-result",
                "/tmp/route-strategy-batch-result.json",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "inspect_route_strategy_batch_result")
        self.assertEqual(
            response.result["kind"], "native_route_strategy_batch_result_artifact"
        )

    @patch("server_runtime.subprocess.run")
    def test_validates_native_project_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=1,
            stdout=(
                '{"action":"validate_project","project_root":"/tmp/native-project",'
                '"valid":false,"issue_count":1}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.validate_project("/tmp/native-project")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "validate",
                "/tmp/native-project",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "validate_project")
        self.assertEqual(response.result["project_root"], "/tmp/native-project")
        self.assertEqual(response.result["valid"], False)

    @patch("server_runtime.subprocess.run")
    def test_flips_native_board_component_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"set_board_component_layer","component_uuid":"comp-uuid","layer":2}',
            stderr="",
        )
        response = EngineDaemonClient().flip_board_component(
            "/tmp/native-project", "comp-uuid", 2
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "flip-board-component",
                "/tmp/native-project",
                "--component",
                "comp-uuid",
                "--layer",
                "2",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["component_uuid"], "comp-uuid")
        self.assertEqual(response.result["layer"], 2)

    @patch("server_runtime.subprocess.run")
    def test_moves_native_board_component_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"move_board_component","component_uuid":"comp-uuid","x_nm":15000000,"y_nm":12000000}',
            stderr="",
        )
        response = EngineDaemonClient().move_board_component(
            "/tmp/native-project", "comp-uuid", 15_000_000, 12_000_000
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "move-board-component",
                "/tmp/native-project",
                "--component",
                "comp-uuid",
                "--x-nm",
                "15000000",
                "--y-nm",
                "12000000",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["component_uuid"], "comp-uuid")
        self.assertEqual(response.result["x_nm"], 15000000)

    @patch("server_runtime.subprocess.run")
    def test_rotates_native_board_component_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"rotate_board_component","component_uuid":"comp-uuid","rotation_deg":90}',
            stderr="",
        )
        response = EngineDaemonClient().rotate_board_component(
            "/tmp/native-project", "comp-uuid", 90
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "rotate-board-component",
                "/tmp/native-project",
                "--component",
                "comp-uuid",
                "--rotation-deg",
                "90",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["component_uuid"], "comp-uuid")
        self.assertEqual(response.result["rotation_deg"], 90)

    @patch("server_runtime.subprocess.run")
    def test_native_schematic_connectivity_aliases_use_project_cli(self, run_mock) -> None:
        cases = [
            ("create_sheet", ("/tmp/native-project", "Aux"), ["project", "create-sheet", "/tmp/native-project", "--name", "Aux", "--sheet", "sheet-uuid"], {"sheet": "sheet-uuid"}),
            ("delete_sheet", ("/tmp/native-project", "sheet-uuid"), ["project", "delete-sheet", "/tmp/native-project", "--sheet", "sheet-uuid"]),
            ("rename_sheet", ("/tmp/native-project", "sheet-uuid", "Renamed"), ["project", "rename-sheet", "/tmp/native-project", "--sheet", "sheet-uuid", "--name", "Renamed"]),
            ("create_sheet_definition", ("/tmp/native-project", "sheet-uuid", "Main Definition"), ["project", "create-sheet-definition", "/tmp/native-project", "--root-sheet", "sheet-uuid", "--name", "Main Definition", "--definition", "definition-uuid"], {"definition": "definition-uuid"}),
            ("create_sheet_instance", ("/tmp/native-project", "definition-uuid", "Main Instance", 100, 200), ["project", "create-sheet-instance", "/tmp/native-project", "--definition", "definition-uuid", "--name", "Main Instance", "--x-nm", "100", "--y-nm", "200", "--parent-sheet", "sheet-uuid", "--instance", "instance-uuid"], {"parent_sheet": "sheet-uuid", "instance": "instance-uuid"}),
            ("delete_sheet_instance", ("/tmp/native-project", "instance-uuid"), ["project", "delete-sheet-instance", "/tmp/native-project", "--instance", "instance-uuid"]),
            ("move_sheet_instance", ("/tmp/native-project", "instance-uuid", 300, 400), ["project", "move-sheet-instance", "/tmp/native-project", "--instance", "instance-uuid", "--x-nm", "300", "--y-nm", "400"]),
            ("bind_sheet_instance_port", ("/tmp/native-project", "instance-uuid", "port-uuid"), ["project", "bind-sheet-instance-port", "/tmp/native-project", "--instance", "instance-uuid", "--port", "port-uuid"]),
            ("unbind_sheet_instance_port", ("/tmp/native-project", "instance-uuid", "port-uuid"), ["project", "unbind-sheet-instance-port", "/tmp/native-project", "--instance", "instance-uuid", "--port", "port-uuid"]),
            ("draw_wire", ("/tmp/native-project", "sheet-uuid", 0, 0, 1000, 0), ["project", "draw-wire", "/tmp/native-project", "--sheet", "sheet-uuid", "--from-x-nm", "0", "--from-y-nm", "0", "--to-x-nm", "1000", "--to-y-nm", "0"]),
            ("delete_wire", ("/tmp/native-project", "wire-uuid"), ["project", "delete-wire", "/tmp/native-project", "--wire", "wire-uuid"]),
            ("place_junction", ("/tmp/native-project", "sheet-uuid", 100, 200), ["project", "place-junction", "/tmp/native-project", "--sheet", "sheet-uuid", "--x-nm", "100", "--y-nm", "200"]),
            ("delete_junction", ("/tmp/native-project", "junction-uuid"), ["project", "delete-junction", "/tmp/native-project", "--junction", "junction-uuid"]),
            ("place_noconnect", ("/tmp/native-project", "sheet-uuid", "symbol-uuid", "pin-uuid", 100, 200), ["project", "place-noconnect", "/tmp/native-project", "--sheet", "sheet-uuid", "--symbol", "symbol-uuid", "--pin", "pin-uuid", "--x-nm", "100", "--y-nm", "200"]),
            ("delete_noconnect", ("/tmp/native-project", "noconnect-uuid"), ["project", "delete-noconnect", "/tmp/native-project", "--noconnect", "noconnect-uuid"]),
            ("place_label", ("/tmp/native-project", "sheet-uuid", "VCC", 100, 200), ["project", "place-label", "/tmp/native-project", "--sheet", "sheet-uuid", "--name", "VCC", "--x-nm", "100", "--y-nm", "200", "--kind", "power"], {"kind": "power"}),
            ("rename_label", ("/tmp/native-project", "label-uuid", "VEE"), ["project", "rename-label", "/tmp/native-project", "--label", "label-uuid", "--name", "VEE"]),
            ("delete_label", ("/tmp/native-project", "label-uuid"), ["project", "delete-label", "/tmp/native-project", "--label", "label-uuid"]),
            ("place_port", ("/tmp/native-project", "sheet-uuid", "OUT", "output", 100, 200), ["project", "place-port", "/tmp/native-project", "--sheet", "sheet-uuid", "--name", "OUT", "--direction", "output", "--x-nm", "100", "--y-nm", "200"]),
            ("edit_port", ("/tmp/native-project", "port-uuid"), ["project", "edit-port", "/tmp/native-project", "--port", "port-uuid", "--direction", "input"], {"direction": "input"}),
            ("delete_port", ("/tmp/native-project", "port-uuid"), ["project", "delete-port", "/tmp/native-project", "--port", "port-uuid"]),
            ("create_bus", ("/tmp/native-project", "sheet-uuid", "DATA", ["DATA0", "DATA1"]), ["project", "create-bus", "/tmp/native-project", "--sheet", "sheet-uuid", "--name", "DATA", "--member", "DATA0", "--member", "DATA1"]),
            ("edit_bus_members", ("/tmp/native-project", "bus-uuid", ["DATA0", "DATA1", "DATA2"]), ["project", "edit-bus-members", "/tmp/native-project", "--bus", "bus-uuid", "--member", "DATA0", "--member", "DATA1", "--member", "DATA2"]),
            ("delete_bus", ("/tmp/native-project", "bus-uuid"), ["project", "delete-bus", "/tmp/native-project", "--bus", "bus-uuid"]),
            ("place_bus_entry", ("/tmp/native-project", "sheet-uuid", "bus-uuid", 100, 200), ["project", "place-bus-entry", "/tmp/native-project", "--sheet", "sheet-uuid", "--bus", "bus-uuid", "--x-nm", "100", "--y-nm", "200", "--wire", "wire-uuid"], {"wire": "wire-uuid"}),
            ("delete_bus_entry", ("/tmp/native-project", "bus-entry-uuid"), ["project", "delete-bus-entry", "/tmp/native-project", "--bus-entry", "bus-entry-uuid"]),
            ("place_schematic_text", ("/tmp/native-project", "sheet-uuid", "note", 100, 200), ["project", "place-text", "/tmp/native-project", "--sheet", "sheet-uuid", "--text", "note", "--x-nm", "100", "--y-nm", "200", "--rotation-deg", "90"], {"rotation_deg": 90}),
            ("edit_schematic_text", ("/tmp/native-project", "text-uuid"), ["project", "edit-text", "/tmp/native-project", "--text", "text-uuid", "--value", "new note"], {"value": "new note"}),
            ("delete_schematic_text", ("/tmp/native-project", "text-uuid"), ["project", "delete-text", "/tmp/native-project", "--text", "text-uuid"]),
            ("place_drawing_line", ("/tmp/native-project", "sheet-uuid", 0, 0, 100, 0), ["project", "place-drawing-line", "/tmp/native-project", "--sheet", "sheet-uuid", "--from-x-nm", "0", "--from-y-nm", "0", "--to-x-nm", "100", "--to-y-nm", "0"]),
            ("place_drawing_rect", ("/tmp/native-project", "sheet-uuid", 0, 0, 100, 100), ["project", "place-drawing-rect", "/tmp/native-project", "--sheet", "sheet-uuid", "--min-x-nm", "0", "--min-y-nm", "0", "--max-x-nm", "100", "--max-y-nm", "100"]),
            ("place_drawing_circle", ("/tmp/native-project", "sheet-uuid", 50, 50, 25), ["project", "place-drawing-circle", "/tmp/native-project", "--sheet", "sheet-uuid", "--center-x-nm", "50", "--center-y-nm", "50", "--radius-nm", "25"]),
            ("place_drawing_arc", ("/tmp/native-project", "sheet-uuid", 50, 50, 25, 0, 90000), ["project", "place-drawing-arc", "/tmp/native-project", "--sheet", "sheet-uuid", "--center-x-nm", "50", "--center-y-nm", "50", "--radius-nm", "25", "--start-angle-mdeg", "0", "--end-angle-mdeg", "90000"]),
            ("edit_drawing_line", ("/tmp/native-project", "drawing-uuid"), ["project", "edit-drawing-line", "/tmp/native-project", "--drawing", "drawing-uuid", "--to-x-nm", "200", "--to-y-nm", "0"], {"to_x_nm": 200, "to_y_nm": 0}),
            ("edit_drawing_rect", ("/tmp/native-project", "drawing-uuid"), ["project", "edit-drawing-rect", "/tmp/native-project", "--drawing", "drawing-uuid", "--max-x-nm", "200", "--max-y-nm", "100"], {"max_x_nm": 200, "max_y_nm": 100}),
            ("edit_drawing_circle", ("/tmp/native-project", "drawing-uuid"), ["project", "edit-drawing-circle", "/tmp/native-project", "--drawing", "drawing-uuid", "--radius-nm", "50"], {"radius_nm": 50}),
            ("edit_drawing_arc", ("/tmp/native-project", "drawing-uuid"), ["project", "edit-drawing-arc", "/tmp/native-project", "--drawing", "drawing-uuid", "--end-angle-mdeg", "180000"], {"end_angle_mdeg": 180000}),
            ("delete_drawing", ("/tmp/native-project", "drawing-uuid"), ["project", "delete-drawing", "/tmp/native-project", "--drawing", "drawing-uuid"]),
            ("place_symbol", ("/tmp/native-project", "sheet-uuid", "U1", "OPA", 100, 200), ["project", "place-symbol", "/tmp/native-project", "--sheet", "sheet-uuid", "--reference", "U1", "--value", "OPA", "--x-nm", "100", "--y-nm", "200", "--lib-id", "Device:R", "--rotation-deg", "90", "--mirrored"], {"lib_id": "Device:R", "rotation_deg": 90, "mirrored": True}),
            ("move_symbol", ("/tmp/native-project", "symbol-uuid", 200, 300), ["project", "move-symbol", "/tmp/native-project", "--symbol", "symbol-uuid", "--x-nm", "200", "--y-nm", "300"]),
            ("rotate_symbol", ("/tmp/native-project", "symbol-uuid", 180), ["project", "rotate-symbol", "/tmp/native-project", "--symbol", "symbol-uuid", "--rotation-deg", "180"]),
            ("mirror_symbol", ("/tmp/native-project", "symbol-uuid"), ["project", "mirror-symbol", "/tmp/native-project", "--symbol", "symbol-uuid"]),
            ("delete_symbol", ("/tmp/native-project", "symbol-uuid"), ["project", "delete-symbol", "/tmp/native-project", "--symbol", "symbol-uuid"]),
            ("set_symbol_reference", ("/tmp/native-project", "symbol-uuid", "U2"), ["project", "set-symbol-reference", "/tmp/native-project", "--symbol", "symbol-uuid", "--reference", "U2"]),
            ("set_symbol_value", ("/tmp/native-project", "symbol-uuid", "OPA1656"), ["project", "set-symbol-value", "/tmp/native-project", "--symbol", "symbol-uuid", "--value", "OPA1656"]),
            ("set_symbol_display_mode", ("/tmp/native-project", "symbol-uuid", "library-default"), ["project", "set-symbol-display-mode", "/tmp/native-project", "--symbol", "symbol-uuid", "--mode", "library-default"]),
            ("set_symbol_hidden_power_behavior", ("/tmp/native-project", "symbol-uuid", "explicit"), ["project", "set-symbol-hidden-power-behavior", "/tmp/native-project", "--symbol", "symbol-uuid", "--behavior", "explicit"]),
            ("set_symbol_unit", ("/tmp/native-project", "symbol-uuid", "A"), ["project", "set-symbol-unit", "/tmp/native-project", "--symbol", "symbol-uuid", "--unit", "A"]),
            ("clear_symbol_unit", ("/tmp/native-project", "symbol-uuid"), ["project", "clear-symbol-unit", "/tmp/native-project", "--symbol", "symbol-uuid"]),
            ("set_symbol_gate", ("/tmp/native-project", "symbol-uuid", "gate-uuid"), ["project", "set-symbol-gate", "/tmp/native-project", "--symbol", "symbol-uuid", "--gate", "gate-uuid"]),
            ("clear_symbol_gate", ("/tmp/native-project", "symbol-uuid"), ["project", "clear-symbol-gate", "/tmp/native-project", "--symbol", "symbol-uuid"]),
            ("set_symbol_entity", ("/tmp/native-project", "symbol-uuid", "entity-uuid"), ["project", "set-symbol-entity", "/tmp/native-project", "--symbol", "symbol-uuid", "--entity", "entity-uuid"]),
            ("clear_symbol_entity", ("/tmp/native-project", "symbol-uuid"), ["project", "clear-symbol-entity", "/tmp/native-project", "--symbol", "symbol-uuid"]),
            ("set_symbol_part", ("/tmp/native-project", "symbol-uuid", "part-uuid"), ["project", "set-symbol-part", "/tmp/native-project", "--symbol", "symbol-uuid", "--part", "part-uuid"]),
            ("clear_symbol_part", ("/tmp/native-project", "symbol-uuid"), ["project", "clear-symbol-part", "/tmp/native-project", "--symbol", "symbol-uuid"]),
            ("set_symbol_lib_id", ("/tmp/native-project", "symbol-uuid", "Device:R"), ["project", "set-symbol-lib-id", "/tmp/native-project", "--symbol", "symbol-uuid", "--lib-id", "Device:R"]),
            ("clear_symbol_lib_id", ("/tmp/native-project", "symbol-uuid"), ["project", "clear-symbol-lib-id", "/tmp/native-project", "--symbol", "symbol-uuid"]),
            ("set_pin_override", ("/tmp/native-project", "symbol-uuid", "pin-uuid", False), ["project", "set-pin-override", "/tmp/native-project", "--symbol", "symbol-uuid", "--pin", "pin-uuid", "--visible", "false", "--x-nm", "10"], {"x_nm": 10}),
            ("clear_pin_override", ("/tmp/native-project", "symbol-uuid", "pin-uuid"), ["project", "clear-pin-override", "/tmp/native-project", "--symbol", "symbol-uuid", "--pin", "pin-uuid"]),
            ("add_symbol_field", ("/tmp/native-project", "symbol-uuid", "MPN", "ABC"), ["project", "add-symbol-field", "/tmp/native-project", "--symbol", "symbol-uuid", "--key", "MPN", "--value", "ABC", "--hidden", "--x-nm", "10"], {"hidden": True, "x_nm": 10}),
            ("edit_symbol_field", ("/tmp/native-project", "field-uuid"), ["project", "edit-symbol-field", "/tmp/native-project", "--field", "field-uuid", "--value", "DEF", "--visible", "true"], {"value": "DEF", "visible": True}),
            ("delete_symbol_field", ("/tmp/native-project", "field-uuid"), ["project", "delete-symbol-field", "/tmp/native-project", "--field", "field-uuid"]),
        ]
        client = EngineDaemonClient()
        for case in cases:
            method_name, args, expected_tail, kwargs = (*case, None) if len(case) == 3 else case
            with self.subTest(method_name=method_name):
                run_mock.reset_mock()
                run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"schematic_connectivity_alias"}', stderr="")
                getattr(client, method_name)(*args, **(kwargs or {}))
                run_mock.assert_called_once_with(["datum-eda", "--format", "json", *expected_tail], capture_output=True, text=True, check=False)

    @patch("server_runtime.subprocess.run")
    def test_native_board_component_lifecycle_aliases_use_project_cli(self, run_mock) -> None:
        cases = [
            (
                "place_board_component",
                ("/tmp/native-project", "part-uuid", "pkg-uuid", "U1", "OPA", 15000000, 12000000, 1),
                [
                    "project", "place-board-component", "/tmp/native-project",
                    "--part", "part-uuid", "--package", "pkg-uuid", "--reference", "U1",
                    "--value", "OPA", "--x-nm", "15000000", "--y-nm", "12000000", "--layer", "1",
                ],
            ),
            (
                "delete_board_component",
                ("/tmp/native-project", "comp-uuid"),
                ["project", "delete-board-component", "/tmp/native-project", "--component", "comp-uuid"],
            ),
            (
                "set_board_component_reference",
                ("/tmp/native-project", "comp-uuid", "U2"),
                ["project", "set-board-component-reference", "/tmp/native-project", "--component", "comp-uuid", "--reference", "U2"],
            ),
            (
                "set_board_component_value",
                ("/tmp/native-project", "comp-uuid", "OPA1656"),
                ["project", "set-board-component-value", "/tmp/native-project", "--component", "comp-uuid", "--value", "OPA1656"],
            ),
            (
                "set_board_component_part",
                ("/tmp/native-project", "comp-uuid", "part-next"),
                ["project", "set-board-component-part", "/tmp/native-project", "--component", "comp-uuid", "--part", "part-next"],
            ),
            (
                "set_board_component_package",
                ("/tmp/native-project", "comp-uuid", "pkg-next"),
                ["project", "set-board-component-package", "/tmp/native-project", "--component", "comp-uuid", "--package", "pkg-next"],
            ),
            (
                "lock_board_component",
                ("/tmp/native-project", "comp-uuid"),
                ["project", "set-board-component-locked", "/tmp/native-project", "--component", "comp-uuid"],
            ),
            (
                "unlock_board_component",
                ("/tmp/native-project", "comp-uuid"),
                ["project", "clear-board-component-locked", "/tmp/native-project", "--component", "comp-uuid"],
            ),
        ]
        client = EngineDaemonClient()
        for method_name, args, expected_tail in cases:
            with self.subTest(method_name=method_name):
                run_mock.reset_mock()
                run_mock.return_value = subprocess.CompletedProcess(
                    args=[],
                    returncode=0,
                    stdout='{"action":"board_component_alias","component_uuid":"comp-uuid"}',
                    stderr="",
                )
                getattr(client, method_name)(*args)
                run_mock.assert_called_once_with(
                    ["datum-eda", "--format", "json", *expected_tail],
                    capture_output=True,
                    text=True,
                    check=False,
                )

    @patch("server_runtime.subprocess.run")
    def test_native_board_routing_aliases_use_project_cli(self, run_mock) -> None:
        cases = [
            ("draw_board_track", ("/tmp/native-project", "net-uuid", 0, 0, 1000, 0, 150, 1), ["project", "draw-board-track", "/tmp/native-project", "--net", "net-uuid", "--from-x-nm", "0", "--from-y-nm", "0", "--to-x-nm", "1000", "--to-y-nm", "0", "--width-nm", "150", "--layer", "1"]),
            ("edit_board_track", ("/tmp/native-project", "track-uuid"), ["project", "edit-board-track", "/tmp/native-project", "--track", "track-uuid"]),
            ("delete_board_track", ("/tmp/native-project", "track-uuid"), ["project", "delete-board-track", "/tmp/native-project", "--track", "track-uuid"]),
            ("place_board_via", ("/tmp/native-project", "net-uuid", 100, 200, 300, 600, 1, 2), ["project", "place-board-via", "/tmp/native-project", "--net", "net-uuid", "--x-nm", "100", "--y-nm", "200", "--drill-nm", "300", "--diameter-nm", "600", "--from-layer", "1", "--to-layer", "2"]),
            ("edit_board_via", ("/tmp/native-project", "via-uuid"), ["project", "edit-board-via", "/tmp/native-project", "--via", "via-uuid"]),
            ("delete_board_via", ("/tmp/native-project", "via-uuid"), ["project", "delete-board-via", "/tmp/native-project", "--via", "via-uuid"]),
            ("place_board_zone", ("/tmp/native-project", "net-uuid", ["0:0", "1000:0", "1000:1000"], 1, 250, 200), ["project", "place-board-zone", "/tmp/native-project", "--net", "net-uuid", "--vertex", "0:0", "--vertex", "1000:0", "--vertex", "1000:1000", "--layer", "1", "--thermal-relief", "true", "--thermal-gap-nm", "250", "--thermal-spoke-width-nm", "200"], {"thermal_relief": True}),
            ("edit_board_zone", ("/tmp/native-project", "zone-uuid"), ["project", "edit-board-zone", "/tmp/native-project", "--zone", "zone-uuid", "--vertex", "0:0", "--vertex", "1000:0", "--layer", "2", "--priority", "5", "--thermal-relief", "false", "--thermal-gap-nm", "0", "--thermal-spoke-width-nm", "0"], {"vertices": ["0:0", "1000:0"], "layer": 2, "priority": 5, "thermal_relief": False, "thermal_gap_nm": 0, "thermal_spoke_width_nm": 0}),
            ("delete_board_zone", ("/tmp/native-project", "zone-uuid"), ["project", "delete-board-zone", "/tmp/native-project", "--zone", "zone-uuid"]),
        ]
        client = EngineDaemonClient()
        for case in cases:
            method_name, args, expected_tail, kwargs = (*case, None) if len(case) == 3 else case
            with self.subTest(method_name=method_name):
                run_mock.reset_mock()
                run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"board_routing_alias"}', stderr="")
                getattr(client, method_name)(*args, **(kwargs or {}))
                run_mock.assert_called_once_with(["datum-eda", "--format", "json", *expected_tail], capture_output=True, text=True, check=False)

    @patch("server_runtime.subprocess.run")
    def test_native_board_pad_and_netclass_aliases_use_project_cli(self, run_mock) -> None:
        cases = [
            ("place_board_pad", ("/tmp/native-project", "pkg-uuid", "1", 10, 20, 1), ["project", "place-board-pad", "/tmp/native-project", "--package", "pkg-uuid", "--name", "1", "--x-nm", "10", "--y-nm", "20", "--layer", "1"]),
            ("edit_board_pad", ("/tmp/native-project", "pad-uuid"), ["project", "edit-board-pad", "/tmp/native-project", "--pad", "pad-uuid"]),
            ("delete_board_pad", ("/tmp/native-project", "pad-uuid"), ["project", "delete-board-pad", "/tmp/native-project", "--pad", "pad-uuid"]),
            ("set_board_pad_net", ("/tmp/native-project", "pad-uuid", "net-uuid"), ["project", "set-board-pad-net", "/tmp/native-project", "--pad", "pad-uuid", "--net", "net-uuid"]),
            ("clear_board_pad_net", ("/tmp/native-project", "pad-uuid"), ["project", "clear-board-pad-net", "/tmp/native-project", "--pad", "pad-uuid"]),
            ("place_board_net", ("/tmp/native-project", "VCC", "class-uuid"), ["project", "place-board-net", "/tmp/native-project", "--name", "VCC", "--class", "class-uuid", "--impedance-target-ohms", "50.0", "--impedance-tolerance-pct", "10", "--controlled-dielectric-layer", "2"], {"impedance_target_ohms": "50.0", "impedance_tolerance_pct": "10", "controlled_dielectric_layer": 2}),
            ("edit_board_net", ("/tmp/native-project", "net-uuid"), ["project", "edit-board-net", "/tmp/native-project", "--net", "net-uuid", "--impedance-tolerance-pct", "7.5", "--clear-controlled-impedance"], {"impedance_tolerance_pct": "7.5", "clear_controlled_impedance": True}),
            ("delete_board_net", ("/tmp/native-project", "net-uuid"), ["project", "delete-board-net", "/tmp/native-project", "--net", "net-uuid"], None),
            ("set_board_name", ("/tmp/native-project", "Main Board"), ["project", "set-board-name", "/tmp/native-project", "--name", "Main Board"], None),
            ("set_board_outline", ("/tmp/native-project", ["0,0", "1000,0", "1000,1000"]), ["project", "set-board-outline", "/tmp/native-project", "--vertex", "0,0", "--vertex", "1000,0", "--vertex", "1000,1000"], None),
            ("set_board_stackup", ("/tmp/native-project", ["1:F.Cu:copper:35000", "2:B.Cu:copper:35000"]), ["project", "set-board-stackup", "/tmp/native-project", "--layer", "1:F.Cu:copper:35000", "--layer", "2:B.Cu:copper:35000"], None),
            ("add_default_top_stackup", ("/tmp/native-project",), ["project", "add-default-top-stackup", "/tmp/native-project"], None),
            ("place_board_keepout", ("/tmp/native-project", ["0:0", "1000:0", "1000:1000"], [1, 2], "route"), ["project", "place-board-keepout", "/tmp/native-project", "--kind", "route", "--vertex", "0:0", "--vertex", "1000:0", "--vertex", "1000:1000", "--layer", "1", "--layer", "2"], None),
            ("edit_board_keepout", ("/tmp/native-project", "keepout-uuid"), ["project", "edit-board-keepout", "/tmp/native-project", "--keepout", "keepout-uuid"], None),
            ("delete_board_keepout", ("/tmp/native-project", "keepout-uuid"), ["project", "delete-board-keepout", "/tmp/native-project", "--keepout", "keepout-uuid"], None),
            ("place_board_dimension", ("/tmp/native-project", 0, 0, 1000, 0, 1), ["project", "place-board-dimension", "/tmp/native-project", "--from-x-nm", "0", "--from-y-nm", "0", "--to-x-nm", "1000", "--to-y-nm", "0", "--layer", "1"], None),
            ("edit_board_dimension", ("/tmp/native-project", "dimension-uuid"), ["project", "edit-board-dimension", "/tmp/native-project", "--dimension", "dimension-uuid", "--clear-text"], {"clear_text": True}),
            ("delete_board_dimension", ("/tmp/native-project", "dimension-uuid"), ["project", "delete-board-dimension", "/tmp/native-project", "--dimension", "dimension-uuid"], None),
            ("place_board_text", ("/tmp/native-project", "REF**", 10, 20, 1), ["project", "place-board-text", "/tmp/native-project", "--text", "REF**", "--x-nm", "10", "--y-nm", "20", "--layer", "1"], None),
            ("edit_board_text", ("/tmp/native-project", "text-uuid"), ["project", "edit-board-text", "/tmp/native-project", "--text", "text-uuid", "--mirrored", "false"], {"mirrored": False}),
            ("delete_board_text", ("/tmp/native-project", "text-uuid"), ["project", "delete-board-text", "/tmp/native-project", "--text", "text-uuid"], None),
            ("place_board_net_class", ("/tmp/native-project", "Signal", 150, 150, 300, 600), ["project", "place-board-net-class", "/tmp/native-project", "--name", "Signal", "--clearance-nm", "150", "--track-width-nm", "150", "--via-drill-nm", "300", "--via-diameter-nm", "600"]),
            ("edit_board_net_class", ("/tmp/native-project", "class-uuid"), ["project", "edit-board-net-class", "/tmp/native-project", "--net-class", "class-uuid"]),
            ("delete_board_net_class", ("/tmp/native-project", "class-uuid"), ["project", "delete-board-net-class", "/tmp/native-project", "--net-class", "class-uuid"]),
        ]
        client = EngineDaemonClient()
        for case in cases:
            method_name, args, expected_tail, kwargs = (*case, None) if len(case) == 3 else case
            with self.subTest(method_name=method_name):
                run_mock.reset_mock()
                run_mock.return_value = subprocess.CompletedProcess(args=[], returncode=0, stdout='{"action":"board_pad_or_netclass_alias"}', stderr="")
                getattr(client, method_name)(*args, **(kwargs or {}))
                run_mock.assert_called_once_with(["datum-eda", "--format", "json", *expected_tail], capture_output=True, text=True, check=False)

    @patch("server_runtime.subprocess.run")
    def test_lists_output_jobs_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"output_jobs","output_job_count":1}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.get_output_jobs("/tmp/native-project")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "query",
                "output-jobs",
                "/tmp/native-project",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "output_jobs")
        self.assertEqual(response.result["output_job_count"], 1)

    @patch("server_runtime.subprocess.run")
    def test_generates_artifacts_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"artifact_generate_v1","generated_count":1}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.generate_artifacts(
            "/tmp/native-project", "/tmp/fab", "gerber-set", "doa2526"
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "artifact",
                "generate",
                "/tmp/native-project",
                "--output-dir",
                "/tmp/fab",
                "--include",
                "gerber-set",
                "--prefix",
                "doa2526",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "artifact_generate_v1")
        self.assertEqual(response.result["generated_count"], 1)

    @patch("server_runtime.subprocess.run")
    def test_lists_artifacts_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"artifact_metadata_list","artifact_count":1}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.get_artifacts("/tmp/native-project")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "artifact",
                "list",
                "/tmp/native-project",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "artifact_metadata_list")
        self.assertEqual(response.result["artifact_count"], 1)

    @patch("server_runtime.subprocess.run")
    def test_shows_artifact_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"artifact_metadata_v1","artifact":{"artifact_id":"artifact-test"}}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.show_artifact("/tmp/native-project", "artifact-test")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "artifact",
                "show",
                "/tmp/native-project",
                "--artifact",
                "artifact-test",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "artifact_metadata_v1")
        self.assertEqual(response.result["artifact"]["artifact_id"], "artifact-test")

    @patch("server_runtime.subprocess.run")
    def test_gets_artifact_files_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"artifact_files_v1","file_count":1}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.get_artifact_files("/tmp/native-project", "artifact-test")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "artifact",
                "files",
                "/tmp/native-project",
                "--artifact",
                "artifact-test",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "artifact_files_v1")
        self.assertEqual(response.result["file_count"], 1)

    @patch("server_runtime.subprocess.run")
    def test_previews_artifact_file_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"artifact_file_preview_v1","preview_kind":"gerber_rs274x"}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.preview_artifact_file(
            "/tmp/native-project", "artifact-test", None, "fab/doa2526.gbr"
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "artifact",
                "preview",
                "/tmp/native-project",
                "--artifact",
                "artifact-test",
                "--file",
                "fab/doa2526.gbr",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "artifact_file_preview_v1")
        self.assertEqual(response.result["preview_kind"], "gerber_rs274x")

    @patch("server_runtime.subprocess.run")
    def test_compares_artifacts_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"artifact_metadata_compare_v1","equivalent":false}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.compare_artifacts(
            "/tmp/native-project", "artifact-before", "artifact-after"
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "artifact",
                "compare",
                "/tmp/native-project",
                "--before",
                "artifact-before",
                "--after",
                "artifact-after",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "artifact_metadata_compare_v1")
        self.assertEqual(response.result["equivalent"], False)

    @patch("server_runtime.subprocess.run")
    def test_validates_artifact_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=1,
            stdout='{"contract":"artifact_metadata_validation_v1","valid":false}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.validate_artifact("/tmp/native-project", "artifact-test")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "artifact",
                "validate",
                "/tmp/native-project",
                "--artifact",
                "artifact-test",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "artifact_metadata_validation_v1")
        self.assertEqual(response.result["valid"], False)

    @patch("server_runtime.subprocess.run")
    def test_creates_gerber_output_job_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"create_gerber_output_job","created":true}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.create_gerber_output_job(
            "/tmp/native-project",
            "fab/doa2526",
            "Fabrication Gerbers",
            "fab/doa2526",
            "/tmp/native-project/fab",
            variant="variant-test",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "create-gerber-output-job",
                "/tmp/native-project",
                "--prefix",
                "fab/doa2526",
                "--output-dir",
                "/tmp/native-project/fab",
                "--name",
                "Fabrication Gerbers",
                "--manufacturing-plan",
                "fab/doa2526",
                "--variant",
                "variant-test",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "create_gerber_output_job")
        self.assertEqual(response.result["created"], True)

    @patch("server_runtime.subprocess.run")
    def test_creates_output_job_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"create_output_job","created":true}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.create_output_job(
            "/tmp/native-project",
            "fab/doa2526",
            "drill",
            "Fabrication Drill",
            "fab/doa2526",
            "/tmp/native-project/fab",
            variant="variant-test",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "create-output-job",
                "/tmp/native-project",
                "--prefix",
                "fab/doa2526",
                "--include",
                "drill",
                "--output-dir",
                "/tmp/native-project/fab",
                "--name",
                "Fabrication Drill",
                "--manufacturing-plan",
                "fab/doa2526",
                "--variant",
                "variant-test",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "create_output_job")
        self.assertEqual(response.result["created"], True)

    @patch("server_runtime.subprocess.run")
    def test_proposes_output_job_create_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"proposal_create_v1","action":"propose_create_output_job"}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.create_output_job_proposal(
            "/tmp/native-project",
            "fab/doa2526",
            "drill",
            "Reviewed Drill",
            None,
            "/tmp/native-project/fab",
            "proposal-create-test",
            "review output job creation",
            variant="variant-test",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "proposal",
                "create-output-job",
                "/tmp/native-project",
                "--prefix",
                "fab/doa2526",
                "--include",
                "drill",
                "--output-dir",
                "/tmp/native-project/fab",
                "--name",
                "Reviewed Drill",
                "--variant",
                "variant-test",
                "--proposal",
                "proposal-create-test",
                "--rationale",
                "review output job creation",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "proposal_create_v1")

    @patch("server_runtime.subprocess.run")
    def test_updates_output_job_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"update_output_job","created":false}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.update_output_job(
            "/tmp/native-project",
            "11111111-1111-1111-1111-111111111111",
            "Updated Gerbers",
            "/tmp/native-project/fab",
            None,
            True,
            False,
            variant="variant-test",
            clear_variant=True,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "update-output-job",
                "/tmp/native-project",
                "--output-job",
                "11111111-1111-1111-1111-111111111111",
                "--name",
                "Updated Gerbers",
                "--output-dir",
                "/tmp/native-project/fab",
                "--variant",
                "variant-test",
                "--clear-manufacturing-plan",
                "--clear-variant",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "update_output_job")
        self.assertEqual(response.result["created"], False)

    @patch("server_runtime.subprocess.run")
    def test_proposes_output_job_update_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"proposal_create_v1","action":"propose_update_output_job"}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.update_output_job_proposal(
            "/tmp/native-project",
            "11111111-1111-1111-1111-111111111111",
            "Reviewed Gerbers",
            "/tmp/native-project/fab",
            None,
            True,
            False,
            "proposal-test",
            "review output job update",
            variant="variant-test",
            clear_variant=True,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "proposal",
                "update-output-job",
                "/tmp/native-project",
                "--output-job",
                "11111111-1111-1111-1111-111111111111",
                "--name",
                "Reviewed Gerbers",
                "--output-dir",
                "/tmp/native-project/fab",
                "--variant",
                "variant-test",
                "--clear-manufacturing-plan",
                "--clear-variant",
                "--proposal",
                "proposal-test",
                "--rationale",
                "review output job update",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "proposal_create_v1")

    @patch("server_runtime.subprocess.run")
    def test_deletes_output_job_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"delete_output_job","created":false}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.delete_output_job(
            "/tmp/native-project",
            "11111111-1111-1111-1111-111111111111",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "delete-output-job",
                "/tmp/native-project",
                "--output-job",
                "11111111-1111-1111-1111-111111111111",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "delete_output_job")
        self.assertEqual(response.result["created"], False)

    @patch("server_runtime.subprocess.run")
    def test_proposes_output_job_delete_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"contract":"proposal_create_v1","action":"propose_delete_output_job"}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.delete_output_job_proposal(
            "/tmp/native-project",
            "11111111-1111-1111-1111-111111111111",
            "proposal-delete-test",
            "review output job deletion",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "proposal",
                "delete-output-job",
                "/tmp/native-project",
                "--output-job",
                "11111111-1111-1111-1111-111111111111",
                "--proposal",
                "proposal-delete-test",
                "--rationale",
                "review output job deletion",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["contract"], "proposal_create_v1")

    @patch("server_runtime.subprocess.run")
    def test_exports_manufacturing_set_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"export_manufacturing_set",'
                '"artifact_metadata":{"kind":"manufacturing_set"},'
                '"output_job_run":{"status":"succeeded"}}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.export_manufacturing_set(
            "/tmp/native-project", "/tmp/fab", "doa2526"
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "artifact",
                "export-manufacturing-set",
                "/tmp/native-project",
                "--output-dir",
                "/tmp/fab",
                "--prefix",
                "doa2526",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "export_manufacturing_set")
        self.assertEqual(
            response.result["artifact_metadata"]["kind"], "manufacturing_set"
        )

    @patch("server_runtime.subprocess.run")
    def test_validates_manufacturing_set_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=1,
            stdout=(
                '{"action":"validate_manufacturing_set","valid":false,'
                '"artifact_validation_state":"invalid",'
                '"artifact_file_hash_mismatch_count":1}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.validate_manufacturing_set(
            "/tmp/native-project", "/tmp/fab", "doa2526"
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "artifact",
                "validate-manufacturing-set",
                "/tmp/native-project",
                "--output-dir",
                "/tmp/fab",
                "--prefix",
                "doa2526",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "validate_manufacturing_set")
        self.assertEqual(response.result["valid"], False)
        self.assertEqual(response.result["artifact_validation_state"], "invalid")

    @patch("server_runtime.subprocess.run")
    def test_queries_panel_projections_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"panel_projections","panel_projection_count":1}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.get_panel_projections("/tmp/native-project")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "query",
                "panel-projections",
                "/tmp/native-project",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "panel_projections")
        self.assertEqual(response.result["panel_projection_count"], 1)

    @patch("server_runtime.subprocess.run")
    def test_creates_panel_projection_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"create_panel_projection","created":true}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.create_panel_projection(
            "/tmp/native-project", "main-panel", "Main Panel", "main", 1000, 2000, 90
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "create-panel-projection",
                "/tmp/native-project",
                "--key",
                "main-panel",
                "--name",
                "Main Panel",
                "--board",
                "main",
                "--x-nm",
                "1000",
                "--y-nm",
                "2000",
                "--rotation-deg",
                "90",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "create_panel_projection")
        self.assertEqual(response.result["created"], True)

    @patch("server_runtime.subprocess.run")
    def test_updates_panel_projection_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"update_panel_projection","created":false}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.update_panel_projection(
            "/tmp/native-project",
            "11111111-1111-1111-1111-111111111111",
            "Updated Panel",
            "main",
            3000,
            4000,
            180,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "update-panel-projection",
                "/tmp/native-project",
                "--panel-projection",
                "11111111-1111-1111-1111-111111111111",
                "--name",
                "Updated Panel",
                "--board",
                "main",
                "--x-nm",
                "3000",
                "--y-nm",
                "4000",
                "--rotation-deg",
                "180",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "update_panel_projection")
        self.assertEqual(response.result["created"], False)

    @patch("server_runtime.subprocess.run")
    def test_deletes_panel_projection_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"delete_panel_projection","created":false}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.delete_panel_projection(
            "/tmp/native-project", "11111111-1111-1111-1111-111111111111"
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "delete-panel-projection",
                "/tmp/native-project",
                "--panel-projection",
                "11111111-1111-1111-1111-111111111111",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "delete_panel_projection")
        self.assertEqual(response.result["created"], False)

    @patch("server_runtime.subprocess.run")
    def test_queries_manufacturing_plans_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"manufacturing_plans","manufacturing_plan_count":1}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.get_manufacturing_plans("/tmp/native-project")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "query",
                "manufacturing-plans",
                "/tmp/native-project",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "manufacturing_plans")
        self.assertEqual(response.result["manufacturing_plan_count"], 1)

    @patch("server_runtime.subprocess.run")
    def test_creates_manufacturing_plan_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"create_manufacturing_plan","created":true}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.create_manufacturing_plan(
            "/tmp/native-project",
            "fab/doa2526",
            "Fabrication Plan",
            "default",
            "main-panel",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "create-manufacturing-plan",
                "/tmp/native-project",
                "--prefix",
                "fab/doa2526",
                "--name",
                "Fabrication Plan",
                "--variant",
                "default",
                "--panel-projection",
                "main-panel",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "create_manufacturing_plan")
        self.assertEqual(response.result["created"], True)

    @patch("server_runtime.subprocess.run")
    def test_updates_manufacturing_plan_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"update_manufacturing_plan","created":false}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.update_manufacturing_plan(
            "/tmp/native-project",
            "22222222-2222-2222-2222-222222222222",
            "Updated Fabrication Plan",
            "fab/doa2526-r2",
            None,
            True,
            None,
            False,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "update-manufacturing-plan",
                "/tmp/native-project",
                "--manufacturing-plan",
                "22222222-2222-2222-2222-222222222222",
                "--name",
                "Updated Fabrication Plan",
                "--prefix",
                "fab/doa2526-r2",
                "--clear-variant",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "update_manufacturing_plan")
        self.assertEqual(response.result["created"], False)

    @patch("server_runtime.subprocess.run")
    def test_deletes_manufacturing_plan_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"delete_manufacturing_plan","created":false}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.delete_manufacturing_plan(
            "/tmp/native-project", "22222222-2222-2222-2222-222222222222"
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "delete-manufacturing-plan",
                "/tmp/native-project",
                "--manufacturing-plan",
                "22222222-2222-2222-2222-222222222222",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "delete_manufacturing_plan")
        self.assertEqual(response.result["created"], False)

    @patch("server_runtime.subprocess.run")
    def test_validates_route_strategy_batch_result_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"validate_route_strategy_batch_result","structurally_valid":true,'
                '"version_compatible":true}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.validate_route_strategy_batch_result(
            "/tmp/route-strategy-batch-result.json"
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "validate-route-strategy-batch-result",
                "/tmp/route-strategy-batch-result.json",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "validate_route_strategy_batch_result")
        self.assertEqual(response.result["structurally_valid"], True)

    @patch("server_runtime.subprocess.run")
    def test_compares_route_strategy_batch_result_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"compare_route_strategy_batch_result","comparison_classification":"identical",'
                '"compatible_artifacts":true}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.compare_route_strategy_batch_result(
            "/tmp/before.route-strategy-batch.json",
            "/tmp/after.route-strategy-batch.json",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "compare-route-strategy-batch-result",
                "/tmp/before.route-strategy-batch.json",
                "/tmp/after.route-strategy-batch.json",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "compare_route_strategy_batch_result")
        self.assertEqual(response.result["compatible_artifacts"], True)

    @patch("server_runtime.subprocess.run")
    def test_gates_route_strategy_batch_result_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=2,
            stdout=(
                '{"action":"gate_route_strategy_batch_result","selected_gate_policy":"strict_identical",'
                '"passed":false}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.gate_route_strategy_batch_result(
            "/tmp/before.route-strategy-batch.json",
            "/tmp/after.route-strategy-batch.json",
            "strict_identical",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "gate-route-strategy-batch-result",
                "/tmp/before.route-strategy-batch.json",
                "/tmp/after.route-strategy-batch.json",
                "--policy",
                "strict_identical",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "gate_route_strategy_batch_result")
        self.assertEqual(response.result["passed"], False)

    @patch("server_runtime.subprocess.run")
    def test_summarizes_route_strategy_batch_results_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"summarize_route_strategy_batch_results","summary":{"total_artifacts":2}}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.summarize_route_strategy_batch_results(
            artifacts=["/tmp/run-a.json", "/tmp/run-b.json"],
            baseline="/tmp/run-a.json",
            policy="strict_identical",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "summarize-route-strategy-batch-results",
                "--artifact",
                "/tmp/run-a.json",
                "--artifact",
                "/tmp/run-b.json",
                "--baseline",
                "/tmp/run-a.json",
                "--policy",
                "strict_identical",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "summarize_route_strategy_batch_results")
        self.assertEqual(response.result["summary"]["total_artifacts"], 2)

    @patch("server_runtime.subprocess.run")
    def test_explains_route_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"route_proposal_explain","selected_candidate":"route-path-candidate",'
                '"families":[{"family":"route-path-candidate","status":"selected"}]}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.route_proposal_explain(
            "/tmp/demo",
            "11111111-1111-1111-1111-111111111111",
            "22222222-2222-2222-2222-222222222222",
            "33333333-3333-3333-3333-333333333333",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "route-proposal-explain",
                "/tmp/demo",
                "--net",
                "11111111-1111-1111-1111-111111111111",
                "--from-anchor",
                "22222222-2222-2222-2222-222222222222",
                "--to-anchor",
                "33333333-3333-3333-3333-333333333333",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "route_proposal_explain")
        self.assertEqual(response.result["families"][0]["status"], "selected")

    @patch("server_runtime.subprocess.run")
    def test_exports_selected_route_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"export_route_proposal","selected_candidate":"route-path-candidate",'
                '"artifact_kind":"native_route_proposal_artifact"}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.export_route_proposal(
            "/tmp/demo",
            "11111111-1111-1111-1111-111111111111",
            "22222222-2222-2222-2222-222222222222",
            "33333333-3333-3333-3333-333333333333",
            "/tmp/demo.route-proposal.json",
            None,
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "export-route-proposal",
                "/tmp/demo",
                "--net",
                "11111111-1111-1111-1111-111111111111",
                "--from-anchor",
                "22222222-2222-2222-2222-222222222222",
                "--to-anchor",
                "33333333-3333-3333-3333-333333333333",
                "--out",
                "/tmp/demo.route-proposal.json",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "export_route_proposal")
        self.assertEqual(response.result["artifact_kind"], "native_route_proposal_artifact")

    @patch("server_runtime.subprocess.run")
    def test_applies_selected_route_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"route_apply_selected","proposal_actions":1,'
                '"applied_actions":1}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.route_apply_selected(
            "/tmp/demo",
            "11111111-1111-1111-1111-111111111111",
            "22222222-2222-2222-2222-222222222222",
            "33333333-3333-3333-3333-333333333333",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "route-apply-selected",
                "/tmp/demo",
                "--net",
                "11111111-1111-1111-1111-111111111111",
                "--from-anchor",
                "22222222-2222-2222-2222-222222222222",
                "--to-anchor",
                "33333333-3333-3333-3333-333333333333",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "route_apply_selected")
        self.assertEqual(response.result["applied_actions"], 1)

    @patch("server_runtime.subprocess.run")
    def test_inspects_route_proposal_artifact_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"inspect_route_proposal_artifact","actions":2}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.inspect_route_proposal_artifact("/tmp/demo.route-proposal.json")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "inspect-route-proposal-artifact",
                "/tmp/demo.route-proposal.json",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "inspect_route_proposal_artifact")
        self.assertEqual(response.result["actions"], 2)

    @patch("server_runtime.subprocess.run")
    def test_reviews_live_route_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"review_route_proposal","review_source":"selected_route_proposal","actions":1}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.review_route_proposal(
            "/tmp/demo",
            "00000000-0000-0000-0000-000000000101",
            "00000000-0000-0000-0000-000000000102",
            "00000000-0000-0000-0000-000000000103",
            "default",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "review-route-proposal",
                "/tmp/demo",
                "--net",
                "00000000-0000-0000-0000-000000000101",
                "--from-anchor",
                "00000000-0000-0000-0000-000000000102",
                "--to-anchor",
                "00000000-0000-0000-0000-000000000103",
                "--profile",
                "default",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "review_route_proposal")
        self.assertEqual(response.result["review_source"], "selected_route_proposal")

    @patch("server_runtime.subprocess.run")
    def test_reviews_route_proposal_artifact_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout='{"action":"review_route_proposal","review_source":"route_proposal_artifact","actions":1}',
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.review_route_proposal(artifact="/tmp/demo.route-proposal.json")
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "review-route-proposal",
                "--artifact",
                "/tmp/demo.route-proposal.json",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "review_route_proposal")
        self.assertEqual(response.result["review_source"], "route_proposal_artifact")

    @patch("server_runtime.subprocess.run")
    def test_revalidates_route_proposal_artifact_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"revalidate_route_proposal_artifact","artifact_actions":2,'
                '"live_actions":2,"matches_live":true}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.revalidate_route_proposal_artifact(
            "/tmp/demo",
            "/tmp/demo.route-proposal.json",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "revalidate-route-proposal-artifact",
                "/tmp/demo",
                "--artifact",
                "/tmp/demo.route-proposal.json",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "revalidate_route_proposal_artifact")
        self.assertEqual(response.result["artifact_actions"], 2)
        self.assertEqual(response.result["matches_live"], True)

    @patch("server_runtime.subprocess.run")
    def test_applies_route_proposal_artifact_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"apply_route_proposal_artifact","artifact_actions":2,'
                '"applied_actions":0}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.apply_route_proposal_artifact(
            "/tmp/demo",
            "/tmp/demo.route-proposal.json",
        )
        run_mock.assert_called_once_with(
            [
                "datum-eda",
                "--format",
                "json",
                "project",
                "apply-route-proposal-artifact",
                "/tmp/demo",
                "--artifact",
                "/tmp/demo.route-proposal.json",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "apply_route_proposal_artifact")
        self.assertEqual(response.result["artifact_actions"], 2)
        self.assertEqual(response.result["applied_actions"], 0)

    def test_builds_get_design_rules_request(self) -> None:
        client = EngineDaemonClient()
        request = client.get_design_rules_request()
        self.assertEqual(request.method, "get_design_rules")
        self.assertEqual(request.params, {})
