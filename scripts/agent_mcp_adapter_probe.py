#!/usr/bin/env python3
"""Fake native agent client for the governed MCP adapter acceptance proof."""

from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
from typing import Any


def _argument_after(arguments: list[str], flag: str) -> str:
    try:
        return arguments[arguments.index(flag) + 1]
    except (ValueError, IndexError) as exc:
        raise RuntimeError(f"missing native adapter argument {flag}") from exc


def _json_config(path: str | os.PathLike[str]) -> tuple[str, list[str]]:
    document = json.loads(Path(path).read_text(encoding="utf-8"))
    server = document["mcpServers"]["datum"]
    return server["command"], server["args"]


def _broker_command(adapter: str, arguments: list[str]) -> tuple[str, list[str]]:
    if adapter == "codex":
        overrides: dict[str, Any] = {}
        for index, argument in enumerate(arguments):
            if argument == "-c" and index + 1 < len(arguments):
                key, value = arguments[index + 1].split("=", 1)
                overrides[key] = json.loads(value)
        return overrides["mcp_servers.datum.command"], overrides["mcp_servers.datum.args"]
    if adapter == "claude-code":
        return _json_config(_argument_after(arguments, "--mcp-config"))
    if adapter == "cursor-cli":
        return _json_config(Path.cwd() / ".cursor/mcp.json")
    if adapter == "local-generic":
        return "datum-eda", [
            "mcp",
            "serve",
            "--discovery",
            os.environ["DATUM_AGENT_DISCOVERY"],
        ]
    raise RuntimeError(f"unknown adapter fixture {adapter}")


def _request(identifier: int, method: str) -> str:
    return json.dumps(
        {"jsonrpc": "2.0", "id": identifier, "method": method, "params": {}},
        separators=(",", ":"),
    )


def main() -> int:
    arguments = sys.argv[1:]
    adapter = _argument_after(arguments, "--probe-adapter")
    if not all(os.isatty(fd) for fd in (0, 1, 2)):
        raise RuntimeError("agent fixture did not launch inside the Datum PTY")
    if "DATUM_MCP_ENDPOINT" in os.environ:
        raise RuntimeError("adapter proof must not rely on DATUM_MCP_ENDPOINT")
    command, broker_args = _broker_command(adapter, arguments)
    if command != "datum-eda":
        raise RuntimeError(f"native adapter selected unexpected MCP command {command!r}")
    expected_discovery = os.environ["DATUM_AGENT_DISCOVERY"]
    if broker_args != ["mcp", "serve", "--discovery", expected_discovery]:
        raise RuntimeError(f"native adapter selected unexpected MCP arguments {broker_args!r}")
    discovery_document = json.loads(Path(expected_discovery).read_text(encoding="utf-8"))
    discovery_document["terminal_session_id"] = os.environ["DATUM_TERMINAL_SESSION_ID"]
    Path(expected_discovery).write_text(
        json.dumps(discovery_document, separators=(",", ":")), encoding="utf-8"
    )
    protocol = "\n".join(
        [
            _request(1, "initialize"),
            _request(2, "tools/list"),
            _request(3, "resources/list"),
            _request(4, "prompts/list"),
            "",
        ]
    )
    completed = subprocess.run(
        [os.environ["DATUM_EXPECT_CLI"], *broker_args],
        input=protocol,
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(f"broker failed: {completed.stderr[:1024]}")
    messages = [json.loads(line) for line in completed.stdout.splitlines()]
    if len(messages) != 4:
        raise RuntimeError("broker stdout was not the four requested protocol messages")
    capabilities = messages[0]["result"]["capabilities"]
    if not {"tools", "resources", "prompts"}.issubset(capabilities):
        raise RuntimeError("broker did not declare the required MCP capabilities")
    if not any(tool["name"].startswith("datum.") for tool in messages[1]["result"]["tools"]):
        raise RuntimeError("native client could not enumerate typed Datum tools")
    if not any(
        resource["uri"] == "datum://context/live"
        for resource in messages[2]["result"]["resources"]
    ):
        raise RuntimeError("native client could not enumerate Datum resources")
    if not any(
        prompt["name"] == "datum.prepare-proposal"
        for prompt in messages[3]["result"]["prompts"]
    ):
        raise RuntimeError("native client could not enumerate Datum prompts")
    print(f"AGENT_MCP_OK:{adapter}:{expected_discovery}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
