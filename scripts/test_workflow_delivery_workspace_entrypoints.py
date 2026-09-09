"""Actual CLI/selector subprocesses with owner-pinned workspace fixture policy."""

import hashlib
import importlib.util
import json
from pathlib import Path
import py_compile
import subprocess
import sys
import unittest

import test_workflow_delivery_coverage_entrypoints as coverage_fixture
from workflow_delivery_tree import Tree
from workflow_delivery_workspace_authority import WORKSPACE_POLICY_PATH


class WorkspaceEntrypointsTest(unittest.TestCase):
    def setUp(self):
        fixture = coverage_fixture.CoverageEntrypointsTest()
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        self.f = fixture.f
        self.f.write(".git/info/exclude", b"src/__pycache__/\ntools/local.sh\n")
        raw = b"#!/bin/sh\nexit 0\n"
        self.f.write("tools/local.sh", raw)
        (self.f.root / "tools/local.sh").chmod(0o775)
        policy_path = "specs/workflow_delivery_policy.json"
        coverage_policy = Tree(self.f.root).json(policy_path)
        coverage_policy["coverage"]["production_roots"].append("tools")
        self.f.save(policy_path, coverage_policy)
        self.f.save(WORKSPACE_POLICY_PATH, {
            "schema_version": 1, "kind": "datum.workflow-delivery.workspace-inputs",
            "python_caches": "verified-source-v1", "beads_runtime": "standard-v1",
            "local_files": [{"path": "tools/local.sh", "category": "owner_local",
                             "sha256": hashlib.sha256(raw).hexdigest(), "size": len(raw), "mode": 0o775}]})
        self.f.git("add", WORKSPACE_POLICY_PATH, policy_path)
        self.f.git("commit", "-qm", "fixture exact workspace authority")
        self.authority = self.f.git("rev-parse", "HEAD").decode().strip()
        self.f.git("config", "datum.workflowDeliveryAuthorityRef", self.authority)
        self.f.git("config", "datum.workflowDeliveryBaseRef", self.authority)
        source = self.f.root / "src/read.py"
        py_compile.compile(str(source), doraise=True)
        self.cache = Path(importlib.util.cache_from_source(str(source)))
        self.scripts = Path(__file__).resolve().parent

    def commands(self):
        return [
            [sys.executable, "-B", str(self.scripts / "check_workflow_delivery.py"),
             "--root", str(self.f.root), "--enforce", "--authority-ref", self.authority,
             "--base-ref", self.authority, "--environment-path", "requested-environment.json"],
            [sys.executable, "-B", str(self.scripts / "project_status.py"),
             "--root", str(self.f.root), "check"],
        ]

    def check_commands(self, expected, text=None):
        for command in self.commands():
            result = subprocess.run(command, cwd=self.f.root, capture_output=True, text=True, timeout=30)
            self.assertEqual(expected, result.returncode == 0, result.stdout + result.stderr)
            if text:
                self.assertIn(text, result.stdout + result.stderr)

    def test_actual_worktree_cli_and_selector_accept_pinned_runtime(self):
        original = self.cache.read_bytes()
        self.check_commands(True)
        self.assertEqual(original, self.cache.read_bytes())

    def test_actual_entrypoints_refuse_ignored_source_and_changed_local_file(self):
        self.f.write("src/__pycache__/unknown.py", b"value = 1\n")
        self.check_commands(False, "WDQ-COVERAGE")
        (self.f.root / "src/__pycache__/unknown.py").unlink()
        self.f.write("tools/local.sh", b"changed")
        self.check_commands(False, "WDQ-COVERAGE")

    def test_actual_entrypoints_refuse_changed_policy_and_deleted_pin(self):
        policy = self.f.root / WORKSPACE_POLICY_PATH
        original = policy.read_bytes()
        policy.write_bytes(original + b"\n")
        self.check_commands(False, "WDQ-POLICY")
        policy.write_bytes(original)
        (self.f.root / "tools/local.sh").unlink()
        self.check_commands(False, "required pinned local file is missing")

    def test_pinned_owner_file_inside_proof_roots_still_invalidates_old_proof(self):
        name = "src/runtime-script.sh"
        raw = b"#!/bin/sh\nexit 0\n"
        self.f.write(name, raw)
        (self.f.root / name).chmod(0o775)
        exclusion = self.f.root / ".git/info/exclude"
        exclusion.write_bytes(exclusion.read_bytes() + (name + "\n").encode())
        policy = Tree(self.f.root).json(WORKSPACE_POLICY_PATH)
        policy["local_files"].append({"path": name, "category": "owner_local",
            "sha256": hashlib.sha256(raw).hexdigest(), "size": len(raw), "mode": 0o775})
        self.f.save(WORKSPACE_POLICY_PATH, policy)
        self.f.git("add", WORKSPACE_POLICY_PATH)
        self.f.git("commit", "-qm", "fixture pinned runtime input")
        self.authority = self.f.git("rev-parse", "HEAD").decode().strip()
        self.f.git("config", "datum.workflowDeliveryAuthorityRef", self.authority)
        self.f.git("config", "datum.workflowDeliveryBaseRef", self.authority)
        self.check_commands(False, "WDQ-STALE")

    def test_actual_preflight_cli_uses_pinned_policy_and_preserves_runtime(self):
        from workflow_delivery_capture_state import protected_state
        from workflow_delivery_io import canonical_json, sha256
        from workflow_delivery_publication_delta import publication_delta
        from workflow_delivery_support_bundle import prepare_bundle
        from workflow_delivery_support_store import proposed_local_trust

        prepare_bundle(self.f.root, self.authority)
        tree_id = self.f.git("rev-parse", "HEAD^{tree}").decode().strip()
        candidate = self.f.git("commit-tree", tree_id, "-p", self.authority,
                               "-m", "fixture prospective publication").decode().strip()
        roots = ["src", ".beads", "tools"]
        before = protected_state(self.f.root, roots)
        delta = publication_delta(self.f.root, base=self.authority, candidate=candidate)
        request = {"schema_version": 1, "kind": "datum.workflow-delivery.preflight-request",
            "base": self.authority, "candidate": candidate, "authority": self.authority,
            "input_roots": roots, "prior_local_trust": before["local_trust"],
            "publication_review": {"delta_sha256": sha256(canonical_json(delta)), "paths": delta["touched_paths"]},
            "environment_path": "requested-environment.json",
            "proposed_local_trust": proposed_local_trust(self.f.root, authority=self.authority,
                base=self.authority, environment_path="requested-environment.json")}
        request_path = self.f.root / ".git/datum-wdq/request.json"
        request_path.write_bytes(canonical_json(request))
        output = self.f.root / ".git/datum-wdq/logs/workspace-preflight.json"
        result = subprocess.run([sys.executable, "-I", "-S", "-B",
            str(self.scripts / "workflow_delivery_preflight_cli.py"), "--inspect",
            "--root", str(self.f.root), "--request", str(request_path),
            "--request-sha256", sha256(request_path.read_bytes()), "--output", str(output)],
            capture_output=True, text=True, timeout=30)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        payload = json.loads(output.read_bytes())
        self.assertEqual(before, payload["preflight"]["protected_state"])
        self.assertFalse(payload["activation_asserted"])
        self.assertEqual(before, protected_state(self.f.root, roots))


if __name__ == "__main__":
    unittest.main()
