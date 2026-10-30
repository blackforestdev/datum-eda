#!/usr/bin/env python3
"""Hermetic regressions for dependency-authority policy data."""

from __future__ import annotations

import json
import subprocess
import unittest
from pathlib import Path
from unittest.mock import patch

import check_dependency_authority as authority


ROOT = Path(__file__).resolve().parents[1]


class DependencyAuthorityPolicyTest(unittest.TestCase):
    def test_metadata_collection_stays_locked_offline_and_excludes_workspace_members(self):
        packages = [
            {"name": "app", "dependencies": [{"name": "core"}, {"name": "serde"}]},
            {"name": "core", "dependencies": [{"name": "serde"}, {"name": "uuid"}]},
        ]
        response = subprocess.CompletedProcess([], 0, json.dumps({"packages": packages}), "")
        with patch.object(authority.subprocess, "run", return_value=response) as run:
            self.assertEqual(authority.cargo_external_dependencies(ROOT), {"serde", "uuid"})
        arguments = run.call_args.args[0]
        self.assertEqual(arguments[:2], ["cargo", "metadata"])
        for flag in ("--locked", "--offline", "--no-deps"):
            self.assertIn(flag, arguments)
        self.assertTrue(run.call_args.kwargs["check"])
        self.assertEqual(run.call_args.kwargs["cwd"], ROOT)

    def test_failed_locked_inspection_does_not_retry_online(self):
        error = subprocess.CalledProcessError(101, ["cargo", "metadata"])
        with patch.object(authority.subprocess, "run", side_effect=error) as run:
            with self.assertRaises(subprocess.CalledProcessError):
                authority.cargo_external_dependencies(ROOT)
        self.assertEqual(run.call_count, 1)

    def test_malformed_metadata_does_not_become_an_empty_dependency_set(self):
        response = subprocess.CompletedProcess([], 0, "not JSON", "")
        with patch.object(authority.subprocess, "run", return_value=response):
            with self.assertRaises(json.JSONDecodeError):
                authority.cargo_external_dependencies(ROOT)

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
