#!/usr/bin/env python3
"""Hermetic tests for the exact-file Rust formatter."""

from __future__ import annotations

from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import format_rust


class ExactFileRustfmtTests(unittest.TestCase):
    def test_paths_must_be_rust_files_inside_repository(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "src" / "lib.rs"
            source.parent.mkdir()
            source.write_text("fn main() {}\n", encoding="utf-8")
            self.assertEqual(format_rust.resolve_files(["src/lib.rs"], root), [source])
            with self.assertRaisesRegex(format_rust.FormatError, "not a Rust source"):
                format_rust.resolve_files(["src/lib.txt"], root)
            with self.assertRaisesRegex(format_rust.FormatError, "outside the repository"):
                format_rust.resolve_files([str(root.parent / "outside.rs")], root)

    def test_rustfmt_receives_source_on_stdin_without_a_path(self) -> None:
        completed = subprocess.CompletedProcess([], 0, stdout=b"formatted\n", stderr=b"")
        with mock.patch("format_rust.subprocess.run", return_value=completed) as run:
            self.assertEqual(format_rust.rustfmt_source(b"source\n"), b"formatted\n")
        self.assertEqual(
            run.call_args.args[0],
            ["rustfmt", "--edition", "2024", "--emit", "stdout"],
        )
        self.assertEqual(run.call_args.kwargs["input"], b"source\n")

    def test_only_explicit_files_are_written(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            selected = root / "selected.rs"
            unrelated = root / "unrelated.rs"
            selected.write_bytes(b"selected")
            unrelated.write_bytes(b"unrelated")
            changed = format_rust.format_files(
                [selected], check=False, formatter=lambda source: source + b" formatted"
            )
            self.assertEqual(changed, [selected])
            self.assertEqual(selected.read_bytes(), b"selected formatted")
            self.assertEqual(unrelated.read_bytes(), b"unrelated")

    def test_check_mode_never_writes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "source.rs"
            source.write_bytes(b"source")
            changed = format_rust.format_files(
                [source], check=True, formatter=lambda contents: contents + b" formatted"
            )
            self.assertEqual(changed, [source])
            self.assertEqual(source.read_bytes(), b"source")


if __name__ == "__main__":
    unittest.main()
