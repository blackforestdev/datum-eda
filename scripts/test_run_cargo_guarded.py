#!/usr/bin/env python3
"""Hermetic tests for Datum's Cargo resource guard."""

from __future__ import annotations

import fcntl
from contextlib import nullcontext, redirect_stderr
import io
import json
import os
from pathlib import Path
import tempfile
import subprocess
import sys
import unittest
from dataclasses import replace
from unittest.mock import Mock, patch

import run_cargo_guarded as guard


class CargoResourceGuardTests(unittest.TestCase):
    def test_nested_child_refuses_before_waiting_on_parent_lock(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            lock_path = Path(directory) / "parent.lock"
            policy_path = Path(directory) / "policy.json"
            policy = json.loads(guard.DEFAULT_POLICY.read_text())
            policy["lock_path"] = str(lock_path)
            policy_path.write_text(json.dumps(policy))
            environment = guard.command_environment(
                dict(os.environ), "proof", guard.ROOT / "target", guard.load_policy()
            )
            with guard.acquire_lock(lock_path, 1):
                result = subprocess.run(
                    [sys.executable, str(guard.ROOT / "scripts/run_cargo_guarded.py"),
                     "--workload", "proof", "--policy", str(policy_path), "--check-only"],
                    env=environment, capture_output=True, text=True, timeout=3,
                )
        self.assertEqual(result.returncode, 75)
        self.assertIn("nested Cargo", result.stderr)
        self.assertNotIn("preflight", result.stderr)

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

    def test_small_selected_target_cannot_bypass_retained_cache_limit(self) -> None:
        policy = self.policy(Path("/tmp/lock"))
        snapshot = replace(
            self.snapshot(Path("/workspace/target/small"), target_gib=1),
            retained_cache_bytes=41 * guard.GIB,
        )
        self.assertTrue(any("exceeds hard limit" in error for error in guard.evaluate_resources(policy, snapshot, "proof")))
        self.assertEqual(len(guard.resource_warnings(policy, snapshot)), 1)

    def test_retained_cache_paths_include_archives_without_duplicate_targets(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = root / ".datum/agent-cargo-archive-old"
            archive.mkdir(parents=True)
            paths = guard.retained_cache_paths(root, root / "target/proof")
            self.assertEqual(set(paths), {root / "target", archive})
            external = root / "elsewhere"
            self.assertEqual(set(guard.retained_cache_paths(root, external)), {root / "target", archive, external})


    def invoke(self, command, *, target=None, environment=None, cwd=None, explicit=True):
        target = (target or guard.ROOT / "target").resolve()
        policy = replace(guard.load_policy(), forbid_tmp_target=False)
        self.lock = Mock(return_value=nullcontext())
        self.preflight = Mock(return_value=self.snapshot(target))
        self.native_check = Mock()
        self.child = Mock(return_value=subprocess.CompletedProcess(command, 17))
        error = io.StringIO()
        args = ["--workload", "proof"]
        if explicit:
            args += ["--target-dir", str(target)]
        with patch.dict(os.environ, environment or {}, clear=True), \
                patch.object(guard, "load_policy", return_value=policy), \
                patch.object(guard, "acquire_lock", self.lock), \
                patch.object(guard, "snapshot_resources", self.preflight), \
                patch.object(guard, "refuse_native_projects", self.native_check), \
                patch.object(guard.subprocess, "run", self.child), \
                patch.object(guard.Path, "cwd", return_value=cwd or Path.cwd()), \
                redirect_stderr(error):
            result = guard.main([*args, "--", *command])
        return result, error.getvalue()

    def assert_early_refusal(self, command, **kwargs):
        result, message = self.invoke(command, **kwargs)
        self.assertEqual(result, 75)
        self.lock.assert_not_called()
        self.preflight.assert_not_called()
        self.native_check.assert_not_called()
        self.child.assert_not_called()
        return message

    def test_direct_separate_and_joined_target_conflicts_refuse_before_lock(self):
        for arguments in (["--target-dir", "/foreign/output"], ["--target-dir=/foreign/output"]):
            with self.subTest(arguments=arguments):
                self.assert_early_refusal(["cargo", "test", *arguments])

    def test_matching_direct_paths_preserve_exact_argv_and_child_exit(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "target"
            target.mkdir()
            (root / "target-link").symlink_to(target, target_is_directory=True)
            for arguments in (["--target-dir", "target"], ["--target-dir=target-link"]):
                with self.subTest(arguments=arguments):
                    command = ["cargo", "test", *arguments]
                    result, _ = self.invoke(command, target=target, cwd=root)
                    self.assertEqual(result, 17)
                    self.child.assert_called_once()
                    self.assertEqual(self.child.call_args.args[0], command)
                    self.preflight.assert_called_once_with(target)
                    env = self.child.call_args.kwargs["env"]
                    for key in ("CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR", "CARGO_BUILD_BUILD_DIR"):
                        self.assertEqual(env[key], str(target))
                    self.assertEqual(env["CARGO_INCREMENTAL"], "0")

    def test_config_inline_conflicts_joined_separate_and_quoted_keys(self):
        for argument in ('build.target-dir="/foreign/output"',
                         '"build"."target-dir"="/foreign/output"',
                         'build={build-dir="/foreign/output"}'):
            for flags in (["--config", argument], ["--config=" + argument]):
                with self.subTest(flags=flags):
                    self.assert_early_refusal(["cargo", *flags, "build"])

    def test_effective_config_precedence_allows_shadowed_mismatches(self):
        target = guard.ROOT / "target"
        matching = 'build.target-dir=' + json.dumps(str(target))
        commands = [
            ["cargo", "build", "--config", 'build.target-dir="/foreign/output"',
             "--config=" + matching],
            ["cargo", "build", "--config", 'build.target-dir="/foreign/output"',
             "--target-dir", str(target)],
        ]
        for command in commands:
            with self.subTest(command=command):
                result, _ = self.invoke(command, target=target)
                self.assertEqual(result, 17)
                self.assertEqual(self.child.call_args.args[0], command)
        self.assert_early_refusal(["cargo", "build", "--target-dir", str(target),
                                   "--config", 'build.build-dir="/foreign/output"'])

    def test_config_file_paths_are_relative_two_levels_above_file(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            config = root / "configs" / "special.toml"
            config.parent.mkdir()
            config.write_text('[build]\ntarget-dir="target"\nbuild-dir="target"\n')
            command = ["cargo", "build", "--config", "configs/special.toml"]
            self.assertEqual(self.invoke(command, target=root / "target", cwd=root)[0], 17)
            self.assert_early_refusal(command, target=config.parent / "target", cwd=root)
            config.write_text('[build]\ntarget-dir="/foreign/output"\n')
            self.assert_early_refusal(command, target=root / "target", cwd=root)

    def test_nested_includes_and_own_file_values_follow_cargo_precedence(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            configs = root / ".cargo"
            configs.mkdir()
            (configs / "first.toml").write_text('[build]\ntarget-dir="/foreign/output"\n')
            (configs / "last.toml").write_text('[build]\ntarget-dir="target"\n')
            (configs / "nested.toml").write_text('include=["first.toml", "last.toml"]\n')
            config = configs / "extra.toml"
            config.write_text('include=["nested.toml", {path="missing.toml", optional=true}]\n')
            command = ["cargo", "build", "--config=.cargo/extra.toml"]
            self.assertEqual(self.invoke(command, target=root / "target", cwd=root)[0], 17)
            config.write_text('include=["first.toml"]\n[build]\ntarget-dir="target"\n')
            self.assertEqual(self.invoke(command, target=root / "target", cwd=root)[0], 17)
            config.write_text('include=["first.toml"]\n')
            self.assert_early_refusal(command, target=root / "target", cwd=root)
            config.write_text('include=["extra.toml"]\n')
            self.assert_early_refusal(command, target=root / "target", cwd=root)

    def test_build_directory_env_or_template_cannot_escape_measured_target(self):
        self.assert_early_refusal(["cargo", "build"],
                                   environment={"CARGO_BUILD_BUILD_DIR": "/foreign/intermediates"})
        self.assert_early_refusal(["cargo", "build", "--config",
                                   'build.build-dir="{workspace-root}/target"'])
        target = guard.ROOT / "target"
        command = ["cargo", "build", "--config", 'build.build-dir=' + json.dumps(str(target))]
        self.assertEqual(self.invoke(command, target=target,
                                    environment={"CARGO_BUILD_BUILD_DIR": "/shadowed/path"})[0], 17)

    def test_nested_intermediates_are_preserved_but_escaping_symlinks_refuse(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "target"
            target.mkdir()
            (target / "escape").symlink_to(root / "outside", target_is_directory=True)
            for config in (False, True):
                command = ["cargo", "build"]
                environment = {"CARGO_BUILD_BUILD_DIR": "target/intermediate"}
                if config:
                    command += ["--config", 'build.build-dir="target/nested"']
                self.assertEqual(self.invoke(command, target=target, cwd=root, environment=environment)[0], 17)
                expected = target / ("nested" if config else "intermediate")
                self.assertEqual(self.child.call_args.kwargs["env"]["CARGO_BUILD_BUILD_DIR"], str(expected))
            self.assert_early_refusal(["cargo", "build"], target=target, cwd=root,
                                      environment={"CARGO_BUILD_BUILD_DIR": "target/escape"})

    def test_target_env_aliases_are_selected_and_canonicalized(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with patch.dict(os.environ, {"CARGO_BUILD_TARGET_DIR": "selected"}, clear=True), \
                    patch.object(guard.Path, "cwd", return_value=root):
                self.assertEqual(guard._target_from_arguments(None), root / "selected")
            with patch.dict(os.environ, {"CARGO_TARGET_DIR": "primary",
                                         "CARGO_BUILD_TARGET_DIR": "secondary"}, clear=True), \
                    patch.object(guard.Path, "cwd", return_value=root):
                self.assertEqual(guard._target_from_arguments(None), root / "primary")
                self.assertEqual(guard._target_from_arguments("explicit"), root / "explicit")
            self.invoke(["cargo", "build"], target=root / "explicit",
                        environment={"CARGO_TARGET_DIR": "primary", "CARGO_BUILD_TARGET_DIR": "secondary"})
            env = self.child.call_args.kwargs["env"]
            self.assertEqual(env["CARGO_TARGET_DIR"], env["CARGO_BUILD_TARGET_DIR"])

    def test_mocked_main_default_and_environment_selected_paths(self):
        self.assertEqual(self.invoke(["cargo", "metadata"], explicit=False)[0], 17)
        self.preflight.assert_called_once_with((guard.ROOT / "target").resolve())
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for key in ("CARGO_TARGET_DIR", "CARGO_BUILD_TARGET_DIR"):
                with self.subTest(key=key):
                    result, _ = self.invoke(["cargo", "build"], target=root / "selected",
                                            cwd=root, environment={key: "selected"}, explicit=False)
                    self.assertEqual(result, 17)
                    self.preflight.assert_called_once_with(root / "selected")
                    self.assertEqual(self.child.call_args.kwargs["env"]["CARGO_TARGET_DIR"],
                                     str(root / "selected"))

    def test_unrelated_configuration_metadata_and_post_separator_payload_stay_compatible(self):
        commands = [
            ["cargo", "metadata", "--no-deps", "--config", "net.offline=true"],
            ["cargo", "test", "--", "--target-dir=/foreign/output", "--config", "invalid TOML", "-Celsewhere"],
        ]
        for command in commands:
            with self.subTest(command=command):
                self.assertEqual(self.invoke(command)[0], 17)
                self.assertEqual(self.child.call_args.args[0], command)
        for flag in ("-C", "-Celsewhere"):
            message = self.assert_early_refusal(["cargo", flag, "elsewhere", "build"])
            self.assertIn("run the guard from the desired directory", message)

    def test_config_errors_do_not_echo_secrets_and_missing_values_refuse(self):
        secret = "unit-control-secret-do-not-print"
        for command in (["cargo", "build", "--config", 'http.proxy="' + secret],
                        ["cargo", "build", "--config", 'build.target-dir="/' + secret + '"'],
                        ["cargo", "build", "--config"], ["cargo", "build", "--target-dir", "--"]):
            with self.subTest(command=command):
                message = self.assert_early_refusal(command)
                self.assertNotIn(secret, message)


if __name__ == "__main__":
    unittest.main()
