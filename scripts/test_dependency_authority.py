#!/usr/bin/env python3
"""Hermetic regressions for dependency-authority policy data."""

from __future__ import annotations

import json
import hashlib
import tempfile
import unittest
from pathlib import Path
from check_dependency_authority import check_source_patches


ROOT = Path(__file__).resolve().parents[1]


class DependencyAuthorityPolicyTest(unittest.TestCase):
    def test_policy_is_closed_and_terminal_dependencies_are_forbidden(self) -> None:
        policy = json.loads(
            (ROOT / "specs/third_party_dependency_policy.json").read_text(encoding="utf-8")
        )
        self.assertEqual(
            {
                "schema_version",
                "authority",
                "policy",
                "terminal_policy",
                "inherited_direct_external_dependencies",
                "forbidden_terminal_dependencies",
                "approved_source_patches",
            },
            set(policy),
        )
        self.assertEqual(
            {
                "alacritty_terminal",
                "libghostty-vt",
                "portable-pty",
                "portable_pty",
            },
            set(policy["forbidden_terminal_dependencies"]),
        )
        self.assertEqual(
            policy["inherited_direct_external_dependencies"],
            sorted(set(policy["inherited_direct_external_dependencies"])),
        )

    def test_authority_is_ratified_and_explicit(self) -> None:
        decision = (
            ROOT / "docs/decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md"
        ).read_text(encoding="utf-8")
        self.assertIn("Status: ratified doctrine", decision)
        self.assertIn("sole authority", decision)
        self.assertIn("No external\nterminal implementation", decision)

    def test_patch_exception_rejects_identity_notice_and_scope_expansion(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "authority.md").write_text("Approved fixture")
            package = root / "third_party/example"
            package.mkdir(parents=True)
            notice = b"Fixture notice"
            (package / "LICENSE-MIT").write_bytes(notice)
            manifest = '[package]\nname="example"\nversion="1.0.0"\nlicense="MIT OR Apache-2.0"\n'
            (package / "Cargo.toml").write_text(manifest)
            cargo = '[patch.crates-io]\nexample={path="third_party/example"}\n'
            (root / "Cargo.toml").write_text(cargo)
            policy = {"approved_source_patches": {"example": {
                "path": "third_party/example", "version": "1.0.0", "license": "MIT",
                "license_sha256": hashlib.sha256(notice).hexdigest(), "decision": "authority.md",
            }}}
            self.assertEqual(check_source_patches(root, policy), [])
            (package / "Cargo.toml").write_text(manifest.replace("1.0.0", "2.0.0"))
            self.assertTrue(check_source_patches(root, policy))
            (package / "Cargo.toml").write_text(manifest)
            (package / "LICENSE-MIT").write_text("Changed notice")
            self.assertTrue(check_source_patches(root, policy))
            (package / "LICENSE-MIT").write_bytes(notice)
            (root / "Cargo.toml").write_text(cargo + 'other={path="third_party/other"}\n')
            self.assertTrue(check_source_patches(root, policy))
            (root / "Cargo.toml").write_text(cargo)
            (root / "third_party/other").mkdir()
            self.assertTrue(check_source_patches(root, policy))


if __name__ == "__main__":
    unittest.main()
