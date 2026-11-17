"""Safety controls for exact abandoned compiler objects, never native QA proof."""
from pathlib import Path
import hashlib
import json
import os
import errno
import shutil
import subprocess
import sys
import tempfile
import unittest
from contextlib import ExitStack
import fcntl
from unittest import mock

import cargo_codegen_retention as codegen
import cargo_qa_retention as retention
import run_cargo_guarded as guard


class CodegenRetirementTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        subprocess.run(['git', 'init', '-q', str(self.root)], check=True)
        self.relative = 'target/debug/deps/example-0123456789abcdef.example.0123456789abcdef-cgu.00.rcgu.o'
        self.object = self.root / self.relative
        self.object.parent.mkdir(parents=True)
        header = bytearray(64)
        header[:7] = b'\x7fELF\x02\x01\x01'
        header[16:18] = (1).to_bytes(2, 'little')
        header[20:24] = (1).to_bytes(4, 'little')
        header[52:54] = (64).to_bytes(2, 'little')
        self.object.write_bytes(header)
        self.sentinel = self.object.parent / 'keep-qa-binary'
        self.sentinel.write_bytes(b'current QA remains')
        self.manifest = self.root / 'selection.json'
        self.capture()

    def capture(self, **overrides):
        with ExitStack() as stack:
            held = retention.Bound(self.root, self.relative, stack)
            entry = dict(held.expected, path=self.relative)
        self.value = dict(operation='retire_interrupted_codegen_v1', released=True,
            owner='test', session='test-session', issue='test-issue', reason='abandoned owned test build', files=[entry])
        self.value.update(overrides)
        raw = (json.dumps(self.value)+'\n').encode()
        self.manifest.write_bytes(raw)
        self.sha = hashlib.sha256(raw).hexdigest()

    def retire(self, apply=False, active=False, sha=None, executable=False):
        return codegen.retire(self.root, self.manifest, sha or self.sha,
            self.root/'lock', lambda: ['rustc'] if active else [], apply=apply, executable=executable)

    def test_preview_then_exact_apply_preserves_adjacent_qa(self):
        self.assertEqual(self.retire()['completed'], [])
        self.assertTrue(self.object.exists())
        result = self.retire(apply=True)
        self.assertEqual(result['completed'], [self.relative])
        self.assertFalse(self.object.exists())
        self.assertEqual(self.sentinel.read_bytes(), b'current QA remains')

    def test_empty_interrupted_object_is_admitted(self):
        self.object.write_bytes(b'')
        self.capture()
        self.retire(apply=True)
        self.assertFalse(self.object.exists())

    def test_active_build_wrong_digest_or_missing_release_refuses(self):
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True, active=True)
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True, sha='0'*64)
        self.capture(released=False)
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)
        self.assertTrue(self.object.exists())

    def test_changed_bytes_or_identity_refuses(self):
        self.object.write_bytes(self.object.read_bytes()+b'changed')
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)
        self.capture()
        data = self.object.read_bytes()
        self.object.unlink()
        self.object.write_bytes(data)
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)

    def test_runnable_or_unknown_content_refuses_even_with_fresh_manifest(self):
        for header in (b'ordinary text', b'\x7fELF\x02\x01\x01'+bytes(9)+b'\x02\x00'+bytes(46)):
            self.object.write_bytes(header)
            self.capture()
            with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)
        self.assertTrue(self.object.exists())

    def test_tracked_hardlinked_executable_and_symlink_inputs_refuse(self):
        self.object.chmod(0o755)
        self.capture()
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)
        self.object.chmod(0o644)
        other = self.object.with_name('other-link')
        os.link(self.object, other)
        self.capture()
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)
        other.unlink()
        self.capture()
        subprocess.run(['git', 'add', '-f', '--', self.relative], cwd=self.root, check=True)
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)
        subprocess.run(['git', 'rm', '--cached', '--', self.relative], cwd=self.root, check=True, capture_output=True)
        self.object.unlink()
        self.object.symlink_to(self.sentinel)
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)
        self.assertTrue(self.sentinel.exists())

    def test_parent_symlink_and_contended_lock_refuse(self):
        with (self.root/'lock').open('w') as held:
            fcntl.flock(held, fcntl.LOCK_EX | fcntl.LOCK_NB)
            with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)
        self.assertTrue(self.object.exists())
        directory = self.object.parent
        moved = directory.with_name('moved')
        directory.rename(moved)
        directory.symlink_to(moved, target_is_directory=True)
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)
        self.assertTrue((moved/self.object.name).exists())

    def pair(self, valid=True):
        other = self.object.with_name(self.object.name.replace('cgu.00', 'cgu.01'))
        other.write_bytes(self.object.read_bytes() if valid else b'not compiler output')
        with ExitStack() as stack:
            item = retention.Bound(self.root, str(other.relative_to(self.root)), stack)
            self.value['files'].append(dict(item.expected, path=item.relative))
        raw = (json.dumps(self.value)+'\n').encode()
        self.manifest.write_bytes(raw)
        self.sha = hashlib.sha256(raw).hexdigest()
        return other

    def test_entire_selection_is_validated_before_mutation(self):
        other = self.pair(valid=False)
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)
        self.assertTrue(self.object.exists())
        self.assertTrue(other.exists())

    def test_partial_failure_reports_completed_and_attempted_exact_paths(self):
        other = self.pair()
        unlink = os.unlink
        calls = 0
        def fail_second(*args, **kwargs):
            nonlocal calls
            calls += 1
            if calls == 2: raise OSError('injected selected unlink failure')
            return unlink(*args, **kwargs)
        with mock.patch.object(codegen.os, 'unlink', side_effect=fail_second):
            with self.assertRaises(retention.Failure) as raised: self.retire(apply=True)
        self.assertEqual(raised.exception.receipt['completed'], [self.relative])
        self.assertEqual(len(raised.exception.receipt['attempted']), 2)
        self.assertFalse(self.object.exists())
        self.assertTrue(other.exists())
        self.assertEqual(self.sentinel.read_bytes(), b'current QA remains')

    def executable(self):
        self.relative = 'target/debug/deps/example-0123456789abcdef'
        self.object = self.root/self.relative
        shutil.copyfile('/bin/sleep', self.object)
        self.object.chmod(0o755)
        self.capture(operation='retire_released_compiler_executable_v1')

    def test_compiler_executable_requires_separate_explicit_mode(self):
        self.executable()
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)
        self.assertTrue(self.object.exists())
        self.retire(apply=True, executable=True)
        self.assertFalse(self.object.exists())
        self.assertEqual(self.sentinel.read_bytes(), b'current QA remains')

    def test_unknown_running_executable_refuses_via_kernel_without_proc_assumption(self):
        self.executable()
        child = subprocess.Popen([str(self.object), '10'])
        try:
            self.capture(operation='retire_released_compiler_executable_v1')
            with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True, executable=True)
            self.assertTrue(self.object.exists())
        finally:
            child.kill()
            child.wait()

    def test_held_writable_descriptors_prevent_new_execution_until_retirement(self):
        self.executable()
        unlink = os.unlink
        def execution_blocked(*args, **kwargs):
            with self.assertRaises(OSError) as raised:
                subprocess.Popen([str(self.object), '0'])
            self.assertEqual(raised.exception.errno, errno.ETXTBSY)
            return unlink(*args, **kwargs)
        with mock.patch.object(codegen.os, 'unlink', side_effect=execution_blocked):
            self.retire(apply=True, executable=True)
        self.assertFalse(self.object.exists())


class ArchiveRetirementTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        subprocess.run(['git', 'init', '-q', str(self.root)], check=True)
        self.relative = 'target/debug/deps/libexample-0123456789abcdef.rlib'
        self.archive = self.root / self.relative
        self.archive.parent.mkdir(parents=True)
        data = b'non-runnable fixture member'
        header = f"{'member/':<16}{0:<12}{0:<6}{0:<6}{'644':<8}{len(data):<10}`\n".encode()
        self.assertEqual(len(header), 60)
        self.archive.write_bytes(b'!<arch>\n' + header + data + (b'\n' if len(data) % 2 else b''))
        self.archive.chmod(0o644)
        self.sentinel = self.archive.parent / 'current-protocol-test'
        self.sentinel.write_bytes(b'keep current tests and QA')
        self.manifest = self.root / 'selection.json'
        self.capture()

    def entry(self, relative):
        with ExitStack() as stack:
            item = retention.Bound(self.root, relative, stack)
            return dict(item.expected, path=relative)

    def capture(self, **overrides):
        self.value = dict(operation='retire_released_compiler_archive_v1', released=True,
            owner='test', session='test-session', issue='test-issue',
            reason='completed obsolete archive', files=[self.entry(self.relative)])
        self.value.update(overrides)
        raw = (json.dumps(self.value) + '\n').encode()
        self.manifest.write_bytes(raw)
        self.sha = hashlib.sha256(raw).hexdigest()

    def retire(self, apply=False, active=False, sha=None, **options):
        return codegen.retire(self.root, self.manifest, sha or self.sha,
            self.root / 'lock', lambda: ['rustc'] if active else [],
            apply=apply, archive=True, **options)

    def test_exact_archive_preview_apply_preserves_current_test(self):
        self.assertEqual(self.retire()['completed'], [])
        self.assertTrue(self.archive.exists())
        result = self.retire(apply=True)
        self.assertEqual(result['completed'], [self.relative])
        self.assertFalse(self.archive.exists())
        self.assertEqual(self.sentinel.read_bytes(), b'keep current tests and QA')

    def test_thin_empty_runnable_and_unknown_formats_refuse(self):
        for content in [b'!<thin>\n', b'', b'\x7fELF' + bytes(64), b'#!/bin/sh\n', b'unknown']:
            self.archive.write_bytes(content)
            self.capture()
            with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)
            self.assertEqual(self.archive.read_bytes(), content)

    def test_executable_archive_and_conflicting_kind_refuse(self):
        self.archive.chmod(0o755)
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)
        self.archive.chmod(0o644)
        self.capture()
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True, executable=True)
        self.assertTrue(self.archive.exists())

    def test_active_digest_missing_release_and_wrong_operation_refuse(self):
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True, active=True)
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True, sha='0' * 64)
        for override in [dict(released=False), dict(operation='retire_released_compiler_executable_v1')]:
            self.capture(**override)
            with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)
        self.assertTrue(self.archive.exists())

    def test_changed_bytes_and_replaced_identity_refuse(self):
        original = self.archive.read_bytes()
        self.archive.write_bytes(original + b'changed')
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)
        self.capture()
        self.archive.unlink()
        self.archive.write_bytes(original + b'changed')
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)

    def test_hardlink_and_tracked_archive_refuse(self):
        alias = self.root / 'held-alias'
        os.link(self.archive, alias)
        self.capture()
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)
        alias.unlink()
        self.capture()
        subprocess.run(['git', 'add', '--', self.relative], cwd=self.root, check=True)
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)

    def test_whole_batch_rejects_invalid_second_archive_before_unlink(self):
        other = 'target/debug/deps/libother-fedcba9876543210.rlib'
        (self.root / other).write_bytes(b'!<thin>\n')
        self.capture(files=[self.entry(self.relative), self.entry(other)])
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)
        self.assertTrue(self.archive.exists())
        self.assertTrue((self.root / other).exists())

    def test_non_dependency_path_refuses(self):
        relative = 'target/qa/held/libexample-0123456789abcdef.rlib'
        path = self.root / relative
        path.parent.mkdir(parents=True)
        path.write_bytes(self.archive.read_bytes())
        self.capture(files=[self.entry(relative)])
        with self.assertRaises(guard.ResourcePolicyError): self.retire(apply=True)
        self.assertTrue(path.exists())

    def test_cli_mode_mixing_and_missing_digest_refuse_before_maintenance(self):
        command = [sys.executable, str(Path(__file__).with_name('manage_cargo_cache.py')),
            '--released-compiler-archive-manifest', str(self.manifest)]
        combinations = [[], ['--manifest-sha256', self.sha, '--remove', 'target/debug/incremental'],
            ['--manifest-sha256', self.sha, '--released-qa-build', 'target/qa/held'],
            ['--manifest-sha256', self.sha, '--released-compiler-manifest', str(self.manifest)],
            ['--manifest-sha256', self.sha, '--retain-proof-build', 'target/qa/held']]
        for options in combinations:
            result = subprocess.run(command + options, capture_output=True, text=True)
            self.assertEqual(result.returncode, 2, result.stderr)
            self.assertTrue(self.archive.exists())
        self.assertEqual(self.sentinel.read_bytes(), b'keep current tests and QA')


if __name__ == '__main__':
    unittest.main()
