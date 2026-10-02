"""Regression oracles for dat-7unq; keep private root writers forbidden."""
import unittest
from pathlib import Path
from private_writer_bootstrap_policy import check_project_bootstrap

ROOT = Path(__file__).resolve().parents[1]
CLI = Path("crates/cli/src/commands/project/roots.rs")


class BootstrapPolicyTests(unittest.TestCase):
    def setUp(self):
        self.source = (ROOT / CLI).read_text()

    def test_existing_units_aware_canonical_delegation_passes(self):
        self.assertEqual(check_project_bootstrap(CLI, self.source), [])

    def test_removing_canonical_call_refuses(self):
        changed = self.source.replace(
            "bootstrap_native_project_with_units(", "private_bootstrap("
        )
        self.assertTrue(check_project_bootstrap(CLI, changed))

    def test_private_json_writer_still_refuses(self):
        failures = check_project_bootstrap(
            CLI, self.source + "\nwrite_canonical_json(root, &data)?;"
        )
        self.assertTrue(any("expected 0" in failure for failure in failures))

    def test_direct_filesystem_writer_still_refuses(self):
        failures = check_project_bootstrap(
            CLI, self.source + '\nstd::fs::write(root.join("project.json"), data)?;'
        )
        self.assertTrue(any("forbidden project bootstrap writer" in f for f in failures))

    def test_noncanonical_import_refuses(self):
        changed = self.source.replace(
            "use eda_engine::api::native_write::genesis::", "use private_genesis::"
        )
        self.assertTrue(check_project_bootstrap(CLI, changed))


if __name__ == "__main__":
    unittest.main()
