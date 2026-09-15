#!/usr/bin/env python3
"""Canonical portable workflow inventory and cross-surface validation."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from prompts_catalog import PROMPTS
from resources_catalog import RESOURCE_TEMPLATES


CATALOG_PATH = Path(__file__).with_name("workflow_catalog.json")
TOOL_CATALOG_PATH = Path(__file__).with_name("datum_tool_catalog.json")
CAPABILITIES = {"inspect", "propose", "apply-approved", "unattended"}
REQUIRED_WORKFLOWS = {
    "inspect-current-design",
    "review-active-findings",
    "prepare-proposal",
    "review-proposal",
    "apply-approved-proposal",
    "refresh-stale-context",
    "resume-agent-work",
}
STATIC_RESOURCES = {
    "datum://project/current",
    "datum://context/live",
    "datum://checks/current",
    "datum://workflows",
}


def load_workflow_catalog(path: Path = CATALOG_PATH) -> dict[str, Any]:
    document = json.loads(path.read_text(encoding="utf-8"))
    validate_workflow_catalog(document)
    return document


def validate_workflow_catalog(document: Any) -> None:
    if not isinstance(document, dict) or document.get("catalog_version") != 1:
        raise ValueError("workflow catalog must use catalog_version 1")
    workflows = document.get("workflows")
    if not isinstance(workflows, list):
        raise ValueError("workflow catalog workflows must be an array")

    tools = _tool_catalog()
    cli_roots = _cli_roots(tools)
    prompts = {item["name"] for item in PROMPTS}
    resources = STATIC_RESOURCES | {
        item["uriTemplate"] for item in RESOURCE_TEMPLATES
    }
    seen: set[str] = set()
    used_prompts: set[str] = set()
    for workflow in workflows:
        if not isinstance(workflow, dict):
            raise ValueError("workflow entries must be objects")
        workflow_id = _nonempty_string(workflow, "id")
        if workflow_id in seen:
            raise ValueError(f"duplicate workflow id {workflow_id!r}")
        seen.add(workflow_id)
        _nonempty_string(workflow, "intent")
        capability = _nonempty_string(workflow, "required_capability")
        if capability not in CAPABILITIES:
            raise ValueError(f"unknown capability {capability!r} for {workflow_id}")
        _nonempty_string(workflow, "review_gate")
        cli = _string_array(workflow, "cli", allow_empty=False)
        mcp_tools = _string_array(workflow, "mcp_tools", allow_empty=False)
        mcp_resources = _string_array(workflow, "mcp_resources", allow_empty=False)
        mcp_prompts = _string_array(workflow, "mcp_prompts", allow_empty=True)
        _string_array(workflow, "context_inputs", allow_empty=False)
        _string_array(workflow, "evidence", allow_empty=False)
        if any(not command.startswith("datum-eda ") for command in cli):
            raise ValueError(f"{workflow_id} contains a non-Datum CLI command")
        unknown_cli = {
            " ".join(command.split()[:3])
            for command in cli
            if " ".join(command.split()[:3]) not in cli_roots
        }
        if unknown_cli:
            raise ValueError(
                f"{workflow_id} names unknown CLI command roots {sorted(unknown_cli)}"
            )
        unknown_tools = set(mcp_tools) - tools.keys()
        if unknown_tools:
            raise ValueError(f"{workflow_id} names unknown MCP tools {sorted(unknown_tools)}")
        unknown_resources = set(mcp_resources) - resources
        if unknown_resources:
            raise ValueError(
                f"{workflow_id} names unknown MCP resources {sorted(unknown_resources)}"
            )
        unknown_prompts = set(mcp_prompts) - prompts
        if unknown_prompts:
            raise ValueError(
                f"{workflow_id} names unknown MCP prompts {sorted(unknown_prompts)}"
            )
        used_prompts.update(mcp_prompts)
        for tool_name in mcp_tools:
            dispatch = tools[tool_name].get("dispatch")
            if not isinstance(dispatch, dict) or dispatch.get("kind") != "cli":
                raise ValueError(f"{workflow_id} tool {tool_name} lacks canonical CLI dispatch")

    if seen != REQUIRED_WORKFLOWS:
        raise ValueError(
            "workflow catalog IDs differ from the governed inventory: "
            f"missing={sorted(REQUIRED_WORKFLOWS - seen)}, "
            f"extra={sorted(seen - REQUIRED_WORKFLOWS)}"
        )
    if used_prompts != prompts:
        raise ValueError(
            f"portable prompt coverage drifted: missing={sorted(prompts - used_prompts)}"
        )


def _tool_catalog() -> dict[str, dict[str, Any]]:
    document = json.loads(TOOL_CATALOG_PATH.read_text(encoding="utf-8"))
    return {item["name"]: item for item in document["verbs"]}


def _cli_roots(tools: dict[str, dict[str, Any]]) -> set[str]:
    roots = {"datum-eda agent launch"}
    for tool in tools.values():
        dispatch = tool.get("dispatch")
        if not isinstance(dispatch, dict) or dispatch.get("kind") != "cli":
            continue
        literals = [
            token["lit"]
            for token in dispatch.get("argv", [])
            if isinstance(token, dict) and isinstance(token.get("lit"), str)
        ]
        if len(literals) >= 2:
            roots.add(f"datum-eda {literals[0]} {literals[1]}")
    return roots


def _nonempty_string(document: dict[str, Any], key: str) -> str:
    value = document.get(key)
    if not isinstance(value, str) or not value:
        raise ValueError(f"workflow {key} must be a non-empty string")
    return value


def _string_array(
    document: dict[str, Any], key: str, *, allow_empty: bool
) -> list[str]:
    value = document.get(key)
    if not isinstance(value, list) or not all(
        isinstance(item, str) and item for item in value
    ):
        raise ValueError(f"workflow {key} must be a string array")
    if not allow_empty and not value:
        raise ValueError(f"workflow {key} may not be empty")
    if len(value) != len(set(value)):
        raise ValueError(f"workflow {key} entries must be unique")
    return value
