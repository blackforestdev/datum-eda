"""Tool-name to daemon-method dispatch for tools/call."""

from __future__ import annotations

from typing import Any

from tools_catalog_data import TOOL_BY_NAME


def registered_tool_names() -> list[str]:
    return list(TOOL_BY_NAME)


def dispatch_tool_call(daemon: Any, name: str, arguments: dict[str, Any]) -> Any:
    spec = TOOL_BY_NAME.get(name)
    if spec is None:
        raise RuntimeError(f"unknown tool: {name}")

    method = getattr(daemon, name, None)
    if method is None:
        raise RuntimeError(f"registered tool missing daemon method: {name}")

    return method(*_dispatch_args(spec, arguments))


def _dispatch_args(spec: dict[str, Any], arguments: dict[str, Any]) -> list[Any]:
    input_schema = spec.get("inputSchema", {})
    properties = input_schema.get("properties", {})
    required = set(input_schema.get("required", []))
    defaults = spec.get("x_dispatch_defaults", {})
    values: list[Any] = []
    arg_order = spec.get("x_dispatch_args", list(properties))
    for param_name in arg_order:
        if param_name in required:
            values.append(arguments[param_name])
        else:
            values.append(arguments.get(param_name, defaults.get(param_name)))
    return values
