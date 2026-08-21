#!/usr/bin/env python3
"""Hermetic mutations for the terminal split boundary guard."""

import importlib.util
import unittest
from pathlib import Path


MODULE = Path(__file__).with_name("check_terminal_split_boundary.py")
SPEC = importlib.util.spec_from_file_location("terminal_split_boundary", MODULE)
assert SPEC and SPEC.loader
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


def valid_sources() -> dict[str, str]:
    return {
        name: (guard.ROOT / relative).read_text(encoding="utf-8")
        for name, relative in guard.PATHS.items()
    }


class TerminalSplitBoundaryTest(unittest.TestCase):
    def test_current_tree_passes(self) -> None:
        self.assertEqual([], guard.check_sources(valid_sources()))

    def test_ownership_and_proof_removal_fails(self) -> None:
        sources = valid_sources()
        sources["protocol"] = sources["protocol"].replace("pub enum TerminalSplitChild", "removed")
        sources["viewport"] = sources["viewport"].replace("pub fn terminal_split_dividers", "removed")
        sources["renderer"] = sources["renderer"].replace(
            "HitTarget::TerminalSplitDivider(divider.path)", "removed"
        )
        sources["drag"] = sources["drag"].replace(
            "terminal_split_drag_tracks_axis_and_clamps_to_ten_ninety", "removed"
        )
        self.assertGreaterEqual(len(guard.check_sources(sources)), 4)

    def test_pointer_motion_cannot_resize_pty(self) -> None:
        sources = valid_sources()
        marker = "        self.sync_terminal_tabs();"
        sources["runtime"] = sources["runtime"].replace(
            marker, marker + "\n        self.resize_terminal_to_dock();", 1
        )
        self.assertIn(
            "terminal split drag performs PTY resize during pointer motion",
            guard.check_sources(sources),
        )

    def test_release_and_mouse_routing_order_are_pinned(self) -> None:
        sources = valid_sources()
        sources["runtime"] = sources["runtime"].replace(
            "        self.resize_terminal_to_dock();", "        // removed", 1
        )
        failures = guard.check_sources(sources)
        self.assertIn("terminal split drag release must commit exactly one PTY resize", failures)

        sources = valid_sources()
        begin = "if runtime.begin_terminal_split_drag()"
        report = ".report_terminal_mouse_button(MouseButton::Left, ElementState::Pressed)"
        sources["main"] = (
            sources["main"]
            .replace(begin, "__BEGIN_SPLIT_DRAG__", 1)
            .replace(report, begin, 1)
            .replace("__BEGIN_SPLIT_DRAG__", report, 1)
        )
        self.assertIn(
            "mouse-aware child can consume terminal split-divider press",
            guard.check_sources(sources),
        )


if __name__ == "__main__":
    unittest.main()
