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
                    "8 => self.set_hyperlink\n52 => self.clipboard_request\n"
                    "133 => self.shell_mark\n777 => self.extended_notification\n"
                    "self.state.progress = progress\n"
                )
            if relative == "event.rs":
                text += "\nClipboardRequest\nOpenUriRequest\nShellMark\nNotification\nProgress\n"
            (source / relative).write_text(text, encoding="utf-8")
        (source / "parser_tests.rs").write_text(
            "\n".join(guard.REQUIRED_PARSER_PROOFS), encoding="utf-8"
        )
        (source / "reducer_tests.rs").write_text(
            "\n".join(guard.REQUIRED_REDUCER_PROOFS), encoding="utf-8"
        )
        (source / "semantic_tests.rs").write_text(
            "\n".join(guard.REQUIRED_SEMANTIC_PROOFS + guard.REQUIRED_METADATA_PROOFS),
            encoding="utf-8",
        )
        (source / "unicode_tests.rs").write_text(
            "\n".join(guard.REQUIRED_UNICODE_PROOFS), encoding="utf-8"
        )
        (source / "history_tests.rs").write_text(
            "\n".join(guard.REQUIRED_HISTORY_PROOFS), encoding="utf-8"
        )
        (source / "selection_tests.rs").write_text(
            "\n".join(guard.REQUIRED_SELECTION_PROOFS), encoding="utf-8"
        )
        (source / "search_tests.rs").write_text(
            "\n".join(guard.REQUIRED_SEARCH_PROOFS), encoding="utf-8"
        )
        (source / "input_tests.rs").write_text(
            "\n".join(guard.REQUIRED_INPUT_PROOFS), encoding="utf-8"
        )
        (source / "codec_tests.rs").write_text(
            "\n".join(guard.REQUIRED_CODEC_PROOFS), encoding="utf-8"
        )
        (source / "sixel_tests.rs").write_text(
            "\n".join(guard.REQUIRED_SIXEL_PROOFS), encoding="utf-8"
        )
        (source / "screen.rs").write_text(
            (source / "screen.rs").read_text()
            + "\nprimary: GridBuffer\nalternate: GridBuffer\n",
            encoding="utf-8",
        )
        (source / "reducer.rs").write_text(
            (source / "reducer.rs").read_text()
            + "\nimpl TerminalCore {}\nself.apply_action(action)\nrepair_row(row)\n"
            + "pub(crate) fn prune_graphics(\nclear_buffer\nScreenAction::Reset\n",
            encoding="utf-8",
        )
        (source / "reducer_print.rs").write_text(
            (source / "reducer_print.rs").read_text()
            + "\nfn print_cluster() {}\nhyperlink: self.state.current_hyperlink\n",
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
        (source / "lib.rs").write_text(
            (source / "lib.rs").read_text()
            + "pub use base64::decode_base64;\n"
            + "pub use checksum::{Adler32, Crc32, adler32, crc32};\n"
            + "pub use deflate::{DeflateOutput, decode_deflate};\n"
            + "pub use png::{PngImage, Rgba8, decode_png};\n"
            + "pub use zlib::decode_zlib;\n"
            + "pub use sixel::{SixelColorRegisters, decode_sixel};\n",
            encoding="utf-8",
        )
        (source / "semantics.rs").write_text(
            (source / "semantics.rs").read_text() + "\nScreenAction::AppendCluster\n",
            encoding="utf-8",
        )
        (source / "screen.rs").write_text(
            (source / "screen.rs").read_text()
            + "\ngrapheme_anchor\nresolve_logical_point\n"
            + "pub(crate) selection: Option<crate::Selection>\n",
            encoding="utf-8",
        )
        (source / "selection.rs").write_text(
            (source / "selection.rs").read_text()
            + "\nSelectionScope::Grapheme\nSelectionScope::Word\n"
            + "SelectionScope::WrappedLine\nSelectionScope::LogicalLine\n"
            + "SelectionScope::Block\nSelectionScope::All\n"
            + "resolve_logical_point\nself.limits.clipboard_bytes\n",
            encoding="utf-8",
        )
        (source / "search.rs").write_text(
            (source / "search.rs").read_text()
            + "\nself.limits.search_work.get()\nSearchDirection::Forward\n"
            + "SearchDirection::Backward\nresolve_logical_point\nSearchMatchState::Trimmed\n"
            + "search_all_literal\nsearch_all_regex\n",
            encoding="utf-8",
        )
        (source / "search_regex.rs").write_text(
            (source / "search_regex.rs").read_text()
            + "\nState::Split\nVec<Fragment>\nwork.charge(1)?\n",
            encoding="utf-8",
        )
        (source / "input.rs").write_text(
            (source / "input.rs").read_text()
            + "\nself.limits.input_bytes\nInputDisposition::LocalOnly\n"
            + "self.state.modes.bracketed_paste\nself.state.modes.focus_reporting\n",
            encoding="utf-8",
        )
        (source / "input_key.rs").write_text(
            (source / "input_key.rs").read_text()
            + "\nKITTY_DISAMBIGUATE\nKITTY_REPORT_EVENTS\nKITTY_REPORT_ALTERNATE\nKITTY_REPORT_ALL\n"
            + "KITTY_ASSOCIATED_TEXT\napplication_cursor\napplication_keypad\n"
            + "kitty_keypad_code\n",
            encoding="utf-8",
        )
        (source / "input_modes.rs").write_text(
            (source / "input_modes.rs").read_text()
            + "\n.keyboard_stack\nReplyKind::KeyboardProtocol\n"
            + ".stack.push(\n.stack.pop(\n",
            encoding="utf-8",
        )
        (source / "input_mouse.rs").write_text(
            (source / "input_mouse.rs").read_text()
            + "\nMouseEncoding::Default\nMouseEncoding::Utf8\nMouseEncoding::Sgr\n"
            + "MouseEncoding::Urxvt\nMouseEncoding::SgrPixels\n"
            + "input.local_override\nvalue.clamp(\n",
            encoding="utf-8",
        )
        (source / "history.rs").write_text(
            (source / "history.rs").read_text()
            + "\nself.logical_lines > self.line_limit.get()\n"
            + "self.payload_bytes > self.byte_limit.get()\n",
            encoding="utf-8",
        )
        (source / "reflow.rs").write_text(
            (source / "reflow.rs").read_text()
            + "\nself.state.history.replace_rows(history)\n.reflow_work\n",
            encoding="utf-8",
        )
        (source / "control_string.rs").write_text(
            (source / "control_string.rs").read_text()
            + "\nself.state.graphics.insert_sixel(\n"
            + "self.push_damage(Damage::Graphics, update)?\n"
            + "sixel_aspect(parameters.first().copied().unwrap_or(0))\n",
            encoding="utf-8",
        )
        (source / "csi.rs").write_text(
            (source / "csi.rs").read_text()
            + "\n80 =>\n1070 =>\n8452 =>\nsixel_scrolling\n"
            + "sixel_private_colors\nsixel_cursor_right\n",
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

    def test_history_limit_anchor_and_reflow_proof_removal_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        history = root / guard.CRATE / "src/history.rs"
        history.write_text(
            history.read_text().replace(
                "self.logical_lines > self.line_limit.get()", "false"
            )
        )
        screen = root / guard.CRATE / "src/screen.rs"
        screen.write_text(screen.read_text().replace("resolve_logical_point", "removed"))
        proofs = root / guard.CRATE / "src/history_tests.rs"
        proofs.write_text(
            proofs.read_text().replace(
                "primary_scrollback_preserves_logical_identity_across_reflow", "removed"
            )
        )
        failures = guard.check(root)
        self.assertTrue(any("logical-line limit" in item for item in failures))
        self.assertTrue(any("stable logical anchors" in item for item in failures))
        self.assertTrue(any("primary_scrollback" in item for item in failures))

    def test_selection_scope_limit_and_stable_endpoint_removal_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        selection = root / guard.CRATE / "src/selection.rs"
        selection.write_text(
            selection.read_text()
            .replace("SelectionScope::Block", "removed_block")
            .replace("self.limits.clipboard_bytes", "usize::MAX")
        )
        screen = root / guard.CRATE / "src/screen.rs"
        screen.write_text(
            screen.read_text().replace(
                "pub(crate) selection: Option<crate::Selection>", "removed_selection"
            )
        )
        proofs = root / guard.CRATE / "src/selection_tests.rs"
        proofs.write_text(
            proofs.read_text().replace(
                "logical_endpoints_survive_reflow_then_report_history_trim", "removed"
            )
        )
        failures = guard.check(root)
        self.assertTrue(any("SelectionScope::Block" in item for item in failures))
        self.assertTrue(any("clipboard_bytes" in item for item in failures))
        self.assertTrue(any("stable logical endpoints" in item for item in failures))
        self.assertTrue(any("logical_endpoints_survive" in item for item in failures))

    def test_search_work_nfa_and_stability_proof_removal_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        search = root / guard.CRATE / "src/search.rs"
        search.write_text(
            search.read_text()
            .replace("self.limits.search_work.get()", "usize::MAX")
            .replace("SearchMatchState::Trimmed", "SearchMatchState::Unknown")
            .replace("pub fn search_all(", "pub fn removed_search_all(")
        )
        regex = root / guard.CRATE / "src/search_regex.rs"
        regex.write_text(regex.read_text().replace("State::Split", "removed_split"))
        proofs = root / guard.CRATE / "src/search_tests.rs"
        proofs.write_text(
            proofs.read_text()
            .replace("hostile_regex_exhausts_search_work_without_backtracking", "removed")
            .replace("all_match_search_shares_one_work_budget", "removed_all_match_budget")
        )
        failures = guard.check(root)
        self.assertTrue(any("search_work" in item for item in failures))
        self.assertTrue(any("SearchMatchState::Trimmed" in item for item in failures))
        self.assertTrue(any("State::Split" in item for item in failures))
        self.assertTrue(any("hostile_regex_exhausts" in item for item in failures))
        self.assertTrue(any("search_all" in item for item in failures))
        self.assertTrue(any("all_match_search_shares" in item for item in failures))

    def test_input_limit_kitty_mouse_override_and_proof_removal_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        contract = root / guard.CRATE / "src/input.rs"
        contract.write_text(contract.read_text().replace("self.limits.input_bytes", "usize::MAX"))
        modes = root / guard.CRATE / "src/input_modes.rs"
        modes.write_text(
            modes.read_text().replace(".keyboard_stack", ".removed_stack")
        )
        mouse = root / guard.CRATE / "src/input_mouse.rs"
        mouse.write_text(
            mouse.read_text()
            .replace("MouseEncoding::SgrPixels", "removed_pixels")
            .replace("input.local_override", "false")
        )
        proofs = root / guard.CRATE / "src/input_tests.rs"
        proofs.write_text(
            proofs.read_text().replace(
                "kitty_keyboard_negotiation_is_chunk_invariant_bounded_and_queryable",
                "removed",
            )
        )
        failures = guard.check(root)
        self.assertTrue(any("input_bytes" in item for item in failures))
        self.assertTrue(any("keyboard_stack" in item for item in failures))
        self.assertTrue(any("MouseEncoding::SgrPixels" in item for item in failures))
        self.assertTrue(any("input.local_override" in item for item in failures))
        self.assertTrue(any("kitty_keyboard_negotiation" in item for item in failures))

    def test_metadata_handler_cell_link_and_proof_removal_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        control = root / guard.CRATE / "src/control_string.rs"
        control.write_text(control.read_text().replace("52 => self.clipboard_request", "removed"))
        reducer_print = root / guard.CRATE / "src/reducer_print.rs"
        reducer_print.write_text(
            reducer_print.read_text().replace(
                "hyperlink: self.state.current_hyperlink", "hyperlink: None"
            )
        )
        proofs = root / guard.CRATE / "src/semantic_tests.rs"
        proofs.write_text(
            proofs.read_text().replace(
                "osc8_hyperlinks_attach_to_cells_and_end_without_opening_any_uri", "removed"
            )
        )
        failures = guard.check(root)
        self.assertTrue(any("clipboard_request" in item for item in failures))
        self.assertTrue(any("current OSC 8 hyperlink" in item for item in failures))
        self.assertTrue(any("osc8_hyperlinks" in item for item in failures))

    def test_codec_limit_crc_interlace_and_hostile_proof_removal_fails(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        base64 = root / guard.CRATE / "src/base64.rs"
        base64.write_text(base64.read_text().replace("limits.decoded_bytes", "usize::MAX"))
        png = root / guard.CRATE / "src/png.rs"
        png.write_text(png.read_text().replace("checksum.update(&kind)", "removed_crc"))
        pixels = root / guard.CRATE / "src/png_pixels.rs"
        pixels.write_text(pixels.read_text().replace("const ADAM7", "const REMOVED"))
        proofs = root / guard.CRATE / "src/codec_tests.rs"
        proofs.write_text(
            proofs.read_text().replace(
                "hostile_png_prefixes_and_mutations_never_escape_bounded_errors", "removed"
            )
        )
        failures = guard.check(root)
        self.assertTrue(any("limits.decoded_bytes" in item for item in failures))
        self.assertTrue(any("checksum.update(&kind)" in item for item in failures))
        self.assertTrue(any("const ADAM7" in item for item in failures))
        self.assertTrue(any("hostile_png_prefixes" in item for item in failures))

    def test_sixel_bounds_placement_teardown_and_proof_removal_fail(self) -> None:
        temporary, root = self.fixture()
        self.addCleanup(temporary.cleanup)
        sixel = root / guard.CRATE / "src/sixel.rs"
        sixel.write_text(
            sixel.read_text()
            .replace("limits.pixels.check(pixel_count)?", "removed_pixel_limit")
            .replace("let mut working_registers = registers.clone()", "removed_atomic_palette")
        )
        reducer = root / guard.CRATE / "src/reducer.rs"
        reducer.write_text(reducer.read_text().replace("clear_buffer", "removed_clear"))
        proofs = root / guard.CRATE / "src/sixel_tests.rs"
        proofs.write_text(
            proofs.read_text().replace(
                "sixel_scrolls_into_history_and_history_trim_releases_pixels", "removed"
            )
        )
        failures = guard.check(root)
        self.assertTrue(any("limits.pixels.check" in item for item in failures))
        self.assertTrue(any("working_registers" in item for item in failures))
        self.assertTrue(any("tear down graphics" in item for item in failures))
        self.assertTrue(any("sixel_scrolls_into_history" in item for item in failures))


if __name__ == "__main__":
    unittest.main()
