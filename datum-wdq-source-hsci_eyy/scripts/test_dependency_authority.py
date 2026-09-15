#!/usr/bin/env python3
"""Hermetic regressions for dependency-authority policy data."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


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


if __name__ == "__main__":
    unittest.main()
