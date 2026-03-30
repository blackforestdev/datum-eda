#!/usr/bin/env python3
"""Engine daemon client request construction tests."""

from __future__ import annotations

import subprocess
import unittest
from unittest.mock import patch

from server_runtime import EngineDaemonClient


class TestDaemonClientRequests(unittest.TestCase):
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

    def test_builds_open_project_request(self) -> None:
        client = EngineDaemonClient()
        request = client.open_project_request("/tmp/demo.kicad_pcb")
        self.assertEqual(request.method, "open_project")
        self.assertEqual(request.params, {"path": "/tmp/demo.kicad_pcb"})

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
    def test_exports_authored_copper_graph_policy_proposal_via_cli(self, run_mock) -> None:
        run_mock.return_value = subprocess.CompletedProcess(
            args=[],
            returncode=0,
            stdout=(
                '{"action":"export_route_path_candidate_authored_copper_graph_proposal",'
                '"contract":"m5_route_path_candidate_authored_copper_graph_policy_v1"}'
            ),
            stderr="",
        )
        client = EngineDaemonClient()
        response = client.export_route_path_candidate_authored_copper_graph_proposal(
            "/tmp/demo",
            "11111111-1111-1111-1111-111111111111",
            "22222222-2222-2222-2222-222222222222",
            "33333333-3333-3333-3333-333333333333",
            "zone_obstacle_topology_layer_balance_aware",
            "/tmp/demo.route-proposal.json",
        )
        run_mock.assert_called_once_with(
            [
                "eda",
                "--format",
                "json",
                "project",
                "export-route-path-candidate-authored-copper-graph-proposal",
                "/tmp/demo",
                "--net",
                "11111111-1111-1111-1111-111111111111",
                "--from-anchor",
                "22222222-2222-2222-2222-222222222222",
                "--to-anchor",
                "33333333-3333-3333-3333-333333333333",
                "--policy",
                "zone_obstacle_topology_layer_balance_aware",
                "--out",
                "/tmp/demo.route-proposal.json",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(response.result["action"], "export_route_path_candidate_authored_copper_graph_proposal")
        self.assertEqual(
            response.result["contract"], "m5_route_path_candidate_authored_copper_graph_policy_v1"
        )

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
                "eda",
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
                "eda",
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
                "eda",
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
                "eda",
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
