#!/usr/bin/env python3
"""Validate the T4V-01 final production-verification inventory."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "specs/terminal_production_verification_matrix.json"

REQUIRED_PACKAGES = {
    "TERMINAL-T0-SHELL-TRUTH",
    "TERMINAL-P0-FOCUS",
    "TERMINAL-T1-INPUT",
    "TERMINAL-T1-PTY",
    "TERMINAL-T1-CORE",
    "TERMINAL-T2-NATIVE",
    "TERMINAL-T3-UX",
    "TERMINAL-T3-PROTOCOLS",
    "TERMINAL-T4A-AGENT-LAUNCH",
    "TERMINAL-T4B-MCP",
    "TERMINAL-T4C-CONTEXT-AUTHORITY",
    "TERMINAL-T4D-WORKFLOW-PARITY",
}
REQUIRED_RUNS = {
    "workspace-offline",
    "terminal-core-release",
    "transport-smoke",
    "transport-gui",
    "transport-sustained",
    "transport-lifecycle",
    "transport-soak",
    "wayland-canary",
    "named-shell-tui",
    "agent-workflow",
    "accessibility-live-bus",
    "repository-drift",
}
REQUIRED_DELTAS = {
    "external-vt-suites",
    "orca-cache-extension",
    "platform",
    "local-agent-resume",
}


def failures(root: Path = ROOT) -> list[str]:
    path = root / MATRIX.relative_to(ROOT)
    try:
        matrix = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        return [f"cannot read T4 production matrix: {error}"]
    problems: list[str] = []
    if matrix.get("schema") != "datum-terminal-production-verification-matrix-v1":
        problems.append("T4 production matrix schema is missing or changed")
    if matrix.get("frontier_task") != "TERMINAL-T4-VERIFY" or matrix.get("planning_step") != "T4V-01":
        problems.append("T4 production matrix is not bound to T4V-01")
    packages = matrix.get("completed_packages", [])
    package_ids = {entry.get("key") for entry in packages}
    if package_ids != REQUIRED_PACKAGES:
        problems.append("T4 production matrix package inventory does not match T0 through T4d")
    for entry in packages:
        if not entry.get("landing_commit") or not entry.get("evidence"):
            problems.append(f"completed package {entry.get('key')} lacks commit or evidence")
    for entry in matrix.get("checked_in_evidence", []):
        evidence_path = entry.get("path", "")
        if not evidence_path or not (root / evidence_path).is_file():
            problems.append(f"checked-in evidence {entry.get('id')} lacks a live path")
        if not entry.get("result"):
            problems.append(f"checked-in evidence {entry.get('id')} lacks a result")
    runs = matrix.get("t4v02_runs", [])
    run_ids = {entry.get("id") for entry in runs}
    if run_ids != REQUIRED_RUNS:
        problems.append("T4V-02 run inventory is incomplete or contains an ungoverned run")
    for entry in runs:
        if not entry.get("domain") or not entry.get("command"):
            problems.append(f"T4V-02 run {entry.get('id')} lacks domain or exact command")
        if entry.get("status") not in {"pending", "passed", "failed", "known-delta"}:
            problems.append(f"T4V-02 run {entry.get('id')} has an invalid status")
        if entry.get("status") == "pending" or not entry.get("evidence"):
            problems.append(f"T4V-02 run {entry.get('id')} lacks a completed evidence record")
    deltas = {entry.get("id") for entry in matrix.get("known_deltas", [])}
    if deltas != REQUIRED_DELTAS:
        problems.append("T4 production known-delta inventory changed without review")
    acceptance = matrix.get("owner_acceptance", {})
    if acceptance.get("frontier_step") != "T4V-03" or len(acceptance.get("checklist", [])) < 4:
        problems.append("T4 production matrix lacks the T4V-03 hands-on checklist")
    if acceptance.get("status") != "accepted" or not acceptance.get("owner_response"):
        problems.append("T4V-03 owner acceptance is not durably recorded")
    return problems


def main() -> int:
    problems = failures()
    if problems:
        for problem in problems:
            print(f"terminal production matrix: {problem}")
        return 1
    print("terminal production verification matrix passed (12 packages; 12 live runs; 4 known deltas).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
