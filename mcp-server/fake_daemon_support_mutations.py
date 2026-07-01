#!/usr/bin/env python3
"""Fake daemon client mutation responses for MCP tests."""

from __future__ import annotations

from typing import Any

from server_runtime import JsonRpcResponse


class FakeDaemonClientMutationsMixin:
    def delete_track(self, uuid: str) -> JsonRpcResponse:
        self.calls.append(("delete_track", uuid))
        return JsonRpcResponse(
            "2.0",
            110,
            {
                "diff": {
                    "created": [],
                    "modified": [],
                    "deleted": [{"object_type": "track", "uuid": uuid}],
                },
                "description": f"delete_track {uuid}",
            },
            None,
        )

    def delete_via(self, uuid: str) -> JsonRpcResponse:
        self.calls.append(("delete_via", uuid))
        return JsonRpcResponse(
            "2.0",
            113,
            {
                "diff": {
                    "created": [],
                    "modified": [],
                    "deleted": [{"object_type": "via", "uuid": uuid}],
                },
                "description": f"delete_via {uuid}",
            },
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
        return self._component_modified_response(116, uuid, f"set_value {uuid}")

    def assign_part(self, uuid: str, part_uuid: str) -> JsonRpcResponse:
        self.calls.append(("assign_part", uuid, part_uuid))
        return self._component_modified_response(120, uuid, f"assign_part {uuid}")

    def set_package(self, uuid: str, package_uuid: str) -> JsonRpcResponse:
        self.calls.append(("set_package", uuid, package_uuid))
        return self._component_modified_response(121, uuid, f"set_package {uuid}")

    def set_package_with_part(
        self, uuid: str, package_uuid: str, part_uuid: str
    ) -> JsonRpcResponse:
        self.calls.append(("set_package_with_part", uuid, package_uuid, part_uuid))
        return self._component_modified_response(
            1211, uuid, f"set_package_with_part {uuid}"
        )

    def replace_component(
        self, uuid: str, package_uuid: str, part_uuid: str
    ) -> JsonRpcResponse:
        self.calls.append(("replace_component", uuid, package_uuid, part_uuid))
        return self._component_modified_response(
            1212, uuid, f"replace_component {uuid}"
        )

    def replace_components(self, replacements: list[dict[str, str]]) -> JsonRpcResponse:
        self.calls.append(("replace_components", replacements))
        return JsonRpcResponse(
            "2.0",
            1213,
            {
                "diff": {
                    "created": [],
                    "modified": [
                        {"object_type": "component", "uuid": item["uuid"]}
                        for item in replacements
                    ],
                    "deleted": [],
                },
                "description": f"replace_components {len(replacements)}",
            },
            None,
        )

    def apply_component_replacement_plan(
        self, replacements: list[dict[str, str | None]]
    ) -> JsonRpcResponse:
        self.calls.append(("apply_component_replacement_plan", replacements))
        return JsonRpcResponse(
            "2.0",
            1214,
            {
                "diff": {
                    "created": [],
                    "modified": [
                        {"object_type": "component", "uuid": item["uuid"]}
                        for item in replacements
                    ],
                    "deleted": [],
                },
                "description": f"replace_components {len(replacements)}",
            },
            None,
        )

    def apply_component_replacement_policy(
        self, replacements: list[dict[str, str]]
    ) -> JsonRpcResponse:
        self.calls.append(("apply_component_replacement_policy", replacements))
        return JsonRpcResponse(
            "2.0",
            1215,
            {
                "diff": {
                    "created": [],
                    "modified": [
                        {"object_type": "component", "uuid": item["uuid"]}
                        for item in replacements
                    ],
                    "deleted": [],
                },
                "description": f"replace_components {len(replacements)}",
            },
            None,
        )

    def apply_scoped_component_replacement_policy(
        self, scope: dict[str, str | None], policy: str
    ) -> JsonRpcResponse:
        self.calls.append(("apply_scoped_component_replacement_policy", scope, policy))
        return JsonRpcResponse(
            "2.0",
            1216,
            {
                "diff": {
                    "created": [],
                    "modified": [
                        {"object_type": "component", "uuid": "comp-1"},
                        {"object_type": "component", "uuid": "comp-2"},
                    ],
                    "deleted": [],
                },
                "description": "replace_components 2",
            },
            None,
        )

    def apply_scoped_component_replacement_plan(
        self, plan: dict[str, object]
    ) -> JsonRpcResponse:
        self.calls.append(("apply_scoped_component_replacement_plan", plan))
        return JsonRpcResponse(
            "2.0",
            1217,
            {
                "diff": {
                    "created": [],
                    "modified": [
                        {"object_type": "component", "uuid": "comp-1"},
                        {"object_type": "component", "uuid": "comp-2"},
                    ],
                    "deleted": [],
                },
                "description": "replace_components 2",
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
        return self._component_modified_response(117, uuid, f"set_reference {uuid}")

    def move_component(
        self, uuid: str, x_mm: float, y_mm: float, rotation_deg: float | None
    ) -> JsonRpcResponse:
        self.calls.append(("move_component", uuid, x_mm, y_mm, rotation_deg))
        return self._component_modified_response(115, uuid, f"move_component {uuid}")

    def rotate_component(self, uuid: str, rotation_deg: float) -> JsonRpcResponse:
        self.calls.append(("rotate_component", uuid, rotation_deg))
        return self._component_modified_response(119, uuid, f"rotate_component {uuid}")

    def move_board_component(
        self, path: str, component: str, x_nm: int, y_nm: int
    ) -> JsonRpcResponse:
        self.calls.append(("move_board_component", path, component, x_nm, y_nm))
        return self._component_modified_response(1151, component, f"move_board_component {component}")

    def create_sheet(self, path: str, name: str, sheet: str | None = None) -> JsonRpcResponse:
        self.calls.append(("create_sheet", path, name, sheet))
        sheet_id = sheet or "sheet-1"
        return JsonRpcResponse(
            "2.0",
            7200,
            {
                "action": "create_sheet",
                "project_root": path,
                "schematic_uuid": "schematic-test",
                "sheet_uuid": sheet_id,
                "sheet_path": f"{path}/schematic/sheets/{sheet_id}.json",
                "name": name,
                "cascaded_objects": 0,
            },
            None,
        )

    def delete_sheet(self, path: str, sheet: str) -> JsonRpcResponse:
        self.calls.append(("delete_sheet", path, sheet))
        return JsonRpcResponse(
            "2.0",
            7200,
            {
                "action": "delete_sheet",
                "project_root": path,
                "schematic_uuid": "schematic-test",
                "sheet_uuid": sheet,
                "sheet_path": f"{path}/schematic/sheets/{sheet}.json",
                "name": "Aux",
                "cascaded_objects": 2,
            },
            None,
        )

    def rename_sheet(self, path: str, sheet: str, name: str) -> JsonRpcResponse:
        self.calls.append(("rename_sheet", path, sheet, name))
        return JsonRpcResponse(
            "2.0",
            7200,
            {
                "action": "rename_sheet",
                "project_root": path,
                "schematic_uuid": "schematic-test",
                "sheet_uuid": sheet,
                "sheet_path": f"{path}/schematic/sheets/{sheet}.json",
                "name": name,
                "cascaded_objects": 0,
            },
            None,
        )

    def create_sheet_definition(
        self,
        path: str,
        root_sheet: str,
        name: str,
        definition: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(("create_sheet_definition", path, root_sheet, name, definition))
        definition_id = definition or "definition-1"
        return JsonRpcResponse(
            "2.0",
            7200,
            {
                "action": "create_sheet_definition",
                "project_root": path,
                "schematic_uuid": "schematic-test",
                "definition_uuid": definition_id,
                "definition_path": f"{path}/schematic/definitions/{definition_id}.json",
                "root_sheet_uuid": root_sheet,
                "name": name,
            },
            None,
        )

    def create_sheet_instance(
        self,
        path: str,
        definition: str,
        name: str,
        x_nm: int,
        y_nm: int,
        parent_sheet: str | None = None,
        instance: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(
            ("create_sheet_instance", path, definition, name, x_nm, y_nm, parent_sheet, instance)
        )
        instance_id = instance or "instance-1"
        return JsonRpcResponse(
            "2.0",
            7200,
            {
                "action": "create_sheet_instance",
                "project_root": path,
                "schematic_uuid": "schematic-test",
                "instance_uuid": instance_id,
                "definition_uuid": definition,
                "parent_sheet_uuid": parent_sheet,
                "name": name,
                "x_nm": x_nm,
                "y_nm": y_nm,
            },
            None,
        )

    def delete_sheet_instance(self, path: str, instance: str) -> JsonRpcResponse:
        self.calls.append(("delete_sheet_instance", path, instance))
        return JsonRpcResponse(
            "2.0",
            7200,
            {
                "action": "delete_sheet_instance",
                "project_root": path,
                "schematic_uuid": "schematic-test",
                "instance_uuid": instance,
                "definition_uuid": "definition-1",
                "parent_sheet_uuid": "sheet-1",
                "name": "Main Instance",
                "x_nm": 100,
                "y_nm": 200,
            },
            None,
        )

    def move_sheet_instance(self, path: str, instance: str, x_nm: int, y_nm: int) -> JsonRpcResponse:
        self.calls.append(("move_sheet_instance", path, instance, x_nm, y_nm))
        return JsonRpcResponse(
            "2.0",
            7200,
            {
                "action": "move_sheet_instance",
                "project_root": path,
                "schematic_uuid": "schematic-test",
                "instance_uuid": instance,
                "definition_uuid": "definition-1",
                "parent_sheet_uuid": "sheet-1",
                "name": "Main Instance",
                "x_nm": x_nm,
                "y_nm": y_nm,
            },
            None,
        )

    def bind_sheet_instance_port(self, path: str, instance: str, port: str) -> JsonRpcResponse:
        self.calls.append(("bind_sheet_instance_port", path, instance, port))
        return JsonRpcResponse(
            "2.0",
            7200,
            {
                "action": "bind_sheet_instance_port",
                "project_root": path,
                "schematic_uuid": "schematic-test",
                "instance_uuid": instance,
                "definition_uuid": "definition-1",
                "parent_sheet_uuid": "sheet-1",
                "port_uuid": port,
                "name": "Main Instance",
                "x_nm": 100,
                "y_nm": 200,
            },
            None,
        )

    def unbind_sheet_instance_port(self, path: str, instance: str, port: str) -> JsonRpcResponse:
        self.calls.append(("unbind_sheet_instance_port", path, instance, port))
        return JsonRpcResponse(
            "2.0",
            7200,
            {
                "action": "unbind_sheet_instance_port",
                "project_root": path,
                "schematic_uuid": "schematic-test",
                "instance_uuid": instance,
                "definition_uuid": "definition-1",
                "parent_sheet_uuid": "sheet-1",
                "port_uuid": port,
                "name": "Main Instance",
                "x_nm": 100,
                "y_nm": 200,
            },
            None,
        )

    def draw_wire(self, path: str, sheet: str, from_x_nm: int, from_y_nm: int, to_x_nm: int, to_y_nm: int) -> JsonRpcResponse:
        self.calls.append(("draw_wire", path, sheet, from_x_nm, from_y_nm, to_x_nm, to_y_nm))
        return self._component_modified_response(7201, "wire-1", "draw_wire wire-1")

    def delete_wire(self, path: str, wire: str) -> JsonRpcResponse:
        self.calls.append(("delete_wire", path, wire))
        return self._component_modified_response(7202, wire, f"delete_wire {wire}")

    def place_junction(self, path: str, sheet: str, x_nm: int, y_nm: int) -> JsonRpcResponse:
        self.calls.append(("place_junction", path, sheet, x_nm, y_nm))
        return self._component_modified_response(7203, "junction-1", "place_junction junction-1")

    def delete_junction(self, path: str, junction: str) -> JsonRpcResponse:
        self.calls.append(("delete_junction", path, junction))
        return self._component_modified_response(7204, junction, f"delete_junction {junction}")

    def place_noconnect(self, path: str, sheet: str, symbol: str, pin: str, x_nm: int, y_nm: int) -> JsonRpcResponse:
        self.calls.append(("place_noconnect", path, sheet, symbol, pin, x_nm, y_nm))
        return self._component_modified_response(7205, "noconnect-1", "place_noconnect noconnect-1")

    def delete_noconnect(self, path: str, noconnect: str) -> JsonRpcResponse:
        self.calls.append(("delete_noconnect", path, noconnect))
        return self._component_modified_response(7206, noconnect, f"delete_noconnect {noconnect}")

    def place_label(self, path: str, sheet: str, name: str, x_nm: int, y_nm: int, kind: str | None = None) -> JsonRpcResponse:
        self.calls.append(("place_label", path, sheet, name, x_nm, y_nm, kind))
        return self._component_modified_response(7207, "label-1", "place_label label-1")

    def rename_label(self, path: str, label: str, name: str) -> JsonRpcResponse:
        self.calls.append(("rename_label", path, label, name))
        return self._component_modified_response(7208, label, f"rename_label {label}")

    def delete_label(self, path: str, label: str) -> JsonRpcResponse:
        self.calls.append(("delete_label", path, label))
        return self._component_modified_response(7209, label, f"delete_label {label}")

    def place_port(self, path: str, sheet: str, name: str, direction: str, x_nm: int, y_nm: int) -> JsonRpcResponse:
        self.calls.append(("place_port", path, sheet, name, direction, x_nm, y_nm))
        return self._component_modified_response(7210, "port-1", "place_port port-1")

    def edit_port(self, path: str, port: str, name: str | None = None, direction: str | None = None, x_nm: int | None = None, y_nm: int | None = None) -> JsonRpcResponse:
        self.calls.append(("edit_port", path, port, name, direction, x_nm, y_nm))
        return self._component_modified_response(7211, port, f"edit_port {port}")

    def delete_port(self, path: str, port: str) -> JsonRpcResponse:
        self.calls.append(("delete_port", path, port))
        return self._component_modified_response(7212, port, f"delete_port {port}")

    def create_bus(self, path: str, sheet: str, name: str, members: list[str]) -> JsonRpcResponse:
        self.calls.append(("create_bus", path, sheet, name, members))
        return self._component_modified_response(7213, "bus-1", "create_bus bus-1")

    def edit_bus_members(self, path: str, bus: str, members: list[str]) -> JsonRpcResponse:
        self.calls.append(("edit_bus_members", path, bus, members))
        return self._component_modified_response(7214, bus, f"edit_bus_members {bus}")

    def delete_bus(self, path: str, bus: str) -> JsonRpcResponse:
        self.calls.append(("delete_bus", path, bus))
        return self._component_modified_response(7214, bus, f"delete_bus {bus}")

    def place_bus_entry(self, path: str, sheet: str, bus: str, x_nm: int, y_nm: int, wire: str | None = None) -> JsonRpcResponse:
        self.calls.append(("place_bus_entry", path, sheet, bus, x_nm, y_nm, wire))
        return self._component_modified_response(7215, "bus-entry-1", "place_bus_entry bus-entry-1")

    def delete_bus_entry(self, path: str, bus_entry: str) -> JsonRpcResponse:
        self.calls.append(("delete_bus_entry", path, bus_entry))
        return self._component_modified_response(7216, bus_entry, f"delete_bus_entry {bus_entry}")

    def place_schematic_text(self, path: str, sheet: str, text: str, x_nm: int, y_nm: int, rotation_deg: int | None = None) -> JsonRpcResponse:
        self.calls.append(("place_schematic_text", path, sheet, text, x_nm, y_nm, rotation_deg))
        return self._component_modified_response(7217, "text-1", "place_schematic_text text-1")

    def edit_schematic_text(self, path: str, text: str, value: str | None = None, x_nm: int | None = None, y_nm: int | None = None, rotation_deg: int | None = None) -> JsonRpcResponse:
        self.calls.append(("edit_schematic_text", path, text, value, x_nm, y_nm, rotation_deg))
        return self._component_modified_response(7218, text, f"edit_schematic_text {text}")

    def delete_schematic_text(self, path: str, text: str) -> JsonRpcResponse:
        self.calls.append(("delete_schematic_text", path, text))
        return self._component_modified_response(7219, text, f"delete_schematic_text {text}")

    def place_drawing_line(self, path: str, sheet: str, from_x_nm: int, from_y_nm: int, to_x_nm: int, to_y_nm: int) -> JsonRpcResponse:
        self.calls.append(("place_drawing_line", path, sheet, from_x_nm, from_y_nm, to_x_nm, to_y_nm))
        return self._component_modified_response(7220, "drawing-1", "place_drawing_line drawing-1")

    def place_drawing_rect(self, path: str, sheet: str, min_x_nm: int, min_y_nm: int, max_x_nm: int, max_y_nm: int) -> JsonRpcResponse:
        self.calls.append(("place_drawing_rect", path, sheet, min_x_nm, min_y_nm, max_x_nm, max_y_nm))
        return self._component_modified_response(7221, "drawing-2", "place_drawing_rect drawing-2")

    def place_drawing_circle(self, path: str, sheet: str, center_x_nm: int, center_y_nm: int, radius_nm: int) -> JsonRpcResponse:
        self.calls.append(("place_drawing_circle", path, sheet, center_x_nm, center_y_nm, radius_nm))
        return self._component_modified_response(7222, "drawing-3", "place_drawing_circle drawing-3")

    def place_drawing_arc(self, path: str, sheet: str, center_x_nm: int, center_y_nm: int, radius_nm: int, start_angle_mdeg: int, end_angle_mdeg: int) -> JsonRpcResponse:
        self.calls.append(("place_drawing_arc", path, sheet, center_x_nm, center_y_nm, radius_nm, start_angle_mdeg, end_angle_mdeg))
        return self._component_modified_response(7223, "drawing-4", "place_drawing_arc drawing-4")

    def edit_drawing_line(self, path: str, drawing: str, from_x_nm: int | None = None, from_y_nm: int | None = None, to_x_nm: int | None = None, to_y_nm: int | None = None) -> JsonRpcResponse:
        self.calls.append(("edit_drawing_line", path, drawing, from_x_nm, from_y_nm, to_x_nm, to_y_nm))
        return self._component_modified_response(7224, drawing, f"edit_drawing_line {drawing}")

    def edit_drawing_rect(self, path: str, drawing: str, min_x_nm: int | None = None, min_y_nm: int | None = None, max_x_nm: int | None = None, max_y_nm: int | None = None) -> JsonRpcResponse:
        self.calls.append(("edit_drawing_rect", path, drawing, min_x_nm, min_y_nm, max_x_nm, max_y_nm))
        return self._component_modified_response(7225, drawing, f"edit_drawing_rect {drawing}")

    def edit_drawing_circle(self, path: str, drawing: str, center_x_nm: int | None = None, center_y_nm: int | None = None, radius_nm: int | None = None) -> JsonRpcResponse:
        self.calls.append(("edit_drawing_circle", path, drawing, center_x_nm, center_y_nm, radius_nm))
        return self._component_modified_response(7226, drawing, f"edit_drawing_circle {drawing}")

    def edit_drawing_arc(self, path: str, drawing: str, center_x_nm: int | None = None, center_y_nm: int | None = None, radius_nm: int | None = None, start_angle_mdeg: int | None = None, end_angle_mdeg: int | None = None) -> JsonRpcResponse:
        self.calls.append(("edit_drawing_arc", path, drawing, center_x_nm, center_y_nm, radius_nm, start_angle_mdeg, end_angle_mdeg))
        return self._component_modified_response(7227, drawing, f"edit_drawing_arc {drawing}")

    def delete_drawing(self, path: str, drawing: str) -> JsonRpcResponse:
        self.calls.append(("delete_drawing", path, drawing))
        return self._component_modified_response(7228, drawing, f"delete_drawing {drawing}")

    def place_symbol(self, path: str, sheet: str, reference: str, value: str, x_nm: int, y_nm: int, lib_id: str | None = None, rotation_deg: int | None = None, mirrored: bool | None = None) -> JsonRpcResponse:
        self.calls.append(("place_symbol", path, sheet, reference, value, x_nm, y_nm, lib_id, rotation_deg, mirrored))
        return self._component_modified_response(7229, "symbol-1", "place_symbol symbol-1")

    def move_symbol(self, path: str, symbol: str, x_nm: int, y_nm: int) -> JsonRpcResponse:
        self.calls.append(("move_symbol", path, symbol, x_nm, y_nm))
        return self._component_modified_response(7230, symbol, f"move_symbol {symbol}")

    def rotate_symbol(self, path: str, symbol: str, rotation_deg: int) -> JsonRpcResponse:
        self.calls.append(("rotate_symbol", path, symbol, rotation_deg))
        return self._component_modified_response(7231, symbol, f"rotate_symbol {symbol}")

    def mirror_symbol(self, path: str, symbol: str) -> JsonRpcResponse:
        self.calls.append(("mirror_symbol", path, symbol))
        return self._component_modified_response(7232, symbol, f"mirror_symbol {symbol}")

    def delete_symbol(self, path: str, symbol: str) -> JsonRpcResponse:
        self.calls.append(("delete_symbol", path, symbol))
        return self._component_modified_response(7233, symbol, f"delete_symbol {symbol}")

    def set_symbol_reference(self, path: str, symbol: str, reference: str) -> JsonRpcResponse:
        self.calls.append(("set_symbol_reference", path, symbol, reference))
        return self._component_modified_response(7234, symbol, f"set_symbol_reference {symbol}")

    def set_symbol_value(self, path: str, symbol: str, value: str) -> JsonRpcResponse:
        self.calls.append(("set_symbol_value", path, symbol, value))
        return self._component_modified_response(7235, symbol, f"set_symbol_value {symbol}")

    def set_symbol_display_mode(self, path: str, symbol: str, mode: str) -> JsonRpcResponse:
        self.calls.append(("set_symbol_display_mode", path, symbol, mode)); return self._component_modified_response(7236, symbol, f"set_symbol_display_mode {symbol}")
    def set_symbol_hidden_power_behavior(self, path: str, symbol: str, behavior: str) -> JsonRpcResponse:
        self.calls.append(("set_symbol_hidden_power_behavior", path, symbol, behavior)); return self._component_modified_response(7237, symbol, f"set_symbol_hidden_power_behavior {symbol}")
    def set_symbol_unit(self, path: str, symbol: str, unit: str) -> JsonRpcResponse:
        self.calls.append(("set_symbol_unit", path, symbol, unit)); return self._component_modified_response(7238, symbol, f"set_symbol_unit {symbol}")
    def clear_symbol_unit(self, path: str, symbol: str) -> JsonRpcResponse:
        self.calls.append(("clear_symbol_unit", path, symbol)); return self._component_modified_response(7239, symbol, f"clear_symbol_unit {symbol}")
    def set_symbol_gate(self, path: str, symbol: str, gate: str) -> JsonRpcResponse:
        self.calls.append(("set_symbol_gate", path, symbol, gate)); return self._component_modified_response(7240, symbol, f"set_symbol_gate {symbol}")
    def clear_symbol_gate(self, path: str, symbol: str) -> JsonRpcResponse:
        self.calls.append(("clear_symbol_gate", path, symbol)); return self._component_modified_response(7241, symbol, f"clear_symbol_gate {symbol}")
    def set_symbol_entity(self, path: str, symbol: str, entity: str) -> JsonRpcResponse:
        self.calls.append(("set_symbol_entity", path, symbol, entity)); return self._component_modified_response(7242, symbol, f"set_symbol_entity {symbol}")
    def clear_symbol_entity(self, path: str, symbol: str) -> JsonRpcResponse:
        self.calls.append(("clear_symbol_entity", path, symbol)); return self._component_modified_response(7243, symbol, f"clear_symbol_entity {symbol}")
    def set_symbol_part(self, path: str, symbol: str, part: str) -> JsonRpcResponse:
        self.calls.append(("set_symbol_part", path, symbol, part)); return self._component_modified_response(7244, symbol, f"set_symbol_part {symbol}")
    def clear_symbol_part(self, path: str, symbol: str) -> JsonRpcResponse:
        self.calls.append(("clear_symbol_part", path, symbol)); return self._component_modified_response(7245, symbol, f"clear_symbol_part {symbol}")
    def set_symbol_lib_id(self, path: str, symbol: str, lib_id: str) -> JsonRpcResponse:
        self.calls.append(("set_symbol_lib_id", path, symbol, lib_id)); return self._component_modified_response(7246, symbol, f"set_symbol_lib_id {symbol}")
    def clear_symbol_lib_id(self, path: str, symbol: str) -> JsonRpcResponse:
        self.calls.append(("clear_symbol_lib_id", path, symbol)); return self._component_modified_response(7247, symbol, f"clear_symbol_lib_id {symbol}")
    def set_pin_override(self, path: str, symbol: str, pin: str, visible: bool, x_nm: int | None = None, y_nm: int | None = None) -> JsonRpcResponse:
        self.calls.append(("set_pin_override", path, symbol, pin, visible, x_nm, y_nm)); return self._component_modified_response(7248, symbol, f"set_pin_override {symbol}")
    def clear_pin_override(self, path: str, symbol: str, pin: str) -> JsonRpcResponse:
        self.calls.append(("clear_pin_override", path, symbol, pin)); return self._component_modified_response(7249, symbol, f"clear_pin_override {symbol}")
    def add_symbol_field(self, path: str, symbol: str, key: str, value: str, hidden: bool | None = None, x_nm: int | None = None, y_nm: int | None = None) -> JsonRpcResponse:
        self.calls.append(("add_symbol_field", path, symbol, key, value, hidden, x_nm, y_nm)); return self._component_modified_response(7250, symbol, f"add_symbol_field {symbol}")
    def edit_symbol_field(self, path: str, field: str, key: str | None = None, value: str | None = None, visible: bool | None = None, x_nm: int | None = None, y_nm: int | None = None) -> JsonRpcResponse:
        self.calls.append(("edit_symbol_field", path, field, key, value, visible, x_nm, y_nm)); return self._component_modified_response(7251, field, f"edit_symbol_field {field}")
    def delete_symbol_field(self, path: str, field: str) -> JsonRpcResponse:
        self.calls.append(("delete_symbol_field", path, field)); return self._component_modified_response(7252, field, f"delete_symbol_field {field}")

    def place_board_component(
        self, path: str, part: str, package: str, reference: str, value: str, x_nm: int, y_nm: int, layer: int
    ) -> JsonRpcResponse:
        self.calls.append(("place_board_component", path, part, package, reference, value, x_nm, y_nm, layer))
        return self._component_modified_response(1149, "comp-placed", "place_board_component comp-placed")

    def generate_board_components(
        self,
        path: str,
        apply: bool | None = None,
        as_proposal: bool | None = None,
        proposal: str | None = None,
        rationale: str | None = None,
        origin_x_nm: int | None = None,
        origin_y_nm: int | None = None,
        pitch_nm: int | None = None,
        layer: int | None = None,
    ) -> JsonRpcResponse:
        self.calls.append((
            "generate_board_components",
            path,
            apply,
            as_proposal,
            proposal,
            rationale,
            origin_x_nm,
            origin_y_nm,
            pitch_nm,
            layer,
        ))
        return JsonRpcResponse(
            "2.0",
            1150,
            {
                "contract": "native_project_board_handoff_v1",
                "applied": bool(apply),
                "proposed": bool(as_proposal),
                "proposal_id": proposal,
                "generated_count": 1,
                "generated_packages": [{"package_uuid": "pkg-generated"}],
            },
            None,
        )

    def rotate_board_component(self, path: str, component: str, rotation_deg: int) -> JsonRpcResponse:
        self.calls.append(("rotate_board_component", path, component, rotation_deg))
        return self._component_modified_response(1191, component, f"rotate_board_component {component}")

    def flip_component(self, uuid: str, layer: int) -> JsonRpcResponse:
        self.calls.append(("flip_component", uuid, layer))
        return self._component_modified_response(120, uuid, f"flip_component {uuid}")

    def flip_board_component(self, path: str, component: str, layer: int) -> JsonRpcResponse:
        self.calls.append(("flip_board_component", path, component, layer))
        return self._component_modified_response(1201, component, f"flip_board_component {component}")

    def delete_board_component(self, path: str, component: str) -> JsonRpcResponse:
        self.calls.append(("delete_board_component", path, component))
        return self._component_modified_response(1181, component, f"delete_board_component {component}")

    def set_board_component_reference(self, path: str, component: str, reference: str) -> JsonRpcResponse:
        self.calls.append(("set_board_component_reference", path, component, reference))
        return self._component_modified_response(1171, component, f"set_board_component_reference {component}")

    def set_board_component_value(self, path: str, component: str, value: str) -> JsonRpcResponse:
        self.calls.append(("set_board_component_value", path, component, value))
        return self._component_modified_response(1161, component, f"set_board_component_value {component}")

    def set_board_component_part(self, path: str, component: str, part: str) -> JsonRpcResponse:
        self.calls.append(("set_board_component_part", path, component, part))
        return self._component_modified_response(1202, component, f"set_board_component_part {component}")

    def set_board_component_package(self, path: str, component: str, package: str) -> JsonRpcResponse:
        self.calls.append(("set_board_component_package", path, component, package))
        return self._component_modified_response(1203, component, f"set_board_component_package {component}")

    def lock_board_component(self, path: str, component: str) -> JsonRpcResponse:
        self.calls.append(("lock_board_component", path, component))
        return self._component_modified_response(1204, component, f"lock_board_component {component}")

    def unlock_board_component(self, path: str, component: str) -> JsonRpcResponse:
        self.calls.append(("unlock_board_component", path, component))
        return self._component_modified_response(1205, component, f"unlock_board_component {component}")

    def draw_board_track(self, path: str, net: str, from_x_nm: int, from_y_nm: int, to_x_nm: int, to_y_nm: int, width_nm: int, layer: int) -> JsonRpcResponse:
        self.calls.append(("draw_board_track", path, net, from_x_nm, from_y_nm, to_x_nm, to_y_nm, width_nm, layer))
        return self._component_modified_response(1301, "track-1", "draw_board_track track-1")

    def edit_board_track(self, path: str, track: str, net: str | None = None, from_x_nm: int | None = None, from_y_nm: int | None = None, to_x_nm: int | None = None, to_y_nm: int | None = None, width_nm: int | None = None, layer: int | None = None) -> JsonRpcResponse:
        self.calls.append(("edit_board_track", path, track, net, from_x_nm, from_y_nm, to_x_nm, to_y_nm, width_nm, layer))
        return self._component_modified_response(1302, track, f"edit_board_track {track}")

    def delete_board_track(self, path: str, track: str) -> JsonRpcResponse:
        self.calls.append(("delete_board_track", path, track))
        return self._component_modified_response(1303, track, f"delete_board_track {track}")

    def place_board_via(self, path: str, net: str, x_nm: int, y_nm: int, drill_nm: int, diameter_nm: int, from_layer: int, to_layer: int) -> JsonRpcResponse:
        self.calls.append(("place_board_via", path, net, x_nm, y_nm, drill_nm, diameter_nm, from_layer, to_layer))
        return self._component_modified_response(1304, "via-1", "place_board_via via-1")

    def edit_board_via(self, path: str, via: str, net: str | None = None, x_nm: int | None = None, y_nm: int | None = None, drill_nm: int | None = None, diameter_nm: int | None = None, from_layer: int | None = None, to_layer: int | None = None) -> JsonRpcResponse:
        self.calls.append(("edit_board_via", path, via, net, x_nm, y_nm, drill_nm, diameter_nm, from_layer, to_layer))
        return self._component_modified_response(1305, via, f"edit_board_via {via}")

    def delete_board_via(self, path: str, via: str) -> JsonRpcResponse:
        self.calls.append(("delete_board_via", path, via))
        return self._component_modified_response(1306, via, f"delete_board_via {via}")

    def place_board_zone(self, path: str, net: str, vertices: list[str], layer: int, thermal_gap_nm: int, thermal_spoke_width_nm: int, priority: int | None = None, thermal_relief: bool | None = None) -> JsonRpcResponse:
        self.calls.append(("place_board_zone", path, net, vertices, layer, thermal_gap_nm, thermal_spoke_width_nm, priority, thermal_relief))
        return self._component_modified_response(1307, "zone-1", "place_board_zone zone-1")

    def edit_board_zone(self, path: str, zone: str, net: str | None = None, vertices: list[str] | None = None, layer: int | None = None, priority: int | None = None, thermal_relief: bool | None = None, thermal_gap_nm: int | None = None, thermal_spoke_width_nm: int | None = None) -> JsonRpcResponse:
        self.calls.append(("edit_board_zone", path, zone, net, vertices, layer, priority, thermal_relief, thermal_gap_nm, thermal_spoke_width_nm))
        return self._component_modified_response(1308, zone, f"edit_board_zone {zone}")

    def delete_board_zone(self, path: str, zone: str) -> JsonRpcResponse:
        self.calls.append(("delete_board_zone", path, zone))
        return self._component_modified_response(1309, zone, f"delete_board_zone {zone}")

    def place_board_pad(self, path: str, package: str, name: str, x_nm: int, y_nm: int, layer: int, shape: str | None = None, diameter_nm: int | None = None, width_nm: int | None = None, height_nm: int | None = None, net: str | None = None) -> JsonRpcResponse:
        self.calls.append(("place_board_pad", path, package, name, x_nm, y_nm, layer, shape, diameter_nm, width_nm, height_nm, net))
        return self._component_modified_response(1311, "pad-1", "place_board_pad pad-1")

    def edit_board_pad(self, path: str, pad: str, x_nm: int | None = None, y_nm: int | None = None, layer: int | None = None, shape: str | None = None, diameter_nm: int | None = None, width_nm: int | None = None, height_nm: int | None = None) -> JsonRpcResponse:
        self.calls.append(("edit_board_pad", path, pad, x_nm, y_nm, layer, shape, diameter_nm, width_nm, height_nm))
        return self._component_modified_response(1312, pad, f"edit_board_pad {pad}")

    def delete_board_pad(self, path: str, pad: str) -> JsonRpcResponse:
        self.calls.append(("delete_board_pad", path, pad))
        return self._component_modified_response(1313, pad, f"delete_board_pad {pad}")

    def set_board_pad_net(self, path: str, pad: str, net: str) -> JsonRpcResponse:
        self.calls.append(("set_board_pad_net", path, pad, net))
        return self._component_modified_response(1314, pad, f"set_board_pad_net {pad}")

    def clear_board_pad_net(self, path: str, pad: str) -> JsonRpcResponse:
        self.calls.append(("clear_board_pad_net", path, pad))
        return self._component_modified_response(1315, pad, f"clear_board_pad_net {pad}")

    def place_board_net(self, path: str, name: str, class_uuid: str, impedance_target_ohms: str | None = None, impedance_tolerance_pct: str | None = None, controlled_dielectric_layer: int | None = None) -> JsonRpcResponse:
        self.calls.append(("place_board_net", path, name, class_uuid, impedance_target_ohms, impedance_tolerance_pct, controlled_dielectric_layer))
        return self._component_modified_response(13151, "net-1", "place_board_net net-1")

    def edit_board_net(self, path: str, net: str, name: str | None = None, class_uuid: str | None = None, impedance_target_ohms: str | None = None, impedance_tolerance_pct: str | None = None, controlled_dielectric_layer: int | None = None, clear_controlled_impedance: bool | None = None) -> JsonRpcResponse:
        self.calls.append(("edit_board_net", path, net, name, class_uuid, impedance_target_ohms, impedance_tolerance_pct, controlled_dielectric_layer, clear_controlled_impedance))
        return self._component_modified_response(13152, net, f"edit_board_net {net}")

    def delete_board_net(self, path: str, net: str) -> JsonRpcResponse:
        self.calls.append(("delete_board_net", path, net))
        return self._component_modified_response(13153, net, f"delete_board_net {net}")

    def set_board_name(self, path: str, name: str) -> JsonRpcResponse:
        self.calls.append(("set_board_name", path, name))
        return self._component_modified_response(13154, "board-root", "set_board_name board-root")

    def set_board_outline(self, path: str, vertices: list[str]) -> JsonRpcResponse:
        self.calls.append(("set_board_outline", path, vertices))
        return self._component_modified_response(13155, "board-root", "set_board_outline board-root")

    def set_board_stackup(self, path: str, layers: list[str]) -> JsonRpcResponse:
        self.calls.append(("set_board_stackup", path, layers))
        return self._component_modified_response(13156, "board-root", "set_board_stackup board-root")

    def add_default_top_stackup(self, path: str) -> JsonRpcResponse:
        self.calls.append(("add_default_top_stackup", path))
        return self._component_modified_response(13157, "board-root", "add_default_top_stackup board-root")

    def place_board_keepout(self, path: str, vertices: list[str], layers: list[int], kind: str) -> JsonRpcResponse:
        self.calls.append(("place_board_keepout", path, vertices, layers, kind))
        return self._component_modified_response(13158, "keepout-1", "place_board_keepout keepout-1")

    def edit_board_keepout(self, path: str, keepout: str, vertices: list[str] | None = None, layers: list[int] | None = None, kind: str | None = None) -> JsonRpcResponse:
        self.calls.append(("edit_board_keepout", path, keepout, vertices, layers, kind))
        return self._component_modified_response(13159, keepout, f"edit_board_keepout {keepout}")

    def delete_board_keepout(self, path: str, keepout: str) -> JsonRpcResponse:
        self.calls.append(("delete_board_keepout", path, keepout))
        return self._component_modified_response(13160, keepout, f"delete_board_keepout {keepout}")

    def place_board_dimension(self, path: str, from_x_nm: int, from_y_nm: int, to_x_nm: int, to_y_nm: int, layer: int, text: str | None = None) -> JsonRpcResponse:
        self.calls.append(("place_board_dimension", path, from_x_nm, from_y_nm, to_x_nm, to_y_nm, layer, text))
        return self._component_modified_response(13161, "dimension-1", "place_board_dimension dimension-1")

    def edit_board_dimension(self, path: str, dimension: str, from_x_nm: int | None = None, from_y_nm: int | None = None, to_x_nm: int | None = None, to_y_nm: int | None = None, layer: int | None = None, text: str | None = None, clear_text: bool | None = None) -> JsonRpcResponse:
        self.calls.append(("edit_board_dimension", path, dimension, from_x_nm, from_y_nm, to_x_nm, to_y_nm, layer, text, clear_text))
        return self._component_modified_response(13162, dimension, f"edit_board_dimension {dimension}")

    def delete_board_dimension(self, path: str, dimension: str) -> JsonRpcResponse:
        self.calls.append(("delete_board_dimension", path, dimension))
        return self._component_modified_response(13163, dimension, f"delete_board_dimension {dimension}")

    def place_board_text(self, path: str, text: str, x_nm: int, y_nm: int, layer: int, rotation_deg: int | None = None, height_nm: int | None = None, stroke_width_nm: int | None = None, render_intent: str | None = None, family: str | None = None, style: str | None = None, style_class: str | None = None, h_align: str | None = None, v_align: str | None = None, mirrored: bool | None = None, keep_upright: bool | None = None, line_spacing_ratio_ppm: int | None = None, bold: bool | None = None, italic: bool | None = None) -> JsonRpcResponse:
        self.calls.append(("place_board_text", path, text, x_nm, y_nm, layer, rotation_deg, height_nm, stroke_width_nm, render_intent, family, style, style_class, h_align, v_align, mirrored, keep_upright, line_spacing_ratio_ppm, bold, italic))
        return self._component_modified_response(13164, "text-1", "place_board_text text-1")

    def edit_board_text(self, path: str, text: str, value: str | None = None, x_nm: int | None = None, y_nm: int | None = None, layer: int | None = None, rotation_deg: int | None = None, height_nm: int | None = None, stroke_width_nm: int | None = None, render_intent: str | None = None, family: str | None = None, style: str | None = None, style_class: str | None = None, h_align: str | None = None, v_align: str | None = None, mirrored: bool | None = None, keep_upright: bool | None = None, line_spacing_ratio_ppm: int | None = None, bold: bool | None = None, italic: bool | None = None) -> JsonRpcResponse:
        self.calls.append(("edit_board_text", path, text, value, x_nm, y_nm, layer, rotation_deg, height_nm, stroke_width_nm, render_intent, family, style, style_class, h_align, v_align, mirrored, keep_upright, line_spacing_ratio_ppm, bold, italic))
        return self._component_modified_response(13165, text, f"edit_board_text {text}")

    def delete_board_text(self, path: str, text: str) -> JsonRpcResponse:
        self.calls.append(("delete_board_text", path, text))
        return self._component_modified_response(13166, text, f"delete_board_text {text}")

    def place_board_net_class(self, path: str, name: str, clearance_nm: int, track_width_nm: int, via_drill_nm: int, via_diameter_nm: int, diffpair_width_nm: int | None = None, diffpair_gap_nm: int | None = None) -> JsonRpcResponse:
        self.calls.append(("place_board_net_class", path, name, clearance_nm, track_width_nm, via_drill_nm, via_diameter_nm, diffpair_width_nm, diffpair_gap_nm))
        return self._component_modified_response(1316, "net-class-1", "place_board_net_class net-class-1")

    def edit_board_net_class(self, path: str, net_class: str, name: str | None = None, clearance_nm: int | None = None, track_width_nm: int | None = None, via_drill_nm: int | None = None, via_diameter_nm: int | None = None, diffpair_width_nm: int | None = None, diffpair_gap_nm: int | None = None) -> JsonRpcResponse:
        self.calls.append(("edit_board_net_class", path, net_class, name, clearance_nm, track_width_nm, via_drill_nm, via_diameter_nm, diffpair_width_nm, diffpair_gap_nm))
        return self._component_modified_response(1317, net_class, f"edit_board_net_class {net_class}")

    def delete_board_net_class(self, path: str, net_class: str) -> JsonRpcResponse:
        self.calls.append(("delete_board_net_class", path, net_class))
        return self._component_modified_response(1318, net_class, f"delete_board_net_class {net_class}")
