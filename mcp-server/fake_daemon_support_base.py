#!/usr/bin/env python3
"""Base fake daemon client helpers for MCP server tests."""

from __future__ import annotations

from typing import Any

from server_runtime import JsonRpcResponse


class FakeDaemonClientBase:
    def __init__(self) -> None:
        self.calls: list[tuple[str, Any]] = []

    def _diff_response(
        self, request_id: int, object_type: str, uuid: str, description: str
    ) -> JsonRpcResponse:
        return JsonRpcResponse(
            "2.0",
            request_id,
            {
                "diff": {
                    "created": [],
                    "modified": [{"object_type": object_type, "uuid": uuid}],
                    "deleted": [],
                },
                "description": description,
            },
            None,
        )

    def _component_modified_response(
        self, request_id: int, uuid: str, description: str
    ) -> JsonRpcResponse:
        return self._diff_response(request_id, "component", uuid, description)

    def open_project(self, path: str) -> JsonRpcResponse:
        self.calls.append(("open_project", path))
        return JsonRpcResponse("2.0", 1, {"kind": "kicad_board", "source": path}, None)

    def close_project(self) -> JsonRpcResponse:
        self.calls.append(("close_project", None))
        return JsonRpcResponse("2.0", 100, {"closed": True}, None)

    def save(self, path: str | None) -> JsonRpcResponse:
        self.calls.append(("save", path))
        return JsonRpcResponse(
            "2.0", 109, {"path": path or "/tmp/original.kicad_pcb"}, None
        )

    def undo(self) -> JsonRpcResponse:
        self.calls.append(("undo", None))
        return JsonRpcResponse(
            "2.0",
            111,
            {
                "diff": {
                    "created": [{"object_type": "track", "uuid": "track-1"}],
                    "modified": [],
                    "deleted": [],
                },
                "description": "undo delete_track track-1",
            },
            None,
        )

    def redo(self) -> JsonRpcResponse:
        self.calls.append(("redo", None))
        return JsonRpcResponse(
            "2.0",
            112,
            {
                "diff": {
                    "created": [],
                    "modified": [],
                    "deleted": [{"object_type": "track", "uuid": "track-1"}],
                },
                "description": "redo delete_track track-1",
            },
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
