#!/usr/bin/env python3
"""Hermetic tests for check_source_health.py."""

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("check_source_health.py")
SPEC = importlib.util.spec_from_file_location("check_source_health", MODULE_PATH)
assert SPEC and SPEC.loader
health = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = health
SPEC.loader.exec_module(health)


def git(root: Path, *args: str) -> None:
    subprocess.run(["git", *args], cwd=root, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)


class RepositoryTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        git(self.root, "init", "-q")
        git(self.root, "config", "user.email", "test@example.invalid")
        git(self.root, "config", "user.name", "Test")
        (self.root / "specs").mkdir()
        self.manifest = self.root / "specs/source_module_size_manifest.json"
        self.policy = {
            "schema_version": 2,
            "policy_decision": "PRODUCT_MECHANICS_022",
            "baseline_commit": "bootstrap",
            "entries": {},
            "exclusions": {
                name: {"paths": [], "requirements": "test requirements"}
                for name in ("generated", "vendored", "data_assets")
            },
            "exceptions": [],
        }
        self.write_policy()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def write_policy(self) -> None:
        self.manifest.write_text(json.dumps(self.policy, indent=2) + "\n")

    def write_lines(self, rel: str, count: int, line: str = "x") -> None:
        path = self.root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text((line + "\n") * count)

    def evaluate(self, base: str | None = None) -> list[str]:
        failures, _ = health.evaluate(self.root, self.manifest, base)
        return failures

    def commit(self, message: str = "base") -> str:
        git(self.root, "add", ".")
        git(self.root, "commit", "-qm", message)
        return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=self.root, text=True).strip()

    def legacy(self, rel: str, ceiling: int = 900, target: int = 700) -> None:
        self.policy["entries"][rel] = {
            "kind": "python-production",
            "limits": {"file_lines": ceiling},
            "target_lines": target,
            "status": "decomposition-required",
            "note": "legacy monolith",
            "owner": "architecture",
            "trigger_commit": "deadbeef",
        }
        self.write_policy()

    def test_untracked_source_is_checked_and_ignored_source_is_not(self) -> None:
        self.write_lines("src/new.py", 701)
        failures = self.evaluate()
        self.assertTrue(any("src/new.py" in x for x in failures))
        (self.root / ".gitignore").write_text("ignored/\n")
        self.write_lines("ignored/large.py", 900)
        self.write_lines("src/new.py", 1)
        self.assertEqual(self.evaluate(), [])

    def test_language_thresholds_and_unterminated_line_count(self) -> None:
        self.write_lines("tool.sh", 401)
        self.write_lines("page.html", 701)
        (self.root / "one.py").write_bytes(b"unterminated")
        failures = self.evaluate()
        self.assertTrue(any("tool.sh" in x and "limit 400" in x for x in failures))
        self.assertTrue(any("page.html" in x and "limit 700" in x for x in failures))
        self.assertEqual(health.count_lines(b"unterminated"), 1)

    def test_test_and_inline_test_limits(self) -> None:
        self.write_lines("crates/a/tests/large.rs", 701)
        path = self.root / "crates/a/src/lib.rs"
        path.parent.mkdir(parents=True)
        path.write_text("fn ok() {}\n#[cfg(test)]\n" + "// test\n" * 351)
        failures = self.evaluate()
        self.assertTrue(any("large.rs" in x and "test" in x for x in failures))
        self.assertTrue(any("inline test tail" in x for x in failures))

    def test_recursive_include_cannot_hide_logical_monolith(self) -> None:
        self.write_lines("crates/a/src/chunk.rs", 400)
        path = self.root / "crates/a/src/lib.rs"
        path.write_text('include!("chunk.rs");\n' * 2)
        failures = self.evaluate()
        self.assertTrue(any("lib.rs" in x and "oversized" in x for x in failures))

    def test_include_cycle_and_dynamic_include_fail_closed(self) -> None:
        base = self.root / "crates/a/src"
        base.mkdir(parents=True)
        (base / "a.rs").write_text('include!("b.rs");\n')
        (base / "b.rs").write_text('include!("a.rs");\n')
        (base / "dynamic.rs").write_text('include!(concat!("x", ".rs"));\n')
        failures = self.evaluate()
        self.assertTrue(any("cycle" in x for x in failures))
        self.assertTrue(any("dynamic" in x for x in failures))

    def test_legacy_ceiling_stale_and_delist_are_failures(self) -> None:
        self.write_lines("src/legacy.py", 901)
        self.legacy("src/legacy.py", 900)
        self.legacy("src/missing.py", 900)
        failures = self.evaluate()
        self.assertTrue(any("ceiling exceeded" in x for x in failures))
        self.assertTrue(any("stale" in x for x in failures))
        del self.policy["entries"]["src/missing.py"]
        self.write_lines("src/legacy.py", 600)
        self.write_policy()
        self.assertTrue(any("must be removed" in x for x in self.evaluate()))

    def test_base_rejects_policy_weakening_and_requires_touch_burndown(self) -> None:
        self.write_lines("src/legacy.py", 800)
        self.legacy("src/legacy.py", 900)
        base = self.commit()
        self.policy["entries"]["src/legacy.py"]["limits"]["file_lines"] = 901
        self.write_policy()
        self.write_lines("src/legacy.py", 801)
        failures = self.evaluate(base)
        self.assertTrue(any("limit file_lines" in x for x in failures))
        self.assertTrue(any("must shrink" in x for x in failures))

    def test_base_rejects_a_new_legacy_entry(self) -> None:
        self.write_lines("src/healthy.py", 1)
        base = self.commit()
        self.write_lines("src/new_debt.py", 701)
        self.legacy("src/new_debt.py", 701)
        failures = self.evaluate(base)
        self.assertTrue(any("new legacy entry is forbidden" in x for x in failures))

    def test_base_accepts_downward_ratchet_and_burndown(self) -> None:
        self.write_lines("src/legacy.py", 800)
        self.legacy("src/legacy.py", 900)
        base = self.commit()
        self.policy["entries"]["src/legacy.py"]["limits"]["file_lines"] = 790
        self.write_policy()
        self.write_lines("src/extracted.py", 20, "def extracted(): pass")
        self.write_lines("src/legacy.py", 789)
        with (self.root / "src/legacy.py").open("a") as handle:
            handle.write("import extracted\n")
        self.assertEqual(self.evaluate(base), [])

    def test_deletion_only_burndown_is_not_structural_evidence(self) -> None:
        self.write_lines("src/legacy.py", 800)
        self.legacy("src/legacy.py", 800)
        base = self.commit()
        self.policy["entries"]["src/legacy.py"]["limits"]["file_lines"] = 790
        self.write_policy()
        self.write_lines("src/legacy.py", 790)
        failures = self.evaluate(base)
        self.assertTrue(any("lacks real extraction evidence" in x for x in failures))

    def test_rust_mod_extraction_is_structural_evidence(self) -> None:
        self.write_lines("crates/a/src/lib.rs", 800, "fn legacy() {}")
        self.policy["entries"]["crates/a/src/lib.rs"] = {
            "kind": "rust-production",
            "limits": {"pre_test_lines": 800},
            "target_lines": 700,
            "status": "decomposition-required",
            "note": "legacy monolith",
            "owner": "architecture",
            "trigger_commit": "deadbeef",
        }
        self.write_policy()
        base = self.commit()
        self.policy["entries"]["crates/a/src/lib.rs"]["limits"]["pre_test_lines"] = 790
        self.write_policy()
        self.write_lines("crates/a/src/extracted.rs", 20, "pub fn extracted() {}")
        self.write_lines("crates/a/src/lib.rs", 789, "fn legacy() {}")
        with (self.root / "crates/a/src/lib.rs").open("a") as handle:
            handle.write("mod extracted;\n")
        self.assertEqual(self.evaluate(base), [])

    def test_repository_contract_requires_gate_wiring(self) -> None:
        decision = self.root / "docs/decisions/PRODUCT_MECHANICS_022_SOURCE_HEALTH_GOVERNANCE.md"
        decision.parent.mkdir(parents=True)
        decision.write_text("immutable\n")
        failures = self.evaluate()
        self.assertTrue(any("protected source-health surface missing" in x for x in failures))

    def test_schema_is_strict_and_output_order_is_stable(self) -> None:
        self.policy["surprise"] = True
        self.write_policy()
        with self.assertRaises(health.PolicyError):
            health.load_policy(self.manifest)
        del self.policy["surprise"]
        self.write_policy()
        self.write_lines("z.py", 701)
        self.write_lines("a.py", 701)
        failures = self.evaluate()
        self.assertEqual(failures, sorted(failures))


if __name__ == "__main__":
    unittest.main()
