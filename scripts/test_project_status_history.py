#!/usr/bin/env python3
"""Presentation regressions: collapse history without changing task authority."""
import copy
import io
import re
import unittest
from contextlib import redirect_stdout
import test_project_status as fixtures

status = fixtures.status


class HistoryPresentationTest(unittest.TestCase):
    def test_mixed_runs_preserve_order_and_leave_active_records_visible(self):
        fixture = fixtures.ProjectStatusTest()
        fixture.doc = "docs/fixture.md"
        records = [fixture.item(key=str(i), order=i, state=state, canonical_next=False)
                   for i, state in enumerate(['landed', 'landed', 'ready', 'landed', 'blocked'])]
        manifest = {'frontier': list(reversed(records))}
        original = copy.deepcopy(manifest)
        block = status.render_block(manifest)
        self.assertEqual(['0', '1', '2', '3', '4'], re.findall(r'\(`(\d+)`;', block))
        self.assertEqual(2, block.count('<details>'))
        self.assertEqual(2, block.count('</details>'))
        self.assertIn('Completed history (2 landed records)', block)
        visible = re.sub(r'<details>.*?</details>', '', block, flags=re.S)
        self.assertIn('(`2`;', visible)
        self.assertIn('(`4`;', visible)
        self.assertNotIn('(`0`;', visible)
        self.assertEqual(original, manifest)

    def test_empty_and_active_only_have_no_history_wrapper(self):
        self.assertNotIn('<details>', status.render_block({'frontier': []}))
        fixture = fixtures.ProjectStatusTest()
        fixture.doc = "docs/fixture.md"
        item = fixture.item()
        self.assertNotIn('<details>', status.render_block({'frontier': [item]}))

    def test_render_repair_keeps_next_and_details_output_identical(self):
        fixture = fixtures.ProjectStatusTest()
        fixture.setUp()
        try:
            decision = fixture.root / fixture.doc
            decision.write_text(decision.read_text() + "\n<!-- REQ:history:TEST-C01 -->\nHistory.\n")
            revision = fixture.head()
            history = fixture.item(key="history", order=1, issue_id="dat-history",
                                   state="landed", authorization="none", canonical_next=False,
                                   landing_commit=revision)
            history["completion"]["canonical_next_step_id"] = None
            history["completion"]["steps"][0].update(
                status="complete", completion_evidence=[{"kind": "commit", "revision": revision}])
            fixture.manifest["frontier"].append(history)
            fixture.issues.append(fixture.issue("dat-history", "closed", labels=["roadmap:frontier"]))
            fixture.write_fixture()
            outputs = []
            for _ in range(2):
                current = []
                for command in ['next', 'details']:
                    out = io.StringIO()
                    with redirect_stdout(out):
                        self.assertEqual(0, status.main(['--root', str(fixture.root), command]))
                    current.append(out.getvalue())
                outputs.append(current)
                progress = fixture.root / "specs/PROGRESS.md"
                progress.write_text(progress.read_text().replace("Completed history", "STALE history"))
                self.assertFalse(status.render_status(fixture.root, False)[0])
                status.render_status(fixture.root, True)
            self.assertEqual(outputs[0], outputs[1])
            self.assertTrue(status.render_status(fixture.root, False)[0])
        finally:
            fixture.tearDown()


if __name__ == '__main__':
    unittest.main()
