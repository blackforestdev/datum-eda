"""Scoped parity repair must not refresh or inspect unrelated inventories."""
import contextlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_spec_parity as parity


class ScopedParityTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        (self.root / "specs").mkdir()
        self.doc = self.root / "specs/SPEC_PARITY.md"
        self.original = (
            "Unrelated prose stays unchanged.\n"
            "| `source_health_debt_surface` | `docs/SOURCE_HEALTH_POLICY.md` | 66 | `aaaa` |\n"
            "| `unrelated` | `docs/OTHER.md` | 5 | `bbbb` |\n"
        )
        self.doc.write_text(self.original)
        (self.root / "specs/source_module_size_manifest.json").write_text(json.dumps({
            "entries": {"crates/engine/src/lib.rs": {"limits": {"pre_test_lines": 42}}}
        }))
        self.inventories = [
            {"id": "source_health_debt_surface", "owner_spec": "docs/SOURCE_HEALTH_POLICY.md",
             "kind": "source_health_debt_surface"},
            {"id": "unrelated", "owner_spec": "docs/OTHER.md", "kind": "invalid-unrelated-kind"},
        ]
        for target, value in (("ROOT", self.root), ("PARITY_DOC", self.doc)):
            self.enterContext(patch.object(parity, target, value))
        self.enterContext(patch.object(parity, "load_manifest",
                                       return_value={"inventories": self.inventories}))

    def test_selected_update_does_not_evaluate_or_refresh_unrelated_inventory(self):
        parity.update_doc(["source_health_debt_surface"])
        digest = parity.digest_items(["crates/engine/src/lib.rs:pre_test_lines=42"])
        self.assertEqual(self.doc.read_text(), self.original.replace("66 | `aaaa`", f"1 | `{digest}`"))

    def test_unknown_inventory_refuses_before_any_write(self):
        with self.assertRaisesRegex(ValueError, "unknown inventories: typo"):
            parity.update_doc(["source_health_debt_surface", "typo"])
        self.assertEqual(self.doc.read_text(), self.original)

    def test_batch_failure_leaves_all_rows_unchanged(self):
        self.inventories[1]["kind"] = "source_health_debt_surface"
        self.doc.write_text(self.original.split("| `unrelated`")[0])
        before = self.doc.read_bytes()
        with self.assertRaisesRegex(RuntimeError, "unable to update row for unrelated"):
            parity.update_doc(["source_health_debt_surface", "unrelated"])
        self.assertEqual(self.doc.read_bytes(), before)

    def test_unscoped_update_retains_existing_all_inventory_behavior(self):
        self.inventories[1]["kind"] = "source_health_debt_surface"
        parity.update_doc()
        self.assertNotIn("`aaaa`", self.doc.read_text())
        self.assertNotIn("`bbbb`", self.doc.read_text())

    def test_diagnostics_label_count_and_point_to_scoped_command(self):
        expected = {"source_health_debt_surface": ("docs/SOURCE_HEALTH_POLICY.md", 65, "cccc")}
        errors = io.StringIO()
        with patch.object(parity, "expected_rows", return_value=expected), contextlib.redirect_stderr(errors):
            self.assertEqual(parity.check(), 1)
        message = errors.getvalue()
        self.assertIn("specs/SPEC_PARITY.md records owner=docs/SOURCE_HEALTH_POLICY.md, item_count=66", message)
        self.assertIn("--update --inventory source_health_debt_surface", message)
        self.assertIn("stale SPEC_PARITY row for unrelated", message)

    def test_cli_requires_update_and_does_not_silently_ignore_selection(self):
        for options in (["--inventory", "source_health_debt_surface"],
                        ["--print", "--update", "--inventory", "source_health_debt_surface"]):
            with self.subTest(options=options), patch("sys.argv", ["checker", *options]), \
                    contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as failure:
                    parity.main()
                self.assertEqual(failure.exception.code, 2)
        self.assertEqual(self.doc.read_text(), self.original)


if __name__ == "__main__":
    unittest.main()
