#!/usr/bin/env python3
"""Unit tests for scripts/check_rustfmt.py (no cargo invocation)."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_rustfmt  # noqa: E402


class DiffLineParsing(unittest.TestCase):
    def test_matches_current_rustfmt_format(self) -> None:
        m = check_rustfmt.DIFF_LINE.match("Diff in /repo/crates/engine/src/lib.rs:137:")
        self.assertIsNotNone(m)
        self.assertEqual(m.group("path"), "/repo/crates/engine/src/lib.rs")

    def test_matches_legacy_rustfmt_format(self) -> None:
        m = check_rustfmt.DIFF_LINE.match("Diff in /repo/crates/engine/src/lib.rs at line 137:")
        self.assertIsNotNone(m)
        self.assertEqual(m.group("path"), "/repo/crates/engine/src/lib.rs")

    def test_ignores_diff_body_lines(self) -> None:
        self.assertIsNone(check_rustfmt.DIFF_LINE.match("-    use foo;"))
        self.assertIsNone(check_rustfmt.DIFF_LINE.match("+    use foo;"))


class SelectViolations(unittest.TestCase):
    def test_whole_tree_reports_violations_and_stale_rows(self) -> None:
        v, stale = check_rustfmt.select_violations({"a.rs", "b.rs"}, {"b.rs", "c.rs"}, None)
        self.assertEqual(v, ["a.rs"])
        self.assertEqual(stale, ["c.rs"])

    def test_staged_mode_only_fails_staged_files_and_skips_stale(self) -> None:
        v, stale = check_rustfmt.select_violations({"a.rs", "b.rs"}, {"c.rs"}, {"b.rs"})
        self.assertEqual(v, ["b.rs"])
        self.assertEqual(stale, [])

    def test_staged_mode_passes_when_dirty_files_are_unstaged(self) -> None:
        v, stale = check_rustfmt.select_violations({"a.rs"}, set(), {"other.rs"})
        self.assertEqual(v, [])
        self.assertEqual(stale, [])


class ExemptionManifest(unittest.TestCase):
    def test_manifest_loads_and_every_exempted_file_exists(self) -> None:
        exemptions = check_rustfmt.load_exemptions()
        self.assertIsInstance(exemptions, dict)
        for rel, row in exemptions.items():
            self.assertTrue((check_rustfmt.ROOT / rel).is_file(), rel)
            self.assertTrue(row.get("reason"), rel)


if __name__ == "__main__":
    unittest.main()
