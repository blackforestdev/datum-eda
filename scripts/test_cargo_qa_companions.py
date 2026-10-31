"""Kernel and refusal controls for exact, byte-preserving QA companion dedup."""
import errno
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock

import cargo_qa_companions as qa
import manage_cargo_cache as cache
import run_cargo_guarded as guard


class CompanionTests(unittest.TestCase):
    cohorts = ("initial-prefix-050cdf8c", "manifest-open-94d59290", "manifest-open-ea04ce38")

    def setUp(self):
        base = Path(os.environ.get("DATUM_QA_COMPANION_TEST_ROOT",
            Path(__file__).resolve().parents[1] / ".datum/scratch/cargo-qa-companions-tests"))
        base.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=base)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        self.lock = self.root / "resource.lock"
        self.paths = []
        for cohort in self.cohorts:
            directory = self.root / "target/qa" / cohort / "debug"
            directory.mkdir(parents=True)
            path = directory / "datum-eda"
            # The existing installed runtime is used only as a temporary ELF
            # kernel fixture executing first-party Python control snippets.
            # No dependency/source is fetched, installed or added to the repo.
            shutil.copy2(sys.executable, path)
            self.paths.append(path)
            (directory / "datum-gui").write_bytes(b"retained GUI evidence")
            (directory.parent / "source.json").write_text('{"retained":true}')
            (directory.parent / "result.log").write_text("retained proof")
        self.manifest = self.root / "manifest.json"
        self.write_manifest()

    def entry(self, path):
        descriptor = os.open(path, qa.READ)
        try:
            return dict(path=str(path.relative_to(self.root)), **qa.snapshot(descriptor))
        finally:
            os.close(descriptor)

    def write_manifest(self, change=None):
        entries = [self.entry(path) for path in self.paths]
        value = {"schema_version": 1, "operation": "deduplicate_immutable_qa_binaries",
                 "groups": [{"anchor": entries[0], "duplicates": entries[1:]}]}
        if change:
            change(value)
        self.manifest.write_text(json.dumps(value))
        self.sha = hashlib.sha256(self.manifest.read_bytes()).hexdigest()

    def run_dedup(self, *, apply=False, active=None):
        return qa.deduplicate(self.root, self.manifest, self.sha, self.lock,
                              active or (lambda: []), apply=apply)

    def assert_intact(self, expected=None):
        for path in self.paths:
            self.assertTrue(path.is_file())
            self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(), self.entry(self.paths[0])["sha256"])
            if expected:
                self.assertEqual(qa.info(path.stat()), expected[path])
        self.assertFalse(list(self.root.rglob(".datum-qa-dedup-*")))
        for path in self.root.rglob("result.log"):
            self.assertEqual(path.read_text(), "retained proof")
        for path in self.root.rglob("datum-gui"):
            self.assertEqual(path.read_bytes(), b"retained GUI evidence")

    def test_dry_run_and_positive_dedup_preserve_all_paths_bytes_security_and_mtime(self):
        expected = {path: qa.info(path.stat()) for path in self.paths}
        dry = self.run_dedup()
        self.assertEqual(dry["status"], "dry_run_validated_no_mutation")
        self.assert_intact(expected)
        applied = self.run_dedup(apply=True)
        self.assertEqual(applied["status"], "complete_all_paths_and_bytes_retained")
        self.assertEqual(len(applied["completed"]), 2)
        self.assertEqual(applied["nonselected_before_sha256"], applied["nonselected_after_sha256"])
        self.assertEqual(len({path.stat().st_ino for path in self.paths}), 1)
        for path in self.paths:
            self.assertEqual(path.stat().st_nlink, 3)
            after = applied["after"][str(path.relative_to(self.root))]
            before = applied["before"][str(path.relative_to(self.root))]
            for key in ["sha256", "size", "mode", "uid", "gid", "mtime_ns", "xattrs_sha256"]:
                self.assertEqual(before[key], after[key])
        self.assert_intact()

    def test_all_held_write_descriptors_block_exec_until_return(self):
        original = qa.nonselected_inventory
        def inspect(*args):
            for path in self.paths:
                with self.assertRaises(OSError) as caught:
                    subprocess.run([str(path), "-c", "pass"], check=True)
                self.assertEqual(caught.exception.errno, errno.ETXTBSY)
            return original(*args)
        with mock.patch.object(qa, "nonselected_inventory", side_effect=inspect):
            self.run_dedup()
        subprocess.run([str(self.paths[1]), "-c", "pass"], check=True)

    def test_running_renamed_unknown_elf_refuses_even_if_process_exe_is_uninspectable(self):
        ready = self.root / "ready"
        code = ("from pathlib import Path; import time; "
                "Path('/proc/self/comm').write_text('qa-unknown'); "
                f"Path({str(ready)!r}).write_text('ready'); time.sleep(30)")
        process = subprocess.Popen([str(self.paths[1]), "-c", code])
        try:
            until = time.monotonic() + 5
            while not ready.exists() and time.monotonic() < until:
                self.assertIsNone(process.poll())
                time.sleep(.01)
            self.assertTrue(ready.exists())
            renamed = self.paths[1].with_name("unknown-executable")
            self.paths[1].rename(renamed)
            os.link(renamed, self.paths[1])
            self.write_manifest()
            expected = {path: qa.info(path.stat()) for path in self.paths}
            with mock.patch.object(cache, "active_cache_users", side_effect=PermissionError("unreadable /proc executable")) as inaccessible:
                with self.assertRaises(qa.Failure) as caught:
                    self.run_dedup(apply=True)
                self.assertEqual(caught.exception.__cause__.errno, errno.ETXTBSY)
                inaccessible.assert_not_called()
            self.assert_intact(expected)
        finally:
            process.terminate()
            process.wait(timeout=5)

    def test_active_build_guard_and_lock_refuse_without_mutation(self):
        expected = {path: qa.info(path.stat()) for path in self.paths}
        with self.assertRaises(qa.Failure):
            self.run_dedup(apply=True, active=lambda: ["owned Cargo process"])
        with guard.acquire_lock(self.lock, 0):
            with self.assertRaises(guard.ResourcePolicyError):
                self.run_dedup(apply=True)
        self.assert_intact(expected)

    def test_manifest_sha_and_unapproved_mixed_or_escaped_paths_refuse(self):
        original = self.manifest.read_bytes()
        self.manifest.write_bytes(original + b" ")
        with self.assertRaises(guard.ResourcePolicyError):
            self.run_dedup(apply=True)
        changes = ["target/debug/datum-eda", "target/qa/foreign/debug/datum-eda",
                   "target/qa/initial-prefix-050cdf8c/debug/datum-gui",
                   "target/qa/initial-prefix-050cdf8c/debug/../debug/datum-eda",
                   "/target/qa/initial-prefix-050cdf8c/debug/datum-eda"]
        for path in changes:
            with self.subTest(path=path):
                self.write_manifest(lambda value: value["groups"][0]["duplicates"][0].update(path=path))
                with self.assertRaises(guard.ResourcePolicyError):
                    self.run_dedup(apply=True)
        self.assert_intact()

    def test_tracked_path_is_never_replaced(self):
        subprocess.run(["git", "add", "--", str(self.paths[1].relative_to(self.root))], cwd=self.root, check=True)
        with self.assertRaises(qa.Failure):
            self.run_dedup(apply=True)
        self.assert_intact()

    def test_hash_security_permission_and_timestamp_mismatch_refuse_batch(self):
        self.paths[-1].write_bytes(b"\x7fELFchanged after manifest")
        anchor = self.paths[0].stat().st_ino
        with self.assertRaises(qa.Failure):
            self.run_dedup(apply=True)
        self.assertEqual(self.paths[0].stat().st_ino, anchor)
        self.assertFalse(list(self.root.rglob(".datum-qa-dedup-*")))
        shutil.copy2(sys.executable, self.paths[-1])
        self.write_manifest()
        self.paths[-1].chmod(0o700)
        with self.assertRaises(qa.Failure):
            self.run_dedup(apply=True)
        shutil.copystat(sys.executable, self.paths[-1])
        self.write_manifest()
        os.utime(self.paths[-1], ns=(self.paths[-1].stat().st_atime_ns, 1))
        self.write_manifest()
        with self.assertRaises(guard.ResourcePolicyError):
            self.run_dedup(apply=True)

    def test_security_xattr_mismatch_is_not_normalized(self):
        os.setxattr(self.paths[-1], "user.qa-control", b"explicit mismatch")
        # Preserve the actual current inode/ctime but keep the reviewed absent
        # xattr expectation, so the descriptor validation must detect the change.
        self.write_manifest(lambda value: value["groups"][0]["duplicates"][-1].update(xattrs_sha256={}))
        with self.assertRaises(qa.Failure):
            self.run_dedup(apply=True)
        self.assertEqual(os.getxattr(self.paths[-1], "user.qa-control"), b"explicit mismatch")
        self.assertFalse(list(self.root.rglob(".datum-qa-dedup-*")))

    def test_symlink_file_and_parent_are_never_followed(self):
        target = self.paths[-1]
        target.unlink(); target.symlink_to(self.paths[0])
        with self.assertRaises(qa.Failure):
            self.run_dedup(apply=True)
        self.assertTrue(target.is_symlink())
        target.unlink(); shutil.copy2(sys.executable, target)
        self.write_manifest()
        directory = target.parent
        moved = directory.with_name("moved-debug")
        directory.rename(moved); directory.symlink_to(moved)
        with self.assertRaises(qa.Failure):
            self.run_dedup(apply=True)
        self.assertTrue(directory.is_symlink())

    def test_permission_error_in_positive_descriptor_admission_refuses(self):
        original = os.open
        def denied(path, flags, *args, **kwargs):
            if flags & os.O_RDWR:
                raise PermissionError("selected write descriptor denied")
            return original(path, flags, *args, **kwargs)
        with mock.patch.object(qa.os, "open", side_effect=denied):
            with self.assertRaises(qa.Failure):
                self.run_dedup(apply=True)
        self.assert_intact()

    def test_replaced_named_inode_during_staging_refuses_and_cleans_only_own_links(self):
        original = os.link
        replaced = []
        def substitute(*args, **kwargs):
            if not replaced:
                changed = self.paths[1].with_name("replacement-control")
                shutil.copy2(sys.executable, changed)
                os.replace(changed, self.paths[1])
                replaced.append(self.paths[1].stat().st_ino)
            return original(*args, **kwargs)
        with mock.patch.object(qa.os, "link", side_effect=substitute):
            with self.assertRaises(qa.Failure):
                self.run_dedup(apply=True)
        self.assertEqual(self.paths[1].stat().st_ino, replaced[0])
        self.assert_intact()

    def test_partial_atomic_failure_retains_all_bytes_and_reports_observed_state(self):
        original = os.replace
        calls = []
        def partial(*args, **kwargs):
            calls.append(1)
            if len(calls) == 2:
                raise OSError("injected second atomic replacement refusal")
            return original(*args, **kwargs)
        with mock.patch.object(qa.os, "replace", side_effect=partial):
            with self.assertRaises(qa.Failure) as caught:
                self.run_dedup(apply=True)
        receipt = caught.exception.receipt
        self.assertEqual(len(receipt["completed"]), 1)
        self.assertEqual(receipt["status"], "refused_or_partial")
        self.assertIn("injected second atomic replacement refusal", receipt["error"])
        self.assertEqual(self.paths[0].stat().st_ino, self.paths[1].stat().st_ino)
        self.assertNotEqual(self.paths[0].stat().st_ino, self.paths[2].stat().st_ino)
        self.assertEqual(len(receipt["observed_after_error"]), 3)
        self.assert_intact()

    def test_completed_replace_followed_by_error_is_not_reported_as_no_change(self):
        original = os.replace
        def acknowledged_late(*args, **kwargs):
            original(*args, **kwargs)
            raise OSError("injected error after atomic replacement")
        with mock.patch.object(qa.os, "replace", side_effect=acknowledged_late):
            with self.assertRaises(qa.Failure) as caught:
                self.run_dedup(apply=True)
        receipt = caught.exception.receipt
        self.assertEqual(receipt["completed"], [])
        observed = receipt["observed_after_error"]
        anchor, first = [str(path.relative_to(self.root)) for path in self.paths[:2]]
        self.assertEqual(observed[anchor]["inode"], observed[first]["inode"])
        self.assert_intact()


class ReleasedCompanionTests(CompanionTests):
    """Apply the same real kernel/refusal controls to the newly admitted cohort family."""
    cohorts = ('auto-global-6183c81e', 'browse-affordance-547d406f', 'browse-affordance-7b3455dd')


class GuiCompanionTests(unittest.TestCase):
    def setUp(self):
        base = Path(__file__).resolve().parents[1] / '.datum/scratch/cargo-qa-companions-tests'
        base.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=base)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        subprocess.run(['git', 'init', '-q', str(self.root)], check=True)
        self.lock = self.root / 'resource.lock'
        self.paths = []
        for i, cohort in enumerate(('manifest-open-94d59290', 'manifest-open-ea04ce38',
                                   'browse-affordance-b0158987', 'browse-affordance-7b3455dd')):
            path = self.root / 'target/qa' / cohort / 'debug/datum-gui'
            path.parent.mkdir(parents=True)
            shutil.copy2(sys.executable, path)
            if i >= 2:
                with path.open('ab') as stream:
                    stream.write(b'qa-second-image')
                os.utime(path, ns=(1700000000000000000, 1700000000000000000))
            self.paths.append(path)
            (path.parent / 'datum-eda').write_bytes(b'retained companion')
            (path.parent.parent / 'result.log').write_text('retained evidence')
        self.manifest = self.root / 'manifest.json'
        self.write_manifest()

    def entry(self, path):
        descriptor = os.open(path, qa.READ)
        try:
            return {'path': str(path.relative_to(self.root)), **qa.snapshot(descriptor)}
        finally:
            os.close(descriptor)

    def write_manifest(self, change=None):
        entries = [self.entry(path) for path in self.paths]
        data = {'schema_version': 1, 'operation': 'deduplicate_immutable_qa_binaries',
                'runtime_safety': qa.GUI_RUNTIME_PROOF,
                'groups': [{'anchor': entries[i], 'duplicates': [entries[i + 1]]} for i in (0, 2)]}
        if change:
            change(data)
        self.manifest.write_text(json.dumps(data))
        self.sha = hashlib.sha256(self.manifest.read_bytes()).hexdigest()

    def unrelated_apps(self, *, builds_only=False):
        self.assertTrue(builds_only, 'new mode must not claim unrelated apps are idle')
        return [] if builds_only else ['unrelated Datum runtime']

    def run_dedup(self, *, apply=False, active=None):
        return qa.deduplicate(self.root, self.manifest, self.sha, self.lock,
                              active or self.unrelated_apps, apply=apply)

    def test_two_gui_pairs_preserve_bytes_paths_security_mtime_and_other_evidence(self):
        before = {str(path.relative_to(self.root)): self.entry(path) for path in self.paths}
        dry = self.run_dedup()
        self.assertEqual(dry['status'], 'dry_run_validated_no_mutation')
        result = self.run_dedup(apply=True)
        self.assertEqual(result['status'], 'complete_all_paths_and_bytes_retained')
        self.assertEqual(result['runtime_guard'], qa.GUI_RUNTIME_PROOF)
        self.assertEqual(len(result['completed']), 2)
        self.assertEqual(result['nonselected_before_sha256'], result['nonselected_after_sha256'])
        for i in (0, 2):
            self.assertEqual(self.paths[i].stat().st_ino, self.paths[i + 1].stat().st_ino)
        self.assertNotEqual(self.paths[0].stat().st_ino, self.paths[2].stat().st_ino)
        for path in self.paths:
            key = str(path.relative_to(self.root))
            for field in ['sha256', 'size', 'mode', 'uid', 'gid', 'mtime_ns', 'xattrs_sha256']:
                self.assertEqual(before[key][field], result['after'][key][field])
            self.assertEqual((path.parent / 'datum-eda').read_bytes(), b'retained companion')
            self.assertEqual((path.parent.parent / 'result.log').read_text(), 'retained evidence')

    def test_gui_mode_is_explicit_and_cannot_select_legacy_roles_or_other_cohorts(self):
        for change in [lambda data: data.pop('runtime_safety'),
                       lambda data: data.update(runtime_safety='skip_checks'),
                       lambda data: data['groups'][0]['anchor'].update(
                           path='target/qa/startup-0a8e8b5d/debug/datum-gui'),
                       lambda data: data['groups'][0]['anchor'].update(
                           path='target/qa/manifest-open-94d59290/debug/datum-eda')]:
            with self.subTest(change=change):
                self.write_manifest(change)
                with self.assertRaises(guard.ResourcePolicyError):
                    self.run_dedup(apply=True)
        self.assertEqual(len({path.stat().st_ino for path in self.paths}), 4)

    def test_active_compiler_and_shared_lock_still_refuse(self):
        with self.assertRaises(qa.Failure):
            self.run_dedup(apply=True, active=lambda **kwargs: ['cargo'])
        with guard.acquire_lock(self.lock, 0):
            with self.assertRaises(guard.ResourcePolicyError):
                self.run_dedup(apply=True)
        self.assertEqual(len({path.stat().st_ino for path in self.paths}), 4)

    def test_selected_running_gui_is_kernel_refused_despite_ignoring_unrelated_apps(self):
        ready = self.root / 'ready'
        code = f"from pathlib import Path; import time; Path({str(ready)!r}).touch(); time.sleep(30)"
        process = subprocess.Popen([str(self.paths[0]), '-c', code])
        try:
            until = time.monotonic() + 5
            while not ready.exists() and time.monotonic() < until:
                self.assertIsNone(process.poll())
                time.sleep(.01)
            self.assertTrue(ready.exists())
            with self.assertRaises(qa.Failure) as caught:
                self.run_dedup(apply=True)
            self.assertEqual(caught.exception.__cause__.errno, errno.ETXTBSY)
            self.assertEqual(len({path.stat().st_ino for path in self.paths}), 4)
        finally:
            process.terminate()
            process.wait(timeout=5)

    def test_all_gui_descriptors_prevent_new_execution_through_postverification(self):
        original = qa.nonselected_inventory
        def inspect(*args):
            for path in self.paths:
                with self.assertRaises(OSError) as caught:
                    subprocess.run([str(path), '-c', 'pass'], check=True)
                self.assertEqual(caught.exception.errno, errno.ETXTBSY)
            return original(*args)
        with mock.patch.object(qa, 'nonselected_inventory', side_effect=inspect):
            self.run_dedup(apply=True)
        subprocess.run([str(self.paths[1]), '-c', 'pass'], check=True)


if __name__ == "__main__":
    unittest.main()
