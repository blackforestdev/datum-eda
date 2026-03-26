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
