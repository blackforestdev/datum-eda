#!/usr/bin/env python3
"""Hermetic mutation tests for the production TerminalCore adapter guard."""

from __future__ import annotations

import importlib.util
import shutil
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("check_terminal_core_adapter_boundary.py")
SPEC = importlib.util.spec_from_file_location("terminal_core_adapter_guard", MODULE_PATH)
assert SPEC and SPEC.loader
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class TerminalCoreAdapterBoundaryTest(unittest.TestCase):
    def fixture(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        root = Path(temporary.name)
        source = root / guard.SRC
        source.mkdir(parents=True)
        (root / guard.APP / "Cargo.toml").write_text(
            'datum-terminal-core = { path = "../terminal-core" }\n', encoding="utf-8"
        )
        limits = "\n".join(
            f"    {field}: {value}," for field, value in guard.APPROVED_LIMITS.items()
        )
        (root / guard.ADAPTER).write_text(
            "struct TerminalCoreSessionAdapter {\n"
            "    parser: StreamingParser,\n    core: TerminalCore,\n}\n"
            f"const PRODUCTION_CORE_LIMIT_VALUES: CoreLimitValues = CoreLimitValues {{\n{limits}\n}};\n"
            "fn apply(){ parser.feed(&bytes, |action| core.apply_into(action)); self.core.render_snapshot(); }\n",
            encoding="utf-8",
        )
        (root / guard.SESSION).write_text(
            "struct Slot { core: TerminalCoreSessionAdapter }\n"
            "fn restart(){slot.core = TerminalCoreSessionAdapter::new_with_profile();}\n",
            encoding="utf-8",
        )
        (root / guard.SESSION_RENDER).write_text(
            "fn resize(){slot.core.resize(cols, rows, pixel_width, pixel_height)?;}\n",
            encoding="utf-8",
        )
        (root / guard.SPAWN).write_text(
            "fn spawn(){TerminalCoreSessionAdapter::new_with_profile();}\n", encoding="utf-8"
        )
        (root / guard.TERMINAL_PROCESS).write_text(
            "fn environment(){\n"
            "let request = apply_datum_terminal_identity(request, &terminfo_root);\n"
            ".env(\"TERM\", DATUM_TERM);\n"
            ".env(\"TERMINFO\", terminfo_root.as_os_str());\n"
            ".env(\"TERM_PROGRAM\", DATUM_TERM_PROGRAM);\n"
            "env_remove(\"TERM_PROGRAM\");\n}\n",
            encoding="utf-8",
        )
        (root / guard.TERMINAL_CAPABILITY).write_text(
            'const DATUM_TERM: &str = "datum-256color";\n'
            'const DATUM_TERM_PROGRAM: &str = "Datum";\n'
            'const ENTRY: &[u8] = include_bytes!("datum-256color");\n'
            "fn install_session_terminfo() {}\n",
            encoding="utf-8",
        )
        terminfo_source = root / guard.TERMINFO_SOURCE
        terminfo_source.parent.mkdir(parents=True, exist_ok=True)
        terminfo_source.write_text(
            "datum-256color|Datum EDA terminal with 256 colors,\n"
            "\tcolors#256,\n\tpairs#32767,\n\tTc,\n\tRGB,\n\tSu,\n\tAX,\n",
            encoding="utf-8",
        )
        terminfo_entry = root / guard.TERMINFO_ENTRY
        terminfo_entry.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(guard.ROOT / guard.TERMINFO_ENTRY, terminfo_entry)
        (root / guard.DRAIN).write_text(
            "fn drain(){\n"
            "debug_assert_eq!(slot.core.session_id(), slot.session.session_id());\n"
            "debug_assert_eq!(slot.core.context_id(), slot.session.context_id);\n"
            "slot.core.apply_output(lane, bytes);\n"
            "slot.core.finish(lane);\n}\n",
            encoding="utf-8",
        )
        (root / guard.CORE_EVENTS).write_text(
            "fn consume(){ session.write_bytes(&response); }\n", encoding="utf-8"
        )
        (source / "terminal_core_adapter_tests.rs").write_text(
            "\n".join(f"fn {proof}() {{}}" for proof in guard.REQUIRED_PROOFS),
            encoding="utf-8",
        )
        return temporary, root

    def assert_mutation_fails(self, relative: Path, old: str, new: str) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        self.assertEqual(guard.check(root), [])
        path = root / relative
        path.write_text(path.read_text(encoding="utf-8").replace(old, new), encoding="utf-8")
        self.assertTrue(guard.check(root))

    def test_valid_adapter_boundary_passes(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        self.assertEqual(guard.check(root), [])

    def test_owner_limit_drift_fails(self) -> None:
        self.assert_mutation_fails(guard.ADAPTER, "history_lines: 100_000", "history_lines: 200_000")

    def test_second_parser_or_core_fails(self) -> None:
        self.assert_mutation_fails(
            guard.ADAPTER,
            "    core: TerminalCore,",
            "    core: TerminalCore,\n    shadow_core: TerminalCore,",
        )

    def test_provisional_parser_reentry_fails(self) -> None:
        self.assert_mutation_fails(
            guard.DRAIN,
            "slot.core.apply_output(lane, bytes);",
            "slot.core.apply_output(lane, bytes); apply_bytes_with_responses(lane, bytes);",
        )

    def test_retired_screen_or_protocol_grid_reentry_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        retired = root / guard.RETIRED_AUTHORITIES[0]
        retired.write_text("struct TerminalScreen;", encoding="utf-8")
        lane = root / guard.TERMINAL_LANE
        lane.parent.mkdir(parents=True, exist_ok=True)
        lane.write_text("fn pty_grid_mut() {}", encoding="utf-8")
        failures = guard.check(root)
        self.assertTrue(any("retired provisional terminal authority returned" in item for item in failures))
        self.assertTrue(any("terminal lane regained screen authority" in item for item in failures))

    def test_xterm_capability_overclaim_reentry_fails(self) -> None:
        self.assert_mutation_fails(
            guard.TERMINAL_PROCESS,
            'env_remove("TERM_PROGRAM")',
            '.env("TERM", "xterm-256color")',
        )

    def test_missing_datum_identity_or_compiled_entry_drift_fails(self) -> None:
        mutations = (
            (guard.TERMINAL_PROCESS, '.env("TERM", DATUM_TERM)', ""),
            (guard.TERMINFO_SOURCE, "\tTc,", "\tXT,"),
            (guard.TERMINFO_ENTRY, None, None),
        )
        for relative, old, new in mutations:
            with self.subTest(relative=relative):
                temporary, root = self.fixture()
                self.addCleanup(temporary.cleanup)
                path = root / relative
                if old is None:
                    path.write_bytes(path.read_bytes() + b"drift")
                else:
                    path.write_text(
                        path.read_text(encoding="utf-8").replace(old, new),
                        encoding="utf-8",
                    )
                self.assertTrue(guard.check(root))

    def test_output_projection_cannot_reset_scrollback(self) -> None:
        self.assert_mutation_fails(
            guard.ADAPTER,
            "fn apply(){",
            "fn apply(){ lane.scroll_offset = 0;",
        )

    def test_missing_reply_resize_finish_or_identity_fails(self) -> None:
        mutations = (
            (guard.CORE_EVENTS, "session.write_bytes(&response);", ""),
            (guard.SESSION_RENDER, ".resize(cols, rows, pixel_width, pixel_height)?;", ""),
            (guard.DRAIN, "slot.core.finish(lane);", ""),
            (guard.DRAIN, "debug_assert_eq!(slot.core.context_id(), slot.session.context_id);", ""),
        )
        for relative, old, new in mutations:
            with self.subTest(marker=old):
                self.assert_mutation_fails(relative, old, new)

    def test_session_core_construction_cannot_bypass_profile(self) -> None:
        self.assert_mutation_fails(
            guard.SPAWN,
            "TerminalCoreSessionAdapter::new_with_profile()",
            "TerminalCoreSessionAdapter::new()",
        )

    def test_external_terminal_dependency_fails(self) -> None:
        self.assert_mutation_fails(
            guard.APP / "Cargo.toml",
            'datum-terminal-core = { path = "../terminal-core" }',
            'datum-terminal-core = { path = "../terminal-core" }\nportable-pty = "0.9"',
        )


if __name__ == "__main__":
    unittest.main()
