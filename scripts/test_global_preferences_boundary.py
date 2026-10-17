#!/usr/bin/env python3
"""Hermetic production-adapter discovery and authority rejection controls."""
from pathlib import Path
import tempfile
import unittest

import check_global_preferences_boundary as gate


class ProductionAdapters(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)

    def write(self, relative, content):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)
        return path

    def test_named_test_child_is_not_a_product_adapter(self):
        self.write("crates/gui-app/src/windows/new_project/tests.rs",
                   "GlobalPreferencesProductService PreferenceRepository XDG_CONFIG_HOME")
        self.write("crates/gui-app/src/windows/new_project.rs",
                   "fn create() {}\n")
        users, failures = gate.adapter_authority(self.root)
        self.assertEqual(users, set())
        self.assertEqual(failures, [])

    def test_test_marker_cannot_hide_later_production_authority(self):
        for name, marker in [
            ("after_test", "#[cfg(test)]\nmod tests {}"),
            ("after_string", 'const MARKER: &str = "#[cfg(test)]";'),
            ("after_comment", "// #[cfg(test)]"),
        ]:
            self.write(f"crates/gui-app/src/{name}.rs",
                       marker + "\nGlobalPreferencesProductService\nPreferenceRepository\n")
        users, failures = gate.adapter_authority(self.root)
        self.assertEqual(len(users), 3)
        self.assertEqual(len(failures), 3)

    def test_real_production_adapter_and_private_authority_still_detected(self):
        expected = Path("crates/gui-app/src/windows/global_preferences_runtime.rs")
        self.write(expected, "use engine::GlobalPreferencesProductService;\n")
        rogue = Path("crates/gui-app/src/windows/rogue.rs")
        self.write(rogue, "use engine::GlobalPreferencesProductService;\nPreferenceRepository\n")
        users, failures = gate.adapter_authority(self.root)
        self.assertEqual(users, {expected, rogue})
        self.assertNotEqual(users, gate.ALLOWED_PRODUCT_SERVICE_FILES)
        self.assertTrue(any("rogue.rs: private preference authority" in f for f in failures))

    def test_non_rust_adapter_and_non_test_sibling_remain_scanned(self):
        self.write("mcp-server/adapter.py", "PreferenceRepository\n")
        self.write("crates/gui-app/src/windows/new_project/production.rs",
                   "GlobalPreferencesProductService")
        users, failures = gate.adapter_authority(self.root)
        self.assertIn(Path("crates/gui-app/src/windows/new_project/production.rs"), users)
        self.assertTrue(any("adapter.py" in f for f in failures))


if __name__ == "__main__":
    unittest.main()
