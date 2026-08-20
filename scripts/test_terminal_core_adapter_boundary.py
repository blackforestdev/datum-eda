#!/usr/bin/env python3
"""Hermetic mutation tests for the production TerminalCore adapter guard."""

from __future__ import annotations

import importlib.util
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
            "fn apply(){ parser.feed(bytes, |action| core.apply(action)); self.core.snapshot(); }\n",
            encoding="utf-8",
        )
        (root / guard.SESSION).write_text(
            "struct Slot { core: TerminalCoreSessionAdapter }\n"
            "fn restart(){slot.core = TerminalCoreSessionAdapter::new();}\n",
            encoding="utf-8",
        )
        (root / guard.SESSION_RENDER).write_text(
            "fn resize(){slot.core.resize(cols, rows, pixel_width, pixel_height)?;}\n",
            encoding="utf-8",
        )
        (root / guard.SPAWN).write_text(
            "fn spawn(){TerminalCoreSessionAdapter::new();}\n", encoding="utf-8"
        )
        (root / guard.DRAIN).write_text(
            "fn drain(){\n"
            "debug_assert_eq!(slot.core.session_id(), slot.session.session_id());\n"
            "debug_assert_eq!(slot.core.context_id(), slot.session.context_id);\n"
            "slot.core.apply_output(lane, bytes);\n"
            "session.write_bytes(&response);\n"
            "slot.core.finish(lane);\n}\n",
            encoding="utf-8",
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

    def test_missing_reply_resize_finish_or_identity_fails(self) -> None:
        mutations = (
            (guard.DRAIN, "session.write_bytes(&response);", ""),
            (guard.SESSION_RENDER, ".resize(cols, rows, pixel_width, pixel_height)?;", ""),
            (guard.DRAIN, "slot.core.finish(lane);", ""),
            (guard.DRAIN, "debug_assert_eq!(slot.core.context_id(), slot.session.context_id);", ""),
        )
        for relative, old, new in mutations:
            with self.subTest(marker=old):
                self.assert_mutation_fails(relative, old, new)

    def test_external_terminal_dependency_fails(self) -> None:
        self.assert_mutation_fails(
            guard.APP / "Cargo.toml",
            'datum-terminal-core = { path = "../terminal-core" }',
            'datum-terminal-core = { path = "../terminal-core" }\nportable-pty = "0.9"',
        )


if __name__ == "__main__":
    unittest.main()
