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
        "TerminalKeyAction::Search",
        "terminal_search_shortcut",
    ),
    "main.rs": (
        "encode_active_key",
        "encode_active_focus",
        "encode_active_paste",
        "arm_terminal_link_at_cursor",
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
    "runtime_terminal_dock.rs": (
        "toggle_terminal_maximized",
        "effective_dock_height_px",
        "terminal_maximize_is_transient_and_preserves_the_normal_dock_height",
    ),
    "runtime_view_actions.rs": (
        "terminal_owns_maximize",
        "self.toggle_terminal_maximized()",
        "maximize_action_follows_the_application_focus_authority",
    ),
    "terminal_session_interaction.rs": (
        ".core.encode_key",
        ".core.encode_mouse",
        ".core.copy_selection",
        "pub(crate) fn search_all_active",
        ".core.search_all(query)",
        "pub(crate) fn active_search_match_state",
        "pub(crate) fn active_hyperlink_at",
        "pub(crate) fn active_link_target_at",
        "pub(crate) fn active_accessibility_snapshot",
    ),
    "terminal_core_adapter_interaction.rs": (
        "pub(crate) fn encode_key",
        "pub(crate) fn encode_ime",
        "pub(crate) fn encode_paste",
        "pub(crate) fn encode_mouse",
        "pub(crate) fn set_selection",
        "pub(crate) fn copy_selection",
        "pub(crate) fn search_all",
        "pub(crate) fn search_match_state",
        "TerminalAccessibilitySnapshot",
        "pub(crate) fn link_target_at_visible_cell",
    ),
    "runtime_terminal_search.rs": (
        "handle_terminal_search_key",
        "search_all_active",
        "maintain_terminal_search_after_output",
        "escape_release_pending",
        "search_navigation_wraps_and_stable_refresh_preserves_current_match",
    ),
    "runtime_terminal_links.rs": (
        "validate_http_target",
        'Command::new("/usr/bin/xdg-open")',
        "handle_terminal_link_confirmation_key",
        "only_exact_http_and_https_targets_are_openable",
        "confirmation_owns_all_keys_and_the_matching_escape_release",
    ),
    "runtime_terminal_clipboard.rs": (
        "terminal_link_target_at_cursor",
        "terminal_clipboard_link_target",
        "handle_terminal_clipboard_write_request",
        "decode_clipboard_text",
        "clipboard_request_is_eligible",
        "clipboard_confirmation_exclusively_owns_enter_escape_and_other_keys",
        "only_a_focused_active_running_session_may_arm_osc52_confirmation",
    ),
    "terminal_session_drain.rs": (
        "TerminalClipboardWriteRequest",
        "CoreEvent::ClipboardRequest",
        "TerminalNotificationRequest",
        "CoreEvent::Notification",
    ),
    "terminal_session_drain_tests.rs": (
        "osc52_becomes_a_typed_session_scoped_request_without_changing_cells",
    ),
    "terminal_session_controls.rs": (
        "context_for_new_terminal",
        "current_working_directory",
    ),
    "terminal_session.rs": (
        "label_is_explicit",
        "terminal_tab_label",
    ),
    "terminal_session_naming_tests.rs": (
        "shell_titles_drive_tabs_until_the_user_renames_them",
        "inactive_shell_title_stays_with_its_parked_session",
        "progress_and_latest_notification_are_visible_in_their_session_tab",
    ),
    "runtime_terminal_notifications.rs": (
        "DATUM_TERMINAL_NOTIFICATIONS",
        "TerminalNotificationPolicy",
        "notification_policy_matches_off_unfocused_and_always_contract",
        'Command::new("/usr/bin/notify-send")',
        "sync_channel::<DesktopNotification>(PRODUCTION_CORE_LIMIT_VALUES.pending_events)",
        ".status()",
    ),
    "terminal_working_directory.rs": (
        "context_for_new_terminal",
        "local_working_directory",
        "percent_decode_path",
        "new_terminal_inherits_local_osc7_directory_without_changing_project_identity",
        "remote_malformed_and_stale_reports_fall_back_to_project_root",
    ),
    "terminal_accessibility_bridge.rs": (
        "TerminalAccessibilityEvent",
        "TextChanged",
        "CaretMoved",
        "SelectionChanged",
        "FocusChanged",
        "refresh_terminal_accessibility",
    ),
    "terminal_accessibility_platform/mod.rs": (
        "pub(crate) use worker::PlatformBridge",
        "mod atspi",
        "mod connection",
        "mod dbus",
    ),
    "terminal_accessibility_platform/connection.rs": (
        "AUTH EXTERNAL",
        "org.a11y.Bus",
        "call_blocking_with",
        "wait_fd",
    ),
    "terminal_accessibility_platform/dbus.rs": (
        "MAX_MESSAGE_BYTES",
        "FrameBuffer",
        "D-Bus frame exceeds Datum limit",
    ),
    "terminal_accessibility_platform/body.rs": (
        "validate_signature",
        "parse_complete_type",
        "D-Bus signature nesting exceeds Datum limit",
    ),
    "terminal_accessibility_platform/atspi.rs": (
        'REGISTRY_PATH: &str = "/org/a11y/atspi/accessible/root"',
        "org.a11y.atspi.Accessible",
        "org.a11y.atspi.Application",
        "org.a11y.atspi.Component",
        "org.a11y.atspi.Text",
        "org.a11y.atspi.Hypertext",
    ),
    "terminal_accessibility_platform/events.rs": (
        "org.a11y.atspi.Event.Object",
        "TextCaretMoved",
        "TextSelectionChanged",
        "StateChanged",
    ),
    "terminal_accessibility_platform/worker.rs": (
        'name("datum-atspi"',
        "real_accessibility_bus_accepts_datum_registration",
        "pending_updates_replace_snapshot_and_coalesce_event_kinds",
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

FORBIDDEN_ACCESSIBILITY_SUBSTRATE = (
    "accesskit",
    "use zbus::",
    "use dbus::",
    "libatspi",
    "std::process::Command",
    "datum_terminal_core",
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

        if '/usr/bin/xdg-open' in source and name != "runtime_terminal_links.rs":
            failures.append(f"terminal desktop handoff escaped its policy owner: {name}")

        if name.startswith("terminal_accessibility_platform/"):
            lowered = source.lower()
            for marker in FORBIDDEN_ACCESSIBILITY_SUBSTRATE:
                if marker.lower() in lowered:
                    failures.append(
                        f"Datum-owned accessibility bridge imported forbidden substrate: {name}:{marker}"
                    )

    for marker in (
        "ime_preedit",
        "render_ime_preedit",
        "snapshot.cursor().position",
        "search_highlights",
        "TERMINAL_SEARCH_ALL_BG",
    ):
        if marker not in render:
            failures.append(f"native terminal IME rendering is missing {marker}")


def main() -> int:
    sources = {
        str(path.relative_to(APP)): path.read_text(encoding="utf-8")
        for path in APP.rglob("*.rs")
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
