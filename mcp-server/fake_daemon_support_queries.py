#!/usr/bin/env python3
"""Fake daemon client read/query responses for MCP tests."""

from __future__ import annotations

from server_runtime import JsonRpcResponse


class FakeDaemonClientQueriesMixin:
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
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_proposal",
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
            118,
            {
                "action": "route_proposal",
                "path": path,
                "net_uuid": net_uuid,
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
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_proposal_explain",
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
            119,
            {
                "action": "route_proposal_explain",
                "path": path,
                "net_uuid": net_uuid,
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

    def export_route_proposal(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        out: str,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "export_route_proposal",
                {
                    "path": path,
                    "net_uuid": net_uuid,
                    "from_anchor_pad_uuid": from_anchor_pad_uuid,
                    "to_anchor_pad_uuid": to_anchor_pad_uuid,
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
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "route_apply_selected",
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
                "action": "route_apply_selected",
                "path": path,
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
