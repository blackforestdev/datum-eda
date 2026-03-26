#!/usr/bin/env python3
"""
EDA MCP Server — thin translation layer between MCP clients and the engine daemon.

Communicates with eda-engine-daemon via JSON-RPC over Unix socket.
See specs/MCP_API_SPEC.md for the full tool catalog.

Current slice:
- typed JSON-RPC request/response helpers
- daemon client contract for open_project + read/check methods
- minimal stdio tool registration/dispatch layer
- no MCP SDK dependency yet
"""

from __future__ import annotations

from dataclasses import dataclass
import json
import os
import socket
import sys
import tempfile
import threading
import unittest
from typing import Any


@dataclass(frozen=True)
class JsonRpcRequest:
    jsonrpc: str
    id: int
    method: str
    params: dict[str, Any]

    def to_json(self) -> str:
        return json.dumps(
            {
                "jsonrpc": self.jsonrpc,
                "id": self.id,
                "method": self.method,
                "params": self.params,
            }
        )


@dataclass(frozen=True)
class JsonRpcError:
    code: int
    message: str


@dataclass(frozen=True)
class JsonRpcResponse:
    jsonrpc: str
    id: int
    result: Any | None
    error: JsonRpcError | None

    @staticmethod
    def from_json(payload: str) -> "JsonRpcResponse":
        decoded = json.loads(payload)
        error = decoded.get("error")
        return JsonRpcResponse(
            jsonrpc=decoded["jsonrpc"],
            id=decoded["id"],
            result=decoded.get("result"),
            error=None
            if error is None
            else JsonRpcError(code=error["code"], message=error["message"]),
        )


class EngineDaemonClient:
    """
    Minimal future daemon client contract.

    Transport is intentionally unimplemented in this slice. The important part
    is that MCP-facing methods build the same daemon RPC requests the Rust side
    already exposes.
    """

    def __init__(self, socket_path: str | None = None) -> None:
        self._next_id = 1
        self._socket_path = socket_path or os.environ.get("EDA_ENGINE_SOCKET")

    def build_request(self, method: str, params: dict[str, Any]) -> JsonRpcRequest:
        request = JsonRpcRequest(
            jsonrpc="2.0",
            id=self._next_id,
            method=method,
            params=params,
        )
        self._next_id += 1
        return request

    def open_project_request(self, path: str) -> JsonRpcRequest:
        return self.build_request("open_project", {"path": path})

    def close_project_request(self) -> JsonRpcRequest:
        return self.build_request("close_project", {})

    def save_request(self, path: str | None = None) -> JsonRpcRequest:
        return self.build_request("save", {"path": path})

    def delete_track_request(self, uuid: str) -> JsonRpcRequest:
        return self.build_request("delete_track", {"uuid": uuid})

    def delete_via_request(self, uuid: str) -> JsonRpcRequest:
        return self.build_request("delete_via", {"uuid": uuid})

    def delete_component_request(self, uuid: str) -> JsonRpcRequest:
        return self.build_request("delete_component", {"uuid": uuid})

    def set_value_request(self, uuid: str, value: str) -> JsonRpcRequest:
        return self.build_request("set_value", {"uuid": uuid, "value": value})

    def assign_part_request(self, uuid: str, part_uuid: str) -> JsonRpcRequest:
        return self.build_request("assign_part", {"uuid": uuid, "part_uuid": part_uuid})

    def set_package_request(self, uuid: str, package_uuid: str) -> JsonRpcRequest:
        return self.build_request("set_package", {"uuid": uuid, "package_uuid": package_uuid})

    def set_net_class_request(
        self,
        net_uuid: str,
        class_name: str,
        clearance: int,
        track_width: int,
        via_drill: int,
        via_diameter: int,
        diffpair_width: int = 0,
        diffpair_gap: int = 0,
    ) -> JsonRpcRequest:
        return self.build_request(
            "set_net_class",
            {
                "net_uuid": net_uuid,
                "class_name": class_name,
                "clearance": clearance,
                "track_width": track_width,
                "via_drill": via_drill,
                "via_diameter": via_diameter,
                "diffpair_width": diffpair_width,
                "diffpair_gap": diffpair_gap,
            },
        )

    def set_reference_request(self, uuid: str, reference: str) -> JsonRpcRequest:
        return self.build_request("set_reference", {"uuid": uuid, "reference": reference})

    def set_design_rule_request(
        self,
        rule_type: str,
        scope: dict[str, Any] | str,
        parameters: dict[str, Any],
        priority: int,
        name: str | None = None,
    ) -> JsonRpcRequest:
        return self.build_request(
            "set_design_rule",
            {
                "rule_type": rule_type,
                "scope": scope,
                "parameters": parameters,
                "priority": priority,
                "name": name,
            },
        )

    def move_component_request(
        self,
        uuid: str,
        x_mm: float,
        y_mm: float,
        rotation_deg: float | None = None,
    ) -> JsonRpcRequest:
        return self.build_request(
            "move_component",
            {
                "uuid": uuid,
                "x_mm": x_mm,
                "y_mm": y_mm,
                "rotation_deg": rotation_deg,
            },
        )

    def rotate_component_request(self, uuid: str, rotation_deg: float) -> JsonRpcRequest:
        return self.build_request(
            "rotate_component",
            {
                "uuid": uuid,
                "x_mm": 0.0,
                "y_mm": 0.0,
                "rotation_deg": rotation_deg,
            },
        )

    def undo_request(self) -> JsonRpcRequest:
        return self.build_request("undo", {})

    def redo_request(self) -> JsonRpcRequest:
        return self.build_request("redo", {})

    def search_pool_request(self, query: str) -> JsonRpcRequest:
        return self.build_request("search_pool", {"query": query})

    def get_part_request(self, uuid: str) -> JsonRpcRequest:
        return self.build_request("get_part", {"uuid": uuid})

    def get_package_request(self, uuid: str) -> JsonRpcRequest:
        return self.build_request("get_package", {"uuid": uuid})

    def get_board_summary_request(self) -> JsonRpcRequest:
        return self.build_request("get_board_summary", {})

    def get_components_request(self) -> JsonRpcRequest:
        return self.build_request("get_components", {})

    def get_netlist_request(self) -> JsonRpcRequest:
        return self.build_request("get_netlist", {})

    def get_schematic_summary_request(self) -> JsonRpcRequest:
        return self.build_request("get_schematic_summary", {})

    def get_sheets_request(self) -> JsonRpcRequest:
        return self.build_request("get_sheets", {})

    def get_labels_request(self) -> JsonRpcRequest:
        return self.build_request("get_labels", {})

    def get_symbols_request(self) -> JsonRpcRequest:
        return self.build_request("get_symbols", {})

    def get_symbol_fields_request(self, symbol_uuid: str) -> JsonRpcRequest:
        return self.build_request("get_symbol_fields", {"symbol_uuid": symbol_uuid})

    def get_ports_request(self) -> JsonRpcRequest:
        return self.build_request("get_ports", {})

    def get_buses_request(self) -> JsonRpcRequest:
        return self.build_request("get_buses", {})

    def get_bus_entries_request(self) -> JsonRpcRequest:
        return self.build_request("get_bus_entries", {})

    def get_noconnects_request(self) -> JsonRpcRequest:
        return self.build_request("get_noconnects", {})

    def get_hierarchy_request(self) -> JsonRpcRequest:
        return self.build_request("get_hierarchy", {})

    def get_net_info_request(self) -> JsonRpcRequest:
        return self.build_request("get_net_info", {})

    def get_unrouted_request(self) -> JsonRpcRequest:
        return self.build_request("get_unrouted", {})

    def get_schematic_net_info_request(self) -> JsonRpcRequest:
        return self.build_request("get_schematic_net_info", {})

    def get_check_report_request(self) -> JsonRpcRequest:
        return self.build_request("get_check_report", {})

    def get_connectivity_diagnostics_request(self) -> JsonRpcRequest:
        return self.build_request("get_connectivity_diagnostics", {})

    def get_design_rules_request(self) -> JsonRpcRequest:
        return self.build_request("get_design_rules", {})

    def run_erc_request(self) -> JsonRpcRequest:
        return self.build_request("run_erc", {})

    def run_drc_request(self) -> JsonRpcRequest:
        return self.build_request("run_drc", {})

    def explain_violation_request(self, domain: str, index: int) -> JsonRpcRequest:
        return self.build_request("explain_violation", {"domain": domain, "index": index})

    def call(self, request: JsonRpcRequest) -> JsonRpcResponse:
        if not self._socket_path:
            raise RuntimeError("EDA_ENGINE_SOCKET is not configured")

        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
            client.connect(self._socket_path)
            client.sendall(request.to_json().encode("utf-8") + b"\n")
            data = b""
            while not data.endswith(b"\n"):
                chunk = client.recv(4096)
                if not chunk:
                    break
                data += chunk

        if not data:
            raise RuntimeError("no response from engine daemon")
        return JsonRpcResponse.from_json(data.decode("utf-8").strip())

    def get_check_report(self) -> JsonRpcResponse:
        return self.call(self.get_check_report_request())

    def open_project(self, path: str) -> JsonRpcResponse:
        return self.call(self.open_project_request(path))

    def close_project(self) -> JsonRpcResponse:
        return self.call(self.close_project_request())

    def save(self, path: str | None = None) -> JsonRpcResponse:
        return self.call(self.save_request(path))

    def delete_track(self, uuid: str) -> JsonRpcResponse:
        return self.call(self.delete_track_request(uuid))

    def delete_via(self, uuid: str) -> JsonRpcResponse:
        return self.call(self.delete_via_request(uuid))

    def delete_component(self, uuid: str) -> JsonRpcResponse:
        return self.call(self.delete_component_request(uuid))

    def set_value(self, uuid: str, value: str) -> JsonRpcResponse:
        return self.call(self.set_value_request(uuid, value))

    def assign_part(self, uuid: str, part_uuid: str) -> JsonRpcResponse:
        return self.call(self.assign_part_request(uuid, part_uuid))

    def set_package(self, uuid: str, package_uuid: str) -> JsonRpcResponse:
        return self.call(self.set_package_request(uuid, package_uuid))

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
        return self.call(
            self.set_net_class_request(
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

    def set_reference(self, uuid: str, reference: str) -> JsonRpcResponse:
        return self.call(self.set_reference_request(uuid, reference))

    def set_design_rule(
        self,
        rule_type: str,
        scope: dict[str, Any] | str,
        parameters: dict[str, Any],
        priority: int,
        name: str | None = None,
    ) -> JsonRpcResponse:
        return self.call(
            self.set_design_rule_request(rule_type, scope, parameters, priority, name)
        )

    def move_component(
        self,
        uuid: str,
        x_mm: float,
        y_mm: float,
        rotation_deg: float | None = None,
    ) -> JsonRpcResponse:
        return self.call(self.move_component_request(uuid, x_mm, y_mm, rotation_deg))

    def rotate_component(self, uuid: str, rotation_deg: float) -> JsonRpcResponse:
        return self.call(self.rotate_component_request(uuid, rotation_deg))

    def undo(self) -> JsonRpcResponse:
        return self.call(self.undo_request())

    def redo(self) -> JsonRpcResponse:
        return self.call(self.redo_request())

    def search_pool(self, query: str) -> JsonRpcResponse:
        return self.call(self.search_pool_request(query))

    def get_part(self, uuid: str) -> JsonRpcResponse:
        return self.call(self.get_part_request(uuid))

    def get_package(self, uuid: str) -> JsonRpcResponse:
        return self.call(self.get_package_request(uuid))

    def get_board_summary(self) -> JsonRpcResponse:
        return self.call(self.get_board_summary_request())

    def get_schematic_summary(self) -> JsonRpcResponse:
        return self.call(self.get_schematic_summary_request())

    def get_sheets(self) -> JsonRpcResponse:
        return self.call(self.get_sheets_request())

    def get_components(self) -> JsonRpcResponse:
        return self.call(self.get_components_request())

    def get_netlist(self) -> JsonRpcResponse:
        return self.call(self.get_netlist_request())

    def get_labels(self) -> JsonRpcResponse:
        return self.call(self.get_labels_request())

    def get_symbols(self) -> JsonRpcResponse:
        return self.call(self.get_symbols_request())

    def get_symbol_fields(self, symbol_uuid: str) -> JsonRpcResponse:
        return self.call(self.get_symbol_fields_request(symbol_uuid))

    def get_ports(self) -> JsonRpcResponse:
        return self.call(self.get_ports_request())

    def get_buses(self) -> JsonRpcResponse:
        return self.call(self.get_buses_request())

    def get_bus_entries(self) -> JsonRpcResponse:
        return self.call(self.get_bus_entries_request())

    def get_noconnects(self) -> JsonRpcResponse:
        return self.call(self.get_noconnects_request())

    def get_hierarchy(self) -> JsonRpcResponse:
        return self.call(self.get_hierarchy_request())

    def get_net_info(self) -> JsonRpcResponse:
        return self.call(self.get_net_info_request())

    def get_unrouted(self) -> JsonRpcResponse:
        return self.call(self.get_unrouted_request())

    def get_schematic_net_info(self) -> JsonRpcResponse:
        return self.call(self.get_schematic_net_info_request())

    def get_connectivity_diagnostics(self) -> JsonRpcResponse:
        return self.call(self.get_connectivity_diagnostics_request())

    def get_design_rules(self) -> JsonRpcResponse:
        return self.call(self.get_design_rules_request())

    def run_erc(self) -> JsonRpcResponse:
        return self.call(self.run_erc_request())

    def run_drc(self) -> JsonRpcResponse:
        return self.call(self.run_drc_request())

    def explain_violation(self, domain: str, index: int) -> JsonRpcResponse:
        return self.call(self.explain_violation_request(domain, index))


TOOLS: list[dict[str, Any]] = [
    {
        "name": "open_project",
        "description": "Import a KiCad or Eagle design into the engine session.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
        },
    },
    {
        "name": "close_project",
        "description": "Close the current in-memory project session.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "save",
        "description": "Save the current imported design to a path or back to its original file.",
        "inputSchema": {
            "type": "object",
            "properties": {"path": {"type": ["string", "null"]}},
        },
    },
    {
        "name": "delete_track",
        "description": "Delete one board track by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {"uuid": {"type": "string"}},
            "required": ["uuid"],
        },
    },
    {
        "name": "delete_component",
        "description": "Delete one board component by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {"uuid": {"type": "string"}},
            "required": ["uuid"],
        },
    },
    {
        "name": "move_component",
        "description": "Move one board component by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "uuid": {"type": "string"},
                "x_mm": {"type": "number"},
                "y_mm": {"type": "number"},
                "rotation_deg": {"type": ["number", "null"]},
            },
            "required": ["uuid", "x_mm", "y_mm"],
        },
    },
    {
        "name": "rotate_component",
        "description": "Rotate one board component by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "uuid": {"type": "string"},
                "rotation_deg": {"type": "number"},
            },
            "required": ["uuid", "rotation_deg"],
        },
    },
    {
        "name": "set_value",
        "description": "Set one board component value by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "uuid": {"type": "string"},
                "value": {"type": "string"},
            },
            "required": ["uuid", "value"],
        },
    },
    {
        "name": "assign_part",
        "description": "Assign one pool part to a board component by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "uuid": {"type": "string"},
                "part_uuid": {"type": "string"},
            },
            "required": ["uuid", "part_uuid"],
        },
    },
    {
        "name": "set_package",
        "description": "Assign one pool package to a board component by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "uuid": {"type": "string"},
                "package_uuid": {"type": "string"},
            },
            "required": ["uuid", "package_uuid"],
        },
    },
    {
        "name": "set_reference",
        "description": "Set one board component reference by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "uuid": {"type": "string"},
                "reference": {"type": "string"},
            },
            "required": ["uuid", "reference"],
        },
    },
    {
        "name": "set_net_class",
        "description": "Assign one board net to a concrete net class in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "net_uuid": {"type": "string"},
                "class_name": {"type": "string"},
                "clearance": {"type": "integer"},
                "track_width": {"type": "integer"},
                "via_drill": {"type": "integer"},
                "via_diameter": {"type": "integer"},
                "diffpair_width": {"type": "integer"},
                "diffpair_gap": {"type": "integer"},
            },
            "required": [
                "net_uuid",
                "class_name",
                "clearance",
                "track_width",
                "via_drill",
                "via_diameter",
            ],
        },
    },
    {
        "name": "delete_via",
        "description": "Delete one board via by UUID in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {"uuid": {"type": "string"}},
            "required": ["uuid"],
        },
    },
    {
        "name": "set_design_rule",
        "description": "Create or update one board design rule in the current M3 slice.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "rule_type": {"type": "string"},
                "scope": {"type": ["object", "string"]},
                "parameters": {"type": "object"},
                "priority": {"type": "integer"},
                "name": {"type": ["string", "null"]},
            },
            "required": ["rule_type", "scope", "parameters", "priority"],
        },
    },
    {
        "name": "undo",
        "description": "Undo the last board transaction in the current session.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "redo",
        "description": "Redo the last undone board transaction in the current session.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "search_pool",
        "description": "Search imported pool parts by keyword.",
        "inputSchema": {
            "type": "object",
            "properties": {"query": {"type": "string"}},
            "required": ["query"],
        },
    },
    {
        "name": "get_part",
        "description": "Return detailed pool part metadata for a UUID.",
        "inputSchema": {
            "type": "object",
            "properties": {"uuid": {"type": "string"}},
            "required": ["uuid"],
        },
    },
    {
        "name": "get_package",
        "description": "Return detailed package geometry/footprint metadata for a UUID.",
        "inputSchema": {
            "type": "object",
            "properties": {"uuid": {"type": "string"}},
            "required": ["uuid"],
        },
    },
    {
        "name": "get_components",
        "description": "Return the imported board component list for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_netlist",
        "description": "Return canonical netlist view for the open board or schematic project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_check_report",
        "description": "Return the unified board/schematic checking report.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_net_info",
        "description": "Return the current imported board net list and routing metrics.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_unrouted",
        "description": "Return unrouted airwires for the current imported board.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_schematic_net_info",
        "description": "Return the current imported schematic net list and connectivity counts.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_board_summary",
        "description": "Return the imported board summary for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_schematic_summary",
        "description": "Return the imported schematic summary for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_sheets",
        "description": "Return imported schematic sheets for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_labels",
        "description": "Return the imported schematic labels for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_symbols",
        "description": "Return the imported schematic symbols for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_symbol_fields",
        "description": "Return authored fields for a specific schematic symbol UUID.",
        "inputSchema": {
            "type": "object",
            "properties": {"symbol_uuid": {"type": "string"}},
            "required": ["symbol_uuid"],
        },
    },
    {
        "name": "get_ports",
        "description": "Return the imported schematic interface ports for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_buses",
        "description": "Return the imported schematic buses for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_bus_entries",
        "description": "Return the imported schematic bus-entry objects for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_hierarchy",
        "description": "Return the imported schematic hierarchy for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_noconnects",
        "description": "Return the imported schematic no-connect markers for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_connectivity_diagnostics",
        "description": "Return raw connectivity diagnostics for the open project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "get_design_rules",
        "description": "Return authored design-rule entries for the open board project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "run_erc",
        "description": "Return raw ERC findings for the open schematic project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "run_drc",
        "description": "Return DRC report for the open board project.",
        "inputSchema": {"type": "object", "properties": {}},
    },
    {
        "name": "explain_violation",
        "description": "Explain a specific ERC/DRC finding by domain and index.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "domain": {"type": "string", "enum": ["erc", "drc"]},
                "index": {"type": "integer"},
            },
            "required": ["domain", "index"],
        },
    },
]


class StdioToolHost:
    def __init__(self, daemon: EngineDaemonClient) -> None:
        self._daemon = daemon

    def handle_message(self, message: dict[str, Any]) -> dict[str, Any] | None:
        method = message.get("method")
        msg_id = message.get("id")
        params = message.get("params", {})

        # Standard MCP initialization and liveness methods.
        if method == "initialize":
            return {
                "jsonrpc": "2.0",
                "id": msg_id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": "datum-eda", "version": "0.1.0"},
                },
            }

        # Notification: no response.
        if method == "notifications/initialized":
            return None

        if method == "ping":
            return {"jsonrpc": "2.0", "id": msg_id, "result": {}}

        if method == "tools/list":
            return {"jsonrpc": "2.0", "id": msg_id, "result": {"tools": TOOLS}}

        if method == "tools/call":
            name = params.get("name")
            arguments = params.get("arguments", {})
            try:
                result = self._call_tool(name, arguments)
            except Exception as exc:
                return {
                    "jsonrpc": "2.0",
                    "id": msg_id,
                    "error": {"code": -32010, "message": str(exc)},
                }
            return {"jsonrpc": "2.0", "id": msg_id, "result": result}

        # For unknown notifications, remain silent. For unknown requests, return error.
        if msg_id is None:
            return None

        return {
            "jsonrpc": "2.0",
            "id": msg_id,
            "error": {"code": -32601, "message": "method not found"},
        }

    def _call_tool(self, name: str, arguments: dict[str, Any]) -> dict[str, Any]:
        if name == "open_project":
            response = self._daemon.open_project(arguments["path"])
        elif name == "close_project":
            response = self._daemon.close_project()
        elif name == "save":
            response = self._daemon.save(arguments.get("path"))
        elif name == "delete_track":
            response = self._daemon.delete_track(arguments["uuid"])
        elif name == "delete_component":
            response = self._daemon.delete_component(arguments["uuid"])
        elif name == "delete_via":
            response = self._daemon.delete_via(arguments["uuid"])
        elif name == "move_component":
            response = self._daemon.move_component(
                arguments["uuid"],
                arguments["x_mm"],
                arguments["y_mm"],
                arguments.get("rotation_deg"),
            )
        elif name == "rotate_component":
            response = self._daemon.rotate_component(
                arguments["uuid"], arguments["rotation_deg"]
            )
        elif name == "set_value":
            response = self._daemon.set_value(arguments["uuid"], arguments["value"])
        elif name == "assign_part":
            response = self._daemon.assign_part(arguments["uuid"], arguments["part_uuid"])
        elif name == "set_package":
            response = self._daemon.set_package(arguments["uuid"], arguments["package_uuid"])
        elif name == "set_reference":
            response = self._daemon.set_reference(
                arguments["uuid"], arguments["reference"]
            )
        elif name == "set_net_class":
            response = self._daemon.set_net_class(
                arguments["net_uuid"],
                arguments["class_name"],
                arguments["clearance"],
                arguments["track_width"],
                arguments["via_drill"],
                arguments["via_diameter"],
                arguments.get("diffpair_width", 0),
                arguments.get("diffpair_gap", 0),
            )
        elif name == "set_design_rule":
            response = self._daemon.set_design_rule(
                arguments["rule_type"],
                arguments["scope"],
                arguments["parameters"],
                arguments["priority"],
                arguments.get("name"),
            )
        elif name == "undo":
            response = self._daemon.undo()
        elif name == "redo":
            response = self._daemon.redo()
        elif name == "search_pool":
            response = self._daemon.search_pool(arguments["query"])
        elif name == "get_part":
            response = self._daemon.get_part(arguments["uuid"])
        elif name == "get_package":
            response = self._daemon.get_package(arguments["uuid"])
        elif name == "get_components":
            response = self._daemon.get_components()
        elif name == "get_netlist":
            response = self._daemon.get_netlist()
        elif name == "get_board_summary":
            response = self._daemon.get_board_summary()
        elif name == "get_schematic_summary":
            response = self._daemon.get_schematic_summary()
        elif name == "get_sheets":
            response = self._daemon.get_sheets()
        elif name == "get_labels":
            response = self._daemon.get_labels()
        elif name == "get_symbols":
            response = self._daemon.get_symbols()
        elif name == "get_symbol_fields":
            response = self._daemon.get_symbol_fields(arguments["symbol_uuid"])
        elif name == "get_ports":
            response = self._daemon.get_ports()
        elif name == "get_buses":
            response = self._daemon.get_buses()
        elif name == "get_bus_entries":
            response = self._daemon.get_bus_entries()
        elif name == "get_noconnects":
            response = self._daemon.get_noconnects()
        elif name == "get_hierarchy":
            response = self._daemon.get_hierarchy()
        elif name == "get_net_info":
            response = self._daemon.get_net_info()
        elif name == "get_unrouted":
            response = self._daemon.get_unrouted()
        elif name == "get_schematic_net_info":
            response = self._daemon.get_schematic_net_info()
        elif name == "get_check_report":
            response = self._daemon.get_check_report()
        elif name == "get_connectivity_diagnostics":
            response = self._daemon.get_connectivity_diagnostics()
        elif name == "get_design_rules":
            response = self._daemon.get_design_rules()
        elif name == "run_erc":
            response = self._daemon.run_erc()
        elif name == "run_drc":
            response = self._daemon.run_drc()
        elif name == "explain_violation":
            response = self._daemon.explain_violation(arguments["domain"], arguments["index"])
        else:
            raise RuntimeError(f"unknown tool: {name}")

        if response.error is not None:
            raise RuntimeError(response.error.message)

        return {
            "content": [
                {
                    "type": "json",
                    "json": response.result,
                }
            ]
        }

    def run_stdio(self) -> None:
        for line in sys.stdin:
            line = line.strip()
            if not line:
                continue
            message = json.loads(line)
            response = self.handle_message(message)
            if response is not None:
                print(json.dumps(response), flush=True)


class ServerTests(unittest.TestCase):
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
        self.assertEqual(part.method, "get_part")
        self.assertEqual(part.params, {"uuid": "11111111-1111-1111-1111-111111111111"})
        self.assertEqual(package.method, "get_package")
        self.assertEqual(package.params, {"uuid": "22222222-2222-2222-2222-222222222222"})

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
        symbol_fields = client.get_symbol_fields_request("11111111-1111-1111-1111-111111111111")
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

    def test_builds_get_design_rules_request(self) -> None:
        client = EngineDaemonClient()
        request = client.get_design_rules_request()
        self.assertEqual(request.method, "get_design_rules")
        self.assertEqual(request.params, {})

    def test_response_decodes_success_payload(self) -> None:
        response = JsonRpcResponse.from_json(
            json.dumps(
                {
                    "jsonrpc": "2.0",
                    "id": 7,
                    "result": {
                        "domain": "board",
                        "summary": {"status": "warning"},
                        "diagnostics": [],
                    },
                    "error": None,
                }
            )
        )
        self.assertEqual(response.id, 7)
        self.assertIsNone(response.error)
        assert isinstance(response.result, dict)
        self.assertEqual(response.result["domain"], "board")

    def test_response_decodes_error_payload(self) -> None:
        response = JsonRpcResponse.from_json(
            json.dumps(
                {
                    "jsonrpc": "2.0",
                    "id": 9,
                    "result": None,
                    "error": {"code": -32001, "message": "no project open"},
                }
            )
        )
        self.assertIsNone(response.result)
        self.assertIsNotNone(response.error)
        assert response.error is not None
        self.assertEqual(response.error.code, -32001)
        self.assertEqual(response.error.message, "no project open")

    def test_call_requires_socket_configuration(self) -> None:
        client = EngineDaemonClient(socket_path=None)
        with self.assertRaisesRegex(RuntimeError, "EDA_ENGINE_SOCKET is not configured"):
            client.get_check_report()

    def test_get_check_report_round_trips_over_unix_socket(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            socket_path = os.path.join(tmp, "eda.sock")
            probe = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            try:
                probe.bind(socket_path)
            except PermissionError as exc:
                self.skipTest(f"unix socket bind not permitted in this environment: {exc}")
            finally:
                probe.close()
                if os.path.exists(socket_path):
                    os.unlink(socket_path)
            ready = threading.Event()

            def serve_once() -> None:
                with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as server:
                    server.bind(socket_path)
                    server.listen(1)
                    ready.set()
                    conn, _ = server.accept()
                    with conn:
                        data = b""
                        while not data.endswith(b"\n"):
                            chunk = conn.recv(4096)
                            if not chunk:
                                break
                            data += chunk
                        request = json.loads(data.decode("utf-8").strip())
                        self.assertEqual(request["method"], "get_check_report")
                        response = json.dumps(
                            {
                                "jsonrpc": "2.0",
                                "id": request["id"],
                                "result": {
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
                                "error": None,
                            }
                        )
                        conn.sendall(response.encode("utf-8") + b"\n")

            thread = threading.Thread(target=serve_once)
            thread.start()
            ready.wait(timeout=2)

            client = EngineDaemonClient(socket_path=socket_path)
            response = client.get_check_report()
            self.assertIsNone(response.error)
            assert isinstance(response.result, dict)
            self.assertEqual(response.result["domain"], "board")
            self.assertEqual(response.result["summary"]["status"], "warning")
            self.assertEqual(
                response.result["summary"]["by_code"][0]["code"], "partially_routed_net"
            )
            thread.join(timeout=2)
            self.assertFalse(thread.is_alive())

    def test_tools_list_returns_registered_tools(self) -> None:
        host = StdioToolHost(self.FakeDaemonClient())
        response = host.handle_message({"jsonrpc": "2.0", "id": 1, "method": "tools/list"})
        self.assertIn("result", response)
        tools = response["result"]["tools"]
        self.assertEqual([tool["name"] for tool in tools], [
            "open_project",
            "close_project",
            "save",
            "delete_track",
            "delete_component",
            "move_component",
            "rotate_component",
            "set_value",
            "assign_part",
            "set_package",
            "set_reference",
            "set_net_class",
            "delete_via",
            "set_design_rule",
            "undo",
            "redo",
            "search_pool",
            "get_part",
            "get_package",
            "get_components",
            "get_netlist",
            "get_check_report",
            "get_net_info",
            "get_unrouted",
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
            "get_hierarchy",
            "get_noconnects",
            "get_connectivity_diagnostics",
            "get_design_rules",
            "run_erc",
            "run_drc",
            "explain_violation",
        ])

    def test_initialize_returns_server_info_and_capabilities(self) -> None:
        host = StdioToolHost(self.FakeDaemonClient())
        response = host.handle_message({"jsonrpc": "2.0", "id": 1, "method": "initialize"})
        assert isinstance(response, dict)
        self.assertEqual(response["jsonrpc"], "2.0")
        self.assertEqual(response["id"], 1)
        self.assertEqual(response["result"]["protocolVersion"], "2024-11-05")
        self.assertEqual(response["result"]["serverInfo"]["name"], "datum-eda")
        self.assertIn("tools", response["result"]["capabilities"])

    def test_ping_returns_empty_result(self) -> None:
        host = StdioToolHost(self.FakeDaemonClient())
        response = host.handle_message({"jsonrpc": "2.0", "id": 7, "method": "ping"})
        assert isinstance(response, dict)
        self.assertEqual(response, {"jsonrpc": "2.0", "id": 7, "result": {}})

    def test_initialized_notification_returns_no_response(self) -> None:
        host = StdioToolHost(self.FakeDaemonClient())
        response = host.handle_message(
            {"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}}
        )
        self.assertIsNone(response)

    def test_tools_call_dispatches_open_project(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {
                    "name": "open_project",
                    "arguments": {"path": "/tmp/demo.kicad_pcb"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("open_project", "/tmp/demo.kicad_pcb")])
        self.assertEqual(response["result"]["content"][0]["json"]["kind"], "kicad_board")

    def test_tools_call_dispatches_close_project(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 200,
                "method": "tools/call",
                "params": {
                    "name": "close_project",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("close_project", None)])
        self.assertEqual(response["result"]["content"][0]["json"]["closed"], True)

    def test_tools_call_dispatches_save(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 204,
                "method": "tools/call",
                "params": {
                    "name": "save",
                    "arguments": {"path": "/tmp/out.kicad_pcb"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("save", "/tmp/out.kicad_pcb")])
        self.assertEqual(response["result"]["content"][0]["json"]["path"], "/tmp/out.kicad_pcb")

    def test_tools_call_dispatches_delete_track(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 205,
                "method": "tools/call",
                "params": {
                    "name": "delete_track",
                    "arguments": {"uuid": "track-1"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("delete_track", "track-1")])
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["deleted"][0]["uuid"],
            "track-1",
        )

    def test_tools_call_dispatches_delete_component(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 206,
                "method": "tools/call",
                "params": {
                    "name": "delete_component",
                    "arguments": {"uuid": "comp-1"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("delete_component", "comp-1")])
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["deleted"][0]["uuid"],
            "comp-1",
        )

    def test_tools_call_dispatches_delete_via(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 208,
                "method": "tools/call",
                "params": {
                    "name": "delete_via",
                    "arguments": {"uuid": "via-1"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("delete_via", "via-1")])
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["deleted"][0]["uuid"],
            "via-1",
        )

    def test_tools_call_dispatches_set_design_rule(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 209,
                "method": "tools/call",
                "params": {
                    "name": "set_design_rule",
                    "arguments": {
                        "rule_type": "ClearanceCopper",
                        "scope": "All",
                        "parameters": {"Clearance": {"min": 125000}},
                        "priority": 10,
                        "name": "default clearance",
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "set_design_rule",
                    "ClearanceCopper",
                    "All",
                    {"Clearance": {"min": 125000}},
                    10,
                    "default clearance",
                )
            ],
        )
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["created"][0]["object_type"],
            "rule",
        )

    def test_tools_call_dispatches_set_value(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2091,
                "method": "tools/call",
                "params": {
                    "name": "set_value",
                    "arguments": {"uuid": "comp-1", "value": "22k"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("set_value", "comp-1", "22k")])
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "component",
        )

    def test_tools_call_dispatches_assign_part(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 20915,
                "method": "tools/call",
                "params": {
                    "name": "assign_part",
                    "arguments": {"uuid": "comp-1", "part_uuid": "part-1"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("assign_part", "comp-1", "part-1")])
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "component",
        )

    def test_tools_call_dispatches_set_package(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2093,
                "method": "tools/call",
                "params": {
                    "name": "set_package",
                    "arguments": {"uuid": "comp-1", "package_uuid": "package-1"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("set_package", "comp-1", "package-1")])
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "component",
        )

    def test_tools_call_dispatches_set_net_class(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 20916,
                "method": "tools/call",
                "params": {
                    "name": "set_net_class",
                    "arguments": {
                        "net_uuid": "net-1",
                        "class_name": "power",
                        "clearance": 125000,
                        "track_width": 250000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [
                (
                    "set_net_class",
                    "net-1",
                    "power",
                    125000,
                    250000,
                    300000,
                    600000,
                    0,
                    0,
                )
            ],
        )
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "net",
        )

    def test_tools_call_dispatches_set_reference(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2092,
                "method": "tools/call",
                "params": {
                    "name": "set_reference",
                    "arguments": {"uuid": "comp-1", "reference": "R10"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("set_reference", "comp-1", "R10")])
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "component",
        )

    def test_tools_call_dispatches_move_component(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 210,
                "method": "tools/call",
                "params": {
                    "name": "move_component",
                    "arguments": {
                        "uuid": "comp-1",
                        "x_mm": 15.0,
                        "y_mm": 12.0,
                        "rotation_deg": 90.0,
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("move_component", "comp-1", 15.0, 12.0, 90.0)],
        )
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "component",
        )

    def test_tools_call_dispatches_rotate_component(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 221,
                "method": "tools/call",
                "params": {
                    "name": "rotate_component",
                    "arguments": {
                        "uuid": "comp-1",
                        "rotation_deg": 180.0,
                    },
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("rotate_component", "comp-1", 180.0)],
        )
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "component",
        )

    def test_tools_call_dispatches_set_value(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 223,
                "method": "tools/call",
                "params": {
                    "name": "set_value",
                    "arguments": {
                        "uuid": "comp-1",
                        "value": "22k",
                    },
                },
            }
        )
        self.assertEqual(daemon.calls, [("set_value", "comp-1", "22k")])
        self.assertEqual(
            response["result"]["content"][0]["json"]["diff"]["modified"][0]["object_type"],
            "component",
        )

    def test_tools_call_move_component_changes_followup_unrouted_response(self) -> None:
        class StatefulDaemon(self.FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.distance_nm = 20_000_000

            def move_component(
                self, uuid: str, x_mm: float, y_mm: float, rotation_deg: float | None
            ) -> JsonRpcResponse:
                response = super().move_component(uuid, x_mm, y_mm, rotation_deg)
                self.distance_nm = 16_278_820
                return response

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
                            "distance_nm": self.distance_nm,
                        }
                    ],
                    None,
                )

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        before = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 211,
                "method": "tools/call",
                "params": {
                    "name": "get_unrouted",
                    "arguments": {},
                },
            }
        )
        before_distance = before["result"]["content"][0]["json"][0]["distance_nm"]
        self.assertEqual(before_distance, 20_000_000)

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 212,
                "method": "tools/call",
                "params": {
                    "name": "move_component",
                    "arguments": {
                        "uuid": "comp-1",
                        "x_mm": 15.0,
                        "y_mm": 12.0,
                        "rotation_deg": 90.0,
                    },
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 213,
                "method": "tools/call",
                "params": {
                    "name": "get_unrouted",
                    "arguments": {},
                },
            }
        )
        after_distance = after["result"]["content"][0]["json"][0]["distance_nm"]
        self.assertNotEqual(after_distance, before_distance)
        self.assertEqual(
            daemon.calls,
            [
                ("get_unrouted", None),
                ("move_component", "comp-1", 15.0, 12.0, 90.0),
                ("get_unrouted", None),
            ],
        )

    def test_tools_call_rotate_component_changes_followup_components_response(self) -> None:
        class StatefulDaemon(self.FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.rotation = 0

            def rotate_component(self, uuid: str, rotation_deg: float) -> JsonRpcResponse:
                response = super().rotate_component(uuid, rotation_deg)
                self.rotation = int(rotation_deg)
                return response

            def get_components(self) -> JsonRpcResponse:
                self.calls.append(("get_components", None))
                return JsonRpcResponse(
                    "2.0",
                    27,
                    [
                        {
                            "uuid": "comp-1",
                            "reference": "R1",
                            "value": "10k",
                            "position": {"x": 10_000_000, "y": 10_000_000},
                            "rotation": self.rotation,
                            "layer": 0,
                            "locked": False,
                        }
                    ],
                    None,
                )

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        before = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 240,
                "method": "tools/call",
                "params": {"name": "get_components", "arguments": {}},
            }
        )
        before_components = before["result"]["content"][0]["json"]
        self.assertEqual(before_components[0]["rotation"], 0)

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 241,
                "method": "tools/call",
                "params": {
                    "name": "rotate_component",
                    "arguments": {"uuid": "comp-1", "rotation_deg": 180.0},
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 242,
                "method": "tools/call",
                "params": {"name": "get_components", "arguments": {}},
            }
        )
        after_components = after["result"]["content"][0]["json"]
        self.assertEqual(after_components[0]["rotation"], 180)
        self.assertEqual(
            daemon.calls,
            [
                ("get_components", None),
                ("rotate_component", "comp-1", 180.0),
                ("get_components", None),
            ],
        )

    def test_tools_call_delete_track_changes_followup_check_report(self) -> None:
        class StatefulDaemon(self.FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.deleted = False

            def delete_track(self, uuid: str) -> JsonRpcResponse:
                response = super().delete_track(uuid)
                self.deleted = True
                return response

            def get_check_report(self) -> JsonRpcResponse:
                self.calls.append(("get_check_report", None))
                return JsonRpcResponse(
                    "2.0",
                    2,
                    {
                        "domain": "board",
                        "summary": {
                            "status": "info" if self.deleted else "warning",
                            "errors": 0,
                            "warnings": 0 if self.deleted else 1,
                            "infos": 1 if self.deleted else 0,
                            "waived": 0,
                            "by_code": [
                                {
                                    "code": "net_without_copper" if self.deleted else "partially_routed_net",
                                    "count": 1,
                                }
                            ],
                        },
                        "diagnostics": [
                            {
                                "kind": "net_without_copper" if self.deleted else "partially_routed_net",
                                "severity": "info" if self.deleted else "warning",
                            }
                        ],
                    },
                    None,
                )

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        before = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 214,
                "method": "tools/call",
                "params": {"name": "get_check_report", "arguments": {}},
            }
        )
        before_kinds = [d["kind"] for d in before["result"]["content"][0]["json"]["diagnostics"]]
        self.assertEqual(before_kinds, ["partially_routed_net"])

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 215,
                "method": "tools/call",
                "params": {"name": "delete_track", "arguments": {"uuid": "track-1"}},
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 216,
                "method": "tools/call",
                "params": {"name": "get_check_report", "arguments": {}},
            }
        )
        after_kinds = [d["kind"] for d in after["result"]["content"][0]["json"]["diagnostics"]]
        self.assertEqual(after_kinds, ["net_without_copper"])
        self.assertEqual(
            daemon.calls,
            [
                ("get_check_report", None),
                ("delete_track", "track-1"),
                ("get_check_report", None),
            ],
        )

    def test_tools_call_delete_component_changes_followup_components_response(self) -> None:
        class StatefulDaemon(self.FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.present = True

            def delete_component(self, uuid: str) -> JsonRpcResponse:
                response = super().delete_component(uuid)
                self.present = False
                return response

            def get_components(self) -> JsonRpcResponse:
                self.calls.append(("get_components", None))
                components: list[dict[str, Any]] = []
                if self.present:
                    components.append(
                        {
                            "uuid": "comp-1",
                            "reference": "R1",
                            "value": "10k",
                            "position": {"x": 10_000_000, "y": 10_000_000},
                            "rotation": 0,
                            "layer": 0,
                            "locked": False,
                        }
                    )
                return JsonRpcResponse("2.0", 26, components, None)

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        before = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 211,
                "method": "tools/call",
                "params": {"name": "get_components", "arguments": {}},
            }
        )
        before_components = before["result"]["content"][0]["json"]
        self.assertEqual(len(before_components), 1)

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 212,
                "method": "tools/call",
                "params": {
                    "name": "delete_component",
                    "arguments": {"uuid": "comp-1"},
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 213,
                "method": "tools/call",
                "params": {"name": "get_components", "arguments": {}},
            }
        )
        after_components = after["result"]["content"][0]["json"]
        self.assertEqual(after_components, [])
        self.assertEqual(
            daemon.calls,
            [
                ("get_components", None),
                ("delete_component", "comp-1"),
                ("get_components", None),
            ],
        )

    def test_tools_call_delete_via_changes_followup_net_info_response(self) -> None:
        class StatefulDaemon(self.FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.vias = 1

            def delete_via(self, uuid: str) -> JsonRpcResponse:
                response = super().delete_via(uuid)
                self.vias = 0
                return response

            def get_net_info(self) -> JsonRpcResponse:
                self.calls.append(("get_net_info", None))
                return JsonRpcResponse(
                    "2.0",
                    22,
                    [
                        {"name": "GND", "tracks": 1, "vias": self.vias, "zones": 0},
                        {"name": "VCC", "tracks": 0, "vias": 0, "zones": 0},
                    ],
                    None,
                )

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        before = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 220,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        before_gnd = before["result"]["content"][0]["json"][0]
        self.assertEqual(before_gnd["name"], "GND")
        self.assertEqual(before_gnd["vias"], 1)

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 221,
                "method": "tools/call",
                "params": {"name": "delete_via", "arguments": {"uuid": "via-1"}},
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 222,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        after_gnd = after["result"]["content"][0]["json"][0]
        self.assertEqual(after_gnd["vias"], 0)
        self.assertEqual(
            daemon.calls,
            [
                ("get_net_info", None),
                ("delete_via", "via-1"),
                ("get_net_info", None),
            ],
        )

    def test_tools_call_dispatches_undo_and_redo(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        undo = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 206,
                "method": "tools/call",
                "params": {"name": "undo", "arguments": {}},
            }
        )
        redo = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 207,
                "method": "tools/call",
                "params": {"name": "redo", "arguments": {}},
            }
        )
        self.assertEqual(daemon.calls, [("undo", None), ("redo", None)])
        self.assertEqual(
            undo["result"]["content"][0]["json"]["diff"]["created"][0]["uuid"],
            "track-1",
        )
        self.assertEqual(
            redo["result"]["content"][0]["json"]["diff"]["deleted"][0]["uuid"],
            "track-1",
        )

    def test_tools_call_dispatches_search_pool(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 201,
                "method": "tools/call",
                "params": {
                    "name": "search_pool",
                    "arguments": {"query": "sot23"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("search_pool", "sot23")])
        self.assertEqual(response["result"]["content"][0]["json"][0]["package"], "SOT23")

    def test_tools_call_dispatches_get_part(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 202,
                "method": "tools/call",
                "params": {
                    "name": "get_part",
                    "arguments": {"uuid": "part-uuid"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_part", "part-uuid")])
        self.assertEqual(response["result"]["content"][0]["json"]["mpn"], "MMBT3904")

    def test_tools_call_dispatches_get_package(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 203,
                "method": "tools/call",
                "params": {
                    "name": "get_package",
                    "arguments": {"uuid": "package-uuid"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_package", "package-uuid")])
        self.assertEqual(response["result"]["content"][0]["json"]["name"], "SOT23")

    def test_tools_call_dispatches_check_report(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "get_check_report",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_check_report", None)])
        self.assertEqual(response["result"]["content"][0]["json"]["domain"], "board")
        self.assertEqual(response["result"]["content"][0]["json"]["summary"]["status"], "warning")

    def test_tools_call_dispatches_check_report_with_input_without_explicit_driver(self) -> None:
        class AnalogCheckDaemon(self.FakeDaemonClient):
            def get_check_report(self) -> JsonRpcResponse:
                self.calls.append(("get_check_report", None))
                return JsonRpcResponse(
                    "2.0",
                    2,
                    {
                        "domain": "schematic",
                        "summary": {
                            "status": "info",
                            "errors": 0,
                            "warnings": 0,
                            "infos": 1,
                            "waived": 0,
                            "by_code": [
                                {"code": "input_without_explicit_driver", "count": 1},
                            ],
                        },
                        "diagnostics": [],
                        "erc": [
                            {
                                "code": "input_without_explicit_driver",
                                "severity": "Info",
                                "net_name": "IN_P",
                            }
                        ],
                    },
                    None,
                )

        daemon = AnalogCheckDaemon()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 31,
                "method": "tools/call",
                "params": {
                    "name": "get_check_report",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_check_report", None)])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["domain"], "schematic")
        self.assertEqual(payload["summary"]["status"], "info")
        self.assertEqual(payload["summary"]["by_code"][0]["code"], "input_without_explicit_driver")
        self.assertEqual(payload["erc"][0]["code"], "input_without_explicit_driver")

    def test_tools_call_dispatches_board_summary(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {
                    "name": "get_board_summary",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_board_summary", None)])
        self.assertEqual(response["result"]["content"][0]["json"]["name"], "simple-demo")

    def test_tools_call_dispatches_schematic_summary(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 5,
                "method": "tools/call",
                "params": {
                    "name": "get_schematic_summary",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_schematic_summary", None)])
        self.assertEqual(response["result"]["content"][0]["json"]["sheet_count"], 1)

    def test_tools_call_dispatches_sheets(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 51,
                "method": "tools/call",
                "params": {
                    "name": "get_sheets",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_sheets", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["name"], "Root")

    def test_tools_call_dispatches_components(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 50,
                "method": "tools/call",
                "params": {
                    "name": "get_components",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_components", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["reference"], "R1")

    def test_tools_call_dispatches_symbol_fields(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 53,
                "method": "tools/call",
                "params": {
                    "name": "get_symbol_fields",
                    "arguments": {"symbol_uuid": "abcd"},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_symbol_fields", "abcd")])
        self.assertEqual(response["result"]["content"][0]["json"][0]["key"], "Reference")

    def test_tools_call_dispatches_netlist(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 52,
                "method": "tools/call",
                "params": {
                    "name": "get_netlist",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_netlist", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["name"], "GND")

    def test_tools_call_dispatches_bus_entries(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 54,
                "method": "tools/call",
                "params": {
                    "name": "get_bus_entries",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_bus_entries", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["uuid"], "be1")

    def test_tools_call_dispatches_board_net_info(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 6,
                "method": "tools/call",
                "params": {
                    "name": "get_net_info",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_net_info", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["name"], "GND")

    def test_tools_call_dispatches_schematic_net_info(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 7,
                "method": "tools/call",
                "params": {
                    "name": "get_schematic_net_info",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_schematic_net_info", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["name"], "SCL")

    def test_tools_call_dispatches_unrouted(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 8,
                "method": "tools/call",
                "params": {
                    "name": "get_unrouted",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_unrouted", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["net_name"], "SIG")

    def test_tools_call_dispatches_connectivity_diagnostics(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 76,
                "method": "tools/call",
                "params": {
                    "name": "get_connectivity_diagnostics",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_connectivity_diagnostics", None)])
        diagnostics = response["result"]["content"][0]["json"]
        self.assertEqual(len(diagnostics), 2)
        self.assertTrue(
            any(diagnostic["kind"] == "partially_routed_net" for diagnostic in diagnostics)
        )

    def test_tools_call_dispatches_design_rules(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 202,
                "method": "tools/call",
                "params": {
                    "name": "get_design_rules",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_design_rules", None)])
        self.assertEqual(response["result"]["content"][0]["json"], [])

    def test_tools_call_set_design_rule_changes_followup_design_rules_response(self) -> None:
        class StatefulDaemon(self.FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.rules: list[dict[str, Any]] = []

            def set_design_rule(
                self,
                rule_type: str,
                scope: dict[str, Any] | str,
                parameters: dict[str, Any],
                priority: int,
                name: str | None,
            ) -> JsonRpcResponse:
                response = super().set_design_rule(
                    rule_type, scope, parameters, priority, name
                )
                self.rules = [
                    {
                        "uuid": "rule-1",
                        "name": name or "default clearance",
                        "scope": scope,
                        "priority": priority,
                        "enabled": True,
                        "rule_type": rule_type,
                        "parameters": parameters,
                    }
                ]
                return response

            def get_design_rules(self) -> JsonRpcResponse:
                self.calls.append(("get_design_rules", None))
                return JsonRpcResponse("2.0", 102, self.rules, None)

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        before = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 217,
                "method": "tools/call",
                "params": {"name": "get_design_rules", "arguments": {}},
            }
        )
        self.assertEqual(before["result"]["content"][0]["json"], [])

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 218,
                "method": "tools/call",
                "params": {
                    "name": "set_design_rule",
                    "arguments": {
                        "rule_type": "ClearanceCopper",
                        "scope": "All",
                        "parameters": {"Clearance": {"min": 125000}},
                        "priority": 10,
                        "name": "default clearance",
                    },
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 219,
                "method": "tools/call",
                "params": {"name": "get_design_rules", "arguments": {}},
            }
        )
        rules = after["result"]["content"][0]["json"]
        self.assertEqual(len(rules), 1)
        self.assertEqual(rules[0]["name"], "default clearance")
        self.assertEqual(
            daemon.calls,
            [
                ("get_design_rules", None),
                (
                    "set_design_rule",
                    "ClearanceCopper",
                    "All",
                    {"Clearance": {"min": 125000}},
                    10,
                    "default clearance",
                ),
                ("get_design_rules", None),
            ],
        )

    def test_tools_call_set_value_changes_followup_components_response(self) -> None:
        class StatefulDaemon(self.FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.value = "10k"

            def set_value(self, uuid: str, value: str) -> JsonRpcResponse:
                response = super().set_value(uuid, value)
                self.value = value
                return response

            def get_components(self) -> JsonRpcResponse:
                self.calls.append(("get_components", None))
                return JsonRpcResponse(
                    "2.0",
                    24,
                    [
                        {
                            "uuid": "comp-1",
                            "reference": "R1",
                            "value": self.value,
                            "position": {"x": 10_000_000, "y": 10_000_000},
                            "rotation": 0,
                            "layer": 0,
                            "locked": False,
                        }
                    ],
                    None,
                )

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        before = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 224,
                "method": "tools/call",
                "params": {"name": "get_components", "arguments": {}},
            }
        )
        before_components = before["result"]["content"][0]["json"]
        self.assertEqual(before_components[0]["value"], "10k")

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 225,
                "method": "tools/call",
                "params": {
                    "name": "set_value",
                    "arguments": {"uuid": "comp-1", "value": "22k"},
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 226,
                "method": "tools/call",
                "params": {"name": "get_components", "arguments": {}},
            }
        )
        after_components = after["result"]["content"][0]["json"]
        self.assertEqual(after_components[0]["value"], "22k")
        self.assertEqual(
            daemon.calls,
            [
                ("get_components", None),
                ("set_value", "comp-1", "22k"),
                ("get_components", None),
            ],
        )

    def test_tools_call_assign_part_changes_followup_net_info_response(self) -> None:
        class StatefulDaemon(self.FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.pin_count = 2

            def assign_part(self, uuid: str, part_uuid: str) -> JsonRpcResponse:
                response = super().assign_part(uuid, part_uuid)
                self.pin_count = 1
                return response

            def get_net_info(self) -> JsonRpcResponse:
                self.calls.append(("get_net_info", None))
                return JsonRpcResponse(
                    "2.0",
                    28,
                    [
                        {
                            "uuid": "net-1",
                            "name": "SIG",
                            "class": "Default",
                            "pins": [
                                {"component": "R1", "pin": "1"}
                                for _ in range(self.pin_count)
                            ],
                            "tracks": 1,
                            "vias": 0,
                            "zones": 0,
                            "routed_length_nm": 11000000,
                            "routed_pct": 1.0,
                        }
                    ],
                    None,
                )

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        before = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 243,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        before_nets = before["result"]["content"][0]["json"]
        self.assertEqual(len(before_nets[0]["pins"]), 2)

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 244,
                "method": "tools/call",
                "params": {
                    "name": "assign_part",
                    "arguments": {"uuid": "comp-1", "part_uuid": "part-1"},
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 245,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        after_nets = after["result"]["content"][0]["json"]
        self.assertEqual(len(after_nets[0]["pins"]), 1)
        self.assertEqual(
            daemon.calls,
            [
                ("get_net_info", None),
                ("assign_part", "comp-1", "part-1"),
                ("get_net_info", None),
            ],
        )

    def test_tools_call_set_package_changes_followup_net_info_response(self) -> None:
        class StatefulDaemon(self.FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.pin_count = 2

            def set_package(self, uuid: str, package_uuid: str) -> JsonRpcResponse:
                response = super().set_package(uuid, package_uuid)
                self.pin_count = 1
                return response

            def get_net_info(self) -> JsonRpcResponse:
                self.calls.append(("get_net_info", None))
                return JsonRpcResponse(
                    "2.0",
                    28,
                    [
                        {
                            "uuid": "net-1",
                            "name": "SIG",
                            "class": "Default",
                            "pins": [
                                {"component": "R1", "pin": "1"}
                                for _ in range(self.pin_count)
                            ],
                            "tracks": 1,
                            "vias": 0,
                            "zones": 0,
                            "routed_length_nm": 11000000,
                            "routed_pct": 1.0,
                        }
                    ],
                    None,
                )

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        before = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2431,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        before_nets = before["result"]["content"][0]["json"]
        self.assertEqual(len(before_nets[0]["pins"]), 2)

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2432,
                "method": "tools/call",
                "params": {
                    "name": "set_package",
                    "arguments": {"uuid": "comp-1", "package_uuid": "package-1"},
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 2433,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        after_nets = after["result"]["content"][0]["json"]
        self.assertEqual(len(after_nets[0]["pins"]), 1)
        self.assertEqual(
            daemon.calls,
            [
                ("get_net_info", None),
                ("set_package", "comp-1", "package-1"),
                ("get_net_info", None),
            ],
        )

    def test_tools_call_set_net_class_changes_followup_net_info_response(self) -> None:
        class StatefulDaemon(self.FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.net_class = "Default"

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
                response = super().set_net_class(
                    net_uuid,
                    class_name,
                    clearance,
                    track_width,
                    via_drill,
                    via_diameter,
                    diffpair_width,
                    diffpair_gap,
                )
                self.net_class = class_name
                return response

            def get_net_info(self) -> JsonRpcResponse:
                self.calls.append(("get_net_info", None))
                return JsonRpcResponse(
                    "2.0",
                    29,
                    [
                        {
                            "uuid": "net-1",
                            "name": "GND",
                            "class": self.net_class,
                            "pins": [],
                            "tracks": 1,
                            "vias": 1,
                            "zones": 0,
                            "routed_length_nm": 1000000,
                            "routed_pct": 1.0,
                        }
                    ],
                    None,
                )

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        before = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 246,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        before_nets = before["result"]["content"][0]["json"]
        self.assertEqual(before_nets[0]["class"], "Default")

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 247,
                "method": "tools/call",
                "params": {
                    "name": "set_net_class",
                    "arguments": {
                        "net_uuid": "net-1",
                        "class_name": "power",
                        "clearance": 125000,
                        "track_width": 250000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                    },
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 248,
                "method": "tools/call",
                "params": {"name": "get_net_info", "arguments": {}},
            }
        )
        after_nets = after["result"]["content"][0]["json"]
        self.assertEqual(after_nets[0]["class"], "power")
        self.assertEqual(
            daemon.calls,
            [
                ("get_net_info", None),
                (
                    "set_net_class",
                    "net-1",
                    "power",
                    125000,
                    250000,
                    300000,
                    600000,
                    0,
                    0,
                ),
                ("get_net_info", None),
            ],
        )

    def test_tools_call_set_reference_changes_followup_components_response(self) -> None:
        class StatefulDaemon(self.FakeDaemonClient):
            def __init__(self) -> None:
                super().__init__()
                self.reference = "R1"

            def set_reference(self, uuid: str, reference: str) -> JsonRpcResponse:
                response = super().set_reference(uuid, reference)
                self.reference = reference
                return response

            def get_components(self) -> JsonRpcResponse:
                self.calls.append(("get_components", None))
                return JsonRpcResponse(
                    "2.0",
                    25,
                    [
                        {
                            "uuid": "comp-1",
                            "reference": self.reference,
                            "value": "10k",
                            "position": {"x": 10_000_000, "y": 10_000_000},
                            "rotation": 0,
                            "layer": 0,
                            "locked": False,
                        }
                    ],
                    None,
                )

        daemon = StatefulDaemon()
        host = StdioToolHost(daemon)
        before = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 227,
                "method": "tools/call",
                "params": {"name": "get_components", "arguments": {}},
            }
        )
        before_components = before["result"]["content"][0]["json"]
        self.assertEqual(before_components[0]["reference"], "R1")

        host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 228,
                "method": "tools/call",
                "params": {
                    "name": "set_reference",
                    "arguments": {"uuid": "comp-1", "reference": "R10"},
                },
            }
        )
        after = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 229,
                "method": "tools/call",
                "params": {"name": "get_components", "arguments": {}},
            }
        )
        after_components = after["result"]["content"][0]["json"]
        self.assertEqual(after_components[0]["reference"], "R10")
        self.assertEqual(
            daemon.calls,
            [
                ("get_components", None),
                ("set_reference", "comp-1", "R10"),
                ("get_components", None),
            ],
        )

    def test_tools_call_dispatches_labels(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 70,
                "method": "tools/call",
                "params": {
                    "name": "get_labels",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_labels", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["name"], "SCL")

    def test_tools_call_dispatches_ports(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 71,
                "method": "tools/call",
                "params": {
                    "name": "get_ports",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_ports", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["name"], "SUB_IN")

    def test_tools_call_dispatches_symbols(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 74,
                "method": "tools/call",
                "params": {
                    "name": "get_symbols",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_symbols", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["reference"], "R1")

    def test_tools_call_dispatches_buses(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 72,
                "method": "tools/call",
                "params": {
                    "name": "get_buses",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_buses", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["name"], "DATA")

    def test_tools_call_dispatches_hierarchy(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 73,
                "method": "tools/call",
                "params": {
                    "name": "get_hierarchy",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_hierarchy", None)])
        self.assertEqual(response["result"]["content"][0]["json"]["instances"][0]["name"], "child")

    def test_tools_call_dispatches_noconnects(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 75,
                "method": "tools/call",
                "params": {
                    "name": "get_noconnects",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("get_noconnects", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["pin"], "2")

    def test_tools_call_dispatches_run_erc(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 76,
                "method": "tools/call",
                "params": {
                    "name": "run_erc",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("run_erc", None)])
        self.assertEqual(response["result"]["content"][0]["json"][0]["code"], "undriven_power_net")

    def test_tools_call_dispatches_run_drc(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 77,
                "method": "tools/call",
                "params": {
                    "name": "run_drc",
                    "arguments": {},
                },
            }
        )
        self.assertEqual(daemon.calls, [("run_drc", None)])
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["passed"], False)
        self.assertEqual(payload["violations"][0]["code"], "connectivity_unrouted_net")

    def test_tools_call_dispatches_explain_violation(self) -> None:
        daemon = self.FakeDaemonClient()
        host = StdioToolHost(daemon)
        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 78,
                "method": "tools/call",
                "params": {
                    "name": "explain_violation",
                    "arguments": {"domain": "drc", "index": 0},
                },
            }
        )
        self.assertEqual(
            daemon.calls,
            [("explain_violation", {"domain": "drc", "index": 0})],
        )
        payload = response["result"]["content"][0]["json"]
        self.assertEqual(payload["rule_detail"], "drc connectivity_unrouted_net")


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--self-test":
        unittest.main(argv=[sys.argv[0]])
    else:
        host = StdioToolHost(EngineDaemonClient())
        host.run_stdio()
