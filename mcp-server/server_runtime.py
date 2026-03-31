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

    def validate_project_request(self, path: str) -> JsonRpcRequest:
        return self.build_request("validate_project", {"path": path})
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

    def review_route_proposal(
        self,
        path: str | None = None,
        net_uuid: str | None = None,
        from_anchor_pad_uuid: str | None = None,
        to_anchor_pad_uuid: str | None = None,
        profile: str | None = None,
        artifact: str | None = None,
    ) -> JsonRpcResponse:
        request = self.build_request(
            "review_route_proposal",
            {
                "path": path,
                "net_uuid": net_uuid,
                "from_anchor_pad_uuid": from_anchor_pad_uuid,
                "to_anchor_pad_uuid": to_anchor_pad_uuid,
                "profile": profile,
                "artifact": artifact,
            },
        )
        args = ["project", "review-route-proposal"]
        if artifact is not None:
            args.extend(["--artifact", artifact])
            return self._run_cli_json(request, args)
        if path is None or net_uuid is None or from_anchor_pad_uuid is None or to_anchor_pad_uuid is None:
            raise RuntimeError(
                "review_route_proposal requires either artifact or path/net_uuid/from_anchor_pad_uuid/to_anchor_pad_uuid"
            )
        args.extend(
            [
                path,
                "--net",
                net_uuid,
                "--from-anchor",
                from_anchor_pad_uuid,
                "--to-anchor",
                to_anchor_pad_uuid,
            ]
        )
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


_REQUIRED = object()

DAEMON_CLIENT_METHOD_SPECS: list[dict[str, Any]] = [
    {"name": "open_project", "params": [("path", _REQUIRED)]},
    {"name": "close_project", "params": []},
    {"name": "save", "params": [("path", None)]},
    {"name": "delete_track", "params": [("uuid", _REQUIRED)]},
    {"name": "delete_via", "params": [("uuid", _REQUIRED)]},
    {"name": "delete_component", "params": [("uuid", _REQUIRED)]},
    {
        "name": "move_component",
        "params": [
            ("uuid", _REQUIRED),
            ("x_mm", _REQUIRED),
            ("y_mm", _REQUIRED),
            ("rotation_deg", None),
        ],
    },
    {
        "name": "rotate_component",
        "params": [("uuid", _REQUIRED), ("rotation_deg", _REQUIRED)],
        "fixed": {"x_mm": 0.0, "y_mm": 0.0},
    },
    {"name": "set_value", "params": [("uuid", _REQUIRED), ("value", _REQUIRED)]},
    {"name": "assign_part", "params": [("uuid", _REQUIRED), ("part_uuid", _REQUIRED)]},
    {
        "name": "set_package",
        "params": [("uuid", _REQUIRED), ("package_uuid", _REQUIRED)],
    },
    {
        "name": "set_package_with_part",
        "params": [
            ("uuid", _REQUIRED),
            ("package_uuid", _REQUIRED),
            ("part_uuid", _REQUIRED),
        ],
    },
    {
        "name": "replace_component",
        "params": [
            ("uuid", _REQUIRED),
            ("package_uuid", _REQUIRED),
            ("part_uuid", _REQUIRED),
        ],
    },
    {"name": "replace_components", "params": [("replacements", _REQUIRED)]},
    {
        "name": "apply_component_replacement_plan",
        "params": [("replacements", _REQUIRED)],
    },
    {
        "name": "apply_component_replacement_policy",
        "params": [("replacements", _REQUIRED)],
    },
    {
        "name": "apply_scoped_component_replacement_policy",
        "params": [("scope", _REQUIRED), ("policy", _REQUIRED)],
    },
    {
        "name": "apply_scoped_component_replacement_plan",
        "params": [("plan", _REQUIRED)],
    },
    {
        "name": "set_net_class",
        "params": [
            ("net_uuid", _REQUIRED),
            ("class_name", _REQUIRED),
            ("clearance", _REQUIRED),
            ("track_width", _REQUIRED),
            ("via_drill", _REQUIRED),
            ("via_diameter", _REQUIRED),
            ("diffpair_width", 0),
            ("diffpair_gap", 0),
        ],
    },
    {
        "name": "set_reference",
        "params": [("uuid", _REQUIRED), ("reference", _REQUIRED)],
    },
    {
        "name": "set_design_rule",
        "params": [
            ("rule_type", _REQUIRED),
            ("scope", _REQUIRED),
            ("parameters", _REQUIRED),
            ("priority", _REQUIRED),
            ("name", None),
        ],
    },
    {"name": "undo", "params": []},
    {"name": "redo", "params": []},
    {"name": "search_pool", "params": [("query", _REQUIRED)]},
    {"name": "get_part", "params": [("uuid", _REQUIRED)]},
    {"name": "get_package", "params": [("uuid", _REQUIRED)]},
    {"name": "get_package_change_candidates", "params": [("uuid", _REQUIRED)]},
    {"name": "get_part_change_candidates", "params": [("uuid", _REQUIRED)]},
    {"name": "get_component_replacement_plan", "params": [("uuid", _REQUIRED)]},
    {
        "name": "get_scoped_component_replacement_plan",
        "params": [("scope", _REQUIRED), ("policy", _REQUIRED)],
    },
    {
        "name": "edit_scoped_component_replacement_plan",
        "params": [
            ("plan", _REQUIRED),
            ("exclude_component_uuids", _REQUIRED),
            ("overrides", _REQUIRED),
        ],
    },
    {"name": "get_board_summary", "params": []},
    {"name": "get_components", "params": []},
    {"name": "get_netlist", "params": []},
    {"name": "get_schematic_summary", "params": []},
    {"name": "get_sheets", "params": []},
    {"name": "get_labels", "params": []},
    {"name": "get_symbols", "params": []},
    {"name": "get_symbol_fields", "params": [("symbol_uuid", _REQUIRED)]},
    {"name": "get_ports", "params": []},
    {"name": "get_buses", "params": []},
    {"name": "get_bus_entries", "params": []},
    {"name": "get_noconnects", "params": []},
    {"name": "get_hierarchy", "params": []},
    {"name": "get_net_info", "params": []},
    {"name": "get_unrouted", "params": []},
    {"name": "get_schematic_net_info", "params": []},
    {"name": "get_check_report", "params": []},
    {"name": "get_connectivity_diagnostics", "params": []},
    {"name": "get_design_rules", "params": []},
    {"name": "run_erc", "params": []},
    {"name": "run_drc", "params": []},
    {
        "name": "explain_violation",
        "params": [("domain", _REQUIRED), ("index", _REQUIRED)],
    },
]


def _build_client_params(
    method_name: str,
    param_specs: list[tuple[str, Any]],
    args: tuple[Any, ...],
    kwargs: dict[str, Any],
) -> dict[str, Any]:
    if len(args) > len(param_specs):
        raise TypeError(
            f"{method_name} expected at most {len(param_specs)} arguments, got {len(args)}"
        )
    remaining_kwargs = dict(kwargs)
    params: dict[str, Any] = {}
    for index, (param_name, default) in enumerate(param_specs):
        if index < len(args):
            value = args[index]
        elif param_name in remaining_kwargs:
            value = remaining_kwargs.pop(param_name)
        elif default is not _REQUIRED:
            value = default
        else:
            raise TypeError(f"{method_name} missing required argument: {param_name}")
        params[param_name] = value
    if remaining_kwargs:
        unknown = ", ".join(sorted(remaining_kwargs))
        raise TypeError(f"{method_name} got unexpected keyword arguments: {unknown}")
    return params


def _install_daemon_client_methods() -> None:
    for spec in DAEMON_CLIENT_METHOD_SPECS:
        name = spec["name"]
        param_specs = spec["params"]
        fixed = dict(spec.get("fixed", {}))

        def request_method(
            self: EngineDaemonClient,
            *args: Any,
            _name: str = name,
            _param_specs: list[tuple[str, Any]] = param_specs,
            _fixed: dict[str, Any] = fixed,
            **kwargs: Any,
        ) -> JsonRpcRequest:
            params = _build_client_params(_name, _param_specs, args, kwargs)
            return self.build_request(_name, {**_fixed, **params})

        def call_method(
            self: EngineDaemonClient,
            *args: Any,
            _name: str = name,
            **kwargs: Any,
        ) -> JsonRpcResponse:
            request = getattr(self, f"{_name}_request")(*args, **kwargs)
            return self.call(request)

        setattr(EngineDaemonClient, f"{name}_request", request_method)
        setattr(EngineDaemonClient, name, call_method)


_install_daemon_client_methods()


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
