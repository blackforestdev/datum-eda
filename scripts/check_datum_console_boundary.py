#!/usr/bin/env python3
"""Fail when the output-only Datum Console boundary regresses."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROTOCOL = ROOT / "crates/gui-protocol/src/console_feedback.rs"
RENDERER = ROOT / "crates/gui-render/src/datum_console.rs"
APP_ROOT = ROOT / "crates/gui-app/src"
UNTYPED_CONSOLE_CALL = re.compile(
    r"\blog_console_(?:echo|refusal|tool_prompt)\s*\((?P<body>.*?)\n\s*\);",
    re.DOTALL,
)
INTERNAL_INTERPOLATION = re.compile(
    r"\{(?P<name>[A-Za-z_][A-Za-z0-9_]*)[^}]*\}"
)
INTERNAL_NAMES = {
    "action_id",
    "artifact_id",
    "command",
    "event_label",
    "fingerprint",
    "layer_id",
    "object_id",
    "other",
    "pane_id",
    "path",
    "target",
    "target_id",
}


def fail(message: str) -> None:
    print(f"DATUM-CONSOLE-BOUNDARY: FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def code_without_comments_or_strings(source: str) -> str:
    """Keep Rust identifiers/punctuation while blanking comments and literals."""
    out: list[str] = []
    index = 0
    block_depth = 0
    while index < len(source):
        pair = source[index : index + 2]
        if block_depth:
            if pair == "/*":
                block_depth += 1
                index += 2
            elif pair == "*/":
                block_depth -= 1
                index += 2
            else:
                index += 1
            out.append(" ")
            continue
        if pair == "//":
            newline = source.find("\n", index)
            if newline < 0:
                break
            out.append("\n")
            index = newline + 1
            continue
        if pair == "/*":
            block_depth = 1
            out.append(" ")
            index += 2
            continue
        if source[index] in {'"', "'"}:
            quote = source[index]
            out.append(" ")
            index += 1
            while index < len(source):
                if source[index] == "\\":
                    index += 2
                    continue
                if source[index] == quote:
                    index += 1
                    break
                index += 1
            continue
        out.append(source[index])
        index += 1
    return "".join(out)


def assert_state_has_no_authoring_or_terminal_api() -> None:
    code = code_without_comments_or_strings(PROTOCOL.read_text())
    forbidden = [
        "ApplicationFocus",
        "SessionCommand",
        "Operation",
        "OperationBatch",
        "TerminalLaneState",
        "TerminalCell",
        "write_bytes",
        "write_foreign_shell_bytes",
        "dispatch_verb",
        "commit",
    ]
    for identifier in forbidden:
        if re.search(rf"\b{re.escape(identifier)}\b", code):
            fail(f"typed Console state references forbidden identifier {identifier}")

    category = re.search(
        r"enum\s+ConsoleFeedbackCategory\s*\{(?P<body>[^}}]+)\}", code, re.DOTALL
    )
    if category is None:
        fail("ConsoleFeedbackCategory enum is missing")
    variants = {
        value.strip().split()[0]
        for value in category.group("body").split(",")
        if value.strip()
    }
    expected = {"ActionEcho", "ToolPrompt", "ActionRefusal"}
    if variants != expected:
        fail(f"Console category surface changed: expected {sorted(expected)}, got {sorted(variants)}")


def assert_renderer_is_consumer_only() -> None:
    code = code_without_comments_or_strings(RENDERER.read_text())
    for identifier in [
        "ApplicationFocus",
        "SessionCommand",
        "Operation",
        "OperationBatch",
        "TerminalLaneState",
        "write_bytes",
        "dispatch_verb",
        "commit",
    ]:
        if re.search(rf"\b{re.escape(identifier)}\b", code):
            fail(f"Console renderer references forbidden identifier {identifier}")


def assert_producer_routes_remain_separated() -> None:
    app_sources = list(APP_ROOT.glob("*.rs"))
    for path in app_sources:
        code = code_without_comments_or_strings(path.read_text())
        if re.search(r"\blog_review_event\b", code):
            fail(f"legacy catch-all narration returned in {path.relative_to(ROOT)}")

    terminal_owned = [
        *APP_ROOT.glob("runtime_terminal_*.rs"),
        *APP_ROOT.glob("terminal_*.rs"),
        APP_ROOT / "application_terminal_shutdown.rs",
    ]
    for path in terminal_owned:
        if path.exists() and re.search(
            r"\blog_console_\w*\b", code_without_comments_or_strings(path.read_text())
        ):
            fail(f"terminal-owned producer routes into Console in {path.relative_to(ROOT)}")

    production_refresh = APP_ROOT / "production_status_refresh.rs"
    if re.search(
        r"\blog_console_\w*\b",
        code_without_comments_or_strings(production_refresh.read_text()),
    ):
        fail("background production refresh routes into Console")

    runtime = (APP_ROOT / "main.rs").read_text()
    marker = "HitTarget::CheckFinding(fingerprint) =>"
    if marker not in runtime:
        fail("check-finding routing branch was not found")
    finding_branch = runtime.split(marker, 1)[1].split("HitTarget::", 1)[0]
    if "log_console_" in finding_branch or "publish_console_feedback" in finding_branch:
        fail("ERC/DRC finding branch routes into Console")


def assert_internal_ids_stay_typed() -> None:
    """Reject internal identity interpolation through untyped prose helpers."""
    for path in APP_ROOT.glob("*.rs"):
        source = path.read_text()
        for call in UNTYPED_CONSOLE_CALL.finditer(source):
            names = {
                match.group("name")
                for match in INTERNAL_INTERPOLATION.finditer(call.group("body"))
            }
            leaked = sorted(names & INTERNAL_NAMES)
            if leaked:
                fail(
                    f"Console prose interpolates internal identity {', '.join(leaked)} in "
                    f"{path.relative_to(ROOT)}; use a typed action_id/target_id helper"
                )

    boundary = (APP_ROOT / "console_feedback.rs").read_text()
    for helper, typed_builder in [
        ("log_console_echo_for_action", ".with_action_id("),
        ("log_console_echo_for_target", ".with_target_id("),
        ("log_console_refusal_for_action", ".with_action_id("),
        ("log_console_refusal_for_target", ".with_target_id("),
    ]:
        if helper not in boundary or typed_builder not in boundary:
            fail(f"typed Console publication helper is incomplete: {helper}")


def main() -> int:
    assert_state_has_no_authoring_or_terminal_api()
    assert_renderer_is_consumer_only()
    assert_producer_routes_remain_separated()
    assert_internal_ids_stay_typed()
    print(
        "Datum Console boundary passed: typed consequence-only state, consumer-only renderer, "
        "and terminal/progress/finding routes remain separated."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
