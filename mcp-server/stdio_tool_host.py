#!/usr/bin/env python3
"""stdio MCP host and Datum target-envelope shaping."""

from __future__ import annotations

import json
import sys
from typing import Any

from datum_result_normalization import normalize_datum_result
from tool_dispatch import dispatch_tool_call
from tools_catalog import TOOLS


class StdioToolHost:
    def __init__(self, daemon: Any) -> None:
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
                if isinstance(name, str) and name.startswith("datum."):
                    result = {"content": [{"type": "json", "json": _datum_error_envelope(name, exc)}]}
                    return {"jsonrpc": "2.0", "id": msg_id, "result": result}
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

        result = response.result
        if isinstance(name, str) and name.startswith("datum."):
            result = _datum_target_envelope(name, result)

        return {"content": [{"type": "json", "json": result}]}

    def run_stdio(self) -> None:
        for line in sys.stdin:
            line = line.strip()
            if not line:
                continue
            response = self.handle_message(json.loads(line))
            if response is not None:
                print(json.dumps(response), flush=True)


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
    return {
        "ok": False,
        "schema": {"name": name, "version": 1},
        "context": _datum_result_context(None),
        "error": {
            "code": "tool_call_failed",
            "message": str(exc),
            "details": {"exception_type": exc.__class__.__name__},
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
