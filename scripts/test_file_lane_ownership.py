#!/usr/bin/env python3
"""Hermetic proof for the staged visual-truth file-lane gate."""

from __future__ import annotations

import importlib.util
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).with_name("check_file_lane_ownership.py")
HOOK = Path(__file__).parent / "git-hooks" / "pre-commit"
SPEC = importlib.util.spec_from_file_location("file_lane_gate", SCRIPT)
assert SPEC and SPEC.loader
gate = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = gate
SPEC.loader.exec_module(gate)


class Repository:
    def __init__(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.git("init", "-q")
        self.git("config", "user.name", "Datum Test")
        self.git("config", "user.email", "datum-test@example.invalid")

    def close(self) -> None:
        self.temporary.cleanup()

    def git(self, *arguments: str, check: bool = True) -> subprocess.CompletedProcess[bytes]:
        return subprocess.run(
            ["git", *arguments], cwd=self.root, check=check, capture_output=True
        )

    def write(self, relative: str, content: str = "fixture") -> Path:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return path

    def commit_all(self) -> None:
        self.git("add", "--all")
        self.git("commit", "-q", "-m", "fixture")

    def run_gate(self, marker: str | None = None) -> subprocess.CompletedProcess[bytes]:
        environment = os.environ.copy()
        environment.pop("DATUM_VISUAL_TRUTH_LANE", None)
        if marker is not None:
            environment["DATUM_VISUAL_TRUTH_LANE"] = marker
        return subprocess.run(
            ["python3", str(SCRIPT), "--staged"],
            cwd=self.root,
            env=environment,
            check=False,
            capture_output=True,
        )


class FileLaneOwnershipTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo = Repository()

    def tearDown(self) -> None:
        self.repo.close()

    def stage(self, relative: str, content: str = "fixture") -> None:
        self.repo.write(relative, content)
        self.repo.git("add", relative)

    def assert_refused(self, marker: str | None = None) -> None:
        result = self.repo.run_gate(marker)
        self.assertEqual(result.returncode, 1, result.stderr.decode())
        self.assertIn(b"Claude-owned visual-truth lane", result.stderr)

    def test_unrelated_staged_file_passes_without_marker(self) -> None:
        self.stage("docs/ordinary.md")
        result = self.repo.run_gate()
        self.assertEqual(result.returncode, 0, result.stderr.decode())

    def test_protected_add_copy_modify_mode_and_delete_refuse(self) -> None:
        protected = "docs/gui/prototypes/study.html"
        operations = ("add", "copy", "modify", "mode", "delete")
        for operation in operations:
            with self.subTest(operation=operation):
                self.repo.close()
                self.repo = Repository()
                if operation == "add":
                    self.stage(protected)
                elif operation == "copy":
                    self.repo.write("source.html")
                    self.repo.commit_all()
                    self.stage(protected, self.repo.write("source.html").read_text())
                else:
                    path = self.repo.write(protected)
                    self.repo.commit_all()
                    if operation == "modify":
                        path.write_text("changed", encoding="utf-8")
                    elif operation == "mode":
                        path.chmod(0o755)
                    else:
                        path.unlink()
                    self.repo.git("add", "--all")
                self.assert_refused()

    def test_rename_into_and_out_of_lane_refuse(self) -> None:
        cases = (
            ("outside.html", "docs/gui/prototypes/inside.html"),
            ("docs/gui/prototypes/inside.html", "outside.html"),
        )
        for source, destination in cases:
            with self.subTest(source=source, destination=destination):
                self.repo.close()
                self.repo = Repository()
                self.repo.write(source)
                self.repo.commit_all()
                (self.repo.root / destination).parent.mkdir(parents=True, exist_ok=True)
                self.repo.git("mv", source, destination)
                self.assert_refused()

    def test_only_exact_marker_value_permits_protected_path(self) -> None:
        self.stage("docs/gui/prototypes/study.html")
        for marker in (None, "", "Claude", "CLAUDE", "codex", "claude "):
            with self.subTest(marker=marker):
                self.assert_refused(marker)
        accepted = self.repo.run_gate("claude")
        self.assertEqual(accepted.returncode, 0, accepted.stderr.decode())
        self.assertIn(b"marked Claude session", accepted.stdout)

    def test_nested_spaces_and_unicode_paths_are_unambiguous(self) -> None:
        first = "docs/gui/prototypes/nested/a study.html"
        second = "docs/gui/prototypes/nested/μ-study.html"
        self.stage(first)
        self.stage(second)
        result = self.repo.run_gate()
        self.assertEqual(result.returncode, 1)
        decoded = result.stderr.decode("utf-8")
        self.assertIn(repr(first), decoded)
        self.assertIn(repr(second), decoded)

    def test_gate_does_not_mutate_index_or_worktree(self) -> None:
        self.stage("docs/gui/prototypes/study.html")
        before_index = self.repo.git("diff", "--cached", "--binary").stdout
        before_status = self.repo.git("status", "--porcelain=v1", "-z").stdout
        self.assert_refused()
        after_index = self.repo.git("diff", "--cached", "--binary").stdout
        after_status = self.repo.git("status", "--porcelain=v1", "-z").stdout
        self.assertEqual(before_index, after_index)
        self.assertEqual(before_status, after_status)

    def test_git_failure_is_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run(
                ["python3", str(SCRIPT), "--staged"],
                cwd=directory,
                check=False,
                capture_output=True,
            )
        self.assertEqual(result.returncode, 2)
        self.assertIn(b"failed closed", result.stderr)

    def test_malformed_records_fail_parser(self) -> None:
        malformed = (b"M\0path", b"R100\0only-one\0", b"X\0path\0", b"M\0\0")
        for record in malformed:
            with self.subTest(record=record):
                with self.assertRaises(gate.StagedPathError):
                    gate.parse_name_status_z(record)

    def test_copy_and_rename_records_preserve_both_paths(self) -> None:
        changes = gate.parse_name_status_z(
            b"C100\0source.html\0docs/gui/prototypes/copied.html\0"
            b"R095\0docs/gui/prototypes/old.html\0moved.html\0"
        )
        self.assertEqual(changes[0].paths, (b"source.html", b"docs/gui/prototypes/copied.html"))
        self.assertEqual(
            changes[1].paths,
            (b"docs/gui/prototypes/old.html", b"moved.html"),
        )
        self.assertEqual(
            gate.protected_paths(changes),
            [
                b"docs/gui/prototypes/copied.html",
                b"docs/gui/prototypes/old.html",
            ],
        )

    def test_hook_runs_lane_gate_before_rustfmt(self) -> None:
        hook = HOOK.read_text(encoding="utf-8")
        lane_position = hook.index("check_file_lane_ownership.py --staged")
        rustfmt_position = hook.index("check_rustfmt.py --staged")
        self.assertLess(lane_position, rustfmt_position)


if __name__ == "__main__":
    unittest.main()
