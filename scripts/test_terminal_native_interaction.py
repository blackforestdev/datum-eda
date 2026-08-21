#!/usr/bin/env python3
"""Hermetic mutation tests for the native terminal interaction boundary."""

import importlib.util
import unittest
from pathlib import Path


MODULE = Path(__file__).with_name("check_terminal_native_interaction.py")
SPEC = importlib.util.spec_from_file_location("terminal_native_interaction", MODULE)
assert SPEC and SPEC.loader
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


def valid_sources() -> dict[str, str]:
    return {name: "\n".join(markers) for name, markers in guard.REQUIRED.items()}


class TerminalNativeInteractionGuardTest(unittest.TestCase):
    def test_valid_multifile_boundary_passes(self) -> None:
        failures: list[str] = []
        guard.check(
            valid_sources(),
            "ime_preedit render_ime_preedit snapshot.cursor().position search_highlights TERMINAL_SEARCH_ALL_BG",
            failures,
        )
        self.assertEqual([], failures)

    def test_missing_core_routes_and_ime_projection_fail(self) -> None:
        sources = valid_sources()
        sources["runtime_terminal_pointer.rs"] = sources[
            "runtime_terminal_pointer.rs"
        ].replace("encode_active_mouse", "encode_mouse_locally")
        sources["terminal_accessibility_bridge.rs"] = sources[
            "terminal_accessibility_bridge.rs"
        ].replace("CaretMoved", "removed")
        failures: list[str] = []
        guard.check(sources, "snapshot.cursor().position", failures)
        self.assertTrue(any("encode_active_mouse" in failure for failure in failures))
        self.assertTrue(any("CaretMoved" in failure for failure in failures))
        self.assertTrue(any("ime_preedit" in failure for failure in failures))

    def test_legacy_encoder_or_byte_action_cannot_return_to_production(self) -> None:
        sources = valid_sources()
        sources["terminal_input.rs"] += (
            "\nTerminalKeyAction::Write\nfn terminal_sgr_mouse_button_sequence"
        )
        failures: list[str] = []
        guard.check(
            sources,
            "ime_preedit render_ime_preedit snapshot.cursor().position search_highlights TERMINAL_SEARCH_ALL_BG",
            failures,
        )
        self.assertIn(
            "production terminal input must not retain a byte-encoder action",
            failures,
        )
        self.assertTrue(any("legacy terminal encoder escaped" in failure for failure in failures))

    def test_accessibility_bridge_cannot_lose_limits_or_import_external_runtime(self) -> None:
        sources = valid_sources()
        sources["terminal_accessibility_platform/dbus.rs"] = sources[
            "terminal_accessibility_platform/dbus.rs"
        ].replace("MAX_MESSAGE_BYTES", "unbounded_message")
        sources["terminal_accessibility_platform/worker.rs"] = sources[
            "terminal_accessibility_platform/worker.rs"
        ].replace("real_accessibility_bus_accepts_datum_registration", "deleted_live_proof")
        sources["terminal_accessibility_platform/connection.rs"] += "\nuse zbus::Connection;"
        failures: list[str] = []
        guard.check(
            sources,
            "ime_preedit render_ime_preedit snapshot.cursor().position search_highlights TERMINAL_SEARCH_ALL_BG",
            failures,
        )
        self.assertTrue(any("MAX_MESSAGE_BYTES" in failure for failure in failures))
        self.assertTrue(any("real_accessibility_bus" in failure for failure in failures))
        self.assertTrue(any("forbidden substrate" in failure for failure in failures))

    def test_search_wrap_stability_or_all_match_rendering_removal_fails(self) -> None:
        sources = valid_sources()
        sources["runtime_terminal_search.rs"] = sources[
            "runtime_terminal_search.rs"
        ].replace(
            "search_navigation_wraps_and_stable_refresh_preserves_current_match",
            "deleted_search_proof",
        )
        failures: list[str] = []
        guard.check(
            sources,
            "ime_preedit render_ime_preedit snapshot.cursor().position",
            failures,
        )
        self.assertTrue(any("stable_refresh" in failure for failure in failures))
        self.assertTrue(any("search_highlights" in failure for failure in failures))
        self.assertTrue(any("TERMINAL_SEARCH_ALL_BG" in failure for failure in failures))


if __name__ == "__main__":
    unittest.main()
