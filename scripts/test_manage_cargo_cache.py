#!/usr/bin/env python3
"""Hermetic safety checks for explicit Cargo cache cleanup."""

import fcntl
from pathlib import Path
import subprocess
import tempfile
import unittest

import manage_cargo_cache as cache
import run_cargo_guarded as guard


class CargoCacheTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        self.proc = self.root / "fake-proc"
        self.proc.mkdir()
        self.lock = self.root / "resource.lock"

    def build_target(self, relative: str) -> Path:
        path = self.root / relative
        (path / "debug").mkdir(parents=True)
        (path / ".rustc_info.json").write_text("{}")
        (path / "debug/.cargo-lock").touch()
        (path / "debug/test-binary").write_text("rebuildable")
        return path

    def clean(self, *paths: str) -> None:
        cache.clean_caches(self.root, list(paths), self.lock, self.proc)

    def test_only_selected_disposable_target_is_removed(self) -> None:
        selected = self.build_target("target/proof-a")
        other = self.build_target("target/proof-b")
        self.clean("target/proof-a")
        self.assertFalse(selected.exists())
        self.assertTrue(other.exists())

    def test_shared_cache_fixtures_tracked_and_escape_paths_refuse(self) -> None:
        self.build_target("target")
        tracked = self.build_target("target/tracked")
        subprocess.run(["git", "add", "--", "target/tracked/.rustc_info.json"], cwd=self.root, check=True)
        (self.root / "target/fixture").mkdir()
        cases = ("target", "target/debug", "target/release", "target/fixture", "target/tracked", "../outside", str(self.root))
        for relative in cases:
            with self.subTest(relative=relative), self.assertRaises(guard.ResourcePolicyError):
                self.clean(relative)
        self.assertTrue(tracked.exists())

    def test_symlink_and_symlink_parent_refuse(self) -> None:
        path = self.build_target("elsewhere")
        (self.root / "target").mkdir()
        (self.root / "target/proof-link").symlink_to(path, target_is_directory=True)
        with self.assertRaises(guard.ResourcePolicyError):
            self.clean("target/proof-link")
        (self.root / ".datum").symlink_to(self.root / "target", target_is_directory=True)
        self.build_target("target/agent-cargo-archive-old")
        with self.assertRaises(guard.ResourcePolicyError):
            self.clean(".datum/agent-cargo-archive-old")
        self.assertTrue(path.exists())

    def test_active_build_or_application_refuses_cleanup(self) -> None:
        path = self.build_target("target/proof")
        process = self.proc / "123"
        process.mkdir()
        for name in ("cargo", "rustc", "datum-gui", "datum-eda", "eda-engine-daem"):
            (process / "comm").write_text(name)
            with self.subTest(name=name), self.assertRaisesRegex(guard.ResourcePolicyError, "active"):
                self.clean("target/proof")
            self.assertTrue(path.exists())

    def test_contended_build_lock_refuses_cleanup(self) -> None:
        path = self.build_target("target/proof")
        with self.lock.open("w") as held:
            fcntl.flock(held.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            with self.assertRaisesRegex(guard.ResourcePolicyError, "timed out"):
                self.clean("target/proof")
        self.assertTrue(path.exists())

    def test_explicit_incremental_cleanup_preserves_shared_binaries(self) -> None:
        shared = self.build_target("target")
        incremental = shared / "debug/incremental"
        incremental.mkdir()
        (incremental / "object").write_text("rebuildable")
        self.clean("target/debug/incremental")
        self.assertFalse(incremental.exists())
        self.assertTrue((shared / "debug/test-binary").exists())

    def test_mixed_content_target_refuses_cleanup(self) -> None:
        path = self.build_target("target/proof")
        for name in ("fixture.kicad_pcb", "native-proof.log"):
            output = path / name
            output.write_text("not a compiler cache")
            with self.subTest(name=name), self.assertRaisesRegex(guard.ResourcePolicyError, "mixed-content"):
                self.clean("target/proof")
            self.assertTrue(output.exists())
            output.unlink()

    def test_unknown_running_cache_executable_refuses_cleanup(self) -> None:
        path = self.build_target("target/proof")
        process = self.proc / "123"
        process.mkdir()
        (process / "comm").write_text("custom-worker")
        (process / "exe").symlink_to(path / "debug/test-binary")
        with self.assertRaisesRegex(guard.ResourcePolicyError, "active cache executables"):
            self.clean("target/proof")
        self.assertTrue(path.exists())

    def test_archive_cleanup_and_all_paths_validated_before_deletion(self) -> None:
        archive = self.root / ".datum/agent-cargo-archive-old"
        archive.mkdir(parents=True)
        (archive / "binary").write_text("rebuildable")
        with self.assertRaises(guard.ResourcePolicyError):
            self.clean(".datum/agent-cargo-archive-old", "target/missing")
        self.assertTrue(archive.exists())
        self.clean(".datum/agent-cargo-archive-old")
        self.assertFalse(archive.exists())


if __name__ == "__main__":
    unittest.main()
