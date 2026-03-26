#!/usr/bin/env python3
"""Shared MCP server test fixtures and fake daemon responses."""

from __future__ import annotations

from typing import Any

from server_runtime import JsonRpcResponse

class FakeDaemonClient:
    def __init__(self) -> None:
        self.calls: list[tuple[str, Any]] = []

    def open_project(self, path: str) -> JsonRpcResponse:
        self.calls.append(("open_project", path))
        return JsonRpcResponse("2.0", 1, {"kind": "kicad_board", "source": path}, None)

    def close_project(self) -> JsonRpcResponse:
        self.calls.append(("close_project", None))
        return JsonRpcResponse("2.0", 100, {"closed": True}, None)

    def save(self, path: str | None) -> JsonRpcResponse:
        self.calls.append(("save", path))
        return JsonRpcResponse("2.0", 109, {"path": path or "/tmp/original.kicad_pcb"}, None)

    def delete_track(self, uuid: str) -> JsonRpcResponse:
        self.calls.append(("delete_track", uuid))
        return JsonRpcResponse(
            "2.0",
            110,
            {"diff": {"created": [], "modified": [], "deleted": [{"object_type": "track", "uuid": uuid}]},
             "description": f"delete_track {uuid}"},
            None,
        )

    def delete_via(self, uuid: str) -> JsonRpcResponse:
        self.calls.append(("delete_via", uuid))
        return JsonRpcResponse(
            "2.0",
            113,
            {"diff": {"created": [], "modified": [], "deleted": [{"object_type": "via", "uuid": uuid}]},
             "description": f"delete_via {uuid}"},
            None,
        )

    def delete_component(self, uuid: str) -> JsonRpcResponse:
        self.calls.append(("delete_component", uuid))
        return JsonRpcResponse(
            "2.0",
            118,
            {
                "diff": {
                    "created": [],
                    "modified": [],
                    "deleted": [{"object_type": "component", "uuid": uuid}],
                },
                "description": f"delete_component {uuid}",
            },
            None,
        )

    def set_design_rule(
        self,
        rule_type: str,
        scope: dict[str, Any] | str,
        parameters: dict[str, Any],
        priority: int,
        name: str | None,
    ) -> JsonRpcResponse:
        self.calls.append(
            ("set_design_rule", rule_type, scope, parameters, priority, name)
        )
        return JsonRpcResponse(
            "2.0",
            114,
            {
                "diff": {
                    "created": [{"object_type": "rule", "uuid": "rule-1"}],
                    "modified": [],
                    "deleted": [],
                },
                "description": "set_design_rule rule-1",
            },
            None,
        )

    def set_value(self, uuid: str, value: str) -> JsonRpcResponse:
        self.calls.append(("set_value", uuid, value))
        return JsonRpcResponse(
            "2.0",
            116,
            {
                "diff": {
                    "created": [],
                    "modified": [{"object_type": "component", "uuid": uuid}],
                    "deleted": [],
                },
                "description": f"set_value {uuid}",
            },
            None,
        )

    def assign_part(self, uuid: str, part_uuid: str) -> JsonRpcResponse:
        self.calls.append(("assign_part", uuid, part_uuid))
        return JsonRpcResponse(
            "2.0",
            120,
            {
                "diff": {
                    "created": [],
                    "modified": [{"object_type": "component", "uuid": uuid}],
                    "deleted": [],
                },
                "description": f"assign_part {uuid}",
            },
            None,
        )

    def set_package(self, uuid: str, package_uuid: str) -> JsonRpcResponse:
        self.calls.append(("set_package", uuid, package_uuid))
        return JsonRpcResponse(
            "2.0",
            121,
            {
                "diff": {
                    "created": [],
                    "modified": [{"object_type": "component", "uuid": uuid}],
                    "deleted": [],
                },
                "description": f"set_package {uuid}",
            },
            None,
        )

    def set_package_with_part(
        self, uuid: str, package_uuid: str, part_uuid: str
    ) -> JsonRpcResponse:
        self.calls.append(("set_package_with_part", uuid, package_uuid, part_uuid))
        return JsonRpcResponse(
            "2.0",
            1211,
            {
                "diff": {
                    "created": [],
                    "modified": [{"object_type": "component", "uuid": uuid}],
                    "deleted": [],
                },
                "description": f"set_package_with_part {uuid}",
            },
            None,
        )

    def set_net_class(
        self,
        net_uuid: str,
        class_name: str,
        clearance: int,
        track_width: int,
        via_drill: int,
        via_diameter: int,
        diffpair_width: int = 0,
        diffpair_gap: int = 0,
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "set_net_class",
                net_uuid,
                class_name,
                clearance,
                track_width,
                via_drill,
                via_diameter,
                diffpair_width,
                diffpair_gap,
            )
        )
        return JsonRpcResponse(
            "2.0",
            121,
            {
                "diff": {
                    "created": [],
                    "modified": [{"object_type": "net", "uuid": net_uuid}],
                    "deleted": [],
                },
                "description": f"set_net_class {net_uuid}",
            },
            None,
        )

    def set_reference(self, uuid: str, reference: str) -> JsonRpcResponse:
        self.calls.append(("set_reference", uuid, reference))
        return JsonRpcResponse(
            "2.0",
            117,
            {
                "diff": {
                    "created": [],
                    "modified": [{"object_type": "component", "uuid": uuid}],
                    "deleted": [],
                },
                "description": f"set_reference {uuid}",
            },
            None,
        )

    def move_component(
        self, uuid: str, x_mm: float, y_mm: float, rotation_deg: float | None
    ) -> JsonRpcResponse:
        self.calls.append(("move_component", uuid, x_mm, y_mm, rotation_deg))
        return JsonRpcResponse(
            "2.0",
            115,
            {
                "diff": {
                    "created": [],
                    "modified": [{"object_type": "component", "uuid": uuid}],
                    "deleted": [],
                },
                "description": f"move_component {uuid}",
            },
            None,
        )

    def rotate_component(self, uuid: str, rotation_deg: float) -> JsonRpcResponse:
        self.calls.append(("rotate_component", uuid, rotation_deg))
        return JsonRpcResponse(
            "2.0",
            119,
            {
                "diff": {
                    "created": [],
                    "modified": [{"object_type": "component", "uuid": uuid}],
                    "deleted": [],
                },
                "description": f"rotate_component {uuid}",
            },
            None,
        )

    def undo(self) -> JsonRpcResponse:
        self.calls.append(("undo", None))
        return JsonRpcResponse(
            "2.0",
            111,
            {"diff": {"created": [{"object_type": "track", "uuid": "track-1"}], "modified": [], "deleted": []},
             "description": "undo delete_track track-1"},
            None,
        )

    def redo(self) -> JsonRpcResponse:
        self.calls.append(("redo", None))
        return JsonRpcResponse(
            "2.0",
            112,
            {"diff": {"created": [], "modified": [], "deleted": [{"object_type": "track", "uuid": "track-1"}]},
             "description": "redo delete_track track-1"},
            None,
        )

    def search_pool(self, query: str) -> JsonRpcResponse:
        self.calls.append(("search_pool", query))
        return JsonRpcResponse(
            "2.0",
            101,
            [
                {
                    "uuid": "00000000-0000-0000-0000-000000000000",
                    "mpn": "MMBT3904",
                    "manufacturer": "onsemi",
                    "value": "NPN",
                    "package": "SOT23",
                }
            ],
            None,
        )

    def get_part(self, uuid: str) -> JsonRpcResponse:
        self.calls.append(("get_part", uuid))
        return JsonRpcResponse(
            "2.0",
            106,
            {
                "uuid": uuid,
                "mpn": "MMBT3904",
                "manufacturer": "onsemi",
                "value": "NPN",
                "description": "NPN transistor",
                "datasheet": "",
                "entity": {"name": "Q_NPN", "prefix": "Q", "gates": []},
                "package": {"name": "SOT23", "pads": 3},
                "parametric": {"hfe": "100"},
                "lifecycle": "active",
            },
            None,
        )

    def get_package(self, uuid: str) -> JsonRpcResponse:
        self.calls.append(("get_package", uuid))
        return JsonRpcResponse(
            "2.0",
            107,
            {
                "uuid": uuid,
                "name": "SOT23",
                "pads": [{"name": "1", "x_mm": 0.0, "y_mm": 0.0, "layer": "1"}],
                "courtyard_mm": {"width": 3.0, "height": 1.5},
            },
            None,
        )

    def get_package_change_candidates(self, uuid: str) -> JsonRpcResponse:
        self.calls.append(("get_package_change_candidates", uuid))
        return JsonRpcResponse(
            "2.0",
            108,
            {
                "component_uuid": uuid,
                "current_part_uuid": "part-uuid",
                "current_package_uuid": "package-uuid",
                "current_package_name": "SOT23",
                "current_value": "LMV321",
                "status": "candidates_available",
                "ambiguous_package_count": 0,
                "candidates": [
                    {
                        "package_uuid": "alt-package-uuid",
                        "package_name": "ALT-3",
                        "compatible_part_uuid": "alt-part-uuid",
                        "compatible_part_value": "ALTAMP",
                        "pin_names": ["IN+", "IN-", "OUT"],
                    }
                ],
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

    def run_erc(self) -> JsonRpcResponse:
        self.calls.append(("run_erc", None))
        return JsonRpcResponse("2.0", 4, [{"code": "undriven_power_net"}], None)

    def run_drc(self) -> JsonRpcResponse:
        self.calls.append(("run_drc", None))
        return JsonRpcResponse(
            "2.0",
            5,
            {
                "passed": False,
                "violations": [{"code": "connectivity_unrouted_net"}],
                "summary": {"errors": 1, "warnings": 0},
            },
            None,
        )

    def explain_violation(self, domain: str, index: int) -> JsonRpcResponse:
        self.calls.append(("explain_violation", {"domain": domain, "index": index}))
        return JsonRpcResponse(
            "2.0",
            108,
            {
                "explanation": "net SIG has 1 unrouted connection(s)",
                "rule_detail": "drc connectivity_unrouted_net",
                "objects_involved": [],
                "suggestion": "Route the remaining airwires.",
            },
            None,
        )
