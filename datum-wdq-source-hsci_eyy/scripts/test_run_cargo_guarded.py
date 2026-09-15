#!/usr/bin/env python3
"""Hermetic tests for Datum's Cargo resource guard."""

from __future__ import annotations

import fcntl
import os
from pathlib import Path
import tempfile
import unittest

import run_cargo_guarded as guard


class CargoResourceGuardTests(unittest.TestCase):
    def policy(self, lock_path: Path) -> guard.ResourcePolicy:
        return guard.ResourcePolicy(
            lock_path=lock_path,
            lock_timeout_seconds=1,
            minimum_tmp_free_bytes=6 * guard.GIB,
            minimum_target_filesystem_free_bytes=20 * guard.GIB,
            target_soft_limit_bytes=20 * guard.GIB,
            target_hard_limit_bytes=40 * guard.GIB,
            forbid_tmp_target=True,
            proof_incremental=False,
        )

    def snapshot(
        self,
        target: Path,
        *,
        target_gib: int = 10,
        target_free_gib: int = 100,
        tmp_free_gib: int = 10,
    ) -> guard.ResourceSnapshot:
        return guard.ResourceSnapshot(
            target_dir=target,
            target_bytes=target_gib * guard.GIB,
            target_filesystem_free_bytes=target_free_gib * guard.GIB,
            tmp_free_bytes=tmp_free_gib * guard.GIB,
        )

    def test_proof_target_on_tmp_is_refused(self) -> None:
        policy = self.policy(Path("/tmp/lock"))
        errors = guard.evaluate_resources(
            policy, self.snapshot(Path("/tmp/proof-target")), "proof"
        )
        self.assertTrue(any("must not reside on /tmp" in error for error in errors))

    def test_interactive_target_on_tmp_is_not_subject_to_proof_rule(self) -> None:
        policy = self.policy(Path("/tmp/lock"))
        errors = guard.evaluate_resources(
            policy, self.snapshot(Path("/tmp/interactive-target")), "interactive"
        )
        self.assertEqual(errors, [])

    def test_low_tmp_and_target_filesystem_reserves_are_refused(self) -> None:
        policy = self.policy(Path("/tmp/lock"))
        errors = guard.evaluate_resources(
            policy,
            self.snapshot(
                Path("/workspace/target"), target_free_gib=19, tmp_free_gib=5
            ),
            "proof",
        )
        self.assertEqual(len(errors), 2)
        self.assertTrue(any("/tmp reserve" in error for error in errors))
        self.assertTrue(any("target-filesystem reserve" in error for error in errors))

    def test_hard_target_limit_refuses_and_soft_limit_warns(self) -> None:
        policy = self.policy(Path("/tmp/lock"))
        snapshot = self.snapshot(Path("/workspace/target"), target_gib=41)
        self.assertTrue(
            any(
                "exceeds hard limit" in error
                for error in guard.evaluate_resources(policy, snapshot, "proof")
            )
        )
        self.assertEqual(len(guard.resource_warnings(policy, snapshot)), 1)

    def test_proof_environment_disables_incremental_compilation(self) -> None:
        policy = self.policy(Path("/tmp/lock"))
        environment = guard.command_environment(
            {"CARGO_INCREMENTAL": "1"}, "proof", Path("/disk/target"), policy
        )
        self.assertEqual(environment["CARGO_INCREMENTAL"], "0")
        self.assertEqual(environment["CARGO_TARGET_DIR"], "/disk/target")

    def test_interactive_environment_preserves_incremental_choice(self) -> None:
        policy = self.policy(Path("/tmp/lock"))
        environment = guard.command_environment(
            {"CARGO_INCREMENTAL": "1"},
            "interactive",
            Path("/disk/target"),
            policy,
        )
        self.assertEqual(environment["CARGO_INCREMENTAL"], "1")

    def test_second_guard_times_out_while_lock_is_held(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            lock_path = Path(directory) / "cargo.lock"
            with lock_path.open("a+", encoding="utf-8") as first:
                fcntl.flock(first.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                with self.assertRaisesRegex(
                    guard.ResourcePolicyError, "timed out waiting"
                ):
                    with guard.acquire_lock(lock_path, 0.01):
                        self.fail("contended lock must not be acquired")

    def test_checked_in_policy_loads(self) -> None:
        policy = guard.load_policy()
        self.assertFalse(policy.proof_incremental)
        self.assertGreater(policy.target_hard_limit_bytes, policy.target_soft_limit_bytes)


if __name__ == "__main__":
    unittest.main()
