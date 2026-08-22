#!/usr/bin/env python3
"""stdio MCP host and Datum target-envelope shaping."""

from __future__ import annotations

import json
import sys
from typing import Any, TextIO

from context_revision_fence import (
    ContextFenceError,
    scoped_tool_catalog,
    validate_context_fence,
)
from datum_result_normalization import normalize_datum_result
from discovery_scope import DiscoveryScope
from prompts_catalog import PROMPTS, get_prompt
from resources_catalog import DatumResourceCatalog, RESOURCE_TEMPLATES
from tool_dispatch import dispatch_tool_call
from tools_catalog import TOOLS
from tools_catalog_data import TOOL_BY_NAME


class StdioToolHost:
    def __init__(
        self,
        daemon: Any,
        discovery: DiscoveryScope | None = None,
        *,
        notifications_enabled: bool = True,
    ) -> None:
        self._daemon = daemon
        self._resources = None if discovery is None else DatumResourceCatalog(discovery)
        self._discovery = discovery
        self._notifications_enabled = notifications_enabled
        self._subscriptions: set[str] = set()
        self._pending_notifications: list[dict[str, Any]] = []

    def handle_message(self, message: dict[str, Any]) -> dict[str, Any] | None:
        if not _valid_request(message):
            return _protocol_error(None, -32600, "invalid request")
        method = message.get("method")
        msg_id = message.get("id")
        params = message.get("params", {})

        if method == "initialize":
            requested_version = params.get("protocolVersion")
            protocol_version = (
                requested_version
                if requested_version in {"2024-11-05", "2025-03-26", "2025-06-18"}
                else "2024-11-05"
            )
            capabilities: dict[str, Any] = {"tools": {"listChanged": False}}
            if self._resources is not None:
                capabilities["resources"] = {
                    "subscribe": self._notifications_enabled,
                    "listChanged": self._notifications_enabled,
                }
                capabilities["prompts"] = {"listChanged": False}
            return {
                "jsonrpc": "2.0",
                "id": msg_id,
                "result": {
                    "protocolVersion": protocol_version,
                    "capabilities": capabilities,
                    "serverInfo": {"name": "datum-eda", "version": "0.1.0"},
                },
            }

        if method == "notifications/initialized":
            return None
        if method == "ping":
            return {"jsonrpc": "2.0", "id": msg_id, "result": {}}
        if method == "tools/list":
            return {
                "jsonrpc": "2.0",
                "id": msg_id,
                "result": {
                    "tools": scoped_tool_catalog(TOOLS, TOOL_BY_NAME, self._discovery)
                },
            }

        if method == "resources/list":
            return self._resource_result(msg_id, "resources", self._resources.list_resources())
        if method == "resources/templates/list":
            return self._resource_result(msg_id, "resourceTemplates", RESOURCE_TEMPLATES)
        if method == "resources/read":
            return self._read_resource(msg_id, params)
        if method in {"resources/subscribe", "resources/unsubscribe"}:
            return self._change_subscription(
                msg_id, params, method == "resources/subscribe"
            )
        if method == "prompts/list":
            return self._resource_result(msg_id, "prompts", PROMPTS)
        if method == "prompts/get":
            return self._get_prompt(msg_id, params)

        if method == "tools/call":
            name = params.get("name")
            arguments = params.get("arguments", {})
            try:
                result = self._call_tool(name, arguments)
            except Exception as exc:
                if isinstance(exc, ContextFenceError):
                    result = {
                        "content": [
                            {"type": "json", "json": _datum_error_envelope(name, exc)}
                        ]
                    }
                    return {"jsonrpc": "2.0", "id": msg_id, "result": result}
                if isinstance(name, str) and name.startswith("datum."):
                    result = {"content": [{"type": "json", "json": _datum_error_envelope(name, exc)}]}
                    return {"jsonrpc": "2.0", "id": msg_id, "result": result}
                return {
                    "jsonrpc": "2.0",
                    "id": msg_id,
                    "error": {"code": -32010, "message": str(exc)},
                }
            if TOOL_BY_NAME.get(name, {}).get("x_public_write_surface_class"):
                self._publish_resource_changes()
            return {"jsonrpc": "2.0", "id": msg_id, "result": result}

        if msg_id is None:
            return None

        return {
            "jsonrpc": "2.0",
            "id": msg_id,
            "error": {"code": -32601, "message": "method not found"},
        }

    def drain_notifications(self) -> list[dict[str, Any]]:
        pending, self._pending_notifications = self._pending_notifications, []
        return pending

    def _resource_result(self, msg_id: Any, key: str, value: Any) -> dict[str, Any]:
        if self._resources is None or self._discovery is None:
            return _protocol_error(msg_id, -32601, "method not found")
        return {"jsonrpc": "2.0", "id": msg_id, "result": {key: value}}

    def _read_resource(self, msg_id: Any, params: Any) -> dict[str, Any]:
        if self._resources is None or not isinstance(params, dict):
            return _protocol_error(msg_id, -32602, "invalid resource request")
        uri = params.get("uri")
        if not isinstance(uri, str):
            return _protocol_error(msg_id, -32602, "resource uri must be a string")
        try:
            result = self._resources.read(uri)
        except ValueError as exc:
            return _protocol_error(msg_id, -32002, str(exc))
        return {"jsonrpc": "2.0", "id": msg_id, "result": result}

    def _change_subscription(
        self, msg_id: Any, params: Any, subscribe: bool
    ) -> dict[str, Any]:
        if self._resources is None or not self._notifications_enabled:
            return _protocol_error(msg_id, -32601, "resource subscriptions are unavailable")
        uri = params.get("uri") if isinstance(params, dict) else None
        if not isinstance(uri, str) or not self._resources.supports(uri):
            return _protocol_error(msg_id, -32602, "resource uri is unavailable")
        if subscribe:
            self._subscriptions.add(uri)
        else:
            self._subscriptions.discard(uri)
        return {"jsonrpc": "2.0", "id": msg_id, "result": {}}

    def _get_prompt(self, msg_id: Any, params: Any) -> dict[str, Any]:
        if self._discovery is None or not isinstance(params, dict):
            return _protocol_error(msg_id, -32601, "prompts are unavailable")
        try:
            result = get_prompt(
                self._discovery, params.get("name"), params.get("arguments", {})
            )
        except ValueError as exc:
            return _protocol_error(msg_id, -32602, str(exc))
        return {"jsonrpc": "2.0", "id": msg_id, "result": result}

    def _publish_resource_changes(self) -> None:
        if not self._notifications_enabled:
            return
        for uri in sorted(self._subscriptions):
            self._pending_notifications.append(
                {
                    "jsonrpc": "2.0",
                    "method": "notifications/resources/updated",
                    "params": {"uri": uri},
                }
            )

    def _call_tool(self, name: str, arguments: dict[str, Any]) -> dict[str, Any]:
        fenced_arguments = validate_context_fence(
            self._discovery, name, arguments, TOOL_BY_NAME
        )
        response = dispatch_tool_call(self._daemon, name, fenced_arguments)
        if response.error is not None:
            raise RuntimeError(response.error.message)

        result = response.result
        if isinstance(name, str) and name.startswith("datum."):
            result = _datum_target_envelope(name, result)

        return {"content": [{"type": "json", "json": result}]}

    def run_stdio(self, stdin: TextIO | None = None, stdout: TextIO | None = None) -> None:
        input_stream = stdin or sys.stdin
        output_stream = stdout or sys.stdout
        for line in input_stream:
            line = line.strip()
            if not line:
                continue
            try:
                message = json.loads(line)
            except json.JSONDecodeError:
                response = _protocol_error(None, -32700, "parse error")
            else:
                if not _valid_request(message):
                    response = _protocol_error(None, -32600, "invalid request")
                else:
                    response = self.handle_message(message)
            if response is not None:
                print(json.dumps(response, separators=(",", ":")), file=output_stream, flush=True)
            for notification in self.drain_notifications():
                print(
                    json.dumps(notification, separators=(",", ":")),
                    file=output_stream,
                    flush=True,
                )


def _protocol_error(msg_id: Any, code: int, message: str) -> dict[str, Any]:
    return {
        "jsonrpc": "2.0",
        "id": msg_id,
        "error": {"code": code, "message": message},
    }


def _valid_request(message: Any) -> bool:
    if not isinstance(message, dict):
        return False
    if message.get("jsonrpc") != "2.0" or not isinstance(message.get("method"), str):
        return False
    msg_id = message.get("id")
    return "id" not in message or msg_id is None or (
        isinstance(msg_id, (str, int)) and not isinstance(msg_id, bool)
    )


def _datum_target_envelope(name: str, result: Any) -> dict[str, Any]:
    normalized = normalize_datum_result(name, result)
    envelope: dict[str, Any] = {
        "ok": True,
        "schema": {"name": name, "version": 1},
        "context": _datum_result_context(result),
        "result": normalized,
    }
    if isinstance(result, dict):
        # Transitional compatibility while callers migrate to `result`.
        for key, value in result.items():
            if key not in {"ok", "schema", "context", "result", "error"}:
                envelope[key] = value
    return envelope


def _datum_error_envelope(name: str, exc: Exception) -> dict[str, Any]:
    code = "tool_call_failed"
    details = {"exception_type": exc.__class__.__name__}
    if isinstance(exc, ContextFenceError):
        code = exc.code
        details = exc.details
    return {
        "ok": False,
        "schema": {"name": name, "version": 1},
        "context": _datum_result_context(None),
        "error": {
            "code": code,
            "message": str(exc),
            "details": details,
        },
    }


def _datum_result_context(result: Any) -> dict[str, Any]:
    project_id = model_revision = variant = output_context = None
    if isinstance(result, dict):
        project_id = result.get("project_id")
        model_revision = result.get("model_revision")
        variant = result.get("variant")
        output_context = result.get("output_context")
        for nested_key in ("check_run", "artifact", "artifact_metadata"):
            nested = result.get(nested_key)
            if isinstance(nested, dict):
                project_id = project_id or nested.get("project_id")
                model_revision = model_revision or nested.get("model_revision")
                variant = variant or nested.get("variant")
                output_context = output_context or nested.get("output_context")
    return {
        "project_id": project_id,
        "model_revision": model_revision,
        "variant": variant,
        "output_context": output_context,
    }
