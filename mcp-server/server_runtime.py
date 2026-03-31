#!/usr/bin/env python3
"""
EDA MCP Server — thin translation layer between MCP clients and the engine daemon.

Communicates with eda-engine-daemon via JSON-RPC over Unix socket.
See specs/MCP_API_SPEC.md for the full tool catalog.
"""

from __future__ import annotations

from dataclasses import dataclass
import json
import os
import shlex
import socket
import subprocess
import sys
from typing import Any

from tool_dispatch import dispatch_tool_call
from tools_catalog import TOOLS

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

    def validate_project_request(self, path: str) -> JsonRpcRequest:
        return self.build_request("validate_project", {"path": path})

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

    def set_package_with_part_request(
        self, uuid: str, package_uuid: str, part_uuid: str
    ) -> JsonRpcRequest:
        return self.build_request(
            "set_package_with_part",
            {"uuid": uuid, "package_uuid": package_uuid, "part_uuid": part_uuid},
        )

    def replace_component_request(
        self, uuid: str, package_uuid: str, part_uuid: str
    ) -> JsonRpcRequest:
        return self.build_request(
            "replace_component",
            {"uuid": uuid, "package_uuid": package_uuid, "part_uuid": part_uuid},
        )

    def replace_components_request(
        self, replacements: list[dict[str, str]]
    ) -> JsonRpcRequest:
        return self.build_request("replace_components", {"replacements": replacements})

    def apply_component_replacement_plan_request(
        self, replacements: list[dict[str, str | None]]
    ) -> JsonRpcRequest:
        return self.build_request(
            "apply_component_replacement_plan", {"replacements": replacements}
        )

    def apply_component_replacement_policy_request(
        self, replacements: list[dict[str, str]]
    ) -> JsonRpcRequest:
        return self.build_request(
            "apply_component_replacement_policy", {"replacements": replacements}
        )

    def apply_scoped_component_replacement_policy_request(
        self, scope: dict[str, str | None], policy: str
    ) -> JsonRpcRequest:
        return self.build_request(
            "apply_scoped_component_replacement_policy",
            {"scope": scope, "policy": policy},
        )

    def apply_scoped_component_replacement_plan_request(
        self, plan: dict[str, object]
    ) -> JsonRpcRequest:
        return self.build_request(
            "apply_scoped_component_replacement_plan",
            {"plan": plan},
        )

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

    def get_package_change_candidates_request(self, uuid: str) -> JsonRpcRequest:
        return self.build_request("get_package_change_candidates", {"uuid": uuid})

    def get_part_change_candidates_request(self, uuid: str) -> JsonRpcRequest:
        return self.build_request("get_part_change_candidates", {"uuid": uuid})

    def get_component_replacement_plan_request(self, uuid: str) -> JsonRpcRequest:
        return self.build_request("get_component_replacement_plan", {"uuid": uuid})

    def get_scoped_component_replacement_plan_request(
        self, scope: dict[str, str | None], policy: str
    ) -> JsonRpcRequest:
        return self.build_request(
            "get_scoped_component_replacement_plan",
            {"scope": scope, "policy": policy},
        )

    def edit_scoped_component_replacement_plan_request(
        self,
        plan: dict[str, object],
        exclude_component_uuids: list[str],
        overrides: list[dict[str, str]],
    ) -> JsonRpcRequest:
        return self.build_request(
            "edit_scoped_component_replacement_plan",
            {
                "plan": plan,
                "exclude_component_uuids": exclude_component_uuids,
                "overrides": overrides,
            },
        )

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

    def _cli_prefix(self) -> list[str]:
        configured = os.environ.get("EDA_CLI_BIN", "eda")
        prefix = shlex.split(configured)
        if not prefix:
            raise RuntimeError("EDA_CLI_BIN resolved to an empty command")
        return prefix

    def _run_cli_json(
        self,
        request: JsonRpcRequest,
        cli_args: list[str],
    ) -> JsonRpcResponse:
        return self._run_cli_json_allowing_statuses(request, cli_args, {0})

    def _run_cli_json_allowing_statuses(
        self,
        request: JsonRpcRequest,
        cli_args: list[str],
        allowed_statuses: set[int],
    ) -> JsonRpcResponse:
        completed = subprocess.run(
            [*self._cli_prefix(), "--format", "json", *cli_args],
            capture_output=True,
            text=True,
            check=False,
        )
        if completed.returncode not in allowed_statuses:
            detail = completed.stderr.strip() or completed.stdout.strip() or "unknown CLI failure"
            raise RuntimeError(detail)
        stdout = completed.stdout.strip()
        if not stdout:
            raise RuntimeError("eda CLI returned no JSON payload")
        try:
            result = json.loads(stdout)
        except json.JSONDecodeError as exc:
            raise RuntimeError(f"failed to parse eda CLI JSON: {exc}") from exc
        return JsonRpcResponse("2.0", request.id, result, None)

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

    def set_package_with_part(
        self, uuid: str, package_uuid: str, part_uuid: str
    ) -> JsonRpcResponse:
        return self.call(self.set_package_with_part_request(uuid, package_uuid, part_uuid))

    def replace_component(
        self, uuid: str, package_uuid: str, part_uuid: str
    ) -> JsonRpcResponse:
        return self.call(self.replace_component_request(uuid, package_uuid, part_uuid))

    def replace_components(
        self, replacements: list[dict[str, str]]
    ) -> JsonRpcResponse:
        return self.call(self.replace_components_request(replacements))

    def apply_component_replacement_plan(
        self, replacements: list[dict[str, str | None]]
    ) -> JsonRpcResponse:
        return self.call(self.apply_component_replacement_plan_request(replacements))

    def apply_component_replacement_policy(
        self, replacements: list[dict[str, str]]
    ) -> JsonRpcResponse:
        return self.call(self.apply_component_replacement_policy_request(replacements))

    def apply_scoped_component_replacement_policy(
        self, scope: dict[str, str | None], policy: str
    ) -> JsonRpcResponse:
        return self.call(
            self.apply_scoped_component_replacement_policy_request(scope, policy)
        )

    def apply_scoped_component_replacement_plan(
        self, plan: dict[str, object]
    ) -> JsonRpcResponse:
        return self.call(self.apply_scoped_component_replacement_plan_request(plan))

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

    def get_package_change_candidates(self, uuid: str) -> JsonRpcResponse:
        return self.call(self.get_package_change_candidates_request(uuid))

    def get_part_change_candidates(self, uuid: str) -> JsonRpcResponse:
        return self.call(self.get_part_change_candidates_request(uuid))

    def get_component_replacement_plan(self, uuid: str) -> JsonRpcResponse:
        return self.call(self.get_component_replacement_plan_request(uuid))

    def get_scoped_component_replacement_plan(
        self, scope: dict[str, str | None], policy: str
    ) -> JsonRpcResponse:
        return self.call(
            self.get_scoped_component_replacement_plan_request(scope, policy)
        )

    def edit_scoped_component_replacement_plan(
        self,
        plan: dict[str, object],
        exclude_component_uuids: list[str],
        overrides: list[dict[str, str]],
    ) -> JsonRpcResponse:
        return self.call(
            self.edit_scoped_component_replacement_plan_request(
                plan, exclude_component_uuids, overrides
            )
        )

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
        request = self.build_request(
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
        args = [
            "project",
            "export-route-path-proposal",
            path,
            "--net",
            net_uuid,
            "--from-anchor",
            from_anchor_pad_uuid,
            "--to-anchor",
            to_anchor_pad_uuid,
            "--candidate",
            candidate,
        ]
        if policy is not None:
            args.extend(["--policy", policy])
        args.extend(["--out", out])
        return self._run_cli_json(request, args)

    def route_proposal(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        profile: str | None = None,
    ) -> JsonRpcResponse:
        request = self.build_request(
            "route_proposal",
            {
                "path": path,
                "net_uuid": net_uuid,
                "from_anchor_pad_uuid": from_anchor_pad_uuid,
                "to_anchor_pad_uuid": to_anchor_pad_uuid,
                "profile": profile,
            },
        )
        args = [
            "project",
            "route-proposal",
            path,
            "--net",
            net_uuid,
            "--from-anchor",
            from_anchor_pad_uuid,
            "--to-anchor",
            to_anchor_pad_uuid,
        ]
        if profile is not None:
            args.extend(["--profile", profile])
        return self._run_cli_json(request, args)

    def route_strategy_report(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        objective: str | None = None,
    ) -> JsonRpcResponse:
        request = self.build_request(
            "route_strategy_report",
            {
                "path": path,
                "net_uuid": net_uuid,
                "from_anchor_pad_uuid": from_anchor_pad_uuid,
                "to_anchor_pad_uuid": to_anchor_pad_uuid,
                "objective": objective,
            },
        )
        args = [
            "project",
            "route-strategy-report",
            path,
            "--net",
            net_uuid,
            "--from-anchor",
            from_anchor_pad_uuid,
            "--to-anchor",
            to_anchor_pad_uuid,
        ]
        if objective is not None:
            args.extend(["--objective", objective])
        return self._run_cli_json(request, args)

    def route_strategy_compare(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
    ) -> JsonRpcResponse:
        request = self.build_request(
            "route_strategy_compare",
            {
                "path": path,
                "net_uuid": net_uuid,
                "from_anchor_pad_uuid": from_anchor_pad_uuid,
                "to_anchor_pad_uuid": to_anchor_pad_uuid,
            },
        )
        args = [
            "project",
            "route-strategy-compare",
            path,
            "--net",
            net_uuid,
            "--from-anchor",
            from_anchor_pad_uuid,
            "--to-anchor",
            to_anchor_pad_uuid,
        ]
        return self._run_cli_json(request, args)

    def route_strategy_delta(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
    ) -> JsonRpcResponse:
        request = self.build_request(
            "route_strategy_delta",
            {
                "path": path,
                "net_uuid": net_uuid,
                "from_anchor_pad_uuid": from_anchor_pad_uuid,
                "to_anchor_pad_uuid": to_anchor_pad_uuid,
            },
        )
        args = [
            "project",
            "route-strategy-delta",
            path,
            "--net",
            net_uuid,
            "--from-anchor",
            from_anchor_pad_uuid,
            "--to-anchor",
            to_anchor_pad_uuid,
        ]
        return self._run_cli_json(request, args)

    def write_route_strategy_curated_fixture_suite(
        self, out_dir: str, manifest: str | None = None
    ) -> JsonRpcResponse:
        request = self.build_request(
            "write_route_strategy_curated_fixture_suite",
            {
                "out_dir": out_dir,
                "manifest": manifest,
            },
        )
        args = [
            "project",
            "write-route-strategy-curated-fixture-suite",
            "--out-dir",
            out_dir,
        ]
        if manifest is not None:
            args.extend(["--manifest", manifest])
        return self._run_cli_json(request, args)

    def capture_route_strategy_curated_baseline(
        self, out_dir: str, manifest: str | None = None, result: str | None = None
    ) -> JsonRpcResponse:
        request = self.build_request(
            "capture_route_strategy_curated_baseline",
            {
                "out_dir": out_dir,
                "manifest": manifest,
                "result": result,
            },
        )
        args = [
            "project",
            "capture-route-strategy-curated-baseline",
            "--out-dir",
            out_dir,
        ]
        if manifest is not None:
            args.extend(["--manifest", manifest])
        if result is not None:
            args.extend(["--result", result])
        return self._run_cli_json(request, args)

    def route_strategy_batch_evaluate(self, requests: str) -> JsonRpcResponse:
        request = self.build_request(
            "route_strategy_batch_evaluate",
            {
                "requests": requests,
            },
        )
        args = [
            "project",
            "route-strategy-batch-evaluate",
            "--requests",
            requests,
        ]
        return self._run_cli_json(request, args)

    def validate_project(self, path: str) -> JsonRpcResponse:
        request = self.build_request(
            "validate_project",
            {
                "path": path,
            },
        )
        args = [
            "project",
            "validate",
            path,
        ]
        return self._run_cli_json_allowing_statuses(request, args, {0, 1})

    def inspect_route_strategy_batch_result(self, artifact: str) -> JsonRpcResponse:
        request = self.build_request(
            "inspect_route_strategy_batch_result",
            {
                "artifact": artifact,
            },
        )
        args = [
            "project",
            "inspect-route-strategy-batch-result",
            artifact,
        ]
        return self._run_cli_json(request, args)

    def validate_route_strategy_batch_result(self, artifact: str) -> JsonRpcResponse:
        request = self.build_request(
            "validate_route_strategy_batch_result",
            {
                "artifact": artifact,
            },
        )
        args = [
            "project",
            "validate-route-strategy-batch-result",
            artifact,
        ]
        return self._run_cli_json(request, args)

    def compare_route_strategy_batch_result(
        self, before: str, after: str
    ) -> JsonRpcResponse:
        request = self.build_request(
            "compare_route_strategy_batch_result",
            {
                "before": before,
                "after": after,
            },
        )
        args = [
            "project",
            "compare-route-strategy-batch-result",
            before,
            after,
        ]
        return self._run_cli_json(request, args)

    def gate_route_strategy_batch_result(
        self, before: str, after: str, policy: str | None = None
    ) -> JsonRpcResponse:
        request = self.build_request(
            "gate_route_strategy_batch_result",
            {
                "before": before,
                "after": after,
                "policy": policy,
            },
        )
        args = [
            "project",
            "gate-route-strategy-batch-result",
            before,
            after,
        ]
        if policy is not None:
            args.extend(["--policy", policy])
        return self._run_cli_json_allowing_statuses(request, args, {0, 2})

    def summarize_route_strategy_batch_results(
        self,
        dir: str | None = None,
        artifacts: list[str] | None = None,
        baseline: str | None = None,
        policy: str | None = None,
    ) -> JsonRpcResponse:
        request = self.build_request(
            "summarize_route_strategy_batch_results",
            {
                "dir": dir,
                "artifacts": artifacts,
                "baseline": baseline,
                "policy": policy,
            },
        )
        args = ["project", "summarize-route-strategy-batch-results"]
        if dir is not None:
            args.extend(["--dir", dir])
        for artifact in artifacts or []:
            args.extend(["--artifact", artifact])
        if baseline is not None:
            args.extend(["--baseline", baseline])
        if policy is not None:
            args.extend(["--policy", policy])
        return self._run_cli_json(request, args)

    def route_proposal_explain(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        profile: str | None = None,
    ) -> JsonRpcResponse:
        request = self.build_request(
            "route_proposal_explain",
            {
                "path": path,
                "net_uuid": net_uuid,
                "from_anchor_pad_uuid": from_anchor_pad_uuid,
                "to_anchor_pad_uuid": to_anchor_pad_uuid,
                "profile": profile,
            },
        )
        args = [
            "project",
            "route-proposal-explain",
            path,
            "--net",
            net_uuid,
            "--from-anchor",
            from_anchor_pad_uuid,
            "--to-anchor",
            to_anchor_pad_uuid,
        ]
        if profile is not None:
            args.extend(["--profile", profile])
        return self._run_cli_json(request, args)

    def export_route_proposal(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        out: str,
        profile: str | None = None,
    ) -> JsonRpcResponse:
        request = self.build_request(
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
        args = [
            "project",
            "export-route-proposal",
            path,
            "--net",
            net_uuid,
            "--from-anchor",
            from_anchor_pad_uuid,
            "--to-anchor",
            to_anchor_pad_uuid,
        ]
        if profile is not None:
            args.extend(["--profile", profile])
        args.extend(["--out", out])
        return self._run_cli_json(request, args)

    def route_apply(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        candidate: str,
        policy: str | None,
    ) -> JsonRpcResponse:
        request = self.build_request(
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
        args = [
            "project",
            "route-apply",
            path,
            "--net",
            net_uuid,
            "--from-anchor",
            from_anchor_pad_uuid,
            "--to-anchor",
            to_anchor_pad_uuid,
            "--candidate",
            candidate,
        ]
        if policy is not None:
            args.extend(["--policy", policy])
        return self._run_cli_json(request, args)

    def route_apply_selected(
        self,
        path: str,
        net_uuid: str,
        from_anchor_pad_uuid: str,
        to_anchor_pad_uuid: str,
        profile: str | None = None,
    ) -> JsonRpcResponse:
        request = self.build_request(
            "route_apply_selected",
            {
                "path": path,
                "net_uuid": net_uuid,
                "from_anchor_pad_uuid": from_anchor_pad_uuid,
                "to_anchor_pad_uuid": to_anchor_pad_uuid,
                "profile": profile,
            },
        )
        args = [
            "project",
            "route-apply-selected",
            path,
            "--net",
            net_uuid,
            "--from-anchor",
            from_anchor_pad_uuid,
            "--to-anchor",
            to_anchor_pad_uuid,
        ]
        if profile is not None:
            args.extend(["--profile", profile])
        return self._run_cli_json(request, args)

    def inspect_route_proposal_artifact(self, artifact: str) -> JsonRpcResponse:
        request = self.build_request(
            "inspect_route_proposal_artifact",
            {
                "artifact": artifact,
            },
        )
        return self._run_cli_json(
            request,
            [
                "project",
                "inspect-route-proposal-artifact",
                artifact,
            ],
        )

    def revalidate_route_proposal_artifact(
        self,
        path: str,
        artifact: str,
    ) -> JsonRpcResponse:
        request = self.build_request(
            "revalidate_route_proposal_artifact",
            {
                "path": path,
                "artifact": artifact,
            },
        )
        return self._run_cli_json(
            request,
            [
                "project",
                "revalidate-route-proposal-artifact",
                path,
                "--artifact",
                artifact,
            ],
        )

    def apply_route_proposal_artifact(
        self,
        path: str,
        artifact: str,
    ) -> JsonRpcResponse:
        request = self.build_request(
            "apply_route_proposal_artifact",
            {
                "path": path,
                "artifact": artifact,
            },
        )
        return self._run_cli_json(
            request,
            [
                "project",
                "apply-route-proposal-artifact",
                path,
                "--artifact",
                artifact,
            ],
        )


class StdioToolHost:
    def __init__(self, daemon: EngineDaemonClient) -> None:
        self._daemon = daemon

    def handle_message(self, message: dict[str, Any]) -> dict[str, Any] | None:
        method = message.get("method")
        msg_id = message.get("id")
        params = message.get("params", {})

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

        if msg_id is None:
            return None

        return {
            "jsonrpc": "2.0",
            "id": msg_id,
            "error": {"code": -32601, "message": "method not found"},
        }

    def _call_tool(self, name: str, arguments: dict[str, Any]) -> dict[str, Any]:
        response = dispatch_tool_call(self._daemon, name, arguments)

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
def run_server() -> None:
    host = StdioToolHost(EngineDaemonClient())
    host.run_stdio()
if __name__ == "__main__":
    run_server()
