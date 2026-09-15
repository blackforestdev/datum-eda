#!/usr/bin/env python3
"""Hermetic tests for the Cargo resource-policy static gate."""

from __future__ import annotations

from pathlib import Path
import tempfile
import unittest

import check_cargo_resource_policy as checker


class CargoResourcePolicyGateTests(unittest.TestCase):
    def fixture(self, command: str) -> Path:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        scripts = root / "scripts"
        scripts.mkdir()
        for name in checker.REQUIRED_GUARDED_RUNNERS:
            (scripts / name).write_text(
                "#!/usr/bin/env bash\n"
                f"python3 scripts/{checker.GUARD} --workload proof -- "
                f"{command}\n",
                encoding="utf-8",
            )
        return root

    def test_direct_cargo_command_is_rejected(self) -> None:
        root = self.fixture("true")
        path = root / "scripts/run_drift_gates.sh"
        path.write_text("#!/usr/bin/env bash\ncargo test --workspace\n", encoding="utf-8")
        errors = checker.check(root)
        self.assertTrue(any("unguarded Cargo command" in error for error in errors))
        self.assertTrue(any("does not invoke" in error for error in errors))

    def test_guarded_cargo_command_passes(self) -> None:
        root = self.fixture("cargo test --workspace")
        self.assertEqual(checker.check(root), [])

    def test_comments_do_not_count_as_commands(self) -> None:
        root = self.fixture("cargo test --workspace")
        path = root / "scripts/extra.sh"
        path.write_text("# cargo test is intentionally guarded\n", encoding="utf-8")
        self.assertEqual(checker.check(root), [])


if __name__ == "__main__":
    unittest.main()
