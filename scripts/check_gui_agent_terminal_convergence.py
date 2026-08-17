#!/usr/bin/env python3
"""Guard that GUI agent entry points stay inside the PTY terminal lane."""

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
MAIN = ROOT / "crates" / "gui-app" / "src" / "main.rs"
KEYBOARD_FOCUS = ROOT / "crates" / "gui-app" / "src" / "keyboard_focus.rs"
BOTTOM_DOCK = ROOT / "crates" / "gui-render" / "src" / "bottom_dock.rs"
LAUNCHER = ROOT / "crates" / "gui-app" / "src" / "terminal_agent_launcher.rs"
TERMINAL_CONTROLS = ROOT / "crates" / "gui-app" / "src" / "terminal_session_controls.rs"
TERMINAL_SESSION_SPAWN = ROOT / "crates" / "gui-app" / "src" / "terminal_session_spawn.rs"
RUNTIME_TERMINAL_CONTEXT = ROOT / "crates" / "gui-app" / "src" / "runtime_terminal_context.rs"
PRODUCTION_REFRESH = ROOT / "crates" / "gui-app" / "src" / "production_status_refresh.rs"
RUNTIME_TERMINAL_DOCK = ROOT / "crates" / "gui-app" / "src" / "runtime_terminal_dock.rs"
TERMINAL_DRAIN = ROOT / "crates" / "gui-app" / "src" / "terminal_session_drain.rs"
TERMINAL_DRAIN_TESTS = ROOT / "crates" / "gui-app" / "src" / "terminal_session_drain_tests.rs"
TERMINAL_CLOSE_TESTS = ROOT / "crates" / "gui-app" / "src" / "terminal_session_close_tests.rs"
GUI_PROTOCOL = ROOT / "crates" / "gui-protocol" / "src" / "lib.rs"
TERMINAL_LANE = ROOT / "crates" / "gui-protocol" / "src" / "terminal_lane.rs"
WORKSPACE_LAYOUT = ROOT / "crates" / "gui-protocol" / "src" / "workspace_layout.rs"
RENDER_LAYOUT = ROOT / "crates" / "gui-render" / "src" / "render" / "layout.rs"
RENDER_SCENE = ROOT / "crates" / "gui-render" / "src" / "render" / "scene.rs"
HIT_CLIPPING = ROOT / "crates" / "gui-render" / "src" / "render" / "hit_clipping.rs"
TERMINAL_FOCUS_TESTS = ROOT / "crates" / "gui-app" / "src" / "terminal_focus_convergence_tests.rs"
TERMINAL_HIT_TESTS = ROOT / "crates" / "gui-render" / "src" / "terminal_hit_ownership_tests.rs"
TERMINAL_ESCAPE = ROOT / "crates" / "gui-app" / "src" / "terminal_screen" / "terminal_escape.rs"
TERMINAL_SCREEN_BASIC_TESTS = ROOT / "crates" / "gui-app" / "src" / "terminal_screen" / "terminal_screen_basic_tests.rs"
RENDER_GEOMETRY = ROOT / "crates" / "gui-render" / "src" / "render" / "geometry.rs"
TEXT_BUFFER_CACHE = ROOT / "crates" / "gui-render" / "src" / "render" / "text_buffer_cache.rs"
RENDER_GPU = ROOT / "crates" / "gui-render" / "src" / "render" / "gpu.rs"
TERMINAL_FONT_TESTS = ROOT / "crates" / "gui-render" / "src" / "terminal_font_tests.rs"
TERMINAL_CURSOR = ROOT / "crates" / "gui-render" / "src" / "terminal_cursor.rs"
TERMINAL_TAB_STRIP = ROOT / "crates" / "gui-render" / "src" / "terminal_tab_strip.rs"
TERMINAL_TAB_STRIP_TESTS = ROOT / "crates" / "gui-render" / "src" / "terminal_tab_strip_tests.rs"
TERMINAL_GRID_GEOMETRY = ROOT / "crates" / "gui-viewport" / "src" / "terminal_grid_geometry.rs"
TERMINAL_INPUT = ROOT / "crates" / "gui-app" / "src" / "terminal_input.rs"
TERMINAL_SESSION = ROOT / "crates" / "gui-app" / "src" / "terminal_session.rs"
TERMINAL_SESSION_NAMING_TESTS = ROOT / "crates" / "gui-app" / "src" / "terminal_session_naming_tests.rs"
TERMINAL_TRANSPORT = ROOT / "crates" / "gui-app" / "src" / "terminal_transport"
RETIRED_BRIDGE_FILES = [
    ROOT / "crates" / "gui-app" / "src" / "assistant_bridge.rs",
    ROOT / "scripts" / "datum_assistant_bridge.py",
]


def check_terminal_grid_writers(failures: list[str]) -> None:
    """Keep the public cross-crate grid gateway inside PTY interpretation."""
    declaration = Path("crates/gui-protocol/src/terminal_lane.rs")
    terminal_core = Path("crates/gui-app/src/terminal_screen")
    for path in sorted((ROOT / "crates").rglob("*.rs")):
        source = path.read_text(encoding="utf-8")
        if "pty_grid_mut(" not in source:
            continue
        relative = path.relative_to(ROOT)
        is_test = relative.name.endswith("_tests.rs")
        is_terminal_core = relative == Path("crates/gui-app/src/terminal_screen.rs") or (
            terminal_core in relative.parents
        )
        if relative != declaration and not is_test and not is_terminal_core:
            failures.append(
                f"terminal grid mutation escaped PTY interpretation: {relative}"
            )


def check_terminal_focus_reporting(
    main: str,
    authority: str,
    workspace_layout: str,
    terminal_lane: str,
    mutation_sources: str,
    failures: list[str],
) -> None:
    """Keep one application focus authority for editors, terminal, and overlays."""
    window_marker = "            WindowEvent::Focused(focused) => {"
    setter_marker = "    pub(crate) fn set_application_focus"
    if window_marker not in main or setter_marker not in authority:
        failures.append("terminal focus-report authority markers are missing")
        return
    window_body = main.split(window_marker, 1)[1].split("            WindowEvent::", 1)[0]
    setter_body = authority.split(setter_marker, 1)[1].split("\n    }", 1)[0]
    if "report_terminal_focus_event" in window_body:
        failures.append("OS window focus must not emit terminal focus-report bytes")
    if "terminal_focus_report_transition" not in setter_body:
        failures.append("application-focus transitions must own terminal focus reporting")
    for marker in (
        "pub enum ApplicationFocus",
        "Editor(PaneId)",
        "Terminal",
        "Overlay",
    ):
        if marker not in workspace_layout:
            failures.append(f"shared application focus is missing {marker}")
    if re.search(r"\bkeyboard_focus\s*:(?!:)", main) or "self.keyboard_focus" in mutation_sources:
        failures.append("runtime must not retain a rival keyboard-focus field")
    if "has_keyboard_focus" in terminal_lane:
        failures.append("terminal lane must not retain a rival focus projection")
    assignments = re.findall(r"\.ui\.focus\s*=(?!=)", mutation_sources)
    if len(assignments) != 2:
        failures.append(
            "application focus must mutate only through initialization and set_application_focus"
        )


def check_terminal_hit_ownership(
    render_layout: str,
    render_scene: str,
    hit_clipping: str,
    focus_tests: str,
    hit_tests: str,
    failures: list[str],
) -> None:
    """Clip editor hits and prove terminal selection through production routing."""
    for marker in ("pub fn intersect", "right > x", "bottom > y"):
        if marker not in render_layout:
            failures.append(f"viewport hit clipping is missing {marker}")
    for marker in ("scene_hit_start", "hit_clipping::clip_new_hit_regions"):
        if marker not in render_scene:
            failures.append(f"editor scene hit clipping is missing {marker}")
    for marker in (".drain(first_new_region..)", "region.rect.intersect(viewport)"):
        if marker not in hit_clipping:
            failures.append(f"editor scene hit clipping is missing {marker}")
    for marker in (
        "non_mouse_child_click_selects_terminal_and_tab_never_cycles_editor_panes",
        "mouse_reporting_press_selects_same_terminal_authority_before_forwarding",
        "workspace_action_should_fire",
        "terminal_tab_sequence",
    ):
        if marker not in focus_tests:
            failures.append(f"terminal focus convergence proof is missing {marker}")
    if "editor_scene_hits_cannot_shadow_terminal_screen_at_adversarial_cameras" not in hit_tests:
        failures.append("adversarial editor-hit ownership proof is missing")


def check_workspace_hotkey_timing(authority: str, failures: list[str]) -> None:
    """Keep editor actions on initial Press and outside terminal ownership."""
    if "workspace_action_should_fire(focus, dock_visible, event.state, event.repeat)" not in authority:
        failures.append("workspace hotkeys must use the focus-aware initial-Press predicate")
    timing_body = authority.split("pub(crate) fn workspace_action_should_fire", 1)
    if len(timing_body) != 2:
        return
    timing_body = timing_body[1].split("\n}", 1)[0]
    for marker in ("ElementState::Pressed", "!repeat", "KeyClass::WorkspaceHotkey"):
        if marker not in timing_body:
            failures.append(f"workspace hotkey timing predicate is missing {marker}")
    dispatch_tail = authority.split("// Pane focus cycling", 1)
    if len(dispatch_tail) == 2:
        dispatch_tail = dispatch_tail[1].split("    if escape_released", 1)[0]
        if dispatch_tail.count("workspace_action_pressed") != 2:
            failures.append("pane and character hotkeys must share the Press predicate")
        if "ElementState::Released" in dispatch_tail:
            failures.append("workspace hotkey dispatch must not fire on key release")


def check_claude_completion_controls(
    terminal_escape: str,
    terminal_screen_tests: str,
    failures: list[str],
) -> None:
    """Keep kitty keyboard management distinct from legacy cursor restore."""
    if "b'u' if self.params.is_empty()" not in terminal_escape:
        failures.append("parameterized CSI u must not restore the terminal cursor")
    for marker in (
        "claude_keyboard_controls_cannot_restore_a_stale_cursor",
        "split_claude_keyboard_controls_are_cursor_state_invariant",
        "colored_bash_prompt_places_cursor_after_dollar_and_trailing_space",
        'b"\\x1b[>1u\\x1b[?u\\x1b[<uclaude"',
    ):
        if marker not in terminal_screen_tests:
            failures.append(f"Claude completion control proof is missing {marker}")


def check_agent_tui_runtime(
    main: str,
    runtime_dock: str,
    drain: str,
    render_geometry: str,
    text_cache: str,
    render_gpu: str,
    bottom_dock: str,
    terminal_font_tests: str,
    terminal_cursor: str,
    failures: list[str],
) -> None:
    """Keep mouse-aware agent TUIs focused, responsive, and bounded."""
    window_event = main.split("    fn window_event", 1)
    if len(window_event) != 2:
        failures.append("window-event dispatch boundary is missing")
    else:
        before_match = window_event[1].split("        match event {", 1)[0]
        if "poll_terminal_output" in before_match:
            failures.append("terminal output must not drain before window input dispatch")
    press = main.split("MouseButton::Left,", 1)
    if len(press) != 2:
        failures.append("terminal primary-press routing is missing")
    else:
        press = press[1].split("MouseButton::Left,", 1)[0]
        focus_at = press.find("focus_terminal_screen_before_mouse_report")
        report_at = press.find("report_terminal_mouse_button")
        if focus_at < 0 or report_at < 0 or focus_at > report_at:
            failures.append("terminal focus must precede child mouse-report forwarding")
    for marker in (
        "terminal_mouse_report_allowed",
        "self.terminal_screen_cell_at(x, y)",
    ):
        if marker not in main:
            failures.append(f"terminal mouse routing is missing {marker}")
    for marker in (
        "focus_before_terminal_mouse_press",
        "terminal_screen_cell_at",
    ):
        if marker not in runtime_dock:
            failures.append(f"terminal focus-entry boundary is missing {marker}")
    for marker in (
        "flush_output_batch",
        "tiny_chunk_flood_is_applied_once_per_session_per_turn",
        "slot.remove_when_closed = is_active",
        "natural_shell_exit_removes_its_tab_without_second_close",
    ):
        if marker not in drain:
            failures.append(f"terminal tiny-output batching is missing {marker}")
    for marker in ("JetBrainsMono-Regular.ttf", "TextFace::Terminal"):
        if marker not in render_geometry:
            failures.append(f"terminal glyph face is missing {marker}")
    if "if run.face != TextFace::Terminal" not in render_geometry:
        failures.append("HiDPI text scaling must preserve the terminal device-pixel grid")
    for marker in (
        "begin_text_buffer_frame",
        "last_used_frame",
        "animated_agent_text_cache_retains_only_two_visible_generations",
    ):
        if marker not in text_cache:
            failures.append(f"terminal text-cache bound is missing {marker}")
    begin_at = render_gpu.find("self.begin_text_buffer_frame();")
    lookup_at = render_gpu.find("self.cached_text_buffer_indices(")
    if render_gpu.count("self.begin_text_buffer_frame();") != 1:
        failures.append("renderer must begin exactly one text-cache generation per frame")
    elif lookup_at < 0 or begin_at > lookup_at:
        failures.append("renderer must prune text buffers before cache lookup")
    for marker in (
        "TERMINAL_FONT_SIZE_PX: f32 = 12.0",
        "TERMINAL_LETTER_SPACING_EM",
        "draw_rich_text",
    ):
        if marker not in bottom_dock:
            failures.append(f"terminal ink/advance separation is missing {marker}")
    if "set_rich_text" not in text_cache:
        failures.append("terminal styled rows must use one rich-text shaping buffer")
    for marker in (
        "terminal_font_advance_matches_shared_logical_cell_width",
        "terminal_cell_advance_combines_smaller_ink_with_explicit_spacing",
        "styled_terminal_colors_share_one_shaping_origin",
        "colored_shell_prompt_preserves_dollar_space_command_and_cursor_cells",
        "hidpi_keeps_terminal_glyphs_and_cursor_on_the_same_device_pixel_grid",
        "prompt_style_boundaries_do_not_restart_glyph_positioning",
        "terminal_rich_span_colors_participate_in_the_buffer_cache_key",
    ):
        if marker not in terminal_font_tests:
            failures.append(f"terminal cell-metric convergence proof is missing {marker}")
    for marker in (
        "CURSOR_HORIZONTAL_INSET_PX",
        "trailing_slash_cursor_paint_stays_inside_the_next_logical_cell",
    ):
        if marker not in terminal_cursor:
            failures.append(f"terminal cursor-cell separation is missing {marker}")


def check_terminal_tab_strip(
    tab_strip: str,
    tests: str,
    bottom_dock: str,
    grid_geometry: str,
    failures: list[str],
) -> None:
    """Keep projected terminal sessions in stable left-to-right tab order."""
    for marker in (
        "for tab in tabs {",
        "HitTarget::TerminalSessionTab(tab.session_id.clone())",
        "x += tab_width + TAB_GAP_PX",
        "target: HitTarget::TerminalSessionNew",
    ):
        if marker not in tab_strip:
            failures.append(f"ordered terminal tab strip is missing {marker}")
    if "new_terminal_tabs_append_left_to_right_and_plus_follows_last_tab" not in tests:
        failures.append("ordered terminal tab-strip production proof is missing")
    for marker in (
        "render_terminal_sessions_row",
        '"SESSIONS"',
        '"+NEW"',
        '"RENAME"',
        '"RESTART"',
        '"CLOSE"',
        "TerminalSessionRenameActive",
        "TerminalSessionRestartActive",
        "TerminalSessionCloseActive",
    ):
        if marker in bottom_dock:
            failures.append(f"redundant terminal session menu remains: {marker}")
    for marker in ("sessions_row", "SESSIONS_BAND_PX"):
        if marker in grid_geometry:
            failures.append(f"retired terminal session row still reserves space: {marker}")
    if "default_dock_keeps_compact_header_and_reclaims_session_menu_row" not in grid_geometry:
        failures.append("terminal session-row reclamation proof is missing")


def check_terminal_session_creation(
    main: str,
    keyboard_focus: str,
    terminal_controls: str,
    terminal_input: str,
    terminal_session: str,
    terminal_session_spawn: str,
    production_refresh: str,
    production_sources: str,
    naming_tests: str,
    failures: list[str],
) -> None:
    """Keep new-session shortcuts real and default labels monotonic."""
    for marker in ("TerminalKeyAction::NewSession", "terminal_new_session_shortcut(", "KeyCode::KeyT"):
        if marker not in terminal_input:
            failures.append(f"new-terminal shortcut dispatch is missing {marker}")
    if "TerminalKeyAction::NewSession => self.spawn_terminal_session_tab()" not in main:
        failures.append("terminal-focus Ctrl+Shift+T does not spawn a session")
    for marker in ("terminal_new_session_shortcut(", "runtime.spawn_terminal_session_tab()"):
        if marker not in keyboard_focus:
            failures.append(f"editor-focus Ctrl+Shift+T dispatch is missing {marker}")
    spawn_body = terminal_controls.split(
        "pub(super) fn spawn_terminal_session_tab", 1
    )[-1].split("pub(super) fn close_active_terminal_session", 1)[0]
    if "self.sync_terminal_tabs();" not in spawn_body or "self.invalidate_frame();" not in spawn_body:
        failures.append("new terminal tab projection is not followed by frame invalidation")
    if "begin_spawn_and_activate" not in spawn_body or "spawn_and_activate_with_lane" in spawn_body:
        failures.append("new terminal tabs still perform PTY spawn work on the GUI event path")
    for marker in (
        'name(format!("terminal-spawn-{pending_id}"))',
        'status: "starting".to_string()',
        "completion_wake.request();",
        "pending_tab_is_projected_before_spawn_work_finishes",
        "active_pending_id",
    ):
        if marker not in terminal_session + terminal_session_spawn:
            failures.append(f"asynchronous terminal tab creation is missing {marker}")
    if "complete_pending_spawns" not in production_refresh:
        failures.append("terminal spawn completion is not consumed from the GUI wake path")
    if "if !registry.active_attached()" not in production_sources:
        failures.append("active pending terminal tabs do not reject input before PTY readiness")
    session_creation_sources = terminal_session + terminal_session_spawn
    if "DatumToolSessionLifecycle::Attached" in session_creation_sources:
        failures.append("terminal creation/switch still persists retired attach lifecycle")
    if "write_terminal_context_files_scoped(&terminal_context, context, false)" not in naming_tests:
        failures.append("terminal bootstrap publishes redundant pre-spawn aliases")
    if "bootstrap_publishes_only_child_discovery_until_pid_is_known" not in naming_tests:
        failures.append("bounded terminal bootstrap proof is missing")
    for marker in ("next_session_ordinal: usize", 'let label = format!("shell {}", self.next_session_ordinal)', "self.next_session_ordinal += 1"):
        if marker not in session_creation_sources:
            failures.append(f"monotonic terminal session naming is missing {marker}")
    if "self.sessions.len() + 1" in session_creation_sources:
        failures.append("terminal labels still reuse the live-session count")
    if "default_session_labels_never_reuse_a_removed_ordinal" not in naming_tests:
        failures.append("terminal label non-reuse production proof is missing")


def check_terminal_input_identity(
    terminal_lane: str,
    production_sources: str,
    failures: list[str],
) -> None:
    """Keep the PTY screen cursor explicit and retired rename state absent."""
    for marker in (
        "pub screen_cursor_row: usize",
        "pub screen_cursor_col: usize",
    ):
        if marker not in terminal_lane:
            failures.append(f"terminal input classification is missing {marker}")
    if re.search(r"pub\s+(?:input|cursor)\s*:", terminal_lane):
        failures.append("terminal protocol must not expose generic input/cursor fields")
    for marker in (".terminal.input", ".terminal.cursor"):
        if marker in production_sources:
            failures.append(f"terminal production code must not use generic {marker}")
    for marker in (
        "fn dock_accepts_text_input",
        "fn append_dock_text",
        "fn dock_tab_accepts_edit",
        "fn current_dock_input",
        "fn current_dock_input_mut",
        "fn backspace_dock_input",
        "fn move_dock_cursor",
        "fn move_dock_cursor_to_edge",
        "fn complete_dock_input",
        "fn submit_dock_input",
        "KeyClass::DockLineEdit",
        "KeyClass::EscapeWithEmptyInput",
    ):
        if marker in production_sources:
            failures.append(f"terminal chrome editor must not use generic marker {marker}")
    for marker in (
        "rename_session_id",
        "rename_input",
        "rename_cursor",
        "append_terminal_rename_text",
        "RenameChrome",
    ):
        if marker in terminal_lane or marker in production_sources:
            failures.append(f"retired terminal rename chrome remains: {marker}")


def check_terminal_input_mode(
    production_sources: str,
    bottom_dock: str,
    failures: list[str],
) -> None:
    """Require one exclusive owned-PTY input-mode authority."""
    for marker in (
        "enum TerminalInputOwner",
        "AttachedPty",
        "fn terminal_input_owner",
        "fn commit_terminal_ime_text",
        "fn write_attached_terminal_bytes",
    ):
        if marker not in production_sources:
            failures.append(f"terminal input-mode authority is missing {marker}")
    for marker in (
        "LegacyDockLineEdit",
        "complete_terminal_rename_input",
        "terminal_rename_editor_active",
        "RenameChrome",
    ):
        if marker in production_sources:
            failures.append(f"dead terminal line-edit marker must not remain: {marker}")
    for marker in ("\"DETACH\"", "\"REATTACH\"", "TerminalSessionReattachActive", "DetachedReadOnly"):
        if marker in production_sources or marker in bottom_dock:
            failures.append(f"retired detached terminal mode remains: {marker}")
    mode_marker = "pub(crate) fn terminal_input_owner"
    if mode_marker in production_sources:
        mode_body = production_sources.split(mode_marker, 1)[1].split("\n}", 1)[0]
        for marker in ("AttachedPty", "Unowned"):
            if marker not in mode_body:
                failures.append(f"terminal input owner does not classify {marker}")
    writer_marker = "fn write_attached_terminal_bytes"
    if writer_marker in production_sources:
        writer_body = production_sources.split(writer_marker, 1)[1].split("\n}", 1)[0]
        for marker in ("write_bytes",):
            if marker not in writer_body:
                failures.append(f"attached terminal byte gate is missing {marker}")


def check_terminal_transport_boundary(
    production_sources: str,
    transport: str,
    failures: list[str],
) -> None:
    """Keep Linux PTY ownership inside Datum and outside terminal semantics."""
    for marker in (
        "/dev/ptmx",
        "grantpt",
        "unlockpt",
        "ptsname_r",
        "TIOCSCTTY",
        "configure_child_pty",
    ):
        if marker not in transport:
            failures.append(f"Datum PTY boundary is missing {marker}")
        elif production_sources.count(marker) != transport.count(marker):
            failures.append(f"Datum PTY ownership escaped terminal_transport/: {marker}")
    if "TIOCSWINSZ" not in production_sources:
        failures.append("Datum PTY resize ownership is missing TIOCSWINSZ")
    for marker in ("TerminalScreen", "TerminalLaneState", "pty_grid_mut", "apply_bytes"):
        if marker in transport:
            failures.append(f"terminal transport must not own cell/core marker {marker}")
    for marker in (
        "libghostty",
        "alacritty_terminal",
        "portable_pty",
        "portable-pty",
    ):
        if marker in production_sources:
            failures.append(f"third-party terminal dependency must not remain: {marker}")


def main() -> int:
    failures: list[str] = []
    check_terminal_grid_writers(failures)
    main = MAIN.read_text()
    keyboard_focus = KEYBOARD_FOCUS.read_text()
    focus_mutation_sources = "\n".join(
        path.read_text(encoding="utf-8")
        for path in sorted((ROOT / "crates/gui-app/src").rglob("*.rs"))
        if not path.name.endswith("_tests.rs")
    )
    bottom_dock = BOTTOM_DOCK.read_text()
    launcher = LAUNCHER.read_text() if LAUNCHER.exists() else ""
    terminal_controls = TERMINAL_CONTROLS.read_text()
    terminal_session_spawn = TERMINAL_SESSION_SPAWN.read_text()
    runtime_terminal_context = RUNTIME_TERMINAL_CONTEXT.read_text()
    production_refresh = PRODUCTION_REFRESH.read_text()
    runtime_terminal_dock = RUNTIME_TERMINAL_DOCK.read_text()
    terminal_drain = (
        TERMINAL_DRAIN.read_text()
        + TERMINAL_DRAIN_TESTS.read_text()
        + TERMINAL_CLOSE_TESTS.read_text()
    )
    render_geometry = RENDER_GEOMETRY.read_text()
    text_buffer_cache = TEXT_BUFFER_CACHE.read_text()
    render_gpu = RENDER_GPU.read_text()
    terminal_font_tests = TERMINAL_FONT_TESTS.read_text()
    terminal_cursor = TERMINAL_CURSOR.read_text()
    terminal_tab_strip = TERMINAL_TAB_STRIP.read_text()
    terminal_tab_strip_tests = TERMINAL_TAB_STRIP_TESTS.read_text()
    terminal_grid_geometry = TERMINAL_GRID_GEOMETRY.read_text()
    terminal_input = TERMINAL_INPUT.read_text()
    terminal_session = TERMINAL_SESSION.read_text()
    terminal_session_naming_tests = (
        TERMINAL_SESSION_NAMING_TESTS.read_text()
        + TERMINAL_SESSION.with_name("terminal_context.rs").read_text()
    )
    terminal_dock_sources = bottom_dock + "\n" + terminal_tab_strip
    gui_protocol = GUI_PROTOCOL.read_text()
    terminal_lane = TERMINAL_LANE.read_text()
    workspace_layout = WORKSPACE_LAYOUT.read_text()
    render_layout = RENDER_LAYOUT.read_text()
    render_scene = RENDER_SCENE.read_text()
    hit_clipping = HIT_CLIPPING.read_text()
    terminal_focus_tests = TERMINAL_FOCUS_TESTS.read_text()
    terminal_hit_tests = TERMINAL_HIT_TESTS.read_text()
    terminal_escape = TERMINAL_ESCAPE.read_text()
    terminal_screen_basic_tests = TERMINAL_SCREEN_BASIC_TESTS.read_text()
    terminal_transport = "\n".join(
        path.read_text(encoding="utf-8")
        for path in sorted(TERMINAL_TRANSPORT.rglob("*.rs"))
    )
    check_terminal_focus_reporting(
        main,
        keyboard_focus,
        workspace_layout,
        terminal_lane,
        focus_mutation_sources,
        failures,
    )
    check_terminal_hit_ownership(
        render_layout,
        render_scene,
        hit_clipping,
        terminal_focus_tests,
        terminal_hit_tests,
        failures,
    )
    check_workspace_hotkey_timing(keyboard_focus, failures)
    check_claude_completion_controls(
        terminal_escape,
        terminal_screen_basic_tests,
        failures,
    )
    check_agent_tui_runtime(
        main,
        runtime_terminal_dock,
        terminal_drain,
        render_geometry,
        text_buffer_cache,
        render_gpu,
        bottom_dock,
        terminal_font_tests,
        terminal_cursor,
        failures,
    )
    check_terminal_tab_strip(
        terminal_tab_strip,
        terminal_tab_strip_tests,
        bottom_dock,
        terminal_grid_geometry,
        failures,
    )
    check_terminal_session_creation(
        main,
        keyboard_focus,
        terminal_controls,
        terminal_input,
        terminal_session,
        terminal_session_spawn,
        production_refresh,
        focus_mutation_sources,
        terminal_session_naming_tests,
        failures,
    )
    check_terminal_input_identity(terminal_lane, focus_mutation_sources, failures)
    check_terminal_input_mode(focus_mutation_sources, bottom_dock, failures)
    check_terminal_transport_boundary(focus_mutation_sources, terminal_transport, failures)

    raw_write_marker = "    fn write_foreign_shell_bytes"
    if raw_write_marker not in main:
        failures.append("foreign-shell byte writer must remain an explicit runtime boundary")
    else:
        raw_write_body = main.split(raw_write_marker, 1)[1].split("\n    fn ", 1)[0]
        if "write_attached_terminal_bytes" not in raw_write_body:
            failures.append("foreign-shell writer must use the attached-session byte gate")
        if "mark_terminal_" in raw_write_body or "refresh_pending" in raw_write_body:
            failures.append(
                "raw foreign-shell input must not infer mutation or schedule workspace refresh"
            )
    authoring_marker = "    fn queue_authoring_terminal_handoff"
    if authoring_marker not in main or "self.mark_terminal_workspace_refresh_pending();" not in main.split(
        authoring_marker, 1
    )[1].split("\n    fn ", 1)[0]:
        failures.append("typed authoring handoffs must explicitly schedule workspace refresh")

    for path in RETIRED_BRIDGE_FILES:
        if path.exists():
            failures.append(
                f"retired embedded assistant bridge artifact must not exist: {path.relative_to(ROOT)}"
            )

    if '"terminal"' not in terminal_dock_sources:
        failures.append("bottom dock must render the terminal tab")
    for forbidden_label in ('"AGENTS"', '"ASSISTANT"', '"OUTPUT"'):
        if forbidden_label in terminal_dock_sources:
            failures.append(f"bottom dock must not render {forbidden_label} as a dock tab")
    for marker in ("AssistantTab", "OutputsTab", "DockTab::Assistant", "DockTab::Outputs"):
        if marker in main or marker in terminal_dock_sources or marker in gui_protocol:
            failures.append(f"terminal-only dock must not contain {marker}")
    # T0-C02 removes application activity summaries from the foreign-shell
    # viewport entirely. The data remains available to the future Command
    # Console, but must not reclaim terminal rows or a terminal hit target.
    for surface, source in (
        ("GUI runtime", main),
        ("bottom dock", bottom_dock),
        ("GUI protocol", gui_protocol),
    ):
        if "TerminalActivitySummary" in source:
            failures.append(
                f"{surface} must not expose a terminal activity-summary surface"
            )
    if "mod terminal_agent_launcher;" in main:
        failures.append("agent launcher module must not be wired as a dock tab surface")
    forbidden_runtime_markers = [
        "AssistantLaneState",
        "AssistantMessage",
        "assistant: AssistantLaneState",
        "mod assistant_bridge;",
        "spawn_assistant_session",
        "AssistantSession",
        "AssistantBridgeInput",
        "poll_assistant_output",
        "send_assistant_message",
        "sync_assistant_context",
        "push_assistant_message",
        "submit_assistant_input",
        "complete_assistant_input",
        "handle_assistant_meta_command",
        "ui.assistant.input",
        "ui.assistant.transcript",
    ]
    for marker in forbidden_runtime_markers:
        if marker in main or marker in production_refresh:
            failures.append(f"GUI runtime must not own embedded assistant bridge marker {marker!r}")
    forbidden_protocol_markers = [
        "AssistantLaneState",
        "AssistantMessage",
        "assistant: AssistantLaneState",
    ]
    for marker in forbidden_protocol_markers:
        if marker in gui_protocol:
            failures.append(f"GUI protocol must not own assistant lane marker {marker!r}")
    forbidden_render_markers = [
        "render_assistant_lane",
        "state.ui.assistant",
        "ui.assistant",
    ]
    for marker in forbidden_render_markers:
        if marker in bottom_dock:
            failures.append(f"GUI render must not own assistant lane marker {marker!r}")
    close_refresh_marker = (
        "close_active(&mut self.session.workspace_mut().ui.terminal)\n"
        "        {\n"
        "            Ok(()) => {\n"
        "                self.refresh_terminal_context_snapshot();"
    )
    if close_refresh_marker not in terminal_controls:
        failures.append(
            "closing the active terminal tab must refresh the surviving session context alias"
        )
    if "self.terminal_launch_context = context;" not in runtime_terminal_context:
        failures.append(
            "terminal context refresh must update Runtime.terminal_launch_context for future tabs/restarts"
        )

    if failures:
        print("GUI agent/terminal convergence guard failed:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("GUI agent/terminal convergence guard passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
