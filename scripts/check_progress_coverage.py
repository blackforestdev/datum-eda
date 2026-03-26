#!/usr/bin/env python3
"""Enforce PLAN/PROGRESS/spec parity against live repository state."""

from __future__ import annotations

from pathlib import Path
import re
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]


def read_text(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def run(command: list[str]) -> tuple[int, str]:
    completed = subprocess.run(
        command,
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
    )
    return completed.returncode, completed.stdout.strip()


def parse_daemon_methods() -> set[str]:
    text = read_text("crates/engine-daemon/src/dispatch.rs")
    return set(re.findall(r'^\s*"([a-z_]+)"\s*=>', text, flags=re.M))


def parse_tool_catalog_methods() -> set[str]:
    text = read_text("mcp-server/tools_catalog_data.py")
    return set(re.findall(r'"name":\s*"([a-z_]+)"', text))


def parse_current_mcp_method_block(spec_text: str) -> set[str]:
    match = re.search(
        r"### Current Implemented Methods \([^\n]+\)\n\n(.*?)(?:\n\n### |\Z)",
        spec_text,
        flags=re.S,
    )
    if not match:
        return set()
    return set(re.findall(r"`([a-z_]+)`", match.group(1)))


def parse_mcp_headings(spec_text: str) -> set[str]:
    return set(re.findall(r"^#### `([a-z_]+)`", spec_text, flags=re.M))


def markdown_row_status(text: str, label: str) -> tuple[str, str] | None:
    pattern = re.compile(
        rf"^\|\s*{re.escape(label)}\s*\|\s*(\[[x~ —]\])\s*\|\s*(.*?)\s*\|$",
        flags=re.M,
    )
    match = pattern.search(text)
    if not match:
        return None
    return match.group(1), match.group(2)


def section_slice(text: str, heading: str) -> str | None:
    match = re.search(
        rf"^### {re.escape(heading)}.*?(?=^### |\Z)",
        text,
        flags=re.S | re.M,
    )
    return match.group(0) if match else None


def check_plan_progress_coverage(plan_text: str, failures: list[str]) -> None:
    for heading in (
        "M0: Canonical IR + Foundation",
        "M1: Design Ingestion + Query Engine",
        "M2: ERC + DRC + Reporting + MCP/CLI",
        "R1: Commercial Interop Research Track",
        "M3: Write Operations on Imported Designs",
        "M4: Native Project Creation + Editing",
    ):
        section = section_slice(plan_text, heading)
        if section is None:
            failures.append(f"PLAN.md: missing section: {heading}")
            continue
        if "**Progress (" not in section:
            failures.append(f"PLAN.md: missing progress block in section: {heading}")


def check_progress_sections(progress_text: str, failures: list[str]) -> None:
    required_headers = (
        "## PROGRAM_SPEC.md — M0 Exit Criteria",
        "## PROGRAM_SPEC.md — M1 Exit Criteria",
        "## PROGRAM_SPEC.md — M2 Exit Criteria",
        "## PROGRAM_SPEC.md — M3 Exit Criteria",
        "## PROGRAM_SPEC.md — M4 Exit Criteria",
        "## Infrastructure",
    )
    for header in required_headers:
        if header not in progress_text:
            failures.append(f"specs/PROGRESS.md: missing section header: {header}")


def check_infrastructure_rows(progress_text: str, failures: list[str]) -> None:
    daemon_row = markdown_row_status(progress_text, "Daemon JSON-RPC dispatch")
    if daemon_row is None:
        failures.append("specs/PROGRESS.md: missing 'Daemon JSON-RPC dispatch' row")
    else:
        _, notes = daemon_row
        expected_count = len(parse_daemon_methods())
        match = re.search(r"(\d+)\s+methods", notes)
        if not match:
            failures.append(
                "specs/PROGRESS.md: daemon dispatch row must include explicit '<N> methods' note"
            )
        elif int(match.group(1)) != expected_count:
            failures.append(
                "specs/PROGRESS.md: daemon method count mismatch "
                f"(doc={match.group(1)} code={expected_count})"
            )

    git_row = markdown_row_status(progress_text, "Git repository initialized")
    if git_row is None:
        failures.append("specs/PROGRESS.md: missing 'Git repository initialized' row")
    else:
        status, _ = git_row
        code, output = run(["git", "rev-parse", "--is-inside-work-tree"])
        in_repo = code == 0 and output == "true"
        expected = "[x]" if in_repo else "[ ]"
        if status != expected:
            failures.append(
                f"specs/PROGRESS.md: git row status mismatch (doc={status}, expected={expected})"
            )

    ci_row = markdown_row_status(progress_text, "CI pipeline")
    if ci_row is None:
        failures.append("specs/PROGRESS.md: missing 'CI pipeline' row")
    else:
        status, _ = ci_row
        has_ci = (ROOT / ".github/workflows/alignment.yml").exists()
        expected = "[x]" if has_ci else "[ ]"
        if status != expected:
            failures.append(
                f"specs/PROGRESS.md: CI row status mismatch (doc={status}, expected={expected})"
            )


def check_mcp_contract_parity(mcp_text: str, failures: list[str]) -> None:
    daemon_methods = parse_daemon_methods()
    tool_methods = parse_tool_catalog_methods()
    listed_methods = parse_current_mcp_method_block(mcp_text)
    heading_methods = parse_mcp_headings(mcp_text)

    if daemon_methods != tool_methods:
        failures.append(
            "MCP parity mismatch: daemon dispatch methods and MCP tool catalog differ "
            f"(daemon_only={sorted(daemon_methods - tool_methods)}, "
            f"tool_only={sorted(tool_methods - daemon_methods)})"
        )

    if listed_methods != daemon_methods:
        failures.append(
            "specs/MCP_API_SPEC.md current method list mismatch with daemon methods "
            f"(missing={sorted(daemon_methods - listed_methods)}, "
            f"extra={sorted(listed_methods - daemon_methods)})"
        )

    undocumented = sorted(daemon_methods - heading_methods)
    if undocumented:
        failures.append(
            "specs/MCP_API_SPEC.md missing method sections for current methods: "
            + ", ".join(undocumented)
        )

    deferred_phrase = "Write operations and save remain deferred."
    if deferred_phrase in mcp_text:
        failures.append(
            "specs/MCP_API_SPEC.md contains stale deferral text for current write/save support"
        )


def main() -> int:
    failures: list[str] = []
    plan_text = read_text("PLAN.md")
    progress_text = read_text("specs/PROGRESS.md")
    mcp_text = read_text("specs/MCP_API_SPEC.md")

    check_plan_progress_coverage(plan_text, failures)
    check_progress_sections(progress_text, failures)
    check_infrastructure_rows(progress_text, failures)
    check_mcp_contract_parity(mcp_text, failures)

    if failures:
        print("Progress coverage audit failed:")
        for failure in failures:
            print(f"  - {failure}")
        return 1

    print("Progress coverage audit passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
