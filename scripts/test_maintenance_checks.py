"""Regression checks for relocated inventory and frozen progress readers."""

from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import check_progress_coverage as progress
import check_spec_parity as parity


class MaintenanceCheckTests(unittest.TestCase):
    def test_supervision_inventory_follows_owners_without_using_test_decoys(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "crates/gui-protocol/src"
            source.mkdir(parents=True)
            for name in ("lib.rs", "supervision.rs", "workspace_state.rs", "workspace_loading.rs", "source_shard_status.rs"):
                (source / name).write_text("")
            (source / "supervision.rs").write_text("pub struct GuiSupervisionSnapshot {}")
            (source / "tests.rs").write_text("pub struct GuiJournalSupervision {}")
            with patch.object(parity, "ROOT", root):
                self.assertEqual(parity.gui_supervision_surface(), ["protocol:pub struct GuiSupervisionSnapshot"])
                (source / "supervision.rs").write_text("")
                self.assertEqual(parity.gui_supervision_surface(), [])

    def test_compact_historical_coverage_keeps_count_and_authority_guards(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            shard = root / "specs/progress/m3_details.md"
            shard.parent.mkdir(parents=True)
            shard.write_text("# Historical notes\n\n## Criterion A\nRecorded evidence.\n")
            text = (
                "## PROGRAM_SPEC.md — M3 Exit Criteria\n\n"
                "Recorded coverage: 1 completed criteria. "
                "[Criterion-level evidence](progress/m3_details.md) remains unchanged. "
                "**M3 overall**: [x] complete for that recorded slice; this\n"
                "historical record grants no current import execution or broader qualification.\n"
            )
            with patch.object(progress, "ROOT", root):
                failures = []
                progress.check_milestone_progress_compaction(text, failures)
                self.assertEqual(failures, [])
                for broken in (text.replace("1 completed", "2 completed"),
                               text.replace("historical record grants no current import execution or broader qualification", "current execution approved")):
                    failures = []
                    progress.check_milestone_progress_compaction(broken, failures)
                    self.assertTrue(failures)
            self.assertIsNone(progress.markdown_overall_status(text + "**M3 overall**: [~] inconsistent\n", "M3 overall"))


if __name__ == "__main__":
    unittest.main()
