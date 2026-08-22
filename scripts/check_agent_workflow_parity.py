#!/usr/bin/env python3
"""Reject drift between agent instruction projections and canonical workflows."""

from __future__ import annotations

import json
from pathlib import Path
import re
import sys
from typing import Any


MARKER = "<!-- DATUM-WORKFLOW-CATALOG:datum://workflows -->"
EXPECTED_ADAPTERS = ("codex", "claude-code", "cursor-cli", "local-generic")
SEMANTIC_KEYS = {
    "workflow_ids",
    "required_capability",
    "context_inputs",
    "review_gate",
    "evidence",
    "mcp_tools",
    "mcp_resources",
    "mcp_prompts",
}


def check(root: Path) -> list[str]:
    failures: list[str] = []
    try:
        projections = json.loads(
            (root / "mcp-server/workflow_projections.json").read_text(encoding="utf-8")
        )
        catalog = json.loads(
            (root / "mcp-server/workflow_catalog.json").read_text(encoding="utf-8")
        )
        adapter_source = (root / "crates/cli/src/agent_adapters.rs").read_text(
            encoding="utf-8"
        )
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        return [f"workflow parity inputs are unreadable: {exc}"]

    if projections.get("projection_version") != 1:
        failures.append("workflow projections must use projection_version 1")
    if projections.get("workflow_catalog_uri") != "datum://workflows":
        failures.append("workflow projections must target datum://workflows")
    if catalog.get("catalog_version") != 1 or not catalog.get("workflows"):
        failures.append("canonical workflow catalog is missing or empty")

    adapters = projections.get("adapters")
    if not isinstance(adapters, list):
        return failures + ["workflow projection adapters must be an array"]
    ids = tuple(item.get("adapter_id") for item in adapters if isinstance(item, dict))
    if ids != EXPECTED_ADAPTERS:
        failures.append(f"workflow projection adapter order drifted: {ids!r}")

    for item in adapters:
        if not isinstance(item, dict):
            failures.append("workflow projection entries must be objects")
            continue
        adapter_id = item.get("adapter_id")
        forbidden = SEMANTIC_KEYS & item.keys()
        if forbidden:
            failures.append(
                f"{adapter_id} projection redefines workflow semantics: {sorted(forbidden)}"
            )
        primary = _string_array(item.get("adapter_instruction_files"))
        supplemental = _string_array(item.get("supplemental_instruction_files"))
        if primary is None or supplemental is None:
            failures.append(f"{adapter_id} instruction files must be string arrays")
            continue
        if not _adapter_declares_files(adapter_source, adapter_id, primary):
            failures.append(
                f"{adapter_id} projection differs from AgentAdapter instruction files"
            )
        expected_delivery = (
            "printed-stdio-command" if adapter_id == "local-generic" else "native-mcp"
        )
        if item.get("delivery") != expected_delivery:
            failures.append(f"{adapter_id} workflow delivery drifted")
        for relative in primary + supplemental:
            path = Path(relative)
            if path.is_absolute() or ".." in path.parts:
                failures.append(f"{adapter_id} projection path escapes repository: {relative}")
                continue
            try:
                text = (root / path).read_text(encoding="utf-8")
            except (OSError, UnicodeError):
                failures.append(f"{adapter_id} projection file is unavailable: {relative}")
                continue
            if MARKER not in text:
                failures.append(f"{adapter_id} projection lacks canonical marker: {relative}")
    return failures


def _string_array(value: Any) -> list[str] | None:
    if not isinstance(value, list) or not all(
        isinstance(item, str) and item for item in value
    ):
        return None
    return value


def _adapter_declares_files(source: str, adapter_id: Any, expected: list[str]) -> bool:
    if not isinstance(adapter_id, str):
        return False
    block = re.search(
        rf'id: "{re.escape(adapter_id)}",(?P<body>.*?)(?=\n    AgentAdapter \{{|\n\];)',
        source,
        re.DOTALL,
    )
    if block is None:
        return False
    files = re.search(r"files: &\[(?P<files>.*?)\]", block.group("body"), re.DOTALL)
    if files is None:
        return False
    declared = re.findall(r'"([^"]+)"', files.group("files"))
    return declared == expected


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    failures = check(root)
    if failures:
        print("Agent workflow parity FAILED:")
        for failure in failures:
            print(f"- {failure}")
        return 1
    print("Agent workflow parity: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
