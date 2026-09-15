#!/usr/bin/env python3
"""Portable, user-invoked MCP prompts for Datum review workflows."""

from __future__ import annotations

from typing import Any

from discovery_scope import DiscoveryScope


PROMPTS = [
    {
        "name": "datum.inspect-current-design",
        "description": "Inspect the current design context without changing it.",
        "arguments": [],
    },
    {
        "name": "datum.review-active-findings",
        "description": "Review current check findings and explain the highest-value next action.",
        "arguments": [],
    },
    {
        "name": "datum.prepare-proposal",
        "description": "Prepare a reviewable proposal; never apply it automatically.",
        "arguments": [
            {"name": "objective", "description": "Desired design outcome", "required": True}
        ],
    },
]


def get_prompt(scope: DiscoveryScope, name: str, arguments: Any) -> dict[str, Any]:
    if not isinstance(arguments, dict):
        raise ValueError("prompt arguments must be an object")
    context_uri = (
        f"datum://context/pinned/{scope.context_id}"
        if scope.context_id
        else "datum://context/live"
    )
    if name == "datum.inspect-current-design":
        text = (
            f"Read {context_uri} and datum://project/current. Inspect the current Datum design "
            "with typed read-only tools. Report stable object IDs and do not mutate the model."
        )
    elif name == "datum.review-active-findings":
        text = (
            f"Read {context_uri} and datum://checks/current. Review the visible findings, explain "
            "their evidence, and recommend the next review action without applying changes."
        )
    elif name == "datum.prepare-proposal":
        objective = arguments.get("objective")
        if not isinstance(objective, str) or not objective.strip():
            raise ValueError("prompt argument objective must be a non-empty string")
        text = (
            f"Read {context_uri}. Prepare a Datum proposal for this objective: {objective.strip()}. "
            "Use proposal-producing tools only, preserve review gates, and do not apply it."
        )
    else:
        raise ValueError(f"unknown prompt: {name}")
    return {
        "description": next(prompt["description"] for prompt in PROMPTS if prompt["name"] == name),
        "messages": [{"role": "user", "content": {"type": "text", "text": text}}],
    }
