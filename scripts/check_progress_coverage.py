#!/usr/bin/env python3
"""Enforce single-source status authority and spec parity checks."""

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


def markdown_overall_status(text: str, label: str) -> tuple[str, str] | None:
    pattern = re.compile(
        rf"^\*\*{re.escape(label)}\*\*:\s*(\[[x~ —]\])\s*(.*?)$",
        flags=re.M,
    )
    match = pattern.search(text)
    if not match:
        return None
    return match.group(1), match.group(2)


def parse_markdown_table_first_column(text: str, section_heading: str) -> list[str]:
    section_match = re.search(
        rf"^## {re.escape(section_heading)}\n(.*?)(?=^## |\Z)",
        text,
        flags=re.S | re.M,
    )
    if not section_match:
        return []
    section = section_match.group(1)

    table_match = re.search(
        r"^\|[^\n]*\|\n^\|[-| :]+\|\n(.*?)(?=\n\n|\Z)",
        section,
        flags=re.S | re.M,
    )
    if not table_match:
        return []

    rows = []
    for line in table_match.group(1).splitlines():
        if not line.startswith("|"):
            continue
        cols = [c.strip() for c in line.strip().strip("|").split("|")]
        if cols and cols[0]:
            rows.append(cols[0])
    return rows


def parse_program_m4_gate_names(program_text: str) -> list[str]:
    match = re.search(
        r"### M4:.*?\n\n\| Criterion \| Threshold \|\n\|[-| ]+\n(.*?)\n\n\*\*Non-goals for M4",
        program_text,
        flags=re.S,
    )
    if not match:
        return []
    names: list[str] = []
    for line in match.group(1).splitlines():
        if not line.startswith("|"):
            continue
        cols = [c.strip() for c in line.strip().strip("|").split("|")]
        if cols and cols[0] and cols[0] != "Criterion":
            names.append(cols[0])
    return names


def parse_integrated_m4_gate_names(integrated_text: str) -> list[str]:
    match = re.search(
        r"\| M4 Gate \(`specs/PROGRAM_SPEC\.md`\) \| Evidence Type \| Required Evidence Hook \|\n"
        r"\|[-| ]+\n(.*?)\n\n### 13\.1",
        integrated_text,
        flags=re.S,
    )
    if not match:
        return []
    names: list[str] = []
    for line in match.group(1).splitlines():
        if not line.startswith("|"):
            continue
        cols = [c.strip() for c in line.strip().strip("|").split("|")]
        if cols and cols[0] and not cols[0].startswith("M4 Gate"):
            names.append(cols[0])
    return names


def parse_progress_m4_gate_names(progress_text: str) -> list[str]:
    return parse_markdown_table_first_column(progress_text, "PROGRAM_SPEC.md — M4 Exit Criteria")


def check_single_status_authority(
    plan_text: str, program_text: str, integrated_text: str, failures: list[str]
) -> None:
    authority_files = (
        ("PLAN.md", plan_text),
        ("specs/PROGRAM_SPEC.md", program_text),
        ("specs/INTEGRATED_PROGRAM_SPEC.md", integrated_text),
    )

    for path, text in authority_files:
        if re.search(r"^- \[[x~ ]\] ", text, flags=re.M):
            failures.append(
                f"{path}: checklist-style status markers are not allowed outside specs/PROGRESS.md"
            )
        if "Current implementation status" in text:
            failures.append(
                f"{path}: found forbidden status heading 'Current implementation status'"
            )

    if "Current State |" in integrated_text:
        failures.append(
            "specs/INTEGRATED_PROGRAM_SPEC.md: remove 'Current State' columns from acceptance tables"
        )

    if "Status source of truth" not in plan_text:
        failures.append("PLAN.md: missing explicit status-authority statement")
    if "Status tracking rule:" not in program_text:
        failures.append("specs/PROGRAM_SPEC.md: missing explicit status-tracking rule")
    if "specs/PROGRESS.md` is the only source of truth for implementation status" not in integrated_text:
        failures.append(
            "specs/INTEGRATED_PROGRAM_SPEC.md: missing explicit single-source status rule"
        )


def check_progress_sections(progress_text: str, failures: list[str]) -> None:
    required_headers = (
        "## PROGRAM_SPEC.md — M0 Exit Criteria",
        "## PROGRAM_SPEC.md — M1 Exit Criteria",
        "## PROGRAM_SPEC.md — M2 Exit Criteria",
        "## PROGRAM_SPEC.md — R1 Research Gates",
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


def check_r1_g0_gate(progress_text: str, failures: list[str]) -> None:
    gate_row = markdown_row_status(progress_text, "R1-G0 Foundation Gate")
    if gate_row is None:
        failures.append("specs/PROGRESS.md: missing 'R1-G0 Foundation Gate' row")
        return

    gate_status, _ = gate_row
    for overall_label in ("M3 overall", "M4 overall"):
        overall = markdown_overall_status(progress_text, overall_label)
        if overall is None:
            failures.append(f"specs/PROGRESS.md: missing '{overall_label}' line")
            continue
        overall_status, _ = overall
        if overall_status == "[x]" and gate_status != "[x]":
            failures.append(
                f"specs/PROGRESS.md: {overall_label} cannot be [x] while R1-G0 Foundation Gate is {gate_status}"
            )


def check_m4_gate_parity(program_text: str, integrated_text: str, progress_text: str, failures: list[str]) -> None:
    program_gates = parse_program_m4_gate_names(program_text)
    integrated_gates = parse_integrated_m4_gate_names(integrated_text)
    progress_gates = parse_progress_m4_gate_names(progress_text)

    if not program_gates:
        failures.append("specs/PROGRAM_SPEC.md: unable to parse M4 gate table")
        return
    if not integrated_gates:
        failures.append("specs/INTEGRATED_PROGRAM_SPEC.md: unable to parse M4 acceptance table")
        return
    if not progress_gates:
        failures.append("specs/PROGRESS.md: unable to parse M4 exit-criteria table")
        return

    if integrated_gates != program_gates:
        failures.append(
            "M4 gate-name mismatch (PROGRAM_SPEC vs INTEGRATED): "
            f"program={program_gates} integrated={integrated_gates}"
        )
    if progress_gates != program_gates:
        failures.append(
            "M4 gate-name mismatch (PROGRAM_SPEC vs PROGRESS): "
            f"program={program_gates} progress={progress_gates}"
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
    program_text = read_text("specs/PROGRAM_SPEC.md")
    integrated_text = read_text("specs/INTEGRATED_PROGRAM_SPEC.md")
    progress_text = read_text("specs/PROGRESS.md")
    mcp_text = read_text("specs/MCP_API_SPEC.md")

    check_single_status_authority(plan_text, program_text, integrated_text, failures)
    check_progress_sections(progress_text, failures)
    check_infrastructure_rows(progress_text, failures)
    check_r1_g0_gate(progress_text, failures)
    check_m4_gate_parity(program_text, integrated_text, progress_text, failures)
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
