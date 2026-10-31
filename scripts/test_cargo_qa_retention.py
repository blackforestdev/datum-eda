"""Hermetic released-QA retention controls; no live process or cache actions."""
from contextlib import redirect_stdout, redirect_stderr
import io
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import cargo_qa_retention as r
import manage_cargo_cache as manager


class RetentionTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        subprocess.run(["git", "init", "-q", self.root], check=True)
        (self.root / ".beads").mkdir()
        (self.root / "specs").mkdir()
        self.write(".beads/issues.jsonl", '{"id":"dat-test","status":"closed"}\n')
        self.frontier([])
        self.bundle = "target/qa/example"
        self.create(self.bundle)

    def write(self, path, data):
        p = self.root / path
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(data)

    def frontier(self, rows):
        self.write("specs/active_frontier.json", json.dumps({"frontier": rows}))

    def create(self, bundle):
        base = self.root / bundle
        (base / "debug").mkdir(parents=True)
        elf = b"\x7fELF\x02\x01" + b"\0" * 10 + b"\x02\0payload"
        p = base / "debug/datum-gui"
        p.write_bytes(elf)
        p.chmod(0o755)
        (base / "debug/datum-gui.d").write_text("dependency data")
        self.write(bundle + "/source-snapshot/file.rs", "source evidence")
        self.write(bundle + "/observations.json", '{"evidence":true}')
        hashes = {"debug/" + p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                  for p in (base / "debug").iterdir()}
        self.write(bundle + "/build-info.json", json.dumps({"build_retention": {
            "state": "released", "issue_id": "dat-test"}, "files_sha256": hashes}))

    def run_retire(self, bundles=None, **kwargs):
        return r.retire(self.root, bundles or [self.bundle], self.root / "guard.lock",
                        kwargs.pop("active_check", lambda: []),
                        kwargs.pop("users_check", lambda paths: []), **kwargs)

    def denied(self, **kwargs):
        with self.assertRaises(r.Failure) as result:
            self.run_retire(apply=True, **kwargs)
        self.assertTrue((self.root / self.bundle / "debug/datum-gui").exists())
        return result.exception.receipt

    def test_dry_run_apply_repeat_preserve_evidence(self):
        kept = {str(p.relative_to(self.root)): p.read_bytes()
                for p in (self.root / self.bundle).rglob("*") if p.is_file() and p.parent.name != "debug"}
        self.assertEqual(self.run_retire()["status"], "validated_dry_run")
        self.assertTrue((self.root / self.bundle / "debug/datum-gui").exists())
        result = self.run_retire(apply=True)
        self.assertEqual(len(result["completed"]), 2)
        self.assertEqual(sorted(result["observed_absent"]), sorted(result["completed"]))
        self.assertEqual(self.run_retire(apply=True)["completed"], [])
        for path, data in kept.items():
            self.assertEqual((self.root / path).read_bytes(), data)

    def test_release_and_issue_must_be_positive(self):
        p = self.root / self.bundle / "build-info.json"
        p.write_text('{}')
        self.denied()
        self.create_metadata()
        self.write(".beads/issues.jsonl", '{"id":"dat-test","status":"open"}\n')
        self.denied()
        self.write(".beads/issues.jsonl", '{"id":"dat-test","status":"closed"}\n' * 2)
        self.denied()

    def create_metadata(self):
        base = self.root / self.bundle
        hashes = {"debug/" + p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                  for p in (base / "debug").iterdir()}
        (base / "build-info.json").write_text(json.dumps({"build_retention": {
            "state": "released", "issue_id": "dat-test"}, "files_sha256": hashes}))

    def test_claims_block_even_when_expired_but_disjoint_allowed(self):
        for row in [
            {"issue_id": "dat-test", "claim": {"scope": [], "expires_at": "2000-01-01"}},
            {"issue_id": "other", "claim": {"scope": ["target/qa"]}},
            {"issue_id": "other", "claim": {"scope": [self.bundle + "/debug/*"]}},
        ]:
            self.frontier([row])
            self.denied()
        self.frontier([{"issue_id": "other", "claim": {"scope": ["crates/engine"]}}])
        self.assertEqual(self.run_retire()["status"], "validated_dry_run")

    def test_unknown_or_hash_mismatch_prevents_entire_batch(self):
        other = "target/qa/second"
        self.create(other)
        self.write(other + "/debug/unknown", "keep")
        self.denied(bundles=[self.bundle, other])
        (self.root / other / "debug/unknown").unlink()
        (self.root / other / "debug/datum-gui.d").write_text("changed")
        self.denied(bundles=[self.bundle, other])

    def test_binary_sha_fallback_preserves_unpinned_dependency_file(self):
        p = self.root / self.bundle / "build-info.json"
        data = json.loads(p.read_text())
        data["binary_sha256"] = {"datum-gui": data.pop("files_sha256")["debug/datum-gui"]}
        p.write_text(json.dumps(data))
        result = self.run_retire(apply=True)
        self.assertEqual(result["completed"], [self.bundle + "/debug/datum-gui"])
        self.assertEqual(result["retained"], [self.bundle + "/debug/datum-gui.d"])
        self.assertEqual((self.root / self.bundle / "debug/datum-gui.d").read_text(), "dependency data")
        self.assertEqual(self.run_retire(apply=True)["completed"], [])

    def test_build_and_selected_mapping_refused(self):
        self.denied(active_check=lambda: ["cargo"])
        self.denied(users_check=lambda paths: ["selected mapping"])

    def test_symlink_leaf_parent_and_escaping_paths_refused(self):
        p = self.root / self.bundle / "debug/datum-gui"
        p.rename(p.with_name("saved"))
        p.symlink_to("saved")
        self.denied()
        p.unlink()
        p.with_name("saved").rename(p)
        debug = p.parent
        debug.rename(debug.with_name("kept"))
        debug.symlink_to("kept", target_is_directory=True)
        self.denied()
        for path in ["/target/qa/example", "target/qa/../example", "target/qa/example/extra"]:
            self.denied(bundles=[path])

    def test_tracked_bundle_refused(self):
        subprocess.run(["git", "add", "-f", self.bundle + "/debug/datum-gui"], cwd=self.root, check=True)
        self.denied()

    def test_hardlinked_selected_images_retired(self):
        other = "target/qa/second"
        self.create(other)
        a = self.root / self.bundle / "debug/datum-gui"
        b = self.root / other / "debug/datum-gui"
        b.unlink()
        os.link(a, b)
        result = self.run_retire([self.bundle, other], apply=True)
        self.assertEqual(len(result["completed"]), 4)

    def test_identity_swap_recheck_preserves_replacement(self):
        replacement = self.root / self.bundle / "debug/datum-gui"
        def swap(paths):
            replacement.rename(replacement.with_name("original"))
            replacement.write_text("foreign replacement")
            return []
        self.denied(users_check=swap)
        self.assertEqual(replacement.read_text(), "foreign replacement")

    def test_new_unknown_file_rechecked_before_mutation(self):
        def add(paths):
            self.write(self.bundle + "/debug/new-output", "keep")
            return []
        self.denied(users_check=add)

    def test_ownership_rechecked_after_process_inspection(self):
        def reopen(paths):
            self.write(".beads/issues.jsonl", '{"id":"dat-test","status":"open"}\n')
            return []
        self.denied(users_check=reopen)

    def test_malformed_shapes_refuse_cleanly(self):
        p = self.root / self.bundle / "build-info.json"
        original = p.read_text()
        for data in [[], {"build_retention": []}, {"build_retention": {
                "state": "released", "issue_id": "dat-test"}, "files_sha256": []}]:
            p.write_text(json.dumps(data))
            self.denied()
        p.write_text(original)
        for value in [[], {"frontier": [[]]}, {"frontier": [{"claim": []}]}, {"frontier": [{"claim": {}}]}]:
            self.write("specs/active_frontier.json", json.dumps(value))
            self.denied()
        self.frontier([])
        self.write(".beads/issues.jsonl", '[]\n')
        self.denied()

    def test_malformed_after_first_unlink_has_partial_receipt(self):
        unlink = os.unlink
        def corrupt(path, *args, **kwargs):
            result = unlink(path, *args, **kwargs)
            self.write("specs/active_frontier.json", "[]")
            return result
        with patch.object(r.os, "unlink", side_effect=corrupt):
            with self.assertRaises(r.Failure) as caught:
                self.run_retire(apply=True)
        result = caught.exception.receipt
        self.assertEqual(result["status"], "partial_failure")
        self.assertEqual(result["completed"], [self.bundle + "/debug/datum-gui"])
        self.assertEqual(result["observed_absent"], result["completed"])

    def test_manager_route_unknown_inspection_is_not_bypassed(self):
        def invoke(root, bundles, lock, active, users, *, apply):
            self.assertEqual(bundles, [self.bundle])
            self.assertFalse(apply)
            self.assertEqual(active(), [])
            users([self.root / self.bundle / "debug/datum-gui"])
            self.fail("unknown inspection must refuse")
        with patch("sys.argv", ["manage_cargo_cache.py", "--released-qa-build", self.bundle]), \
                patch.object(manager.guard, "load_policy", return_value=SimpleNamespace(lock_path=self.root / "lock")), \
                patch.object(manager.retention, "retire", side_effect=invoke), \
                patch.object(manager, "active_processes", return_value=[]) as active, \
                patch.object(manager, "active_cache_users", side_effect=manager.guard.ResourcePolicyError("unknown inspection")) as users, \
                redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
            self.assertEqual(manager.main(), 75)
        active.assert_called_once_with(builds_only=True)
        self.assertEqual(users.call_args.kwargs, {"include_mappings": True})

    def test_partial_failure_receipt_is_honest(self):
        unlink = os.unlink
        def fail_dependency(path, *args, **kwargs):
            if path == "datum-gui.d":
                raise OSError("injected refusal")
            return unlink(path, *args, **kwargs)
        with patch.object(r.os, "unlink", side_effect=fail_dependency):
            with self.assertRaises(r.Failure) as caught:
                self.run_retire(apply=True)
        result = caught.exception.receipt
        self.assertEqual(result["status"], "partial_failure")
        self.assertEqual(result["completed"], [self.bundle + "/debug/datum-gui"])
        self.assertEqual(result["observed_absent"], result["completed"])
        self.assertTrue((self.root / self.bundle / "debug/datum-gui.d").exists())


if __name__ == "__main__":
    unittest.main()
