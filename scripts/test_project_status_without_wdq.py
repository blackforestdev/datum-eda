"""Roadmap queries remain usable without retired WDQ transaction enforcement."""

import io
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch
import unittest

import test_project_status as fixture_support

status = fixture_support.status


class RoadmapWithoutWdqTest(unittest.TestCase):
    def setUp(self):
        self.fixture = fixture_support.ProjectStatusTest()
        self.fixture.setUp()
        self.addCleanup(self.fixture.tearDown)
        self.root = self.fixture.root

    def test_queries_do_not_invoke_wdq_with_generated_files_present(self):
        for name in ("scripts/__pycache__/measurements.cpython-313.pyc",
                     "fixture/.datum/check_runs/report.json"):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"retained generated output")
        with patch("workflow_delivery_selector.selector_failures",
                   side_effect=AssertionError("WDQ must not run for roadmap queries")):
            for command in ("check", "next", "details"):
                with self.subTest(command=command), redirect_stdout(io.StringIO()) as output:
                    result = status.main(["--root", str(self.root), command])
                self.assertEqual(0, result, output.getvalue())
        self.assertTrue((self.root / "fixture/.datum/check_runs/report.json").is_file())

    def test_invalid_roadmap_still_refuses(self):
        self.fixture.manifest["frontier"][0]["canonical_next"] = False
        self.fixture.write_fixture()
        errors, _ = status.validate(self.root)
        self.assertTrue(any("exactly one canonical_next" in error for error in errors))

    def test_legacy_diagnostic_requires_explicit_opt_in(self):
        with patch("workflow_delivery_selector.selector_failures",
                   return_value=["explicit legacy diagnostic"]) as diagnostic:
            errors, _ = status.validate(self.root, delivery_checks=True)
        diagnostic.assert_called_once()
        self.assertIn("explicit legacy diagnostic", errors)

    def test_normal_entrypoints_keep_basic_gates_not_wdq(self):
        root = Path(__file__).resolve().parents[1]
        hook = (root / "scripts/git-hooks/pre-commit").read_text()
        self.assertIn("check_file_lane_ownership.py --staged", hook)
        self.assertIn("check_rustfmt.py --staged", hook)
        for name in ("scripts/git-hooks/pre-commit", "scripts/run_drift_gates.sh",
                     ".github/workflows/alignment.yml"):
            self.assertNotIn("check_workflow_delivery.py", (root / name).read_text())


if __name__ == "__main__":
    unittest.main()
