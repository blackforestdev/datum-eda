#!/usr/bin/env python3
"""Keep production terminal interactions behind the sole TerminalCore authority."""

from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]
APP = ROOT / "crates/gui-app/src"
RENDER = ROOT / "crates/gui-render/src"

REQUIRED = {
    "terminal_input.rs": (
        "TerminalKeyAction::CoreKey",
        "terminal_core_key_input",
    ),
    "main.rs": (
        "encode_active_key",
        "encode_active_focus",
        "encode_active_paste",
    ),
    "runtime_terminal_input.rs": (
        "handle_terminal_ime",
        "encode_active_ime",
        "terminal_ime_cursor_rect",
    ),
    "runtime_terminal_pointer.rs": (
        "encode_active_mouse",
        "set_active_selection",
        "active_logical_point_at",
    ),
    "terminal_session_interaction.rs": (
        ".core.encode_key",
        ".core.encode_mouse",
        ".core.copy_selection",
        "pub(crate) fn search_active",
        ".search(query, cursor)",
        "pub(crate) fn active_hyperlink_at",
        "pub(crate) fn active_accessibility_snapshot",
    ),
    "terminal_core_adapter_interaction.rs": (
        "pub(crate) fn encode_key",
        "pub(crate) fn encode_ime",
        "pub(crate) fn encode_paste",
        "pub(crate) fn encode_mouse",
        "pub(crate) fn set_selection",
        "pub(crate) fn copy_selection",
        "pub(crate) fn search",
        "TerminalAccessibilitySnapshot",
    ),
    "terminal_accessibility_bridge.rs": (
        "TerminalAccessibilityEvent",
        "TextChanged",
        "CaretMoved",
        "SelectionChanged",
        "FocusChanged",
        "refresh_terminal_accessibility",
    ),
}

LEGACY_ENCODERS = (
    "terminal_sgr_mouse_button_sequence",
    "terminal_x10_mouse_button_sequence",
    "terminal_utf8_mouse_button_sequence",
    "terminal_urxvt_mouse_button_sequence",
    "terminal_character_sequence",
    "terminal_tab_sequence",
)


def check(sources: dict[str, str], render: str, failures: list[str]) -> None:
    for name, markers in REQUIRED.items():
        source = sources.get(name, "")
        for marker in markers:
            if marker not in source:
                failures.append(f"{name} is missing TerminalCore interaction marker {marker}")

    terminal_input = sources.get("terminal_input.rs", "")
    if "TerminalKeyAction::Write" in terminal_input or "ConsumeRelease" in terminal_input:
        failures.append("production terminal input must not retain a byte-encoder action")
    for name, source in sources.items():
        if name.endswith("_tests.rs") or "legacy" in name:
            continue
        for marker in LEGACY_ENCODERS:
            if f"fn {marker}" in source:
                failures.append(f"legacy terminal encoder escaped test-only modules: {name}:{marker}")

    for marker in ("ime_preedit", "render_ime_preedit", "snapshot.cursor().position"):
        if marker not in render:
            failures.append(f"native terminal IME rendering is missing {marker}")


def main() -> int:
    sources = {
        path.name: path.read_text(encoding="utf-8")
        for path in APP.glob("*.rs")
    }
    render = (RENDER / "terminal_core_render.rs").read_text(encoding="utf-8")
    failures: list[str] = []
    check(sources, render, failures)
    if failures:
        print("Terminal native-interaction boundary FAILED:")
        for failure in failures:
            print(f"- {failure}")
        return 1
    print("Terminal native-interaction boundary passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
