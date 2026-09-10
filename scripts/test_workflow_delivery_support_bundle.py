"""Pinned runner-byte preparation in fixture Git stores; no live installation."""

import json
import os
import subprocess
import sys
import unittest
from unittest.mock import patch

from workflow_delivery_support_bundle import expected_bundle, prepare_bundle, verify_bundle
from workflow_delivery_support_store import support_locations
from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_native_test_support import environment
from workflow_delivery_tree import Tree
from workflow_delivery_capture_state import git
from workflow_delivery_test_support import Fixture
from workflow_delivery_trust_test_support import accepted_fixture


class SupportBundleTest(unittest.TestCase):
    def setUp(self):
        self.f = Fixture()
        self.addCleanup(self.f.close)
        _, _, _, self.authority = accepted_fixture(self.f)
        self.locations = support_locations(self.f.root, self.authority)

    def test_prepare_uses_git_bytes_and_does_not_install_or_change_checkout(self):
        original = (self.f.root / "scripts/check_workflow_delivery.py").read_bytes()
        self.f.write("scripts/check_workflow_delivery.py", b"candidate-only bytes\n")
        before = self.f.snapshot()
        config = (self.f.root / ".git/config").read_bytes()
        head = self.f.git("rev-parse", "HEAD")
        manifest = prepare_bundle(self.f.root, self.authority)
        self.assertEqual(original, self.locations["runner"].read_bytes())
        self.assertEqual(manifest, verify_bundle(self.f.root, self.authority))
        self.assertEqual(before, self.f.snapshot())
        self.assertEqual(config, (self.f.root / ".git/config").read_bytes())
        self.assertEqual(head, self.f.git("rev-parse", "HEAD"))
        self.assertFalse(manifest["activation_asserted"])

    def test_existing_complete_or_partial_bundle_is_not_overwritten(self):
        self.locations["trusted"].mkdir(parents=True)
        sentinel = self.locations["trusted"] / "partial"
        sentinel.write_text("retain this failed preparation")
        with self.assertRaises(FileExistsError):
            prepare_bundle(self.f.root, self.authority)
        self.assertEqual("retain this failed preparation", sentinel.read_text())
        with self.assertRaisesRegex(ValueError, "incomplete"):
            verify_bundle(self.f.root, self.authority)

    def test_replacement_refs_and_repository_overrides_cannot_redirect_pinned_bytes(self):
        original = (self.f.root / "scripts/check_workflow_delivery.py").read_bytes()
        self.f.write("scripts/check_workflow_delivery.py", b"# substituted authority source\n")
        self.f.stage()
        self.f.git("commit", "-qm", "test(workflow): create replacement input\n\nSynthetic authority attack only.")
        replacement = self.f.git("rev-parse", "HEAD").decode().strip()
        self.f.git("replace", self.authority, replacement)
        with patch.dict(os.environ, {"GIT_DIR": "/nonexistent-foreign-repository",
                                     "GIT_INDEX_FILE": "/nonexistent-foreign-index"}):
            _, sources = expected_bundle(self.f.root, self.authority)
            self.assertEqual(original, sources["scripts/check_workflow_delivery.py"])
            prepare_bundle(self.f.root, self.authority)
            verify_bundle(self.f.root, self.authority)

    def test_prepared_runner_executes_without_bytecode_or_local_activation(self):
        self.f.save("requested-environment.json", environment())
        self.f.stage()
        prepare_bundle(self.f.root, self.authority)
        config = (self.f.root / ".git/config").read_bytes()
        result = subprocess.run([sys.executable, "-I", "-S", "-B",
            str(self.locations["scripts"] / "workflow_delivery_observe_python.py"),
            "--script", str(self.locations["runner"]), "--source-root", str(self.locations["trusted"]),
            "--output", str(self.f.root / "prepared-run.jsonl"), "--function", "main", "--",
            "--root", str(self.f.root), "--enforce", "--staged",
            "--authority-ref", self.authority, "--base-ref", self.authority,
            "--environment-path", "requested-environment.json"], capture_output=True, timeout=30)
        self.assertEqual(0, result.returncode, result.stderr)
        self.assertEqual(["TASK: accept"], json.loads(result.stdout)["checks"])
        verify_bundle(self.f.root, self.authority)
        self.assertEqual(config, (self.f.root / ".git/config").read_bytes())

    def test_local_graft_cannot_rewrite_pinned_history(self):
        before = Tree(self.f.root).git("rev-list", self.authority)
        self.f.write(".git/info/grafts", (self.authority + "\n").encode())
        self.assertEqual(before, Tree(self.f.root).git("rev-list", self.authority))
        self.assertEqual(before, git(self.f.root, "rev-list", self.authority))

    def test_tampered_manifest_cannot_bless_changed_runner_bytes(self):
        prepare_bundle(self.f.root, self.authority)
        runner = self.locations["runner"]
        runner.chmod(0o644)
        runner.write_text("changed implementation")
        runner.chmod(0o444)
        manifest_path = self.locations["trusted"] / "manifest.json"
        manifest = json.loads(manifest_path.read_text())
        for row in manifest["files"]:
            if row["path"] == "scripts/check_workflow_delivery.py":
                row["sha256"] = sha256(runner.read_bytes())
        manifest_path.chmod(0o644)
        manifest_path.write_bytes(canonical_json(manifest))
        manifest_path.chmod(0o444)
        with self.assertRaisesRegex(ValueError, "pinned Git"):
            verify_bundle(self.f.root, self.authority)

    def test_extra_import_or_bytecode_cache_refuses(self):
        prepare_bundle(self.f.root, self.authority)
        for name in ("json.py", "__pycache__", ".hidden.py"):
            path = self.locations["scripts"] / name
            path.write_text("unapproved runtime input")
            with self.assertRaisesRegex(ValueError, "module set differs"):
                verify_bundle(self.f.root, self.authority)
            path.unlink()

    def test_writable_or_redirected_runtime_files_refuse(self):
        prepare_bundle(self.f.root, self.authority)
        runner = self.locations["runner"]
        runner.chmod(0o644)
        with self.assertRaisesRegex(ValueError, "read-only"):
            verify_bundle(self.f.root, self.authority)
        runner.unlink()
        runner.symlink_to(self.f.root / "scripts/check_workflow_delivery.py")
        with self.assertRaisesRegex(ValueError, "nonredirected"):
            verify_bundle(self.f.root, self.authority)

    def test_authority_without_runtime_is_rejected_before_creating_store(self):
        with self.assertRaises((ValueError, KeyError)):
            expected_bundle(self.f.root, self.f.head)
        self.assertFalse(self.locations["trusted"].exists())


if __name__ == "__main__":
    unittest.main()
