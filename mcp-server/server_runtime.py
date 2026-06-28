#!/usr/bin/env python3
from __future__ import annotations
from dataclasses import dataclass
import json, os, shlex, socket, subprocess
from typing import Any
from library_authoring_methods import cli_run_kwargs_for_method
from stdio_tool_host import StdioToolHost
def _append_optional(args: list[str], flag: str, value: Any | None) -> None: args.extend([f"--{flag}", str(value)]) if value is not None else None
def _append_optional_bool(args: list[str], flag: str, value: bool | None) -> None: _append_optional(args, flag, str(value).lower() if value is not None else None)
def _component_instance_symbols(symbol: str | None, symbols: list[str] | None) -> list[str]: result = [str(value) for value in symbols] if symbols is not None else ([] if symbol is None else [symbol]); return result if result else (_ for _ in ()).throw(ValueError("symbol or symbols is required"))
def _component_role_spec(object_id: str, value: Any) -> str: role, label = (value.get("role"), value.get("label")) if isinstance(value, dict) else (value, None); return f"{object_id}={role}:{label}" if label is not None else f"{object_id}={role}"
def _append_component_role_args(args: list[str], flag: str, roles: Any | None) -> None: [args.extend([f"--{flag}", _component_role_spec(str(object_id), value)]) for object_id, value in roles.items()] if isinstance(roles, dict) else [args.extend([f"--{flag}", str(value)]) for value in ([] if roles is None else roles)]
@dataclass(frozen=True)
class JsonRpcRequest:
    jsonrpc: str; id: int; method: str; params: dict[str, Any]
    def to_json(self) -> str: return json.dumps({"jsonrpc": self.jsonrpc, "id": self.id, "method": self.method, "params": self.params})
@dataclass(frozen=True)
class JsonRpcError: code: int; message: str
@dataclass(frozen=True)
class JsonRpcResponse:
    jsonrpc: str; id: int; result: Any | None; error: JsonRpcError | None
    @staticmethod
    def from_json(payload: str) -> "JsonRpcResponse":
        decoded = json.loads(payload)
        error = decoded.get("error")
        return JsonRpcResponse(jsonrpc=decoded["jsonrpc"], id=decoded["id"], result=decoded.get("result"), error=None if error is None else JsonRpcError(code=error["code"], message=error["message"]))
class EngineDaemonClient:
    def __init__(self, socket_path: str | None = None) -> None:
        self._next_id = 1
        self._socket_path = socket_path or os.environ.get("DATUM_ENGINE_SOCKET") or os.environ.get("EDA_ENGINE_SOCKET")
    def build_request(self, method: str, params: dict[str, Any]) -> JsonRpcRequest:
        request = JsonRpcRequest(jsonrpc="2.0", id=self._next_id, method=method, params=params)
        self._next_id += 1
        return request
    def validate_project_request(self, path: str) -> JsonRpcRequest:
        return self.build_request("validate_project", {"path": path})
    def _cli_prefix(self) -> list[str]:
        configured = os.environ.get("DATUM_CLI_BIN") or os.environ.get("EDA_CLI_BIN") or "datum-eda"
        prefix = shlex.split(configured)
        if not prefix:
            raise RuntimeError("EDA_CLI_BIN resolved to an empty command")
        return prefix
    def _run_cli_json(self, request: JsonRpcRequest, cli_args: list[str]) -> JsonRpcResponse:
        return self._run_cli_json_allowing_statuses(request, cli_args, {0})
    def _run_cli_json_allowing_statuses(self, request: JsonRpcRequest, cli_args: list[str], allowed_statuses: set[int]) -> JsonRpcResponse:
        completed = subprocess.run([*self._cli_prefix(), "--format", "json", *cli_args], **cli_run_kwargs_for_method(request.method))
        if completed.returncode not in allowed_statuses:
            detail = completed.stderr.strip() or completed.stdout.strip() or "unknown CLI failure"
            raise RuntimeError(detail)
        stdout = completed.stdout.strip()
        if not stdout:
            raise RuntimeError("datum-eda CLI returned no JSON payload")
        try:
            result = json.loads(stdout)
        except json.JSONDecodeError as exc:
            raise RuntimeError(f"failed to parse datum-eda CLI JSON: {exc}") from exc
        return JsonRpcResponse("2.0", request.id, result, None)
    def call(self, request: JsonRpcRequest) -> JsonRpcResponse:
        if not self._socket_path:
            raise RuntimeError("DATUM_ENGINE_SOCKET/EDA_ENGINE_SOCKET is not configured")
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
            client.connect(self._socket_path)
            client.sendall(request.to_json().encode("utf-8") + b"\n")
            data = b""
            while not data.endswith(b"\n"):
                chunk = client.recv(4096)
                if not chunk: break
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
        if result is not None: args.extend(["--result", result])
        return self._run_cli_json(request, args)
    def route_strategy_batch_evaluate(self, requests: str) -> JsonRpcResponse:
        request = self.build_request("route_strategy_batch_evaluate", {"requests": requests})
        args = ["project", "route-strategy-batch-evaluate", "--requests", requests]
        return self._run_cli_json(request, args)
    def validate_project(self, path: str) -> JsonRpcResponse: return self._run_cli_json_allowing_statuses(self.build_request("validate_project", {"path": path}), ["project", "validate", path], {0, 1})
    def datum_context_get(self, session: str | None = None, path: str | None = None, project_root: str | None = None) -> JsonRpcResponse: args = ["context", "get"]; _append_optional(args, "session", session); _append_optional(args, "path", path); _append_optional(args, "project-root", project_root); return self._run_cli_json(self.build_request("datum.context.get", {"session": session, "path": path, "project_root": project_root}), args)
    def datum_context_refresh(self, session: str | None = None, path: str | None = None, project_root: str | None = None) -> JsonRpcResponse: args = ["context", "refresh"]; _append_optional(args, "session", session); _append_optional(args, "path", path); _append_optional(args, "project-root", project_root); return self._run_cli_json(self.build_request("datum.context.refresh", {"session": session, "path": path, "project_root": project_root}), args)
    def datum_context_session_events(self, session: str | None = None, path: str | None = None, project_root: str | None = None, event_kind: str | None = None, origin: str | None = None, command_id: str | None = None, execution_id: str | None = None, limit: int | None = None) -> JsonRpcResponse: args = ["context", "session-events"]; _append_optional(args, "session", session); _append_optional(args, "path", path); _append_optional(args, "project-root", project_root); _append_optional(args, "event-kind", event_kind); _append_optional(args, "origin", origin); _append_optional(args, "command-id", command_id); _append_optional(args, "execution-id", execution_id); _append_optional(args, "limit", limit); return self._run_cli_json(self.build_request("datum.context.session_events", {"session": session, "path": path, "project_root": project_root, "event_kind": event_kind, "origin": origin, "command_id": command_id, "execution_id": execution_id, "limit": limit}), args)
    def datum_context_session_activity(self, session: str | None = None, path: str | None = None, project_root: str | None = None, event_kind: str | None = None, origin: str | None = None, command_id: str | None = None, execution_id: str | None = None, limit: int | None = None) -> JsonRpcResponse: args = ["context", "session-activity"]; _append_optional(args, "session", session); _append_optional(args, "path", path); _append_optional(args, "project-root", project_root); _append_optional(args, "event-kind", event_kind); _append_optional(args, "origin", origin); _append_optional(args, "command-id", command_id); _append_optional(args, "execution-id", execution_id); _append_optional(args, "limit", limit); return self._run_cli_json(self.build_request("datum.context.session_activity", {"session": session, "path": path, "project_root": project_root, "event_kind": event_kind, "origin": origin, "command_id": command_id, "execution_id": execution_id, "limit": limit}), args)
    def generate_artifacts(self, path: str, output_dir: str | None = None, include: str | None = None, prefix: str | None = None, output_job: str | None = None) -> JsonRpcResponse: args = ["artifact", "generate", path]; _append_optional(args, "output-dir", output_dir); _append_optional(args, "include", include); _append_optional(args, "prefix", prefix); _append_optional(args, "output-job", output_job); return self._run_cli_json(self.build_request("generate_artifacts", {"path": path, "output_dir": output_dir, "include": include, "prefix": prefix, "output_job": output_job}), args)
    def get_artifacts(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_artifacts", {"path": path}), ["artifact", "list", path])
    def show_artifact(self, path: str, artifact: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("show_artifact", {"path": path, "artifact": artifact}), ["artifact", "show", path, "--artifact", artifact])
    def get_artifact_files(self, path: str, artifact: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_artifact_files", {"path": path, "artifact": artifact}), ["artifact", "files", path, "--artifact", artifact])
    def preview_artifact_file(self, path: str, artifact: str, artifact_dir: str | None, file: str) -> JsonRpcResponse: args = ["artifact", "preview", path, "--artifact", artifact]; _append_optional(args, "artifact-dir", artifact_dir); args.extend(["--file", file]); return self._run_cli_json(self.build_request("preview_artifact_file", {"path": path, "artifact": artifact, "artifact_dir": artifact_dir, "file": file}), args)
    def compare_artifacts(self, path: str, before: str, after: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("compare_artifacts", {"path": path, "before": before, "after": after}), ["artifact", "compare", path, "--before", before, "--after", after])
    def validate_artifact(self, path: str, artifact: str) -> JsonRpcResponse: return self._run_cli_json_allowing_statuses(self.build_request("validate_artifact", {"path": path, "artifact": artifact}), ["artifact", "validate", path, "--artifact", artifact], {0, 1})
    def get_output_jobs(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_output_jobs", {"path": path}), ["query", "output-jobs", path])
    def get_component_instances(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_component_instances", {"path": path}), ["query", "component-instances", path])
    def bind_component_instance(self, path: str, symbol: str | None, package: str, component_instance: str | None = None, symbols: list[str] | None = None, part: str | None = None, symbol_roles: Any | None = None, package_roles: Any | None = None) -> JsonRpcResponse: args = ["project", "bind-component-instance", path]; [args.extend(["--symbol", value]) for value in _component_instance_symbols(symbol, symbols)]; args.extend(["--package", package]); _append_optional(args, "part", part); _append_component_role_args(args, "symbol-role", symbol_roles); _append_component_role_args(args, "package-role", package_roles); _append_optional(args, "component-instance", component_instance); return self._run_cli_json(self.build_request("bind_component_instance", {"path": path, "symbol": symbol, "symbols": symbols, "package": package, "part": part, "symbol_roles": symbol_roles, "package_roles": package_roles, "component_instance": component_instance}), args)
    def set_component_instance(self, path: str, component_instance: str, symbol: str | None, package: str, symbols: list[str] | None = None, part: str | None = None, symbol_roles: Any | None = None, package_roles: Any | None = None) -> JsonRpcResponse: args = ["project", "set-component-instance", path, "--component-instance", component_instance]; [args.extend(["--symbol", value]) for value in _component_instance_symbols(symbol, symbols)]; args.extend(["--package", package]); _append_optional(args, "part", part); _append_component_role_args(args, "symbol-role", symbol_roles); _append_component_role_args(args, "package-role", package_roles); return self._run_cli_json(self.build_request("set_component_instance", {"path": path, "component_instance": component_instance, "symbol": symbol, "symbols": symbols, "package": package, "part": part, "symbol_roles": symbol_roles, "package_roles": package_roles}), args)
    def delete_component_instance(self, path: str, component_instance: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_component_instance", {"path": path, "component_instance": component_instance}), ["project", "delete-component-instance", path, "--component-instance", component_instance])
    def get_pool_library_objects(self, path: str, pool: str | None = None, kind: str | None = None, object: str | None = None, include_payload: bool | None = None) -> JsonRpcResponse: args = ["project", "query", path, "pool-library-objects"]; _append_optional(args, "pool", pool); _append_optional(args, "kind", kind); _append_optional(args, "object", object); args.extend(["--include-payload"] if include_payload else []); return self._run_cli_json(self.build_request("get_pool_library_objects", {"path": path, "pool": pool, "kind": kind, "object": object, "include_payload": include_payload}), args)
    def show_pool_library_object(self, path: str, object: str, pool: str | None = None, kind: str | None = None) -> JsonRpcResponse: args = ["project", "query", path, "pool-library-objects", "--object", object, "--include-payload"]; _append_optional(args, "pool", pool); _append_optional(args, "kind", kind); return self._run_cli_json(self.build_request("show_pool_library_object", {"path": path, "object": object, "pool": pool, "kind": kind}), args)
    def create_pool_library_object(self, path: str, pool: str, kind: str, object: str, from_json: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("create_pool_library_object", {"path": path, "pool": pool, "kind": kind, "object": object, "from_json": from_json}), ["project", "create-pool-library-object", path, "--pool", pool, "--kind", kind, "--object", object, "--from-json", from_json])
    def create_pool_unit(self, path: str, pool: str, unit: str, name: str, manufacturer: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("create_pool_unit", {"path": path, "pool": pool, "unit": unit, "name": name, "manufacturer": manufacturer}), ["project", "create-pool-unit", path, "--pool", pool, "--unit", unit, "--name", name, "--manufacturer", manufacturer])
    def create_pool_symbol(self, path: str, pool: str, symbol: str, unit: str, name: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("create_pool_symbol", {"path": path, "pool": pool, "symbol": symbol, "unit": unit, "name": name}), ["project", "create-pool-symbol", path, "--pool", pool, "--symbol", symbol, "--unit", unit, "--name", name])
    def create_pool_entity(self, path: str, pool: str, entity: str, gate: str, unit: str, symbol: str, name: str, prefix: str, manufacturer: str, gate_name: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("create_pool_entity", {"path": path, "pool": pool, "entity": entity, "gate": gate, "unit": unit, "symbol": symbol, "name": name, "prefix": prefix, "manufacturer": manufacturer, "gate_name": gate_name}), ["project", "create-pool-entity", path, "--pool", pool, "--entity", entity, "--gate", gate, "--unit", unit, "--symbol", symbol, "--name", name, "--prefix", prefix, "--manufacturer", manufacturer, "--gate-name", gate_name])
    def create_pool_padstack(self, path: str, pool: str, padstack: str, name: str, aperture: str | None = None, diameter_nm: int | None = None, width_nm: int | None = None, height_nm: int | None = None, drill_nm: int | None = None) -> JsonRpcResponse: args = ["project", "create-pool-padstack", path, "--pool", pool, "--padstack", padstack, "--name", name]; _append_optional(args, "aperture", aperture); _append_optional(args, "diameter-nm", diameter_nm); _append_optional(args, "width-nm", width_nm); _append_optional(args, "height-nm", height_nm); _append_optional(args, "drill-nm", drill_nm); return self._run_cli_json(self.build_request("create_pool_padstack", {"path": path, "pool": pool, "padstack": padstack, "name": name, "aperture": aperture, "diameter_nm": diameter_nm, "width_nm": width_nm, "height_nm": height_nm, "drill_nm": drill_nm}), args)
    def create_pool_package(self, path: str, pool: str, package: str, name: str, pad: str, padstack: str, pad_name: str, x_nm: int, y_nm: int, layer: int) -> JsonRpcResponse: return self._run_cli_json(self.build_request("create_pool_package", {"path": path, "pool": pool, "package": package, "name": name, "pad": pad, "padstack": padstack, "pad_name": pad_name, "x_nm": x_nm, "y_nm": y_nm, "layer": layer}), ["project", "create-pool-package", path, "--pool", pool, "--package", package, "--name", name, "--pad", pad, "--padstack", padstack, "--pad-name", pad_name, "--x-nm", str(x_nm), "--y-nm", str(y_nm), "--layer", str(layer)])
    def create_pool_part(self, path: str, pool: str, part: str, entity: str, package: str, mpn: str, manufacturer: str, value: str, description: str, datasheet: str, lifecycle: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("create_pool_part", {"path": path, "pool": pool, "part": part, "entity": entity, "package": package, "mpn": mpn, "manufacturer": manufacturer, "value": value, "description": description, "datasheet": datasheet, "lifecycle": lifecycle}), ["project", "create-pool-part", path, "--pool", pool, "--part", part, "--entity", entity, "--package", package, "--mpn", mpn, "--manufacturer", manufacturer, "--value", value, "--description", description, "--datasheet", datasheet, "--lifecycle", lifecycle])
    def set_pool_part_pad_map_entry(self, path: str, pool: str, part: str, pad: str, gate: str, pin: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("set_pool_part_pad_map_entry", {"path": path, "pool": pool, "part": part, "pad": pad, "gate": gate, "pin": pin}), ["project", "set-pool-part-pad-map-entry", path, "--pool", pool, "--part", part, "--pad", pad, "--gate", gate, "--pin", pin])
    def set_pool_library_object(self, path: str, pool: str, kind: str, object: str, from_json: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("set_pool_library_object", {"path": path, "pool": pool, "kind": kind, "object": object, "from_json": from_json}), ["project", "set-pool-library-object", path, "--pool", pool, "--kind", kind, "--object", object, "--from-json", from_json])
    def delete_pool_library_object(self, path: str, pool: str, kind: str, object: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_pool_library_object", {"path": path, "pool": pool, "kind": kind, "object": object}), ["project", "delete-pool-library-object", path, "--pool", pool, "--kind", kind, "--object", object])
    def draw_wire(self, path: str, sheet: str, from_x_nm: int, from_y_nm: int, to_x_nm: int, to_y_nm: int) -> JsonRpcResponse: return self._run_cli_json(self.build_request("draw_wire", {"path": path, "sheet": sheet, "from_x_nm": from_x_nm, "from_y_nm": from_y_nm, "to_x_nm": to_x_nm, "to_y_nm": to_y_nm}), ["project", "draw-wire", path, "--sheet", sheet, "--from-x-nm", str(from_x_nm), "--from-y-nm", str(from_y_nm), "--to-x-nm", str(to_x_nm), "--to-y-nm", str(to_y_nm)])
    def delete_wire(self, path: str, wire: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_wire", {"path": path, "wire": wire}), ["project", "delete-wire", path, "--wire", wire])
    def place_junction(self, path: str, sheet: str, x_nm: int, y_nm: int) -> JsonRpcResponse: return self._run_cli_json(self.build_request("place_junction", {"path": path, "sheet": sheet, "x_nm": x_nm, "y_nm": y_nm}), ["project", "place-junction", path, "--sheet", sheet, "--x-nm", str(x_nm), "--y-nm", str(y_nm)])
    def delete_junction(self, path: str, junction: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_junction", {"path": path, "junction": junction}), ["project", "delete-junction", path, "--junction", junction])
    def place_noconnect(self, path: str, sheet: str, symbol: str, pin: str, x_nm: int, y_nm: int) -> JsonRpcResponse: return self._run_cli_json(self.build_request("place_noconnect", {"path": path, "sheet": sheet, "symbol": symbol, "pin": pin, "x_nm": x_nm, "y_nm": y_nm}), ["project", "place-noconnect", path, "--sheet", sheet, "--symbol", symbol, "--pin", pin, "--x-nm", str(x_nm), "--y-nm", str(y_nm)])
    def delete_noconnect(self, path: str, noconnect: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_noconnect", {"path": path, "noconnect": noconnect}), ["project", "delete-noconnect", path, "--noconnect", noconnect])
    def place_label(self, path: str, sheet: str, name: str, x_nm: int, y_nm: int, kind: str | None = None) -> JsonRpcResponse: args = ["project", "place-label", path, "--sheet", sheet, "--name", name, "--x-nm", str(x_nm), "--y-nm", str(y_nm)]; _append_optional(args, "kind", kind); return self._run_cli_json(self.build_request("place_label", {"path": path, "sheet": sheet, "name": name, "x_nm": x_nm, "y_nm": y_nm, "kind": kind}), args)
    def rename_label(self, path: str, label: str, name: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("rename_label", {"path": path, "label": label, "name": name}), ["project", "rename-label", path, "--label", label, "--name", name])
    def delete_label(self, path: str, label: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_label", {"path": path, "label": label}), ["project", "delete-label", path, "--label", label])
    def place_port(self, path: str, sheet: str, name: str, direction: str, x_nm: int, y_nm: int) -> JsonRpcResponse: return self._run_cli_json(self.build_request("place_port", {"path": path, "sheet": sheet, "name": name, "direction": direction, "x_nm": x_nm, "y_nm": y_nm}), ["project", "place-port", path, "--sheet", sheet, "--name", name, "--direction", direction, "--x-nm", str(x_nm), "--y-nm", str(y_nm)])
    def edit_port(self, path: str, port: str, name: str | None = None, direction: str | None = None, x_nm: int | None = None, y_nm: int | None = None) -> JsonRpcResponse: args = ["project", "edit-port", path, "--port", port]; _append_optional(args, "name", name); _append_optional(args, "direction", direction); _append_optional(args, "x-nm", x_nm); _append_optional(args, "y-nm", y_nm); return self._run_cli_json(self.build_request("edit_port", {"path": path, "port": port, "name": name, "direction": direction, "x_nm": x_nm, "y_nm": y_nm}), args)
    def delete_port(self, path: str, port: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_port", {"path": path, "port": port}), ["project", "delete-port", path, "--port", port])
    def create_bus(self, path: str, sheet: str, name: str, members: list[str]) -> JsonRpcResponse: args = ["project", "create-bus", path, "--sheet", sheet, "--name", name]; [args.extend(["--member", member]) for member in members]; return self._run_cli_json(self.build_request("create_bus", {"path": path, "sheet": sheet, "name": name, "members": members}), args)
    def edit_bus_members(self, path: str, bus: str, members: list[str]) -> JsonRpcResponse: args = ["project", "edit-bus-members", path, "--bus", bus]; [args.extend(["--member", member]) for member in members]; return self._run_cli_json(self.build_request("edit_bus_members", {"path": path, "bus": bus, "members": members}), args)
    def delete_bus(self, path: str, bus: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_bus", {"path": path, "bus": bus}), ["project", "delete-bus", path, "--bus", bus])
    def place_bus_entry(self, path: str, sheet: str, bus: str, x_nm: int, y_nm: int, wire: str | None = None) -> JsonRpcResponse: args = ["project", "place-bus-entry", path, "--sheet", sheet, "--bus", bus, "--x-nm", str(x_nm), "--y-nm", str(y_nm)]; _append_optional(args, "wire", wire); return self._run_cli_json(self.build_request("place_bus_entry", {"path": path, "sheet": sheet, "bus": bus, "x_nm": x_nm, "y_nm": y_nm, "wire": wire}), args)
    def delete_bus_entry(self, path: str, bus_entry: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_bus_entry", {"path": path, "bus_entry": bus_entry}), ["project", "delete-bus-entry", path, "--bus-entry", bus_entry])
    def place_schematic_text(self, path: str, sheet: str, text: str, x_nm: int, y_nm: int, rotation_deg: int | None = None) -> JsonRpcResponse: args = ["project", "place-text", path, "--sheet", sheet, "--text", text, "--x-nm", str(x_nm), "--y-nm", str(y_nm)]; _append_optional(args, "rotation-deg", rotation_deg); return self._run_cli_json(self.build_request("place_schematic_text", {"path": path, "sheet": sheet, "text": text, "x_nm": x_nm, "y_nm": y_nm, "rotation_deg": rotation_deg}), args)
    def edit_schematic_text(self, path: str, text: str, value: str | None = None, x_nm: int | None = None, y_nm: int | None = None, rotation_deg: int | None = None) -> JsonRpcResponse: args = ["project", "edit-text", path, "--text", text]; _append_optional(args, "value", value); _append_optional(args, "x-nm", x_nm); _append_optional(args, "y-nm", y_nm); _append_optional(args, "rotation-deg", rotation_deg); return self._run_cli_json(self.build_request("edit_schematic_text", {"path": path, "text": text, "value": value, "x_nm": x_nm, "y_nm": y_nm, "rotation_deg": rotation_deg}), args)
    def delete_schematic_text(self, path: str, text: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_schematic_text", {"path": path, "text": text}), ["project", "delete-text", path, "--text", text])
    def place_board_component(self, path: str, part: str, package: str, reference: str, value: str, x_nm: int, y_nm: int, layer: int) -> JsonRpcResponse: return self._run_cli_json(self.build_request("place_board_component", {"path": path, "part": part, "package": package, "reference": reference, "value": value, "x_nm": x_nm, "y_nm": y_nm, "layer": layer}), ["project", "place-board-component", path, "--part", part, "--package", package, "--reference", reference, "--value", value, "--x-nm", str(x_nm), "--y-nm", str(y_nm), "--layer", str(layer)])
    def move_board_component(self, path: str, component: str, x_nm: int, y_nm: int) -> JsonRpcResponse: return self._run_cli_json(self.build_request("move_board_component", {"path": path, "component": component, "x_nm": x_nm, "y_nm": y_nm}), ["project", "move-board-component", path, "--component", component, "--x-nm", str(x_nm), "--y-nm", str(y_nm)])
    def rotate_board_component(self, path: str, component: str, rotation_deg: int) -> JsonRpcResponse: return self._run_cli_json(self.build_request("rotate_board_component", {"path": path, "component": component, "rotation_deg": rotation_deg}), ["project", "rotate-board-component", path, "--component", component, "--rotation-deg", str(rotation_deg)])
    def flip_board_component(self, path: str, component: str, layer: int) -> JsonRpcResponse: return self._run_cli_json(self.build_request("flip_board_component", {"path": path, "component": component, "layer": layer}), ["project", "flip-board-component", path, "--component", component, "--layer", str(layer)])
    def delete_board_component(self, path: str, component: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_board_component", {"path": path, "component": component}), ["project", "delete-board-component", path, "--component", component])
    def set_board_component_reference(self, path: str, component: str, reference: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("set_board_component_reference", {"path": path, "component": component, "reference": reference}), ["project", "set-board-component-reference", path, "--component", component, "--reference", reference])
    def set_board_component_value(self, path: str, component: str, value: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("set_board_component_value", {"path": path, "component": component, "value": value}), ["project", "set-board-component-value", path, "--component", component, "--value", value])
    def set_board_component_part(self, path: str, component: str, part: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("set_board_component_part", {"path": path, "component": component, "part": part}), ["project", "set-board-component-part", path, "--component", component, "--part", part])
    def set_board_component_package(self, path: str, component: str, package: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("set_board_component_package", {"path": path, "component": component, "package": package}), ["project", "set-board-component-package", path, "--component", component, "--package", package])
    def lock_board_component(self, path: str, component: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("lock_board_component", {"path": path, "component": component}), ["project", "set-board-component-locked", path, "--component", component])
    def unlock_board_component(self, path: str, component: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("unlock_board_component", {"path": path, "component": component}), ["project", "clear-board-component-locked", path, "--component", component])
    def draw_board_track(self, path: str, net: str, from_x_nm: int, from_y_nm: int, to_x_nm: int, to_y_nm: int, width_nm: int, layer: int) -> JsonRpcResponse: return self._run_cli_json(self.build_request("draw_board_track", {"path": path, "net": net, "from_x_nm": from_x_nm, "from_y_nm": from_y_nm, "to_x_nm": to_x_nm, "to_y_nm": to_y_nm, "width_nm": width_nm, "layer": layer}), ["project", "draw-board-track", path, "--net", net, "--from-x-nm", str(from_x_nm), "--from-y-nm", str(from_y_nm), "--to-x-nm", str(to_x_nm), "--to-y-nm", str(to_y_nm), "--width-nm", str(width_nm), "--layer", str(layer)])
    def edit_board_track(self, path: str, track: str, net: str | None = None, from_x_nm: int | None = None, from_y_nm: int | None = None, to_x_nm: int | None = None, to_y_nm: int | None = None, width_nm: int | None = None, layer: int | None = None) -> JsonRpcResponse: args = ["project", "edit-board-track", path, "--track", track]; _append_optional(args, "net", net); _append_optional(args, "from-x-nm", from_x_nm); _append_optional(args, "from-y-nm", from_y_nm); _append_optional(args, "to-x-nm", to_x_nm); _append_optional(args, "to-y-nm", to_y_nm); _append_optional(args, "width-nm", width_nm); _append_optional(args, "layer", layer); return self._run_cli_json(self.build_request("edit_board_track", {"path": path, "track": track, "net": net, "from_x_nm": from_x_nm, "from_y_nm": from_y_nm, "to_x_nm": to_x_nm, "to_y_nm": to_y_nm, "width_nm": width_nm, "layer": layer}), args)
    def delete_board_track(self, path: str, track: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_board_track", {"path": path, "track": track}), ["project", "delete-board-track", path, "--track", track])
    def place_board_via(self, path: str, net: str, x_nm: int, y_nm: int, drill_nm: int, diameter_nm: int, from_layer: int, to_layer: int) -> JsonRpcResponse: return self._run_cli_json(self.build_request("place_board_via", {"path": path, "net": net, "x_nm": x_nm, "y_nm": y_nm, "drill_nm": drill_nm, "diameter_nm": diameter_nm, "from_layer": from_layer, "to_layer": to_layer}), ["project", "place-board-via", path, "--net", net, "--x-nm", str(x_nm), "--y-nm", str(y_nm), "--drill-nm", str(drill_nm), "--diameter-nm", str(diameter_nm), "--from-layer", str(from_layer), "--to-layer", str(to_layer)])
    def edit_board_via(self, path: str, via: str, net: str | None = None, x_nm: int | None = None, y_nm: int | None = None, drill_nm: int | None = None, diameter_nm: int | None = None, from_layer: int | None = None, to_layer: int | None = None) -> JsonRpcResponse: args = ["project", "edit-board-via", path, "--via", via]; _append_optional(args, "net", net); _append_optional(args, "x-nm", x_nm); _append_optional(args, "y-nm", y_nm); _append_optional(args, "drill-nm", drill_nm); _append_optional(args, "diameter-nm", diameter_nm); _append_optional(args, "from-layer", from_layer); _append_optional(args, "to-layer", to_layer); return self._run_cli_json(self.build_request("edit_board_via", {"path": path, "via": via, "net": net, "x_nm": x_nm, "y_nm": y_nm, "drill_nm": drill_nm, "diameter_nm": diameter_nm, "from_layer": from_layer, "to_layer": to_layer}), args)
    def delete_board_via(self, path: str, via: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_board_via", {"path": path, "via": via}), ["project", "delete-board-via", path, "--via", via])
    def place_board_zone(self, path: str, net: str, vertices: list[str], layer: int, thermal_gap_nm: int, thermal_spoke_width_nm: int, priority: int | None = None, thermal_relief: bool | None = None) -> JsonRpcResponse: args = ["project", "place-board-zone", path, "--net", net]; [args.extend(["--vertex", vertex]) for vertex in vertices]; args.extend(["--layer", str(layer)]); _append_optional(args, "priority", priority); _append_optional_bool(args, "thermal-relief", thermal_relief); args.extend(["--thermal-gap-nm", str(thermal_gap_nm), "--thermal-spoke-width-nm", str(thermal_spoke_width_nm)]); return self._run_cli_json(self.build_request("place_board_zone", {"path": path, "net": net, "vertices": vertices, "layer": layer, "priority": priority, "thermal_relief": thermal_relief, "thermal_gap_nm": thermal_gap_nm, "thermal_spoke_width_nm": thermal_spoke_width_nm}), args)
    def edit_board_zone(self, path: str, zone: str, net: str | None = None, vertices: list[str] | None = None, layer: int | None = None, priority: int | None = None, thermal_relief: bool | None = None, thermal_gap_nm: int | None = None, thermal_spoke_width_nm: int | None = None) -> JsonRpcResponse: args = ["project", "edit-board-zone", path, "--zone", zone]; _append_optional(args, "net", net); [args.extend(["--vertex", vertex]) for vertex in (vertices or [])]; _append_optional(args, "layer", layer); _append_optional(args, "priority", priority); _append_optional_bool(args, "thermal-relief", thermal_relief); _append_optional(args, "thermal-gap-nm", thermal_gap_nm); _append_optional(args, "thermal-spoke-width-nm", thermal_spoke_width_nm); return self._run_cli_json(self.build_request("edit_board_zone", {"path": path, "zone": zone, "net": net, "vertices": vertices, "layer": layer, "priority": priority, "thermal_relief": thermal_relief, "thermal_gap_nm": thermal_gap_nm, "thermal_spoke_width_nm": thermal_spoke_width_nm}), args)
    def delete_board_zone(self, path: str, zone: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_board_zone", {"path": path, "zone": zone}), ["project", "delete-board-zone", path, "--zone", zone])
    def place_board_pad(self, path: str, package: str, name: str, x_nm: int, y_nm: int, layer: int, shape: str | None = None, diameter_nm: int | None = None, width_nm: int | None = None, height_nm: int | None = None, net: str | None = None) -> JsonRpcResponse: args = ["project", "place-board-pad", path, "--package", package, "--name", name, "--x-nm", str(x_nm), "--y-nm", str(y_nm), "--layer", str(layer)]; _append_optional(args, "shape", shape); _append_optional(args, "diameter-nm", diameter_nm); _append_optional(args, "width-nm", width_nm); _append_optional(args, "height-nm", height_nm); _append_optional(args, "net", net); return self._run_cli_json(self.build_request("place_board_pad", {"path": path, "package": package, "name": name, "x_nm": x_nm, "y_nm": y_nm, "layer": layer, "shape": shape, "diameter_nm": diameter_nm, "width_nm": width_nm, "height_nm": height_nm, "net": net}), args)
    def edit_board_pad(self, path: str, pad: str, x_nm: int | None = None, y_nm: int | None = None, layer: int | None = None, shape: str | None = None, diameter_nm: int | None = None, width_nm: int | None = None, height_nm: int | None = None) -> JsonRpcResponse: args = ["project", "edit-board-pad", path, "--pad", pad]; _append_optional(args, "x-nm", x_nm); _append_optional(args, "y-nm", y_nm); _append_optional(args, "layer", layer); _append_optional(args, "shape", shape); _append_optional(args, "diameter-nm", diameter_nm); _append_optional(args, "width-nm", width_nm); _append_optional(args, "height-nm", height_nm); return self._run_cli_json(self.build_request("edit_board_pad", {"path": path, "pad": pad, "x_nm": x_nm, "y_nm": y_nm, "layer": layer, "shape": shape, "diameter_nm": diameter_nm, "width_nm": width_nm, "height_nm": height_nm}), args)
    def delete_board_pad(self, path: str, pad: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_board_pad", {"path": path, "pad": pad}), ["project", "delete-board-pad", path, "--pad", pad])
    def set_board_pad_net(self, path: str, pad: str, net: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("set_board_pad_net", {"path": path, "pad": pad, "net": net}), ["project", "set-board-pad-net", path, "--pad", pad, "--net", net])
    def clear_board_pad_net(self, path: str, pad: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("clear_board_pad_net", {"path": path, "pad": pad}), ["project", "clear-board-pad-net", path, "--pad", pad])
    def place_board_net(self, path: str, name: str, class_uuid: str, impedance_target_ohms: str | None = None, impedance_tolerance_pct: str | None = None, controlled_dielectric_layer: int | None = None) -> JsonRpcResponse: args = ["project", "place-board-net", path, "--name", name, "--class", class_uuid]; _append_optional(args, "impedance-target-ohms", impedance_target_ohms); _append_optional(args, "impedance-tolerance-pct", impedance_tolerance_pct); _append_optional(args, "controlled-dielectric-layer", controlled_dielectric_layer); return self._run_cli_json(self.build_request("place_board_net", {"path": path, "name": name, "class": class_uuid, "impedance_target_ohms": impedance_target_ohms, "impedance_tolerance_pct": impedance_tolerance_pct, "controlled_dielectric_layer": controlled_dielectric_layer}), args)
    def edit_board_net(self, path: str, net: str, name: str | None = None, class_uuid: str | None = None, impedance_target_ohms: str | None = None, impedance_tolerance_pct: str | None = None, controlled_dielectric_layer: int | None = None, clear_controlled_impedance: bool | None = None) -> JsonRpcResponse: args = ["project", "edit-board-net", path, "--net", net]; _append_optional(args, "name", name); _append_optional(args, "class", class_uuid); _append_optional(args, "impedance-target-ohms", impedance_target_ohms); _append_optional(args, "impedance-tolerance-pct", impedance_tolerance_pct); _append_optional(args, "controlled-dielectric-layer", controlled_dielectric_layer); args.extend(["--clear-controlled-impedance"] if clear_controlled_impedance else []); return self._run_cli_json(self.build_request("edit_board_net", {"path": path, "net": net, "name": name, "class": class_uuid, "impedance_target_ohms": impedance_target_ohms, "impedance_tolerance_pct": impedance_tolerance_pct, "controlled_dielectric_layer": controlled_dielectric_layer, "clear_controlled_impedance": bool(clear_controlled_impedance)}), args)
    def delete_board_net(self, path: str, net: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_board_net", {"path": path, "net": net}), ["project", "delete-board-net", path, "--net", net])
    def set_board_name(self, path: str, name: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("set_board_name", {"path": path, "name": name}), ["project", "set-board-name", path, "--name", name])
    def set_board_outline(self, path: str, vertices: list[str]) -> JsonRpcResponse: args = ["project", "set-board-outline", path]; [args.extend(["--vertex", vertex]) for vertex in vertices]; return self._run_cli_json(self.build_request("set_board_outline", {"path": path, "vertices": vertices}), args)
    def set_board_stackup(self, path: str, layers: list[str]) -> JsonRpcResponse: args = ["project", "set-board-stackup", path]; [args.extend(["--layer", layer]) for layer in layers]; return self._run_cli_json(self.build_request("set_board_stackup", {"path": path, "layers": layers}), args)
    def add_default_top_stackup(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("add_default_top_stackup", {"path": path}), ["project", "add-default-top-stackup", path])
    def place_board_keepout(self, path: str, vertices: list[str], layers: list[int], kind: str) -> JsonRpcResponse: args = ["project", "place-board-keepout", path, "--kind", kind]; [args.extend(["--vertex", vertex]) for vertex in vertices]; [args.extend(["--layer", str(layer)]) for layer in layers]; return self._run_cli_json(self.build_request("place_board_keepout", {"path": path, "vertices": vertices, "layers": layers, "kind": kind}), args)
    def edit_board_keepout(self, path: str, keepout: str, vertices: list[str] | None = None, layers: list[int] | None = None, kind: str | None = None) -> JsonRpcResponse: args = ["project", "edit-board-keepout", path, "--keepout", keepout]; [args.extend(["--vertex", vertex]) for vertex in vertices or []]; [args.extend(["--layer", str(layer)]) for layer in layers or []]; _append_optional(args, "kind", kind); return self._run_cli_json(self.build_request("edit_board_keepout", {"path": path, "keepout": keepout, "vertices": vertices or [], "layers": layers or [], "kind": kind}), args)
    def delete_board_keepout(self, path: str, keepout: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_board_keepout", {"path": path, "keepout": keepout}), ["project", "delete-board-keepout", path, "--keepout", keepout])
    def place_board_dimension(self, path: str, from_x_nm: int, from_y_nm: int, to_x_nm: int, to_y_nm: int, layer: int, text: str | None = None) -> JsonRpcResponse: args = ["project", "place-board-dimension", path, "--from-x-nm", str(from_x_nm), "--from-y-nm", str(from_y_nm), "--to-x-nm", str(to_x_nm), "--to-y-nm", str(to_y_nm), "--layer", str(layer)]; _append_optional(args, "text", text); return self._run_cli_json(self.build_request("place_board_dimension", {"path": path, "from_x_nm": from_x_nm, "from_y_nm": from_y_nm, "to_x_nm": to_x_nm, "to_y_nm": to_y_nm, "layer": layer, "text": text}), args)
    def edit_board_dimension(self, path: str, dimension: str, from_x_nm: int | None = None, from_y_nm: int | None = None, to_x_nm: int | None = None, to_y_nm: int | None = None, layer: int | None = None, text: str | None = None, clear_text: bool | None = None) -> JsonRpcResponse: args = ["project", "edit-board-dimension", path, "--dimension", dimension]; _append_optional(args, "from-x-nm", from_x_nm); _append_optional(args, "from-y-nm", from_y_nm); _append_optional(args, "to-x-nm", to_x_nm); _append_optional(args, "to-y-nm", to_y_nm); _append_optional(args, "layer", layer); _append_optional(args, "text", text); args.extend(["--clear-text"] if clear_text else []); return self._run_cli_json(self.build_request("edit_board_dimension", {"path": path, "dimension": dimension, "from_x_nm": from_x_nm, "from_y_nm": from_y_nm, "to_x_nm": to_x_nm, "to_y_nm": to_y_nm, "layer": layer, "text": text, "clear_text": bool(clear_text)}), args)
    def delete_board_dimension(self, path: str, dimension: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_board_dimension", {"path": path, "dimension": dimension}), ["project", "delete-board-dimension", path, "--dimension", dimension])
    def place_board_text(self, path: str, text: str, x_nm: int, y_nm: int, layer: int, rotation_deg: int | None = None, height_nm: int | None = None, stroke_width_nm: int | None = None, render_intent: str | None = None, family: str | None = None, style: str | None = None, style_class: str | None = None, h_align: str | None = None, v_align: str | None = None, mirrored: bool | None = None, keep_upright: bool | None = None, line_spacing_ratio_ppm: int | None = None, bold: bool | None = None, italic: bool | None = None) -> JsonRpcResponse: args = ["project", "place-board-text", path, "--text", text, "--x-nm", str(x_nm), "--y-nm", str(y_nm), "--layer", str(layer)]; _append_optional(args, "rotation-deg", rotation_deg); _append_optional(args, "height-nm", height_nm); _append_optional(args, "stroke-width-nm", stroke_width_nm); _append_optional(args, "render-intent", render_intent); _append_optional(args, "family", family); _append_optional(args, "style", style); _append_optional(args, "style-class", style_class); _append_optional(args, "h-align", h_align); _append_optional(args, "v-align", v_align); args.extend(["--mirrored"] if mirrored else []); args.extend(["--keep-upright"] if keep_upright else []); _append_optional(args, "line-spacing-ratio-ppm", line_spacing_ratio_ppm); args.extend(["--bold"] if bold else []); args.extend(["--italic"] if italic else []); return self._run_cli_json(self.build_request("place_board_text", {"path": path, "text": text, "x_nm": x_nm, "y_nm": y_nm, "layer": layer, "rotation_deg": rotation_deg, "height_nm": height_nm, "stroke_width_nm": stroke_width_nm, "render_intent": render_intent, "family": family, "style": style, "style_class": style_class, "h_align": h_align, "v_align": v_align, "mirrored": mirrored, "keep_upright": keep_upright, "line_spacing_ratio_ppm": line_spacing_ratio_ppm, "bold": bold, "italic": italic}), args)
    def edit_board_text(self, path: str, text: str, value: str | None = None, x_nm: int | None = None, y_nm: int | None = None, layer: int | None = None, rotation_deg: int | None = None, height_nm: int | None = None, stroke_width_nm: int | None = None, render_intent: str | None = None, family: str | None = None, style: str | None = None, style_class: str | None = None, h_align: str | None = None, v_align: str | None = None, mirrored: bool | None = None, keep_upright: bool | None = None, line_spacing_ratio_ppm: int | None = None, bold: bool | None = None, italic: bool | None = None) -> JsonRpcResponse: args = ["project", "edit-board-text", path, "--text", text]; _append_optional(args, "value", value); _append_optional(args, "x-nm", x_nm); _append_optional(args, "y-nm", y_nm); _append_optional(args, "layer", layer); _append_optional(args, "rotation-deg", rotation_deg); _append_optional(args, "height-nm", height_nm); _append_optional(args, "stroke-width-nm", stroke_width_nm); _append_optional(args, "render-intent", render_intent); _append_optional(args, "family", family); _append_optional(args, "style", style); _append_optional(args, "style-class", style_class); _append_optional(args, "h-align", h_align); _append_optional(args, "v-align", v_align); _append_optional_bool(args, "mirrored", mirrored); _append_optional_bool(args, "keep-upright", keep_upright); _append_optional(args, "line-spacing-ratio-ppm", line_spacing_ratio_ppm); _append_optional_bool(args, "bold", bold); _append_optional_bool(args, "italic", italic); return self._run_cli_json(self.build_request("edit_board_text", {"path": path, "text": text, "value": value, "x_nm": x_nm, "y_nm": y_nm, "layer": layer, "rotation_deg": rotation_deg, "height_nm": height_nm, "stroke_width_nm": stroke_width_nm, "render_intent": render_intent, "family": family, "style": style, "style_class": style_class, "h_align": h_align, "v_align": v_align, "mirrored": mirrored, "keep_upright": keep_upright, "line_spacing_ratio_ppm": line_spacing_ratio_ppm, "bold": bold, "italic": italic}), args)
    def delete_board_text(self, path: str, text: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_board_text", {"path": path, "text": text}), ["project", "delete-board-text", path, "--text", text])
    def place_board_net_class(self, path: str, name: str, clearance_nm: int, track_width_nm: int, via_drill_nm: int, via_diameter_nm: int, diffpair_width_nm: int | None = None, diffpair_gap_nm: int | None = None) -> JsonRpcResponse: args = ["project", "place-board-net-class", path, "--name", name, "--clearance-nm", str(clearance_nm), "--track-width-nm", str(track_width_nm), "--via-drill-nm", str(via_drill_nm), "--via-diameter-nm", str(via_diameter_nm)]; _append_optional(args, "diffpair-width-nm", diffpair_width_nm); _append_optional(args, "diffpair-gap-nm", diffpair_gap_nm); return self._run_cli_json(self.build_request("place_board_net_class", {"path": path, "name": name, "clearance_nm": clearance_nm, "track_width_nm": track_width_nm, "via_drill_nm": via_drill_nm, "via_diameter_nm": via_diameter_nm, "diffpair_width_nm": diffpair_width_nm, "diffpair_gap_nm": diffpair_gap_nm}), args)
    def edit_board_net_class(self, path: str, net_class: str, name: str | None = None, clearance_nm: int | None = None, track_width_nm: int | None = None, via_drill_nm: int | None = None, via_diameter_nm: int | None = None, diffpair_width_nm: int | None = None, diffpair_gap_nm: int | None = None) -> JsonRpcResponse: args = ["project", "edit-board-net-class", path, "--net-class", net_class]; _append_optional(args, "name", name); _append_optional(args, "clearance-nm", clearance_nm); _append_optional(args, "track-width-nm", track_width_nm); _append_optional(args, "via-drill-nm", via_drill_nm); _append_optional(args, "via-diameter-nm", via_diameter_nm); _append_optional(args, "diffpair-width-nm", diffpair_width_nm); _append_optional(args, "diffpair-gap-nm", diffpair_gap_nm); return self._run_cli_json(self.build_request("edit_board_net_class", {"path": path, "net_class": net_class, "name": name, "clearance_nm": clearance_nm, "track_width_nm": track_width_nm, "via_drill_nm": via_drill_nm, "via_diameter_nm": via_diameter_nm, "diffpair_width_nm": diffpair_width_nm, "diffpair_gap_nm": diffpair_gap_nm}), args)
    def delete_board_net_class(self, path: str, net_class: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_board_net_class", {"path": path, "net_class": net_class}), ["project", "delete-board-net-class", path, "--net-class", net_class])
    def get_relationships(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_relationships", {"path": path}), ["query", "relationships", path])
    def get_variants(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_variants", {"path": path}), ["query", "variants", path])
    def get_import_map(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_import_map", {"path": path}), ["query", "import-map", path])
    def get_source_shards(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_source_shards", {"path": path}), ["project", "query", path, "resolve-debug"])
    def create_proposal(self, path: str, batch: str, rationale: str, proposal: str | None = None, source: str | None = None, checks_run: list[str] | None = None, finding_fingerprints: list[str] | None = None) -> JsonRpcResponse:
        args = ["proposal", "create", path, "--batch", batch, "--rationale", rationale]
        if proposal: args.extend(["--proposal", proposal])
        if source: args.extend(["--source", source])
        for check_run in checks_run or []: args.extend(["--check-run", check_run])
        for fingerprint in finding_fingerprints or []: args.extend(["--finding-fingerprint", fingerprint])
        return self._run_cli_json(self.build_request("create_proposal", {"path": path, "batch": batch, "rationale": rationale, "proposal": proposal, "source": source, "checks_run": checks_run or [], "finding_fingerprints": finding_fingerprints or []}), args)
    def get_proposals(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_proposals", {"path": path}), ["proposal", "list", path])
    def show_proposal(self, path: str, proposal: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("show_proposal", {"path": path, "proposal": proposal}), ["proposal", "show", path, "--proposal", proposal])
    def preview_proposal(self, path: str, proposal: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("preview_proposal", {"path": path, "proposal": proposal}), ["proposal", "preview", path, "--proposal", proposal])
    def validate_proposal(self, path: str, proposal: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("validate_proposal", {"path": path, "proposal": proposal}), ["proposal", "validate", path, "--proposal", proposal])
    def defer_proposal(self, path: str, proposal: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("defer_proposal", {"path": path, "proposal": proposal}), ["proposal", "defer", path, "--proposal", proposal])
    def reject_proposal(self, path: str, proposal: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("reject_proposal", {"path": path, "proposal": proposal}), ["proposal", "reject", path, "--proposal", proposal])
    def review_proposal(self, path: str, proposal: str, status: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("review_proposal", {"path": path, "proposal": proposal, "status": status}), ["proposal", "review", path, "--proposal", proposal, "--status", status])
    def accept_apply_proposal(self, path: str, proposal: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("accept_apply_proposal", {"path": path, "proposal": proposal}), ["proposal", "accept-apply", path, "--proposal", proposal])
    def apply_proposal(self, path: str, proposal: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("apply_proposal", {"path": path, "proposal": proposal}), ["proposal", "apply", path, "--proposal", proposal])
    def get_check_run(self, path: str, profile: str | None = None) -> JsonRpcResponse: args = ["check", "run", path]; _append_optional(args, "profile", profile); return self._run_cli_json(self.build_request("get_check_run", {"path": path, "profile": profile}), args)
    def get_check_runs(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_check_runs", {"path": path}), ["check", "list", path])
    def show_check_run(self, path: str, check_run: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("show_check_run", {"path": path, "check_run": check_run}), ["check", "show", path, "--check-run", check_run])
    def get_check_profiles(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_check_profiles", {"path": path}), ["check", "profiles", path])
    def get_zone_fills(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_zone_fills", {"path": path}), ["query", "zone-fills", path])
    def fill_zones(self, path: str, zone: str | None = None, net: str | None = None) -> JsonRpcResponse:
        args = ["check", "fill-zones", path]; _append_optional(args, "zone", zone); _append_optional(args, "net", net); return self._run_cli_json(self.build_request("fill_zones", {"path": path, "zone": zone, "net": net}), args)
    def generate_standards_repair_proposals(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("generate_standards_repair_proposals", {"path": path}), ["check", "repair-standards", path])
    def waive_finding(self, path: str, fingerprint: str, rationale: str, created_by: str | None = None) -> JsonRpcResponse:
        args = ["check", "waive", path, "--fingerprint", fingerprint, "--rationale", rationale]; _append_optional(args, "created-by", created_by); return self._run_cli_json(self.build_request("waive_finding", {"path": path, "fingerprint": fingerprint, "rationale": rationale, "created_by": created_by}), args)
    def accept_deviation(self, path: str, fingerprint: str, rationale: str, accepted_by: str | None = None) -> JsonRpcResponse:
        args = ["check", "accept-deviation", path, "--fingerprint", fingerprint, "--rationale", rationale]; _append_optional(args, "accepted-by", accepted_by); return self._run_cli_json(self.build_request("accept_deviation", {"path": path, "fingerprint": fingerprint, "rationale": rationale, "accepted_by": accepted_by}), args)
    def get_journal_list(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_journal_list", {"path": path}), ["journal", "list", path])
    def get_journal_transaction(self, path: str, transaction: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_journal_transaction", {"path": path, "transaction": transaction}), ["journal", "show", path, "--transaction", transaction])
    def get_schematic_wires(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_schematic_wires", {"path": path}), ["project", "query", path, "wires"])
    def get_schematic_junctions(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_schematic_junctions", {"path": path}), ["project", "query", path, "junctions"])
    def get_schematic_labels(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_schematic_labels", {"path": path}), ["project", "query", path, "labels"])
    def get_board_tracks(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_board_tracks", {"path": path}), ["project", "query", path, "board-tracks"])
    def get_board_vias(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_board_vias", {"path": path}), ["project", "query", path, "board-vias"])
    def get_board_pads(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_board_pads", {"path": path}), ["project", "query", path, "board-pads"])
    def journal_undo(self, path: str, expected_model_revision: str | None = None, expected_tip_transaction: str | None = None) -> JsonRpcResponse: args = ["journal", "undo", path]; _append_optional(args, "expected-model-revision", expected_model_revision); _append_optional(args, "expected-tip-transaction", expected_tip_transaction); return self._run_cli_json(self.build_request("journal_undo", {"path": path, "expected_model_revision": expected_model_revision, "expected_tip_transaction": expected_tip_transaction}), args)
    def journal_redo(self, path: str, expected_model_revision: str | None = None, expected_tip_transaction: str | None = None) -> JsonRpcResponse: args = ["journal", "redo", path]; _append_optional(args, "expected-model-revision", expected_model_revision); _append_optional(args, "expected-tip-transaction", expected_tip_transaction); return self._run_cli_json(self.build_request("journal_redo", {"path": path, "expected_model_revision": expected_model_revision, "expected_tip_transaction": expected_tip_transaction}), args)
    def get_manufacturing_plans(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_manufacturing_plans", {"path": path}), ["query", "manufacturing-plans", path])
    def get_panel_projections(self, path: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("get_panel_projections", {"path": path}), ["query", "panel-projections", path])
    def create_manufacturing_plan(self, path: str, prefix: str, name: str | None = None, variant: str | None = None, panel_projection: str | None = None) -> JsonRpcResponse:
        args = ["project", "create-manufacturing-plan", path, "--prefix", prefix]; _append_optional(args, "name", name); _append_optional(args, "variant", variant); _append_optional(args, "panel-projection", panel_projection); return self._run_cli_json(self.build_request("create_manufacturing_plan", {"path": path, "prefix": prefix, "name": name, "variant": variant, "panel_projection": panel_projection}), args)
    def create_manufacturing_plan_proposal(self, path: str, prefix: str, name: str | None = None, variant: str | None = None, panel_projection: str | None = None, proposal: str | None = None, rationale: str | None = None) -> JsonRpcResponse:
        args = ["proposal", "create-manufacturing-plan", path, "--prefix", prefix]; _append_optional(args, "name", name); _append_optional(args, "variant", variant); _append_optional(args, "panel-projection", panel_projection); _append_optional(args, "proposal", proposal); _append_optional(args, "rationale", rationale); return self._run_cli_json(self.build_request("create_manufacturing_plan_proposal", {"path": path, "prefix": prefix, "name": name, "variant": variant, "panel_projection": panel_projection, "proposal": proposal, "rationale": rationale}), args)
    def update_manufacturing_plan(self, path: str, manufacturing_plan: str, name: str | None = None, prefix: str | None = None, variant: str | None = None, clear_variant: bool | None = None, panel_projection: str | None = None, clear_panel_projection: bool | None = None) -> JsonRpcResponse:
        args = ["project", "update-manufacturing-plan", path, "--manufacturing-plan", manufacturing_plan]; _append_optional(args, "name", name); _append_optional(args, "prefix", prefix); _append_optional(args, "variant", variant); args.extend(["--clear-variant"] if clear_variant else []); _append_optional(args, "panel-projection", panel_projection); args.extend(["--clear-panel-projection"] if clear_panel_projection else []); return self._run_cli_json(self.build_request("update_manufacturing_plan", {"path": path, "manufacturing_plan": manufacturing_plan, "name": name, "prefix": prefix, "variant": variant, "clear_variant": clear_variant, "panel_projection": panel_projection, "clear_panel_projection": clear_panel_projection}), args)
    def update_manufacturing_plan_proposal(self, path: str, manufacturing_plan: str, name: str | None = None, prefix: str | None = None, variant: str | None = None, clear_variant: bool | None = None, panel_projection: str | None = None, clear_panel_projection: bool | None = None, proposal: str | None = None, rationale: str | None = None) -> JsonRpcResponse:
        args = ["proposal", "update-manufacturing-plan", path, "--manufacturing-plan", manufacturing_plan]; _append_optional(args, "name", name); _append_optional(args, "prefix", prefix); _append_optional(args, "variant", variant); args.extend(["--clear-variant"] if clear_variant else []); _append_optional(args, "panel-projection", panel_projection); args.extend(["--clear-panel-projection"] if clear_panel_projection else []); _append_optional(args, "proposal", proposal); _append_optional(args, "rationale", rationale); return self._run_cli_json(self.build_request("update_manufacturing_plan_proposal", {"path": path, "manufacturing_plan": manufacturing_plan, "name": name, "prefix": prefix, "variant": variant, "clear_variant": clear_variant, "panel_projection": panel_projection, "clear_panel_projection": clear_panel_projection, "proposal": proposal, "rationale": rationale}), args)
    def delete_manufacturing_plan(self, path: str, manufacturing_plan: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_manufacturing_plan", {"path": path, "manufacturing_plan": manufacturing_plan}), ["project", "delete-manufacturing-plan", path, "--manufacturing-plan", manufacturing_plan])
    def delete_manufacturing_plan_proposal(self, path: str, manufacturing_plan: str, proposal: str | None = None, rationale: str | None = None) -> JsonRpcResponse:
        args = ["proposal", "delete-manufacturing-plan", path, "--manufacturing-plan", manufacturing_plan]; _append_optional(args, "proposal", proposal); _append_optional(args, "rationale", rationale); return self._run_cli_json(self.build_request("delete_manufacturing_plan_proposal", {"path": path, "manufacturing_plan": manufacturing_plan, "proposal": proposal, "rationale": rationale}), args)
    def create_panel_projection(self, path: str, key: str, name: str | None = None, board: str | None = None, x_nm: int | None = None, y_nm: int | None = None, rotation_deg: int | None = None) -> JsonRpcResponse:
        args = ["project", "create-panel-projection", path, "--key", key]; _append_optional(args, "name", name); _append_optional(args, "board", board); _append_optional(args, "x-nm", x_nm); _append_optional(args, "y-nm", y_nm); _append_optional(args, "rotation-deg", rotation_deg); return self._run_cli_json(self.build_request("create_panel_projection", {"path": path, "key": key, "name": name, "board": board, "x_nm": x_nm, "y_nm": y_nm, "rotation_deg": rotation_deg}), args)
    def create_panel_projection_proposal(self, path: str, key: str, name: str | None = None, board: str | None = None, x_nm: int | None = None, y_nm: int | None = None, rotation_deg: int | None = None, proposal: str | None = None, rationale: str | None = None) -> JsonRpcResponse:
        args = ["proposal", "create-panel-projection", path, "--key", key]; _append_optional(args, "name", name); _append_optional(args, "board", board); _append_optional(args, "x-nm", x_nm); _append_optional(args, "y-nm", y_nm); _append_optional(args, "rotation-deg", rotation_deg); _append_optional(args, "proposal", proposal); _append_optional(args, "rationale", rationale); return self._run_cli_json(self.build_request("create_panel_projection_proposal", {"path": path, "key": key, "name": name, "board": board, "x_nm": x_nm, "y_nm": y_nm, "rotation_deg": rotation_deg, "proposal": proposal, "rationale": rationale}), args)
    def update_panel_projection(self, path: str, panel_projection: str, name: str | None = None, board: str | None = None, x_nm: int | None = None, y_nm: int | None = None, rotation_deg: int | None = None) -> JsonRpcResponse:
        args = ["project", "update-panel-projection", path, "--panel-projection", panel_projection]; _append_optional(args, "name", name); _append_optional(args, "board", board); _append_optional(args, "x-nm", x_nm); _append_optional(args, "y-nm", y_nm); _append_optional(args, "rotation-deg", rotation_deg); return self._run_cli_json(self.build_request("update_panel_projection", {"path": path, "panel_projection": panel_projection, "name": name, "board": board, "x_nm": x_nm, "y_nm": y_nm, "rotation_deg": rotation_deg}), args)
    def update_panel_projection_proposal(self, path: str, panel_projection: str, name: str | None = None, board: str | None = None, x_nm: int | None = None, y_nm: int | None = None, rotation_deg: int | None = None, proposal: str | None = None, rationale: str | None = None) -> JsonRpcResponse:
        args = ["proposal", "update-panel-projection", path, "--panel-projection", panel_projection]; _append_optional(args, "name", name); _append_optional(args, "board", board); _append_optional(args, "x-nm", x_nm); _append_optional(args, "y-nm", y_nm); _append_optional(args, "rotation-deg", rotation_deg); _append_optional(args, "proposal", proposal); _append_optional(args, "rationale", rationale); return self._run_cli_json(self.build_request("update_panel_projection_proposal", {"path": path, "panel_projection": panel_projection, "name": name, "board": board, "x_nm": x_nm, "y_nm": y_nm, "rotation_deg": rotation_deg, "proposal": proposal, "rationale": rationale}), args)
    def delete_panel_projection(self, path: str, panel_projection: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_panel_projection", {"path": path, "panel_projection": panel_projection}), ["project", "delete-panel-projection", path, "--panel-projection", panel_projection])
    def delete_panel_projection_proposal(self, path: str, panel_projection: str, proposal: str | None = None, rationale: str | None = None) -> JsonRpcResponse:
        args = ["proposal", "delete-panel-projection", path, "--panel-projection", panel_projection]; _append_optional(args, "proposal", proposal); _append_optional(args, "rationale", rationale); return self._run_cli_json(self.build_request("delete_panel_projection_proposal", {"path": path, "panel_projection": panel_projection, "proposal": proposal, "rationale": rationale}), args)
    def create_gerber_output_job(self, path: str, prefix: str, name: str | None = None, manufacturing_plan: str | None = None, output_dir: str | None = None, variant: str | None = None) -> JsonRpcResponse: args = ["project", "create-gerber-output-job", path, "--prefix", prefix]; _append_optional(args, "output-dir", output_dir); _append_optional(args, "name", name); _append_optional(args, "manufacturing-plan", manufacturing_plan); _append_optional(args, "variant", variant); return self._run_cli_json(self.build_request("create_gerber_output_job", {"path": path, "prefix": prefix, "name": name, "manufacturing_plan": manufacturing_plan, "variant": variant, "output_dir": output_dir}), args)
    def create_output_job(self, path: str, prefix: str, include: str, name: str | None = None, manufacturing_plan: str | None = None, output_dir: str | None = None, variant: str | None = None) -> JsonRpcResponse: args = ["project", "create-output-job", path, "--prefix", prefix, "--include", include]; _append_optional(args, "output-dir", output_dir); _append_optional(args, "name", name); _append_optional(args, "manufacturing-plan", manufacturing_plan); _append_optional(args, "variant", variant); return self._run_cli_json(self.build_request("create_output_job", {"path": path, "prefix": prefix, "include": include, "name": name, "manufacturing_plan": manufacturing_plan, "variant": variant, "output_dir": output_dir}), args)
    def create_output_job_proposal(self, path: str, prefix: str, include: str, name: str | None = None, manufacturing_plan: str | None = None, output_dir: str | None = None, proposal: str | None = None, rationale: str | None = None, variant: str | None = None) -> JsonRpcResponse:
        args = ["proposal", "create-output-job", path, "--prefix", prefix, "--include", include]
        _append_optional(args, "output-dir", output_dir)
        _append_optional(args, "name", name)
        _append_optional(args, "manufacturing-plan", manufacturing_plan); _append_optional(args, "variant", variant); _append_optional(args, "proposal", proposal)
        _append_optional(args, "rationale", rationale)
        return self._run_cli_json(self.build_request("create_output_job_proposal", {"path": path, "prefix": prefix, "include": include, "name": name, "manufacturing_plan": manufacturing_plan, "variant": variant, "output_dir": output_dir, "proposal": proposal, "rationale": rationale}), args)
    def update_output_job(self, path: str, output_job: str, name: str | None = None, output_dir: str | None = None, manufacturing_plan: str | None = None, clear_manufacturing_plan: bool | None = None, clear_output_dir: bool | None = None, variant: str | None = None, clear_variant: bool | None = None) -> JsonRpcResponse: args = ["project", "update-output-job", path, "--output-job", output_job]; _append_optional(args, "name", name); _append_optional(args, "output-dir", output_dir); _append_optional(args, "manufacturing-plan", manufacturing_plan); _append_optional(args, "variant", variant); args.extend(["--clear-manufacturing-plan"] if clear_manufacturing_plan else []); args.extend(["--clear-variant"] if clear_variant else []); args.extend(["--clear-output-dir"] if clear_output_dir else []); return self._run_cli_json(self.build_request("update_output_job", {"path": path, "output_job": output_job, "name": name, "output_dir": output_dir, "manufacturing_plan": manufacturing_plan, "variant": variant, "clear_manufacturing_plan": clear_manufacturing_plan, "clear_variant": clear_variant, "clear_output_dir": clear_output_dir}), args)
    def update_output_job_proposal(self, path: str, output_job: str, name: str | None = None, output_dir: str | None = None, manufacturing_plan: str | None = None, clear_manufacturing_plan: bool | None = None, clear_output_dir: bool | None = None, proposal: str | None = None, rationale: str | None = None, variant: str | None = None, clear_variant: bool | None = None) -> JsonRpcResponse:
        args = ["proposal", "update-output-job", path, "--output-job", output_job]
        _append_optional(args, "name", name)
        _append_optional(args, "output-dir", output_dir)
        _append_optional(args, "manufacturing-plan", manufacturing_plan); _append_optional(args, "variant", variant)
        args.extend(["--clear-manufacturing-plan"] if clear_manufacturing_plan else []); args.extend(["--clear-variant"] if clear_variant else [])
        args.extend(["--clear-output-dir"] if clear_output_dir else [])
        _append_optional(args, "proposal", proposal)
        _append_optional(args, "rationale", rationale)
        return self._run_cli_json(self.build_request("update_output_job_proposal", {"path": path, "output_job": output_job, "name": name, "output_dir": output_dir, "manufacturing_plan": manufacturing_plan, "variant": variant, "clear_manufacturing_plan": clear_manufacturing_plan, "clear_variant": clear_variant, "clear_output_dir": clear_output_dir, "proposal": proposal, "rationale": rationale}), args)
    def run_output_job(self, path: str, output_job: str, output_dir: str | None = None) -> JsonRpcResponse: args = ["artifact", "generate", path, "--output-job", output_job]; _append_optional(args, "output-dir", output_dir); return self._run_cli_json_allowing_statuses(self.build_request("run_output_job", {"path": path, "output_job": output_job, "output_dir": output_dir}), args, {0, 1})
    def start_output_job_run(self, path: str, output_job: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("start_output_job_run", {"path": path, "output_job": output_job}), ["artifact", "start-output-job-run", path, "--output-job", output_job])
    def cancel_output_job_run(self, path: str, run: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("cancel_output_job_run", {"path": path, "run": run}), ["artifact", "cancel-output-job-run", path, "--run", run])
    def delete_output_job(self, path: str, output_job: str) -> JsonRpcResponse: return self._run_cli_json(self.build_request("delete_output_job", {"path": path, "output_job": output_job}), ["project", "delete-output-job", path, "--output-job", output_job])
    def delete_output_job_proposal(self, path: str, output_job: str, proposal: str | None = None, rationale: str | None = None) -> JsonRpcResponse:
        args = ["proposal", "delete-output-job", path, "--output-job", output_job]
        _append_optional(args, "proposal", proposal)
        _append_optional(args, "rationale", rationale)
        return self._run_cli_json(self.build_request("delete_output_job_proposal", {"path": path, "output_job": output_job, "proposal": proposal, "rationale": rationale}), args)
    def export_manufacturing_set(self, path: str, output_dir: str, prefix: str | None = None) -> JsonRpcResponse:
        args = ["artifact", "export-manufacturing-set", path, "--output-dir", output_dir]; _append_optional(args, "prefix", prefix); return self._run_cli_json(self.build_request("export_manufacturing_set", {"path": path, "output_dir": output_dir, "prefix": prefix}), args)
    def validate_manufacturing_set(self, path: str, output_dir: str, prefix: str | None = None) -> JsonRpcResponse:
        args = ["artifact", "validate-manufacturing-set", path, "--output-dir", output_dir]; _append_optional(args, "prefix", prefix); return self._run_cli_json_allowing_statuses(self.build_request("validate_manufacturing_set", {"path": path, "output_dir": output_dir, "prefix": prefix}), args, {0, 1})
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
    {"name": "create_board_component_replacement_proposal", "params": [("path", _REQUIRED), ("component", _REQUIRED), ("package", None), ("part", None), ("value", None), ("proposal", None), ("rationale", None)]},
    {"name": "create_board_component_replacements_proposal", "params": [("path", _REQUIRED), ("replacements", _REQUIRED), ("proposal", None), ("rationale", None)]},
    {"name": "create_board_component_replacement_plan_proposal", "params": [("path", _REQUIRED), ("selections", _REQUIRED), ("proposal", None), ("rationale", None)]},
    {
        "name": "set_reference",
        "params": [("uuid", _REQUIRED), ("reference", _REQUIRED)],
    },
    {"name": "undo", "params": []},
    {"name": "redo", "params": []},
    {"name": "search_pool", "params": [("query", _REQUIRED)]},
    {"name": "get_part", "params": [("uuid", _REQUIRED)]},
    {"name": "get_package", "params": [("uuid", _REQUIRED)]},
    {"name": "get_package_change_candidates", "params": [("uuid", _REQUIRED)]},
    {"name": "get_part_change_candidates", "params": [("uuid", _REQUIRED)]},
    {"name": "get_component_replacement_plan", "params": [("uuid", _REQUIRED)]},
    {"name": "get_scoped_component_replacement_plan", "params": [("scope", _REQUIRED), ("policy", _REQUIRED)]},
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
    {"name": "run_drc", "params": [("rules", None)], "omit_none": True},
    {
        "name": "explain_violation",
        "params": [("domain", _REQUIRED), ("index", None), ("fingerprint", None)],
        "omit_none": True,
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
        def request_method(
            self: EngineDaemonClient,
            *args: Any,
            _name: str = name,
            _param_specs: list[tuple[str, Any]] = param_specs,
            _fixed: dict[str, Any] = dict(spec.get("fixed", {})),
            _omit_none: bool = bool(spec.get("omit_none")),
            **kwargs: Any,
        ) -> JsonRpcRequest:
            params = _build_client_params(_name, _param_specs, args, kwargs)
            if _omit_none: params = {key: value for key, value in params.items() if value is not None}
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
from server_runtime_library import install_library_methods; from server_runtime_project_query import install_project_query_methods; from server_runtime_proposals import install_proposal_authoring_methods; from server_runtime_schematic_drawing import install_schematic_drawing_methods; from server_runtime_schematic_sheet import install_schematic_sheet_methods; from server_runtime_schematic_symbol import install_schematic_symbol_methods; install_library_methods(EngineDaemonClient); install_project_query_methods(EngineDaemonClient); install_proposal_authoring_methods(EngineDaemonClient, _append_optional); install_schematic_drawing_methods(EngineDaemonClient, _append_optional); install_schematic_sheet_methods(EngineDaemonClient, _append_optional); install_schematic_symbol_methods(EngineDaemonClient, _append_optional)
def run_server() -> None:
    host = StdioToolHost(EngineDaemonClient())
    host.run_stdio()
if __name__ == "__main__":
    run_server()
