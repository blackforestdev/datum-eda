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


def valid_render() -> str:
    return " ".join(
        (
            "ime_preedit",
            "render_ime_preedit",
            "snapshot.cursor().position",
            "search_highlights",
            "TERMINAL_SEARCH_ALL_BG",
            *guard.RENDER_REQUIRED,
        )
    )


class TerminalNativeInteractionGuardTest(unittest.TestCase):
    def test_valid_multifile_boundary_passes(self) -> None:
        failures: list[str] = []
        guard.check(
            valid_sources(),
            valid_render(),
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

    def test_link_confirmation_and_desktop_handoff_cannot_bypass_policy_owner(self) -> None:
        sources = valid_sources()
        sources["runtime_terminal_links.rs"] = sources[
            "runtime_terminal_links.rs"
        ].replace("validate_http_target", "open_every_scheme")
        sources["main.rs"] += '\nCommand::new("/usr/bin/xdg-open");'
        failures: list[str] = []
        guard.check(
            sources,
            "ime_preedit render_ime_preedit snapshot.cursor().position search_highlights TERMINAL_SEARCH_ALL_BG",
            failures,
        )
        self.assertTrue(any("validate_http_target" in failure for failure in failures))
        self.assertTrue(any("desktop handoff escaped" in failure for failure in failures))

    def test_new_session_cwd_cannot_bypass_local_path_policy(self) -> None:
        sources = valid_sources()
        sources["terminal_session_controls.rs"] = sources[
            "terminal_session_controls.rs"
        ].replace("context_for_new_terminal", "clone_project_context")
        sources["terminal_working_directory.rs"] = sources[
            "terminal_working_directory.rs"
        ].replace("percent_decode_path", "use_uri_as_path")
        failures: list[str] = []
        guard.check(
            sources,
            "ime_preedit render_ime_preedit snapshot.cursor().position search_highlights TERMINAL_SEARCH_ALL_BG",
            failures,
        )
        self.assertTrue(any("context_for_new_terminal" in failure for failure in failures))
        self.assertTrue(any("percent_decode_path" in failure for failure in failures))

    def test_terminal_maximize_cannot_fall_back_to_the_editor_pane(self) -> None:
        sources = valid_sources()
        sources["runtime_view_actions.rs"] = sources[
            "runtime_view_actions.rs"
        ].replace("self.toggle_terminal_maximized()", "self.pane_toggle_zoom()")
        sources["runtime_terminal_dock.rs"] = sources[
            "runtime_terminal_dock.rs"
        ].replace("effective_dock_height_px", "dock_height_px")
        failures: list[str] = []
        guard.check(
            sources,
            "ime_preedit render_ime_preedit snapshot.cursor().position search_highlights TERMINAL_SEARCH_ALL_BG",
            failures,
        )
        self.assertTrue(any("toggle_terminal_maximized" in failure for failure in failures))
        self.assertTrue(any("effective_dock_height_px" in failure for failure in failures))

    def test_shell_title_cannot_override_an_explicit_tab_rename(self) -> None:
        sources = valid_sources()
        sources["terminal_session.rs"] = sources["terminal_session.rs"].replace(
            "label_is_explicit", "always_follow_shell_title"
        )
        sources["terminal_session_naming_tests.rs"] = sources[
            "terminal_session_naming_tests.rs"
        ].replace(
            "shell_titles_drive_tabs_until_the_user_renames_them",
            "deleted_title_authority_proof",
        )
        failures: list[str] = []
        guard.check(
            sources,
            "ime_preedit render_ime_preedit snapshot.cursor().position search_highlights TERMINAL_SEARCH_ALL_BG",
            failures,
        )
        self.assertTrue(any("label_is_explicit" in failure for failure in failures))
        self.assertTrue(any("shell_titles_drive_tabs" in failure for failure in failures))

    def test_osc52_cannot_bypass_focused_confirmation_or_disappear_in_adapter(self) -> None:
        sources = valid_sources()
        sources["runtime_terminal_clipboard.rs"] = sources[
            "runtime_terminal_clipboard.rs"
        ].replace("clipboard_request_is_eligible", "allow_every_session")
        sources["terminal_session_drain.rs"] = sources[
            "terminal_session_drain.rs"
        ].replace("CoreEvent::ClipboardRequest", "ignored_clipboard_event")
        sources["terminal_session_drain_tests.rs"] = sources[
            "terminal_session_drain_tests.rs"
        ].replace(
            "osc52_becomes_a_typed_session_scoped_request_without_changing_cells",
            "deleted_osc52_adapter_proof",
        )
        failures: list[str] = []
        guard.check(
            sources,
            "ime_preedit render_ime_preedit snapshot.cursor().position search_highlights TERMINAL_SEARCH_ALL_BG",
            failures,
        )
        self.assertTrue(any("clipboard_request_is_eligible" in failure for failure in failures))
        self.assertTrue(any("CoreEvent::ClipboardRequest" in failure for failure in failures))
        self.assertTrue(any("osc52_becomes" in failure for failure in failures))

    def test_notifications_and_progress_cannot_return_to_an_invisible_sink(self) -> None:
        sources = valid_sources()
        sources["terminal_session_drain.rs"] = sources[
            "terminal_session_drain.rs"
        ].replace("CoreEvent::Notification", "ignore_notification")
        sources["runtime_terminal_notifications.rs"] = sources[
            "runtime_terminal_notifications.rs"
        ].replace("DATUM_TERMINAL_NOTIFICATIONS", "hard_coded_policy").replace(
            "sync_channel::<DesktopNotification>(PRODUCTION_CORE_LIMIT_VALUES.pending_events)",
            "unbounded_notification_queue()",
        )
        sources["terminal_session_naming_tests.rs"] = sources[
            "terminal_session_naming_tests.rs"
        ].replace(
            "progress_and_latest_notification_are_visible_in_their_session_tab",
            "deleted_progress_projection_proof",
        )
        failures: list[str] = []
        guard.check(
            sources,
            "ime_preedit render_ime_preedit snapshot.cursor().position search_highlights TERMINAL_SEARCH_ALL_BG",
            failures,
        )
        self.assertTrue(any("CoreEvent::Notification" in failure for failure in failures))
        self.assertTrue(any("DATUM_TERMINAL_NOTIFICATIONS" in failure for failure in failures))
        self.assertTrue(any("sync_channel" in failure for failure in failures))
        self.assertTrue(any("progress_and_latest" in failure for failure in failures))

    def test_inactive_output_and_bell_attention_cannot_become_color_only(self) -> None:
        sources = valid_sources()
        sources["terminal_session_drain.rs"] = sources[
            "terminal_session_drain.rs"
        ].replace("slot.unread_output = true", "discard_inactive_attention")
        render = valid_render().replace(
            "inactive_tabs_use_non_color_output_and_bell_attention_markers",
            "deleted_non_color_attention_proof",
        )
        failures: list[str] = []
        guard.check(sources, render, failures)
        self.assertTrue(any("slot.unread_output" in failure for failure in failures))
        self.assertTrue(any("non_color_output" in failure for failure in failures))


if __name__ == "__main__":
    unittest.main()
