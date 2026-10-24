#!/usr/bin/env python3
"""Hermetic safety checks for explicit Cargo cache cleanup."""

import fcntl
import os
from unittest import mock
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

    def clean(self, *paths: str, metadata_only: bool = False, apply: bool = True) -> None:
        cache.clean_caches(self.root, list(paths), self.lock, self.proc, metadata_only=metadata_only, apply=apply)

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


class IncrementalMetadataTests(CargoCacheTests):
    def metadata_tree(self, relative="target/debug/incremental"):
        root = self.root / relative
        session = root / "example-abc123/s-session-abc123-working"
        session.mkdir(parents=True)
        (session.parent / "s-session-abc123.lock").touch()
        (session / "query-cache.bin").write_bytes(b"RSIC compiler metadata")
        (session / "metadata.rmeta").write_bytes(b"rust\x00\x00\x00\x0a metadata")
        return root, session

    def protected_process(self):
        process = self.proc / "123"
        process.mkdir()
        (process / "comm").write_text("system-service")
        original = Path.readlink
        def readlink(path):
            if path == process / "exe":
                raise PermissionError("simulated same-user executable protection")
            return original(path)
        return mock.patch.object(Path, "readlink", readlink)

    def test_metadata_mode_is_explicit_and_preserves_other_storage(self):
        root, _ = self.metadata_tree()
        binary = self.root / "target/debug/application"
        binary.write_bytes(b"retained executable")
        other = self.build_target("target/other-proof")
        with self.protected_process():
            with self.assertRaises(PermissionError):
                self.clean("target/debug/incremental")
            self.clean("target/debug/incremental", metadata_only=True, apply=False)
            self.assertTrue(root.exists())
            self.clean("target/debug/incremental", metadata_only=True)
        self.assertFalse(root.exists())
        self.assertEqual(binary.read_bytes(), b"retained executable")
        self.assertTrue(other.exists())

    def test_metadata_hardlinks_only_release_selected_names(self):
        root, session = self.metadata_tree()
        release, other = self.metadata_tree("target/release/incremental")
        (other / "query-cache.bin").unlink()
        os.link(session / "query-cache.bin", other / "query-cache.bin")
        outside = self.root / "retained-proof-metadata"
        os.link(session / "query-cache.bin", outside)
        self.clean("target/debug/incremental", "target/release/incremental", metadata_only=True)
        self.assertFalse(root.exists())
        self.assertFalse(release.exists())
        self.assertEqual(outside.read_bytes(), b"RSIC compiler metadata")
        self.assertEqual(outside.stat().st_nlink, 1)

    def test_metadata_mode_refuses_general_and_mixed_paths_before_deletion(self):
        root, _ = self.metadata_tree()
        other = self.build_target("target/other-proof")
        for paths in [("target/other-proof",), ("target/debug/incremental", "target/other-proof"), ("./target/debug/incremental",)]:
            with self.subTest(paths=paths), self.assertRaises(guard.ResourcePolicyError):
                self.clean(*paths, metadata_only=True)
            self.assertTrue(root.exists())
            self.assertTrue(other.exists())

    def test_metadata_mode_rejects_every_unsupported_file_kind_without_deletion(self):
        root, session = self.metadata_tree()
        good = session / "query-cache.bin"
        cases = [("unknown.txt", b"RSIC"), ("object.o", b"RSIC"), ("dep-graph.bin", b"\x7fELF"),
                 ("work-products.bin", b"#!/bin/sh"), ("metadata.rmeta", b"rust wrong version")]
        for name, content in cases:
            path = session / name
            old = path.read_bytes() if path.exists() else None
            path.write_bytes(content)
            with self.subTest(name=name), self.assertRaises(guard.ResourcePolicyError):
                self.clean("target/debug/incremental", metadata_only=True)
            self.assertTrue(good.exists())
            if old is None: path.unlink()
            else: path.write_bytes(old)
        good.chmod(0o755)
        with self.assertRaises(guard.ResourcePolicyError): self.clean("target/debug/incremental", metadata_only=True)
        good.chmod(0o644)
        lock = session.parent / "s-session-abc123.lock"
        lock.write_bytes(b"not empty")
        with self.assertRaises(guard.ResourcePolicyError): self.clean("target/debug/incremental", metadata_only=True)
        self.assertTrue(good.exists())

    def test_metadata_mode_rejects_links_special_files_and_read_failures(self):
        root, session = self.metadata_tree()
        outside = self.root / "proof.log"
        outside.write_bytes(b"RSIC retained")
        link = session / "dep-graph.bin"
        link.symlink_to(outside)
        with self.assertRaises(guard.ResourcePolicyError): self.clean("target/debug/incremental", metadata_only=True)
        link.unlink()
        link.symlink_to(self.root, target_is_directory=True)
        with self.assertRaises(guard.ResourcePolicyError): self.clean("target/debug/incremental", metadata_only=True)
        link.unlink()
        os.mkfifo(link)
        with self.assertRaises(guard.ResourcePolicyError): self.clean("target/debug/incremental", metadata_only=True)
        link.unlink()
        original = os.open
        def unreadable(path, *args, **kwargs):
            if path == "query-cache.bin": raise PermissionError("metadata unreadable")
            return original(path, *args, **kwargs)
        with mock.patch.object(os, "open", unreadable), self.assertRaises(PermissionError):
            self.clean("target/debug/incremental", metadata_only=True)
        self.assertEqual(outside.read_bytes(), b"RSIC retained")
        self.assertTrue(root.exists())

    def test_metadata_mode_retains_active_build_application_cache_user_and_lock_guards(self):
        root, session = self.metadata_tree()
        process = self.proc / "123"
        process.mkdir()
        for name in ["cargo", "rustc", "datum-gui", "eda-engine-daem"]:
            (process / "comm").write_text(name)
            with self.subTest(name=name), self.assertRaises(guard.ResourcePolicyError):
                self.clean("target/debug/incremental", metadata_only=True)
        (process / "comm").write_text("custom-worker")
        (process / "exe").symlink_to(session / "query-cache.bin")
        with self.assertRaises(guard.ResourcePolicyError): self.clean("target/debug/incremental", metadata_only=True)
        (process / "exe").unlink()
        with self.lock.open("w") as held:
            fcntl.flock(held.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            with self.assertRaises(guard.ResourcePolicyError): self.clean("target/debug/incremental", metadata_only=True)
        self.assertTrue(root.exists())

    def test_metadata_mode_revalidates_all_roots_and_content_before_any_deletion(self):
        for mutation in ["content", "added", "inode", "mode"]:
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                # A fresh repository prevents one refused mutation from affecting the next.
                saved = self.root
                self.root = Path(temporary)
                subprocess.run(["git", "init", "-q", str(self.root)], check=True)
                first, session = self.metadata_tree()
                second, _ = self.metadata_tree("target/release/incremental")
                original = cache.metadata.MetadataTree.capture
                calls = 0
                def change(tree):
                    nonlocal calls
                    result = original(tree)
                    calls += 1
                    if calls == 2:
                        target = session / "query-cache.bin"
                        if mutation == "content": target.write_bytes(b"RSIC changed metadata")
                        elif mutation == "added": (session / "unknown.txt").write_bytes(b"not cache")
                        elif mutation == "inode":
                            target.unlink(); target.write_bytes(b"RSIC compiler metadata")
                        else: target.chmod(0o600)
                    return result
                try:
                    with mock.patch.object(cache.metadata.MetadataTree, "capture", change), self.assertRaises(guard.ResourcePolicyError):
                        self.clean("target/debug/incremental", "target/release/incremental", metadata_only=True)
                    self.assertTrue(first.exists())
                    self.assertTrue(second.exists())
                    self.assertTrue((session / "metadata.rmeta").exists())
                finally: self.root = saved

    def test_metadata_mode_refuses_directory_replacement_before_deletion(self):
        root, session = self.metadata_tree()
        original = cache.metadata.MetadataTree.capture
        calls = 0
        def replace(tree):
            nonlocal calls
            result = original(tree)
            calls += 1
            if calls == 1:
                root.rename(root.parent / "retained-old")
                self.metadata_tree()
            return result
        with mock.patch.object(cache.metadata.MetadataTree, "capture", replace), self.assertRaises(guard.ResourcePolicyError):
            self.clean("target/debug/incremental", metadata_only=True)
        self.assertTrue((root.parent / "retained-old/example-abc123/s-session-abc123-working/query-cache.bin").exists())
        self.assertTrue((session / "metadata.rmeta").exists())


if __name__ == "__main__":
    unittest.main()
