#!/usr/bin/env python3
"""Fake daemon client replacement-planning responses for MCP tests."""

from __future__ import annotations

from server_runtime import JsonRpcResponse


class FakeDaemonClientReplacementsMixin:
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

    def get_part_change_candidates(self, uuid: str) -> JsonRpcResponse:
        self.calls.append(("get_part_change_candidates", uuid))
        return JsonRpcResponse(
            "2.0",
            1081,
            {
                "component_uuid": uuid,
                "current_part_uuid": "part-uuid",
                "current_package_uuid": "package-uuid",
                "current_package_name": "SOT23",
                "current_value": "LMV321",
                "status": "candidates_available",
                "candidates": [
                    {
                        "part_uuid": "alt-part-uuid",
                        "package_uuid": "alt-package-uuid",
                        "package_name": "ALT-3",
                        "value": "ALTAMP",
                        "mpn": "ALTAMP-001",
                        "manufacturer": "Datum",
                        "pin_names": ["IN+", "IN-", "OUT"],
                    }
                ],
            },
            None,
        )

    def get_component_replacement_plan(self, uuid: str) -> JsonRpcResponse:
        self.calls.append(("get_component_replacement_plan", uuid))
        return JsonRpcResponse(
            "2.0",
            1082,
            {
                "component_uuid": uuid,
                "current_reference": "R1",
                "current_value": "LMV321",
                "current_part_uuid": "part-uuid",
                "current_package_uuid": "package-uuid",
                "current_package_name": "SOT23",
                "package_change": {
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
                "part_change": {
                    "component_uuid": uuid,
                    "current_part_uuid": "part-uuid",
                    "current_package_uuid": "package-uuid",
                    "current_package_name": "SOT23",
                    "current_value": "LMV321",
                    "status": "candidates_available",
                    "candidates": [
                        {
                            "part_uuid": "alt-part-uuid",
                            "package_uuid": "alt-package-uuid",
                            "package_name": "ALT-3",
                            "value": "ALTAMP",
                            "mpn": "ALTAMP-001",
                            "manufacturer": "Datum",
                            "pin_names": ["IN+", "IN-", "OUT"],
                        }
                    ],
                },
            },
            None,
        )

    def get_scoped_component_replacement_plan(
        self, scope: dict[str, str | None], policy: str
    ) -> JsonRpcResponse:
        self.calls.append(("get_scoped_component_replacement_plan", scope, policy))
        return JsonRpcResponse(
            "2.0",
            1083,
            {
                "scope": scope,
                "policy": policy,
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
            None,
        )

    def edit_scoped_component_replacement_plan(
        self,
        plan: dict[str, object],
        exclude_component_uuids: list[str],
        overrides: list[dict[str, str]],
    ) -> JsonRpcResponse:
        self.calls.append(
            (
                "edit_scoped_component_replacement_plan",
                plan,
                exclude_component_uuids,
                overrides,
            )
        )
        replacements = list(plan["replacements"])
        replacements = [
            item
            for item in replacements
            if item["component_uuid"] not in set(exclude_component_uuids)
        ]
        for override in overrides:
            for item in replacements:
                if item["component_uuid"] == override["component_uuid"]:
                    item["target_package_uuid"] = override["target_package_uuid"]
                    item["target_part_uuid"] = override["target_part_uuid"]
                    item["target_value"] = "ALTAMP"
                    item["target_package_name"] = "ALT-3"
        return JsonRpcResponse(
            "2.0",
            1084,
            {"scope": plan["scope"], "policy": plan["policy"], "replacements": replacements},
            None,
        )
