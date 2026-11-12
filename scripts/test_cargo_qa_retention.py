"""Hermetic released-QA retention controls; no live process or cache actions."""
from collections import Counter
from contextlib import redirect_stdout, redirect_stderr
import io
import errno
import hashlib
import json
import os
import shutil
from pathlib import Path
import subprocess
import sys
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

    def released_cli_with_hidden_process(self, *, before_users=None, error=None):
        proc = self.root / "proc"
        hidden = proc / "123"
        hidden.mkdir(parents=True, exist_ok=True)
        (hidden / "maps").write_text("")
        readlink, read_text = Path.readlink, Path.read_text
        inspect = manager.active_cache_users
        def link(path):
            if path == hidden / "exe":
                raise error or PermissionError("unavailable unrelated process")
            return readlink(path)
        def text(path, *args, **kwargs):
            if path == hidden / "maps":
                raise PermissionError("unavailable unrelated mappings")
            return read_text(path, *args, **kwargs)
        def users(paths, ignored_proc, **kwargs):
            if before_users:
                before_users()
            return inspect(paths, proc, **kwargs)
        output, errors = io.StringIO(), io.StringIO()
        with patch.object(manager, "ROOT", self.root), \
                patch.object(manager.guard, "load_policy", return_value=SimpleNamespace(lock_path=self.root / "lock")), \
                patch.object(manager, "active_processes", return_value=[]), \
                patch.object(manager, "active_cache_users", side_effect=users), \
                patch.object(sys, "argv", ["manage_cargo_cache.py", "--released-qa-build", self.bundle, "--apply"]), \
                patch.object(Path, "readlink", link), patch.object(Path, "read_text", text), \
                redirect_stdout(output), redirect_stderr(errors):
            result = manager.main()
        return result, json.loads(output.getvalue()), errors.getvalue()

    def install_real_selected_elf(self):
        image = self.root / self.bundle / "debug/datum-gui"
        shutil.copy2(sys.executable, image)
        info = self.root / self.bundle / "build-info.json"
        data = json.loads(info.read_text())
        data["files_sha256"]["debug/datum-gui"] = hashlib.sha256(image.read_bytes()).hexdigest()
        data["files_sha256"].pop("debug/datum-gui.d")
        info.write_text(json.dumps(data))
        return image

    def test_released_cli_kernel_holds_block_new_exec_despite_hidden_process(self):
        image = self.install_real_selected_elf()
        def blocked():
            with self.assertRaises(OSError) as caught:
                subprocess.run([str(image), "-c", "pass"], check=True)
            self.assertEqual(caught.exception.errno, errno.ETXTBSY)
        result, receipt, _ = self.released_cli_with_hidden_process(before_users=blocked)
        self.assertEqual(result, 0)
        self.assertEqual(receipt["uninspectable_processes"], ["123"])
        self.assertEqual(receipt["completed"], [self.bundle + "/debug/datum-gui"])
        self.assertTrue(image.with_suffix(".d").exists())
        self.assertIn("no unrelated-app idle assertion", receipt["runtime_proof"])
        self.assertTrue((self.root / self.bundle / "source-snapshot/file.rs").exists())

    def test_running_renamed_hardlinked_elf_refuses_before_proc_visibility(self):
        image = self.install_real_selected_elf()
        process = subprocess.Popen([str(image), "-c", "import time; time.sleep(30)"])
        try:
            renamed = self.root / "renamed-running-image"
            image.rename(renamed)
            os.link(renamed, image)
            result, receipt, _ = self.released_cli_with_hidden_process()
            self.assertEqual(result, 75)
            self.assertEqual(receipt["completed"], [])
            self.assertIn("Text file busy", receipt["reason"])
            self.assertTrue(image.exists())
        finally:
            process.terminate()
            process.wait(timeout=5)

    def test_hidden_proc_views_do_not_bypass_hold_hash_or_visible_mapping(self):
        info = self.root / self.bundle / "build-info.json"
        original = info.read_text()
        for fault in ("hold", "hash"):
            data = json.loads(original)
            if fault == "hold":
                data["build_retention"]["owner_hold"] = True
            else:
                data["files_sha256"]["debug/datum-gui"] = "0" * 64
            info.write_text(json.dumps(data))
            result, receipt, _ = self.released_cli_with_hidden_process()
            self.assertEqual(result, 75)
            self.assertEqual(receipt["completed"], [])
        info.write_text(original)
        visible = self.root / "proc/124"
        visible.mkdir()
        (visible / "exe").symlink_to(sys.executable)
        image = self.root / self.bundle / "debug/datum-gui"
        (visible / "maps").write_text(f"1000-2000 r-xp 00000000 00:00 0 {image}\n")
        result, receipt, _ = self.released_cli_with_hidden_process()
        self.assertEqual(result, 75)
        self.assertIn("mapped by an active process", receipt["reason"])
        self.assertTrue(image.exists())

    def test_other_process_inspection_errors_still_refuse_released_cli(self):
        result, receipt, _ = self.released_cli_with_hidden_process(error=OSError(errno.EIO, "inspection failed"))
        self.assertEqual(result, 75)
        self.assertEqual(receipt["completed"], [])

    def test_legacy_cli_preserves_pinned_dependencies_without_accepting_proof_test_role(self):
        dependency = self.root / self.bundle / "debug/datum-gui.d"
        before = dependency.read_bytes()
        result, receipt, _ = self.released_cli_with_hidden_process()
        self.assertEqual(result, 0)
        self.assertEqual(receipt["completed"], [self.bundle + "/debug/datum-gui"])
        self.assertEqual(dependency.read_bytes(), before)
        self.create("target/qa/extra")
        self.bundle = "target/qa/extra"
        extra = self.root / self.bundle / "debug/datum-gui-test"
        extra.write_bytes(b"\x7fELF\x02\x01" + b"\0" * 10 + b"\x02\0payload")
        extra.chmod(0o755)
        info = self.root / self.bundle / "build-info.json"
        data = json.loads(info.read_text())
        data["files_sha256"]["debug/datum-gui-test"] = hashlib.sha256(extra.read_bytes()).hexdigest()
        info.write_text(json.dumps(data))
        result, receipt, _ = self.released_cli_with_hidden_process()
        self.assertEqual(result, 75)
        self.assertEqual(receipt["completed"], [])
        self.assertTrue(extra.exists())

    def prepare_proof(self):
        metadata = self.root / self.bundle / "build-info.json"
        value = json.loads(metadata.read_text())
        value["build_retention"] = {}
        metadata.write_text(json.dumps(value))
        self.write(".beads/issues.jsonl", '{"id":"dat-test","status":"closed","assignee":"codex"}\n')

    def proof(self, **kwargs):
        return r.proof_lifecycle(self.root, self.bundle, self.root / "guard.lock",
            kwargs.pop("active_check", lambda: []), kwargs.pop("users_check", lambda paths: []),
            owner=kwargs.pop("owner", "codex"), session=kwargs.pop("session", "session-a"), **kwargs)

    def retain_proof(self):
        return self.proof(issue="dat-test", reason="Independent visual review needs this executable",
                          pending="Review the presented application")

    def test_proof_resolution_retires_in_same_command_and_preserves_evidence(self):
        self.prepare_proof()
        self.assertEqual(self.retain_proof()["status"], "proof_retained")
        binary = self.root / self.bundle / "debug/datum-gui"
        dependency = binary.with_name("datum-gui.d")
        before = dependency.read_bytes()
        self.assertTrue(binary.exists())  # A closed task alone does not resolve proof.
        with self.assertRaises(r.Failure):
            self.run_retire(apply=True)
        result = self.proof(result="Review passed; source landed", provenance="commit abc; reviewer session-b")
        self.assertFalse(binary.exists())
        self.assertEqual(dependency.read_bytes(), before)
        self.assertEqual(result["cleanup_residue"], [])
        self.assertEqual(result["completed"], [self.bundle + "/debug/datum-gui"])
        persisted = json.loads((self.root / self.bundle / "proof-cleanup.json").read_text())
        self.assertEqual(persisted, result)
        self.assertEqual(self.proof(result="Review passed; source landed",
            provenance="commit abc; reviewer session-b")["completed"], [])
        attempts = [json.loads(p.read_text()) for p in
                    (self.root / self.bundle).glob("proof-cleanup-*.json")]
        self.assertEqual(len(attempts), 2)
        self.assertEqual([p for attempt in attempts for p in attempt["completed"]],
                         [self.bundle + "/debug/datum-gui"])

    def test_proof_foreign_owner_session_and_hold_cannot_resolve(self):
        self.prepare_proof()
        self.retain_proof()
        for values in ({"owner": "other"}, {"session": "other"}):
            with self.assertRaises(r.guard.ResourcePolicyError):
                self.proof(result="done", provenance="review", **values)
        metadata = self.root / self.bundle / "build-info.json"
        value = json.loads(metadata.read_text())
        value["build_retention"]["owner_hold"] = True
        metadata.write_text(json.dumps(value))
        with self.assertRaises(r.guard.ResourcePolicyError):
            self.proof(result="done", provenance="review")
        self.assertTrue((self.root / self.bundle / "debug/datum-gui").exists())

    def test_reassigned_proof_task_cannot_be_resolved_by_previous_owner(self):
        self.prepare_proof()
        self.retain_proof()
        self.write(".beads/issues.jsonl", '{"id":"dat-test","status":"closed","assignee":"other"}\n')
        with self.assertRaises(r.Failure):
            self.proof(result="accepted", provenance="review")
        info = json.loads((self.root / self.bundle / "build-info.json").read_text())
        self.assertEqual(info["build_retention"]["state"], "retained")

    def test_proof_registration_cannot_rewrite_tracked_metadata(self):
        self.prepare_proof()
        metadata = self.root / self.bundle / "build-info.json"
        before = metadata.read_bytes()
        subprocess.run(["git", "add", "-f", str(metadata)], cwd=self.root, check=True)
        with self.assertRaises(r.Failure):
            self.retain_proof()
        self.assertEqual(metadata.read_bytes(), before)

    def test_resolution_records_exact_residue_when_cleanup_is_blocked(self):
        self.prepare_proof()
        self.retain_proof()
        # Cargo starts between durable resolution and cleanup admission.
        checks = iter([[], ["cargo"]])
        with self.assertRaises(r.Failure) as caught:
            self.proof(result="accepted", provenance="reviewer session-b",
                       active_check=lambda: next(checks))
        self.assertEqual(caught.exception.receipt["cleanup_residue"], [self.bundle + "/debug/datum-gui"])
        info = json.loads((self.root / self.bundle / "build-info.json").read_text())
        self.assertEqual(info["build_retention"]["resolution"]["result"], "accepted")
        self.assertTrue((self.root / self.bundle / "proof-cleanup.json").is_file())
        self.assertEqual(self.proof(result="accepted", provenance="reviewer session-b")["cleanup_residue"], [])

    def test_registered_proof_uses_positive_elf_admission_not_unrelated_proc_access(self):
        self.prepare_proof()
        self.retain_proof()
        def unrelated_unknown(paths):
            self.fail("registered proof must not require unrelated process visibility")
        self.assertEqual(self.proof(result="accepted", provenance="review",
                                  users_check=unrelated_unknown)["cleanup_residue"], [])

    def test_kernel_busy_selected_elf_is_retained_with_resolution_and_residue(self):
        self.prepare_proof()
        self.retain_proof()
        original = r.os.open
        def busy(path, flags, *args, **kwargs):
            if str(path) == "datum-gui" and flags & os.O_RDWR:
                raise OSError(26, "Text file busy")
            return original(path, flags, *args, **kwargs)
        with patch.object(r.os, "open", side_effect=busy):
            with self.assertRaises(r.Failure) as caught:
                self.proof(result="accepted", provenance="review")
        self.assertEqual(caught.exception.receipt["cleanup_residue"], [self.bundle + "/debug/datum-gui"])
        self.assertTrue((self.root / self.bundle / "debug/datum-gui").exists())

    def test_legacy_retirement_cannot_bypass_explicit_owner_hold(self):
        metadata = self.root / self.bundle / "build-info.json"
        value = json.loads(metadata.read_text())
        value["build_retention"]["owner_hold"] = True
        metadata.write_text(json.dumps(value))
        self.denied()

    def test_proof_registration_authenticates_owner_hash_and_requires_a_reason(self):
        self.prepare_proof()
        for values in ({"issue": "dat-test", "reason": "", "pending": "review"},
                       {"issue": "foreign", "reason": "needed", "pending": "review"}):
            with self.assertRaises(r.guard.ResourcePolicyError):
                self.proof(**values)
        self.write(self.bundle + "/debug/datum-gui", "changed")
        with self.assertRaises(r.guard.ResourcePolicyError):
            self.retain_proof()

    def test_active_cargo_prevents_resolution_without_falsely_recording_it(self):
        self.prepare_proof()
        self.retain_proof()
        with self.assertRaises(r.Failure) as caught:
            self.proof(result="accepted", provenance="review", active_check=lambda: ["cargo"])
        self.assertFalse(caught.exception.receipt["resolution_recorded"])
        info = json.loads((self.root / self.bundle / "build-info.json").read_text())
        self.assertEqual(info["build_retention"]["state"], "retained")

    def test_open_task_resolution_records_residue_until_released_then_retry_cleans(self):
        self.prepare_proof()
        self.retain_proof()
        self.write(".beads/issues.jsonl", '{"id":"dat-test","status":"in_progress","assignee":"codex"}\n')
        with self.assertRaises(r.Failure) as caught:
            self.proof(result="accepted", provenance="review")
        self.assertEqual(caught.exception.receipt["cleanup_residue"], [self.bundle + "/debug/datum-gui"])
        self.write(".beads/issues.jsonl", '{"id":"dat-test","status":"closed","assignee":"codex"}\n')
        self.assertEqual(self.proof(result="accepted", provenance="review")["cleanup_residue"], [])

    def test_unknown_output_is_reported_and_never_silently_removed(self):
        self.prepare_proof()
        self.retain_proof()
        self.write(self.bundle + "/debug/unclassified", "keep")
        with self.assertRaises(r.Failure) as caught:
            self.proof(result="accepted", provenance="review")
        self.assertEqual(set(caught.exception.receipt["cleanup_residue"]), {
            self.bundle + "/debug/datum-gui", self.bundle + "/debug/unclassified"})

    def test_foreign_wildcard_test_binary_claim_blocks_proof_resolution(self):
        self.prepare_proof()
        self.retain_proof()
        self.frontier([{"issue_id": "other", "claim": {
            "scope": ["target/qa/*/debug/datum-gui-test"]}}])
        with self.assertRaises(r.Failure):
            self.proof(result="accepted", provenance="review")
        self.assertTrue((self.root / self.bundle / "debug/datum-gui").exists())

    def test_proof_cli_retains_and_resolves_without_a_separate_delete_command(self):
        self.prepare_proof()
        common = ["manage_cargo_cache.py", "--owner", "codex", "--session", "session-a"]
        for flags in (["--retain-proof-build", self.bundle, "--issue", "dat-test", "--reason", "review", "--pending-proof", "visual"],
                      ["--resolve-proof-build", self.bundle, "--result", "accepted", "--provenance", "reviewer session-b"]):
            with patch("sys.argv", common + flags), patch.object(manager, "ROOT", self.root), \
                    patch.object(manager.guard, "load_policy", return_value=SimpleNamespace(lock_path=self.root / "guard.lock")), \
                    patch.object(manager, "active_processes", return_value=[]), \
                    patch.object(manager, "active_cache_users", return_value=[]), redirect_stdout(io.StringIO()):
                self.assertEqual(manager.main(), 0)
        self.assertFalse((self.root / self.bundle / "debug/datum-gui").exists())

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
        def invoke(root, bundles, lock, active, users, *, apply, retain_metadata):
            self.assertEqual(bundles, [self.bundle])
            self.assertFalse(apply)
            self.assertTrue(retain_metadata)
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
        self.assertEqual(users.call_args.kwargs, {"uninspectable": [], "include_mappings": True})

    def test_selected_elf_digest_count_is_linear(self):
        bundles = [self.bundle, "target/qa/second", "target/qa/third"]
        images = set()
        for bundle in bundles:
            if bundle != self.bundle:
                self.create(bundle)
            base = self.root / bundle
            blob = (base / "debug/datum-gui").read_bytes()
            for name in ("datum-eda", "eda-engine-daemon"):
                p = base / "debug" / name
                p.write_bytes(blob)
                p.chmod(0o755)
            data = json.loads((base / "build-info.json").read_text())
            data["files_sha256"] = {"debug/" + p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                                   for p in (base / "debug").iterdir()}
            (base / "build-info.json").write_text(json.dumps(data))
            for name in r.BINARIES:
                st = (base / "debug" / name).stat()
                images.add((st.st_dev, st.st_ino))
        counts, digest = Counter(), r.q.digest
        def counted(fd):
            st = os.fstat(fd)
            key = (st.st_dev, st.st_ino)
            if key in images:
                counts[key] += 1
            return digest(fd)
        with patch.object(r.q, "digest", side_effect=counted):
            result = self.run_retire(bundles, apply=True)
        self.assertEqual(result["status"], "complete_source_and_evidence_retained")
        self.assertEqual(set(counts), images)
        self.assertEqual(set(counts.values()), {3})

    def test_tamper_immediately_before_selected_hash_refuses_unlink(self):
        verify = r.Bound.verify
        def tamper(value, *, named=True, content=True):
            if content and value.name == "datum-gui":
                os.pwrite(value.fd, b"X", value.expected["size"] - 1)
            return verify(value, named=named, content=content)
        with patch.object(r.Bound, "verify", new=tamper):
            result = self.denied()
        self.assertEqual(result["completed"], [])
        self.assertEqual(result["reason"], "released QA build refused: held input changed")

    def test_post_unlink_content_change_cannot_hide_in_new_ctime(self):
        p = self.root / self.bundle / "debug/datum-gui"
        original = p.stat()
        fd = os.open(p, os.O_RDWR)
        unlink = os.unlink
        def tamper(path, *args, **kwargs):
            result = unlink(path, *args, **kwargs)
            if path == "datum-gui":
                os.pwrite(fd, b"X", original.st_size - 1)
                # Keep all expected metadata except unlink's allowed ctime/nlink.
                os.utime(fd, ns=(original.st_atime_ns, original.st_mtime_ns))
            return result
        try:
            with patch.object(r.os, "unlink", side_effect=tamper):
                with self.assertRaises(r.Failure) as caught:
                    self.run_retire(apply=True)
        finally:
            os.close(fd)
        result = caught.exception.receipt
        self.assertEqual(result["status"], "partial_failure")
        self.assertEqual(result["completed"], [self.bundle + "/debug/datum-gui"])
        self.assertIn("unexpected bytes or metadata after unlink", result["reason"])
        self.assertTrue((p.parent / "datum-gui.d").exists())

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
