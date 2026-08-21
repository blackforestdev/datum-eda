#!/usr/bin/env python3
"""Guard Datum-owned terminal split identity, rendering, and divider gestures."""

from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]

PATHS = {
    "main": "crates/gui-app/src/main.rs",
    "input": "crates/gui-app/src/terminal_input.rs",
    "controls": "crates/gui-app/src/terminal_session_controls.rs",
    "spawn": "crates/gui-app/src/terminal_session_spawn.rs",
    "state": "crates/gui-app/src/terminal_split_state.rs",
    "drag": "crates/gui-app/src/terminal_split_drag.rs",
    "runtime": "crates/gui-app/src/runtime_terminal_dock.rs",
    "focus_tests": "crates/gui-app/src/terminal_regression_boundary_tests.rs",
    "protocol": "crates/gui-protocol/src/terminal_split.rs",
    "viewport": "crates/gui-viewport/src/terminal_grid_geometry.rs",
    "render_types": "crates/gui-render/src/render/types.rs",
    "renderer": "crates/gui-render/src/bottom_dock.rs",
    "render_tests": "crates/gui-render/src/terminal_core_render_tests.rs",
}


def function_body(source: str, marker: str, next_marker: str) -> str:
    if marker not in source or next_marker not in source.split(marker, 1)[1]:
        return ""
    return source.split(marker, 1)[1].split(next_marker, 1)[0]


def check_sources(sources: dict[str, str]) -> list[str]:
    failures: list[str] = []
    required = {
        "input": (
            "TerminalKeyAction::SplitRight",
            "TerminalKeyAction::SplitDown",
            "terminal_split_shortcut(",
            "KeyCode::KeyO",
            "KeyCode::KeyE",
        ),
        "controls": (
            "spawn_terminal_split",
            "begin_split_and_activate",
            "resize_terminal_to_dock",
        ),
        "main": (
            "TerminalKeyAction::SplitRight =>",
            "TerminalKeyAction::SplitDown =>",
            "HitTarget::TerminalPaneScreen(session_id)",
            "begin_terminal_split_drag()",
            "advance_terminal_split_drag(next_pos)",
            "finish_terminal_split_drag()",
        ),
        "spawn": (
            "split_spawn_stays_in_one_tab_and_focuses_the_completed_leaf",
            "failed_split_spawn_removes_its_leaf_without_removing_the_tab",
        ),
        "state": ("replace_terminal_session_identity", "set_active_split_ratio"),
        "protocol": (
            "pub enum TerminalSplitChild",
            "set_ratio_at_path",
            "recursive_tree_preserves_leaf_order_and_focus_identity",
        ),
        "viewport": (
            "pub fn terminal_split_dividers",
            "recursive_splits_have_distinct_identity_whole_cells_and_gutters",
        ),
        "render_types": ("TerminalSplitDivider(Vec<datum_gui_protocol::TerminalSplitChild>)",),
        "renderer": (
            "render_pane(",
            "HitTarget::TerminalPaneScreen",
            "HitTarget::TerminalSplitDivider(divider.path)",
            "split_divider_renders_one_path_stable_resize_target_in_its_gutter",
        ),
        "render_tests": ("split_panes_retain_independent_rows_geometry_and_hit_identity",),
        "drag": ("terminal_split_drag_tracks_axis_and_clamps_to_ten_ninety",),
        "runtime": ("terminal_split_drag_previews_layout_and_commits_one_pty_resize_on_release",),
        "focus_tests": ("HitTarget::TerminalSplitDivider(Vec::new())",),
    }
    for owner, markers in required.items():
        for marker in markers:
            if marker not in sources.get(owner, ""):
                failures.append(f"terminal split {owner} ownership is missing {marker}")

    runtime = sources.get("runtime", "")
    advance = function_body(
        runtime,
        "pub(super) fn advance_terminal_split_drag",
        "pub(super) fn finish_terminal_split_drag",
    )
    finish = function_body(
        runtime,
        "pub(super) fn finish_terminal_split_drag",
        "pub(super) fn dock_resize_cursor_icon",
    )
    if not advance or "set_active_split_ratio" not in advance or "invalidate_frame" not in advance:
        failures.append("terminal split drag does not preview the persistent layout")
    if "resize_terminal_to_dock" in advance:
        failures.append("terminal split drag performs PTY resize during pointer motion")
    if finish.count("resize_terminal_to_dock") != 1:
        failures.append("terminal split drag release must commit exactly one PTY resize")

    main = sources.get("main", "")
    press = function_body(
        main,
        "state: ElementState::Pressed,\n                button: MouseButton::Left",
        "state: ElementState::Released",
    )
    if not press:
        failures.append("terminal split primary-press routing boundary is missing")
    elif press.find("begin_terminal_split_drag") > press.find("report_terminal_mouse_button"):
        failures.append("mouse-aware child can consume terminal split-divider press")
    move = function_body(main, "WindowEvent::CursorMoved", "WindowEvent::MouseWheel")
    if not move:
        failures.append("terminal split pointer-motion routing boundary is missing")
    elif move.find("advance_terminal_split_drag") > move.find("report_terminal_mouse_motion"):
        failures.append("mouse-aware child can consume terminal split-divider motion")
    return failures


def main() -> int:
    sources = {
        name: (ROOT / relative).read_text(encoding="utf-8")
        for name, relative in PATHS.items()
    }
    failures = check_sources(sources)
    if failures:
        for failure in failures:
            print(f"terminal split boundary: {failure}", file=sys.stderr)
        return 1
    print("terminal split boundary: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
