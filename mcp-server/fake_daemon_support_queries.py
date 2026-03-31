#!/usr/bin/env python3
"""Fake daemon client read/query responses for MCP tests."""

from __future__ import annotations

from server_runtime import JsonRpcResponse


class FakeDaemonClientQueriesMixin:
    def validate_project(self, path: str) -> JsonRpcResponse:
        self.calls.append(("validate_project", path))
        return JsonRpcResponse(
            "2.0",
            1,
            {
                "action": "validate_project",
                "project_root": path,
                "valid": True,
                "schema_compatible": True,
                "required_files_expected": 4,
                "required_files_validated": 4,
                "checked_sheet_files": 0,
                "checked_definition_files": 0,
                "issue_count": 0,
                "issues": [],
            },
            None,
        )

    def get_check_report(self) -> JsonRpcResponse:
        self.calls.append(("get_check_report", None))
        return JsonRpcResponse(
            "2.0",
            2,
            {
                "domain": "board",
                "summary": {
                    "status": "warning",
                    "errors": 0,
                    "warnings": 1,
                    "infos": 1,
                    "waived": 0,
                    "by_code": [
                        {"code": "partially_routed_net", "count": 1},
                        {"code": "net_without_copper", "count": 1},
                    ],
                },
                "diagnostics": [
                    {"kind": "partially_routed_net", "severity": "warning"},
                    {"kind": "net_without_copper", "severity": "info"},
                ],
            },
            None,
        )

    def get_board_summary(self) -> JsonRpcResponse:
        self.calls.append(("get_board_summary", None))
        return JsonRpcResponse(
            "2.0",
            20,
            {
                "name": "simple-demo",
                "layer_count": 3,
                "component_count": 1,
                "net_count": 2,
            },
            None,
        )

    def get_schematic_summary(self) -> JsonRpcResponse:
        self.calls.append(("get_schematic_summary", None))
        return JsonRpcResponse(
            "2.0",
            21,
            {
                "sheet_count": 1,
                "symbol_count": 1,
                "net_label_count": 3,
                "port_count": 1,
            },
            None,
        )

    def get_sheets(self) -> JsonRpcResponse:
        self.calls.append(("get_sheets", None))
        return JsonRpcResponse(
            "2.0",
            32,
            [
                {"name": "Root", "symbols": 1, "ports": 1, "labels": 3, "buses": 1},
            ],
            None,
        )

    def get_net_info(self) -> JsonRpcResponse:
        self.calls.append(("get_net_info", None))
        return JsonRpcResponse(
            "2.0",
            22,
            [
                {"name": "GND", "tracks": 1, "vias": 1, "zones": 0},
                {"name": "VCC", "tracks": 0, "vias": 0, "zones": 0},
            ],
            None,
        )

    def get_unrouted(self) -> JsonRpcResponse:
        self.calls.append(("get_unrouted", None))
        return JsonRpcResponse(
            "2.0",
            31,
            [
                {
                    "net_name": "SIG",
                    "from": {"component": "R1", "pin": "1"},
                    "to": {"component": "R2", "pin": "1"},
                    "distance_nm": 20000000,
                }
            ],
            None,
        )

    def get_components(self) -> JsonRpcResponse:
        self.calls.append(("get_components", None))
        return JsonRpcResponse(
            "2.0",
            24,
            [
                {
                    "uuid": "comp-1",
                    "package_uuid": "00000000-0000-0000-0000-000000000000",
                    "reference": "R1",
                    "value": "10k",
                    "footprint": "Resistor_SMD:R_0603_1608Metric",
                }
            ],
            None,
        )

    def get_netlist(self) -> JsonRpcResponse:
        self.calls.append(("get_netlist", None))
        return JsonRpcResponse(
            "2.0",
            103,
            [
                {
                    "uuid": "11111111-1111-1111-1111-111111111111",
                    "name": "GND",
                    "class": "Default",
                    "pins": [{"component": "R1", "pin": "2"}],
                    "routed_pct": 1.0,
                    "labels": None,
                    "ports": None,
                    "sheets": None,
                    "semantic_class": None,
                }
            ],
            None,
        )

    def get_schematic_net_info(self) -> JsonRpcResponse:
        self.calls.append(("get_schematic_net_info", None))
        return JsonRpcResponse(
            "2.0",
            23,
            [
                {"name": "SCL", "labels": 1, "ports": 0},
                {"name": "VCC", "labels": 1, "ports": 0},
            ],
            None,
        )

    def get_labels(self) -> JsonRpcResponse:
        self.calls.append(("get_labels", None))
        return JsonRpcResponse(
            "2.0",
            25,
            [
                {"name": "SCL"},
                {"name": "VCC"},
                {"name": "SUB_IN"},
            ],
            None,
        )

    def get_symbols(self) -> JsonRpcResponse:
        self.calls.append(("get_symbols", None))
        return JsonRpcResponse(
            "2.0",
            29,
            [
                {"reference": "R1", "value": "10k"},
            ],
            None,
        )

    def get_symbol_fields(self, symbol_uuid: str) -> JsonRpcResponse:
        self.calls.append(("get_symbol_fields", symbol_uuid))
        return JsonRpcResponse(
            "2.0",
            104,
            [
                {"uuid": "f1", "symbol": symbol_uuid, "key": "Reference", "value": "R1"},
                {"uuid": "f2", "symbol": symbol_uuid, "key": "Value", "value": "10k"},
            ],
            None,
        )

    def get_ports(self) -> JsonRpcResponse:
        self.calls.append(("get_ports", None))
        return JsonRpcResponse(
            "2.0",
            26,
            [
                {"name": "SUB_IN"},
            ],
            None,
        )

    def get_buses(self) -> JsonRpcResponse:
        self.calls.append(("get_buses", None))
        return JsonRpcResponse(
            "2.0",
            27,
            [
                {"name": "DATA", "members": ["SCL", "SDA"]},
            ],
            None,
        )

    def get_bus_entries(self) -> JsonRpcResponse:
        self.calls.append(("get_bus_entries", None))
        return JsonRpcResponse(
            "2.0",
            105,
            [{"uuid": "be1", "sheet": "s1", "bus": "b1", "wire": None}],
            None,
        )

    def get_noconnects(self) -> JsonRpcResponse:
        self.calls.append(("get_noconnects", None))
        return JsonRpcResponse(
            "2.0",
            30,
            [
                {"symbol": "R1", "pin": "2"},
            ],
            None,
        )

    def get_hierarchy(self) -> JsonRpcResponse:
        self.calls.append(("get_hierarchy", None))
        return JsonRpcResponse(
            "2.0",
            28,
            {
                "instances": [{"name": "child"}],
                "links": [],
            },
            None,
        )

    def export_route_path_proposal(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        candidate: str,
        policy: str | None,
        out: str,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "export_route_path_proposal",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                    "candidate": candidate,
                    "policy": policy,
                    "out": out,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            116,
            {
                "action": "export_route_path_proposal",
                "contract": (
                    "m5_route_path_candidate_authored_copper_graph_policy_v1"
                    if candidate == "authored-copper-graph"
                    else "m5_route_path_candidate_v2"
                ),
                "path": out,
                "candidate": candidate,
                "policy": policy,
                "artifact_kind": "native_route_proposal_artifact",
            },
            None,
        )

    def route_apply(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        candidate: str,
        policy: str | None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_apply",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                    "candidate": candidate,
                    "policy": policy,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            117,
            {
                "action": "route_apply",
                "contract": (
                    "m5_route_path_candidate_authored_copper_graph_policy_v1"
                    if candidate == "authored-copper-graph"
                    else "m5_route_path_candidate_v2"
                ),
                "path": path,
                "candidate": candidate,
                "policy": policy,
                "proposal_actions": 1,
                "applied_actions": 0 if candidate == "authored-copper-graph" else 1,
            },
            None,
        )

    def route_proposal(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        profile: str | None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_proposal",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                    "profile": profile,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            118,
            {
                "action": "route_proposal",
                "path": path,
                "net_uuid": net_uuid,
                "selection_profile": profile or "default",
                "selected_candidate": "route-path-candidate",
                "selected_contract": "m5_route_path_candidate_v2",
                "selection_reason": "first_selectable_candidate",
                "evaluated_candidates": 2,
            },
            None,
        )

    def route_proposal_explain(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        profile: str | None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_proposal_explain",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                    "profile": profile,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            119,
            {
                "action": "route_proposal_explain",
                "path": path,
                "net_uuid": net_uuid,
                "selection_profile": profile or "default",
                "selected_candidate": "route-path-candidate",
                "selected_family": "route-path-candidate",
                "families": [
                    {
                        "family": "route-path-candidate",
                        "status": "selected",
                        "reason": "first_selectable_candidate",
                    },
                    {
                        "family": "authored-copper-graph",
                        "status": "rejected",
                        "reason": "policy_unavailable",
                    },
                ],
            },
            None,
        )

    def route_strategy_report(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        objective: str | None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_strategy_report",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                    "objective": objective,
                },
            )
        )
        resolved = objective or "default"
        selected_candidate = (
            "authored-copper-graph"
            if resolved == "authored-copper-priority"
            else "route-path-candidate"
        )
        selected_policy = "plain" if resolved == "authored-copper-priority" else None
        return JsonRpcResponse(
            "2.0",
            119,
            {
                "action": "route_strategy_report",
                "path": path,
                "net_uuid": net_uuid,
                "objective": resolved,
                "recommended_profile": resolved,
                "recommendation_rule": (
                    f"objective {resolved} maps directly to selector profile {resolved} "
                    "using the accepted deterministic M6 objective/profile table"
                ),
                "selector_status": "deterministic_route_proposal_selected",
                "selector_rule": "profile default selects the first successful candidate",
                "selected_candidate": selected_candidate,
                "selected_policy": selected_policy,
                "selected_contract": (
                    "m5_route_path_candidate_authored_copper_graph_policy_v1"
                    if selected_candidate == "authored-copper-graph"
                    else "m5_route_path_candidate_v2"
                ),
                "selected_actions": 1,
                "next_step_command": "project route-proposal /tmp/demo --net ...",
            },
            None,
        )

    def route_strategy_compare(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_strategy_compare",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            120,
            {
                "action": "route_strategy_compare",
                "path": path,
                "net_uuid": net_uuid,
                "comparison_rule": (
                    "compare accepted objectives/profiles in deterministic order "
                    "default > authored-copper-priority"
                ),
                "recommended_objective": "default",
                "recommended_profile": "default",
                "recommendation_reason": (
                    "recommended default because it yields a proposal while "
                    "preserving the baseline accepted selector order"
                ),
                "next_step_command": "project route-proposal /tmp/demo --net ...",
                "entries": [
                    {
                        "objective": "default",
                        "profile": "default",
                        "proposal_available": True,
                        "selector_status": "deterministic_route_proposal_selected",
                        "selected_candidate": "route-path-candidate",
                        "selected_policy": None,
                        "selected_contract": "m5_route_path_candidate_v2",
                        "selected_actions": 1,
                        "distinction": (
                            "baseline profile: preserves the accepted selector family order exactly"
                        ),
                    },
                    {
                        "objective": "authored-copper-priority",
                        "profile": "authored-copper-priority",
                        "proposal_available": True,
                        "selector_status": "deterministic_route_proposal_selected",
                        "selected_candidate": "authored-copper-graph",
                        "selected_policy": "plain",
                        "selected_contract": "m5_route_path_candidate_authored_copper_graph_policy_v1",
                        "selected_actions": 1,
                        "distinction": (
                            "reuse-priority profile: prepends the accepted authored-copper-graph policy family ahead of the unchanged default order"
                        ),
                    },
                ],
            },
            None,
        )

    def route_strategy_delta(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_strategy_delta",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            121,
            {
                "action": "route_strategy_delta",
                "path": path,
                "net_uuid": net_uuid,
                "compared_objectives": ["default", "authored-copper-priority"],
                "compared_profiles": ["default", "authored-copper-priority"],
                "outcomes_match": False,
                "outcome_relation": "different",
                "delta_classification": "different_candidate_family",
                "recommendation_summary": (
                    "recommended default because it yields a proposal while "
                    "preserving the baseline accepted selector order"
                ),
                "material_difference": (
                    "the accepted profiles currently resolve to different candidate families, "
                    "so the choice changes whether the engine prefers baseline synthesis or "
                    "authored-copper reuse first"
                ),
                "recommended_objective": "default",
                "recommended_profile": "default",
                "profiles": [
                    {
                        "objective": "default",
                        "profile": "default",
                        "proposal_available": True,
                        "selected_candidate": "route-path-candidate",
                        "selected_policy": None,
                    },
                    {
                        "objective": "authored-copper-priority",
                        "profile": "authored-copper-priority",
                        "proposal_available": True,
                        "selected_candidate": "authored-copper-graph",
                        "selected_policy": "plain",
                    },
                ],
            },
            None,
        )

    def write_route_strategy_curated_fixture_suite(
        self, out_dir: str, manifest: str | None = None
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "write_route_strategy_curated_fixture_suite",
                {
                    "out_dir": out_dir,
                    "manifest": manifest,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            121,
            {
                "action": "write_route_strategy_curated_fixture_suite",
                "suite_id": "m6_route_strategy_curated_fixture_suite_v1",
                "out_dir": out_dir,
                "requests_manifest_path": manifest
                or f"{out_dir}/route-strategy-batch-requests.json",
                "requests_manifest_kind": "native_route_strategy_batch_requests",
                "requests_manifest_version": 1,
                "total_fixtures": 4,
                "total_requests": 4,
                "fixtures": [
                    {
                        "request_id": "same-outcome-default",
                        "fixture_id": "same-outcome-default",
                        "project_root": f"{out_dir}/same-outcome-default",
                        "net_uuid": "00000000-0000-0000-0000-00000000c200",
                        "from_anchor_pad_uuid": "00000000-0000-0000-0000-00000000c205",
                        "to_anchor_pad_uuid": "00000000-0000-0000-0000-00000000c206",
                        "coverage_labels": [
                            "same_outcome",
                            "baseline_route_path_candidate",
                        ],
                    }
                ],
                "next_step_command": (
                    "project route-strategy-batch-evaluate --requests "
                    f"{manifest or f'{out_dir}/route-strategy-batch-requests.json'}"
                ),
            },
            None,
        )

    def capture_route_strategy_curated_baseline(
        self,
        out_dir: str,
        manifest: str | None = None,
        result: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "capture_route_strategy_curated_baseline",
                {
                    "out_dir": out_dir,
                    "manifest": manifest,
                    "result": result,
                },
            )
        )
        result_path = result or f"{out_dir}/route-strategy-batch-result.json"
        return JsonRpcResponse(
            "2.0",
            122,
            {
                "action": "capture_route_strategy_curated_baseline",
                "suite_id": "m6_route_strategy_curated_fixture_suite_v1",
                "out_dir": out_dir,
                "requests_manifest_path": manifest
                or f"{out_dir}/route-strategy-batch-requests.json",
                "result_artifact_path": result_path,
                "requests_manifest_kind": "native_route_strategy_batch_requests",
                "requests_manifest_version": 1,
                "result_kind": "native_route_strategy_batch_result_artifact",
                "result_version": 1,
                "total_fixtures": 4,
                "total_requests": 4,
                "summary": {
                    "total_evaluated_requests": 4,
                    "recommendation_counts_by_profile": {"default": 4},
                    "delta_classification_counts": {
                        "same_outcome": 2,
                        "different_candidate_family": 1,
                        "no_proposal_under_any_profile": 1,
                    },
                    "same_outcome_count": 3,
                    "different_outcome_count": 1,
                    "proposal_available_count": 3,
                    "no_proposal_count": 1,
                },
                "next_inspect_command": (
                    "project inspect-route-strategy-batch-result "
                    f"{result_path}"
                ),
                "next_gate_example_command": (
                    "project gate-route-strategy-batch-result "
                    f"{result_path} {result_path} --policy strict_identical"
                ),
            },
            None,
        )

    def route_strategy_batch_evaluate(self, requests: str) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_strategy_batch_evaluate",
                {
                    "requests": requests,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            122,
            {
                "action": "route_strategy_batch_evaluate",
                "kind": "native_route_strategy_batch_result_artifact",
                "version": 1,
                "requests_manifest_path": requests,
                "requests_manifest_kind": "native_route_strategy_batch_requests",
                "requests_manifest_version": 1,
                "summary": {
                    "total_evaluated_requests": 2,
                    "recommendation_counts_by_profile": {"default": 2},
                    "delta_classification_counts": {
                        "different_candidate_family": 1,
                        "same_outcome": 1,
                    },
                    "same_outcome_count": 1,
                    "different_outcome_count": 1,
                    "proposal_available_count": 2,
                    "no_proposal_count": 0,
                },
                "results": [
                    {
                        "identity": {
                            "request_id": "request-a",
                            "fixture_id": "fixture-a",
                            "project_root": "/tmp/demo-a",
                            "net_uuid": "11111111-1111-1111-1111-111111111111",
                            "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                            "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                        },
                        "recommended_profile": "default",
                        "delta_classification": "different_candidate_family",
                        "outcomes_match": False,
                    },
                    {
                        "identity": {
                            "request_id": "request-b",
                            "fixture_id": "fixture-b",
                            "project_root": "/tmp/demo-b",
                            "net_uuid": "44444444-4444-4444-4444-444444444444",
                            "from_anchor_pad_uuid": "55555555-5555-5555-5555-555555555555",
                            "to_anchor_pad_uuid": "66666666-6666-6666-6666-666666666666",
                        },
                        "recommended_profile": "default",
                        "delta_classification": "same_outcome",
                        "outcomes_match": True,
                    },
                ],
            },
            None,
        )

    def inspect_route_strategy_batch_result(self, artifact: str) -> JsonRpcResponse:
        self.calls.append(("inspect_route_strategy_batch_result", artifact))
        return JsonRpcResponse(
            "2.0",
            123,
            {
                "action": "inspect_route_strategy_batch_result",
                "artifact_path": artifact,
                "kind": "native_route_strategy_batch_result_artifact",
                "source_version": 1,
                "version": 1,
                "requests_manifest_kind": "native_route_strategy_batch_requests",
                "requests_manifest_version": 1,
                "summary": {
                    "total_evaluated_requests": 2,
                    "recommendation_counts_by_profile": {"default": 2},
                    "delta_classification_counts": {
                        "different_candidate_family": 1,
                        "same_outcome": 1,
                    },
                    "same_outcome_count": 1,
                    "different_outcome_count": 1,
                    "proposal_available_count": 2,
                    "no_proposal_count": 0,
                },
                "results": [
                    {
                        "identity": {
                            "request_id": "request-a",
                            "fixture_id": "fixture-a",
                            "project_root": "/tmp/demo-a",
                            "net_uuid": "11111111-1111-1111-1111-111111111111",
                            "from_anchor_pad_uuid": "22222222-2222-2222-2222-222222222222",
                            "to_anchor_pad_uuid": "33333333-3333-3333-3333-333333333333",
                        },
                        "recommended_profile": "default",
                        "delta_classification": "different_candidate_family",
                        "outcomes_match": False,
                    }
                ],
                "malformed_entries": [],
            },
            None,
        )

    def validate_route_strategy_batch_result(self, artifact: str) -> JsonRpcResponse:
        self.calls.append(("validate_route_strategy_batch_result", artifact))
        return JsonRpcResponse(
            "2.0",
            124,
            {
                "action": "validate_route_strategy_batch_result",
                "artifact_path": artifact,
                "kind": "native_route_strategy_batch_result_artifact",
                "source_version": 1,
                "version": 1,
                "structurally_valid": True,
                "version_compatible": True,
                "missing_required_fields": [],
                "request_result_count_matches_summary": True,
                "recommendation_counts_match_summary": True,
                "delta_classification_counts_match_summary": True,
                "outcome_counts_match_summary": True,
                "proposal_counts_match_summary": True,
                "malformed_entries": [],
            },
            None,
        )

    def compare_route_strategy_batch_result(
        self, before: str, after: str
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "compare_route_strategy_batch_result",
                {"before": before, "after": after},
            )
        )
        return JsonRpcResponse(
            "2.0",
            125,
            {
                "action": "compare_route_strategy_batch_result",
                "comparison_classification": "per_request_outcomes_changed",
                "compatibility_rule": "artifacts are compatible only when both use kind native_route_strategy_batch_result_artifact, version 1, and the same requests manifest kind/version",
                "compatible_artifacts": True,
                "before_artifact": {
                    "artifact_path": before,
                    "kind": "native_route_strategy_batch_result_artifact",
                    "version": 1,
                    "requests_manifest_kind": "native_route_strategy_batch_requests",
                    "requests_manifest_version": 1,
                },
                "after_artifact": {
                    "artifact_path": after,
                    "kind": "native_route_strategy_batch_result_artifact",
                    "version": 1,
                    "requests_manifest_kind": "native_route_strategy_batch_requests",
                    "requests_manifest_version": 1,
                },
                "total_request_count_change": {"before": 2, "after": 2, "change": 0},
                "recommendation_distribution_changes": {
                    "default": {"before": 2, "after": 1, "change": -1},
                    "authored-copper-priority": {
                        "before": 0,
                        "after": 1,
                        "change": 1,
                    },
                },
                "delta_classification_distribution_changes": {
                    "different_candidate_family": {
                        "before": 1,
                        "after": 0,
                        "change": -1,
                    },
                    "same_outcome": {"before": 1, "after": 2, "change": 1},
                },
                "same_outcome_count_change": {"before": 1, "after": 2, "change": 1},
                "different_outcome_count_change": {
                    "before": 1,
                    "after": 0,
                    "change": -1,
                },
                "proposal_available_count_change": {
                    "before": 2,
                    "after": 2,
                    "change": 0,
                },
                "no_proposal_count_change": {"before": 0, "after": 0, "change": 0},
                "added_request_ids": [],
                "removed_request_ids": [],
                "common_request_ids": ["request-a", "request-b"],
                "changed_common_requests": [
                    {
                        "request_id": "request-a",
                        "recommendation_changed": True,
                        "delta_classification_changed": True,
                        "selected_live_outcome_changed": True,
                        "before_recommended_profile": "default",
                        "after_recommended_profile": "authored-copper-priority",
                        "before_delta_classification": "different_candidate_family",
                        "after_delta_classification": "same_outcome",
                        "before_selected_candidate": "route-path-candidate",
                        "after_selected_candidate": "authored-copper-graph",
                        "before_selected_policy": None,
                        "after_selected_policy": "plain",
                    }
                ],
            },
            None,
        )

    def gate_route_strategy_batch_result(
        self, before: str, after: str, policy: str | None = None
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "gate_route_strategy_batch_result",
                {"before": before, "after": after, "policy": policy},
            )
        )
        return JsonRpcResponse(
            "2.0",
            126,
            {
                "action": "gate_route_strategy_batch_result",
                "selected_gate_policy": policy or "strict_identical",
                "passed": False,
                "comparison_classification": "per_request_outcomes_changed",
                "pass_fail_reasons": [
                    "failed because strict_identical requires comparison_classification = identical"
                ],
                "threshold_facts": {
                    "changed_recommendations": 1,
                    "changed_delta_classifications": 1,
                    "changed_per_request_outcomes": 1,
                    "added_request_ids": 0,
                    "removed_request_ids": 0,
                },
                "changed_recommendations": 1,
                "changed_delta_classifications": 1,
                "changed_per_request_outcomes": 1,
                "comparison": {
                    "action": "compare_route_strategy_batch_result",
                    "comparison_classification": "per_request_outcomes_changed",
                    "compatible_artifacts": True,
                },
            },
            None,
        )

    def summarize_route_strategy_batch_results(
        self,
        dir: str | None = None,
        artifacts: list[str] | None = None,
        baseline: str | None = None,
        policy: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "summarize_route_strategy_batch_results",
                {
                    "dir": dir,
                    "artifacts": artifacts,
                    "baseline": baseline,
                    "policy": policy,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            127,
            {
                "action": "summarize_route_strategy_batch_results",
                "ordering_basis": "filesystem_modified_time_then_path",
                "baseline_artifact": baseline,
                "selected_gate_policy": policy,
                "summary": {
                    "total_artifacts": 2,
                    "structurally_valid_artifacts": 2,
                    "structurally_invalid_artifacts": 0,
                    "gate_passed_artifacts": 1,
                    "gate_failed_artifacts": 0,
                },
                "artifacts": [
                    {
                        "artifact_path": "/tmp/run-a.json",
                        "kind": "native_route_strategy_batch_result_artifact",
                        "version": 1,
                        "requests_manifest_kind": "native_route_strategy_batch_requests",
                        "requests_manifest_version": 1,
                        "file_modified_unix_seconds": 1710000000,
                        "run_order": 1,
                        "structurally_valid": True,
                        "request_count": 2,
                        "recommendation_distribution": {"default": 2},
                        "delta_classification_distribution": {"same_outcome": 2},
                        "validation_error": None,
                        "is_baseline": True,
                        "baseline_gate": None,
                    },
                    {
                        "artifact_path": "/tmp/run-b.json",
                        "kind": "native_route_strategy_batch_result_artifact",
                        "version": 1,
                        "requests_manifest_kind": "native_route_strategy_batch_requests",
                        "requests_manifest_version": 1,
                        "file_modified_unix_seconds": 1710000100,
                        "run_order": 2,
                        "structurally_valid": True,
                        "request_count": 2,
                        "recommendation_distribution": {"default": 2},
                        "delta_classification_distribution": {"same_outcome": 2},
                        "validation_error": None,
                        "is_baseline": False,
                        "baseline_gate": {
                            "selected_gate_policy": policy or "strict_identical",
                            "passed": True,
                            "comparison_classification": "identical",
                            "pass_fail_reasons": ["passed because the saved artifacts are identical"],
                        },
                    },
                ],
            },
            None,
        )

    def export_route_proposal(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        out: str,
        profile: str | None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "export_route_proposal",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                    "profile": profile,
                    "out": out,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            120,
            {
                "action": "export_route_proposal",
                "path": out,
                "selection_profile": profile or "default",
                "selected_candidate": "route-path-candidate",
                "selected_contract": "m5_route_path_candidate_v2",
                "artifact_kind": "native_route_proposal_artifact",
            },
            None,
        )

    def route_apply_selected(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        profile: str | None,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_apply_selected",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
                    "profile": profile,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            121,
            {
                "action": "route_apply_selected",
                "path": path,
                "selection_profile": profile or "default",
                "selected_candidate": "route-path-candidate",
                "selected_contract": "m5_route_path_candidate_v2",
                "proposal_actions": 1,
                "applied_actions": 1,
            },
            None,
        )

    def inspect_route_proposal_artifact(self, artifact: str) -> JsonRpcResponse:
        self.calls.append(("inspect_route_proposal_artifact", artifact))
        return JsonRpcResponse(
            "2.0",
            114,
            {
                "action": "inspect_route_proposal_artifact",
                "artifact_kind": "native_route_proposal_artifact",
                "contract": "m5_route_path_candidate_authored_copper_graph_policy_v1",
                "path": artifact,
                "actions": 2,
            },
            None,
        )

    def revalidate_route_proposal_artifact(
        self,
        path: str,
        artifact: str,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "revalidate_route_proposal_artifact",
                {
                    "path": path,
                    "artifact": artifact,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            122,
            {
                "action": "revalidate_route_proposal_artifact",
                "project_root": path,
                "artifact_path": artifact,
                "contract": "m5_route_path_candidate_authored_copper_graph_policy_v1",
                "artifact_actions": 2,
                "live_actions": 2,
                "matches_live": True,
                "drift_kind": None,
                "drift_message": None,
                "live_rebuild_error": None,
            },
            None,
        )

    def apply_route_proposal_artifact(self, path: str, artifact: str) -> JsonRpcResponse:
        self.calls.append(
            (
                "apply_route_proposal_artifact",
                {
                    "path": path,
                    "artifact": artifact,
                },
            )
        )
        return JsonRpcResponse(
            "2.0",
            115,
            {
                "action": "apply_route_proposal_artifact",
                "path": path,
                "artifact": artifact,
                "artifact_actions": 2,
                "applied_actions": 0,
            },
            None,
        )

    def get_connectivity_diagnostics(self) -> JsonRpcResponse:
        self.calls.append(("get_connectivity_diagnostics", None))
        return JsonRpcResponse(
            "2.0",
            3,
            [
                {"kind": "partially_routed_net", "severity": "warning"},
                {"kind": "net_without_copper", "severity": "info"},
            ],
            None,
        )

    def get_design_rules(self) -> JsonRpcResponse:
        self.calls.append(("get_design_rules", None))
        return JsonRpcResponse("2.0", 102, [], None)
