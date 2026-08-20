#!/usr/bin/env python3
"""Hermetic mutation tests for the Datum TerminalCore boundary guard."""

from __future__ import annotations

import importlib.util
import shutil
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("check_terminal_core_boundary.py")
SPEC = importlib.util.spec_from_file_location("terminal_core_guard", MODULE_PATH)
assert SPEC and SPEC.loader
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class TerminalCoreBoundaryTest(unittest.TestCase):
    def fixture(self) -> tuple[tempfile.TemporaryDirectory[str], Path]:
        temporary = tempfile.TemporaryDirectory()
        root = Path(temporary.name)
        source = root / guard.CRATE / "src"
        source.mkdir(parents=True)
        shutil.copytree(guard.ROOT / guard.CRATE / "unicode", root / guard.CRATE / "unicode")
        scripts = root / "scripts"
        scripts.mkdir()
        shutil.copy2(
            guard.ROOT / "scripts/generate_terminal_unicode.py",
            scripts / "generate_terminal_unicode.py",
        )
        (root / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["crates/terminal-core"]\n', encoding="utf-8"
        )
        (root / guard.CRATE / "Cargo.toml").write_text(
            '[package]\nname = "datum-terminal-core"\nversion = "0.1.0"\n'
            'edition = "2024"\n[dependencies]\n',
            encoding="utf-8",
        )
        for relative, markers in guard.REQUIRED_MODULES.items():
            text = "\n".join(markers)
            if relative == "limits.rs":
                text += "\n" + "\n".join(guard.REQUIRED_LIMITS)
            if relative == "parser.rs":
                text += (
                    "\nemit: impl FnMut(Action)\nControlStringKind::Osc\n"
                    "ControlStringKind::Dcs\nControlStringKind::Apc\nControlStringKind::Pm\n"
                    "ControlStringKind::Sos\nDiscardState\nMalformedUtf8\nwork_exhausted\n"
                )
            if relative == "parser_action.rs":
                text += "\nCancelled\nCsiParameter\nStringTerminator\nLimitExceeded\n"
            if relative == "charset.rs":
                text += "\nCharacterSet::DecSpecialGraphics\n"
            if relative == "semantics.rs":
                text += (
                    "\nself.state.charsets.map(character)\n"
                    "self.state.synchronized_dirty = true\n"
                    "ReplyKind::DeviceAttributes\n"
                )
            if relative == "csi.rs":
                text += (
                    "\n2026 => self.end_synchronized_output(update)?\n"
                    "ReplyKind::ModeReport\n"
                )
            if relative == "control_string.rs":
                text += (
                    "\nself.state.palette[index as usize] = color\n"
                    "TitleText::new\nWorkingDirectoryText::new\n"
                )
            (source / relative).write_text(text, encoding="utf-8")
        (source / "parser_tests.rs").write_text(
            "\n".join(guard.REQUIRED_PARSER_PROOFS), encoding="utf-8"
        )
        (source / "reducer_tests.rs").write_text(
            "\n".join(guard.REQUIRED_REDUCER_PROOFS), encoding="utf-8"
        )
        (source / "semantic_tests.rs").write_text(
            "\n".join(guard.REQUIRED_SEMANTIC_PROOFS), encoding="utf-8"
        )
        (source / "unicode_tests.rs").write_text(
            "\n".join(guard.REQUIRED_UNICODE_PROOFS), encoding="utf-8"
        )
        (source / "screen.rs").write_text(
            (source / "screen.rs").read_text()
            + "\nprimary: GridBuffer\nalternate: GridBuffer\n",
            encoding="utf-8",
        )
        (source / "reducer.rs").write_text(
            (source / "reducer.rs").read_text()
            + "\nimpl TerminalCore {}\nself.apply_action(action)\nrepair_row(row)\n",
            encoding="utf-8",
        )
        (source / "grid.rs").write_text(
            (source / "grid.rs").read_text() + "\nCellContent::Continuation\n",
            encoding="utf-8",
        )
        (source / "lib.rs").write_text(
            "pub use screen::{ScreenState, TerminalCore};\n"
            "pub use parser::{FeedReport, StreamingParser};\n"
            "pub use reducer::{Reduction, ScreenError};\n"
            "pub use reducer_action::{EraseDisplay, EraseLine, FoundationMode, ScreenAction};\n"
            "pub use semantics::{CoreError, CoreUpdate};\n",
            encoding="utf-8",
        )
        (source / "semantics.rs").write_text(
            (source / "semantics.rs").read_text() + "\nScreenAction::AppendCluster\n",
            encoding="utf-8",
        )
        (source / "screen.rs").write_text(
            (source / "screen.rs").read_text() + "\ngrapheme_anchor\n",
            encoding="utf-8",
        )
        return temporary, root

    def test_valid_std_only_foundation_passes(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        self.assertEqual(guard.check(root), [])

    def test_external_dependency_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        manifest = root / guard.CRATE / "Cargo.toml"
        manifest.write_text(manifest.read_text() + 'serde = "1"\n', encoding="utf-8")
        self.assertTrue(any("std-only" in item for item in guard.check(root)))

    def test_renderer_or_process_authority_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        screen = root / guard.CRATE / "src/screen.rs"
        screen.write_text(screen.read_text() + "\nuse wgpu::Texture;\nstd::process::Command;\n")
        failures = guard.check(root)
        self.assertTrue(any("wgpu" in item for item in failures))
        self.assertTrue(any("std::process" in item for item in failures))

    def test_missing_limit_family_and_numeric_default_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        limits = root / guard.CRATE / "src/limits.rs"
        text = limits.read_text().replace("SnapshotCells", "")
        limits.write_text(text + "\nimpl Default for CoreLimits {}\n")
        failures = guard.check(root)
        self.assertTrue(any("SnapshotCells" in item for item in failures))
        self.assertTrue(any("owner-supplied" in item for item in failures))

    def test_missing_parser_or_core_authority_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        screen = root / guard.CRATE / "src/screen.rs"
        screen.write_text("pub fn feed_pty() {}\n")
        failures = guard.check(root)
        self.assertTrue(any("owned marker" in item for item in failures))

        parser = root / guard.CRATE / "src/parser.rs"
        parser.unlink()
        self.assertTrue(any("parser.rs" in item for item in guard.check(root)))

    def test_lossy_utf8_or_retained_action_queue_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        parser = root / guard.CRATE / "src/parser.rs"
        parser.write_text(parser.read_text() + "\nfrom_utf8_lossy\nVec<Action>\n")
        failures = guard.check(root)
        self.assertTrue(any("from_utf8_lossy" in item for item in failures))
        self.assertTrue(any("action queue" in item for item in failures))

    def test_missing_chunk_recovery_proof_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        proofs = root / guard.CRATE / "src/parser_tests.rs"
        proofs.write_text(proofs.read_text().replace(
            "oversized_sequences_emit_one_error_discard_and_recover", "removed"
        ))
        self.assertTrue(any("oversized_sequences" in item for item in guard.check(root)))

    def test_missing_reducer_or_continuation_repair_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        reducer = root / guard.CRATE / "src/reducer.rs"
        reducer.unlink()
        self.assertTrue(any("reducer.rs" in item for item in guard.check(root)))

        temporary_two, root_two = self.fixture()
        self.addCleanup(temporary_two.cleanup)
        grid = root_two / guard.CRATE / "src/grid.rs"
        grid.write_text(grid.read_text().replace("fn repair_row", "fn removed"))
        self.assertTrue(any("repair_row" in item for item in guard.check(root_two)))

    def test_missing_screen_limit_or_reducer_proof_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        limits = root / guard.CRATE / "src/limits.rs"
        limits.write_text(limits.read_text().replace("ScreenCells", "RemovedCells"))
        proofs = root / guard.CRATE / "src/reducer_tests.rs"
        proofs.write_text(
            proofs.read_text().replace(
                "primary_and_alternate_buffers_are_isolated_and_reset_is_total", "removed"
            )
        )
        failures = guard.check(root)
        self.assertTrue(any("ScreenCells" in item for item in failures))
        self.assertTrue(any("primary_and_alternate" in item for item in failures))

    def test_missing_semantic_owner_or_proof_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        semantics = root / guard.CRATE / "src/semantics.rs"
        semantics.write_text(
            semantics.read_text().replace("self.state.charsets.map(character)", "character")
        )
        proofs = root / guard.CRATE / "src/semantic_tests.rs"
        proofs.write_text(
            proofs.read_text().replace(
                "complete_semantics_are_invariant_across_arbitrary_parser_chunks", "removed"
            )
        )
        failures = guard.check(root)
        self.assertTrue(any("character-set mapping" in item for item in failures))
        self.assertTrue(any("complete_semantics" in item for item in failures))

    def test_synchronized_output_or_metadata_owner_removal_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        csi = root / guard.CRATE / "src/csi.rs"
        csi.write_text(
            csi.read_text().replace(
                "2026 => self.end_synchronized_output(update)?", "2026 => {}"
            )
        )
        control = root / guard.CRATE / "src/control_string.rs"
        control.write_text(control.read_text().replace("WorkingDirectoryText::new", "removed"))
        failures = guard.check(root)
        self.assertTrue(any("synchronized-output" in item for item in failures))
        self.assertTrue(any("WorkingDirectoryText::new" in item for item in failures))

    def test_retained_semantic_update_queue_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        semantics = root / guard.CRATE / "src/semantics.rs"
        semantics.write_text(semantics.read_text() + "\nVec<CoreUpdate>\n")
        self.assertTrue(any("update queue" in item for item in guard.check(root)))

    def test_unicode_input_generator_and_proof_drift_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        width = root / guard.CRATE / "unicode/17.0.0/EastAsianWidth.txt"
        width.write_text(width.read_text() + "\n# drift\n")
        generator = root / "scripts/generate_terminal_unicode.py"
        generator.write_text(generator.read_text().replace("verify_inputs", "removed"))
        proofs = root / guard.CRATE / "src/unicode_tests.rs"
        proofs.write_text(
            proofs.read_text().replace(
                "unicode_17_grapheme_break_corpus_matches_every_normative_boundary", "removed"
            )
        )
        failures = guard.check(root)
        self.assertTrue(any("checksum drifted" in item for item in failures))
        self.assertTrue(any("generator marker" in item for item in failures))
        self.assertTrue(any("grapheme_break_corpus" in item for item in failures))

    def test_unicode_policy_or_reducer_integration_removal_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        unicode = root / guard.CRATE / "src/unicode.rs"
        unicode.write_text(
            unicode.read_text().replace("BidirectionalTextPolicy::LogicalOrder", "removed")
        )
        semantics = root / guard.CRATE / "src/semantics.rs"
        semantics.write_text(semantics.read_text().replace("ScreenAction::AppendCluster", "removed"))
        failures = guard.check(root)
        self.assertTrue(any("owned marker" in item for item in failures))
        self.assertTrue(any("sole screen reducer" in item for item in failures))


if __name__ == "__main__":
    unittest.main()
