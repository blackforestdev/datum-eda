"""Hermetic owned-process teardown controls; no real processes are signalled."""
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch

import native_qa_owned_processes as owned


class OwnedTeardownTests(unittest.TestCase):
    def qa(self):
        app = Mock(pid=123)
        app.poll.return_value = 0
        return SimpleNamespace(process=app, app_log=Mock(), actions=[], observations={}, env={},
                               binary_dir=Path('/owned/bin'), phase='main', compositor=Mock(),
                               compositor_log=Mock(), clipboard_writer=Mock(),
                               runtime=SimpleNamespace(name='/private/runtime', cleanup=Mock(),
                                                       _finalizer=Mock()))

    def proc(self, pid=101, *, uid=1000, binary='/owned/bin/eda-engine-daemon',
             socket='/private/runtime/datum-native-edit-unit/engine.sock'):
        proc = Mock()
        proc.name = str(pid)
        proc.stat.return_value = SimpleNamespace(st_uid=uid)
        exe = Mock()
        exe.resolve.return_value = Path(binary)
        cmdline = Mock()
        cmdline.read_bytes.return_value = f'{binary}\0--socket\0{socket}\0'.encode()
        proc.__truediv__ = Mock(side_effect=lambda name: {'exe': exe, 'cmdline': cmdline}[name])
        proc.exists.return_value = False
        return proc, exe, cmdline

    def processes(self, entries):
        directory = Mock()
        directory.iterdir.return_value = entries
        real_path = Path
        self.enterContext(patch.object(owned, 'Path', side_effect=lambda name:
                                       directory if name == '/proc' else real_path(name)))
        self.enterContext(patch.object(owned.os, 'getuid', return_value=1000))
        opened = self.enterContext(patch.object(owned.os, 'pidfd_open', return_value=42))
        closed = self.enterContext(patch.object(owned.os, 'close'))
        sent = self.enterContext(patch.object(owned.signal, 'pidfd_send_signal'))
        self.enterContext(patch.object(owned.os, 'killpg'))
        self.enterContext(patch.object(owned.time, 'sleep'))
        return directory, opened, closed, sent

    def test_only_exact_owned_companion_is_signalled(self):
        qa = self.qa()
        valid, _, _ = self.proc()
        other_uid, _, _ = self.proc(102, uid=2000)
        other_exe, _, _ = self.proc(103, binary='/foreign/eda-engine-daemon')
        other_runtime, _, _ = self.proc(104, socket='/foreign/datum-native-edit-x/engine.sock')
        near_prefix, _, _ = self.proc(105, socket='/private/runtime-extra/datum-native-edit-x/engine.sock')
        directory, opened, closed, sent = self.processes(
            [valid, other_uid, other_exe, other_runtime, near_prefix])
        result = {}
        owned.close_owned_companions(qa, result)
        opened.assert_called_once_with(101)
        sent.assert_called_once_with(42, signal.SIGTERM)
        closed.assert_called_once_with(42)
        self.assertEqual([row['pid'] for row in result['owned_companion_housekeeping']], [101])
        self.assertIn('not ordinary Close', result['owned_companion_housekeeping'][0]['limit'])

    def test_uid_executable_and_full_argv_are_rechecked_after_pidfd_open(self):
        for changed in ('uid', 'exe', 'argv'):
            with self.subTest(changed=changed):
                qa = self.qa()
                proc, exe, argv = self.proc()
                if changed == 'uid':
                    proc.stat.side_effect = [SimpleNamespace(st_uid=1000), SimpleNamespace(st_uid=2000)]
                elif changed == 'exe':
                    exe.resolve.side_effect = [Path('/owned/bin/eda-engine-daemon'), Path('/foreign/daemon')]
                else:
                    argv.read_bytes.side_effect = [argv.read_bytes.return_value, b'changed\0']
                with self.subTest():
                    _, opened, closed, sent = self.processes([proc])
                    with self.assertRaisesRegex(RuntimeError, 'identity changed'):
                        owned.close_owned_companions(qa, {})
                    opened.assert_called_once_with(101)
                    sent.assert_not_called()
                    closed.assert_called_once_with(42)
                    qa.runtime._finalizer.detach.assert_called_once()
                    qa.runtime.cleanup.assert_not_called()
                    self.assertIn('identity changed',
                                  qa.observations['retained_owned_runtime']['reason'])

    def test_direct_reaper_failure_retains_runtime_and_original_reason(self):
        qa = self.qa()
        proc, _, _ = self.proc()
        proc.exists.return_value = True
        self.processes([proc])
        with patch.object(owned.time, 'monotonic', side_effect=[0, 4]):
            with self.assertRaisesRegex(RuntimeError, 'still present'):
                owned.close_owned_companions(qa, {})
        with self.assertRaisesRegex(RuntimeError, 'Prior owned shutdown failed'):
            owned.close_owned_envelope(qa)
        qa.runtime._finalizer.detach.assert_called_once()
        qa.runtime.cleanup.assert_not_called()
        self.assertIn('still present', qa.observations['retained_owned_runtime']['reason'])

    def test_live_gui_prevents_any_companion_signal_and_runtime_removal(self):
        qa = self.qa()
        qa.process.poll.return_value = None
        directory, opened, _, sent = self.processes([])
        with self.assertRaisesRegex(RuntimeError, 'GUI exit is unconfirmed'):
            owned.close_owned_envelope(qa)
        directory.iterdir.assert_not_called()
        opened.assert_not_called()
        sent.assert_not_called()
        qa.runtime.cleanup.assert_not_called()
        qa.runtime._finalizer.detach.assert_called_once()
        qa.compositor.terminate.assert_not_called()

    def test_repeated_cleanup_preserves_positive_receipts(self):
        qa = self.qa()
        proc, _, _ = self.proc()
        directory, _, _, sent = self.processes([proc])
        first, second = {}, {}
        owned.close_owned_companions(qa, first)
        directory.iterdir.return_value = []
        owned.close_owned_companions(qa, second)
        self.assertEqual(first, second)
        self.assertEqual(len(second['owned_companion_housekeeping']), 1)
        self.assertIs(second['owned_companion_housekeeping'], qa.observations['owned_companion_housekeeping'])
        sent.assert_called_once()

    def test_stop_confirms_gui_exit_reaps_and_preserves_reusable_envelope(self):
        qa = self.qa()
        app = qa.process
        app.poll.return_value = None
        app.wait.side_effect = lambda timeout: setattr(app.poll, 'return_value', -15)
        _, _, _, sent = self.processes([self.proc()[0]])
        owned.stop_owned_gui(qa)
        app.terminate.assert_called_once()
        app.wait.assert_called_once_with(timeout=6)
        sent.assert_called_once()
        self.assertIsNone(qa.process)
        qa.app_log.close.assert_called_once()
        qa.compositor.terminate.assert_not_called()
        qa.clipboard_writer.terminate.assert_not_called()
        qa.runtime.cleanup.assert_not_called()
        self.assertIn('not ordinary window Close', qa.actions[0]['limit'])

    def test_failed_gui_shutdown_retains_runtime_and_blocks_later_cleanup(self):
        qa = self.qa()
        qa.process.poll.return_value = None
        qa.process.wait.side_effect = subprocess.TimeoutExpired('owned-gui', 6)
        _, _, _, sent = self.processes([])
        with self.assertRaises(subprocess.TimeoutExpired):
            owned.stop_owned_gui(qa)
        with self.assertRaisesRegex(RuntimeError, 'Prior owned shutdown failed'):
            owned.close_owned_envelope(qa)
        sent.assert_not_called()
        qa.runtime.cleanup.assert_not_called()
        self.assertTrue(qa.runtime._finalizer.detach.called)
        self.assertIn('retained_owned_runtime', qa.observations)
        self.assertTrue(qa.actions)

    def test_failed_companion_shutdown_retains_runtime(self):
        qa = self.qa()
        proc, _, _ = self.proc()
        proc.exists.return_value = True
        self.processes([proc])
        with patch.object(owned.time, 'monotonic', side_effect=[0, 4]):
            with self.assertRaisesRegex(RuntimeError, 'still present'):
                owned.stop_owned_gui(qa)
        qa.runtime.cleanup.assert_not_called()
        qa.runtime._finalizer.detach.assert_called_once()
        self.assertIsNotNone(qa.process)

    def test_envelope_failure_preserves_runtime_and_success_removes_it(self):
        for failed in (False, True):
            with self.subTest(failed=failed):
                qa = self.qa()
                qa.clipboard_writer.poll.return_value = None
                qa.compositor.poll.return_value = None
                self.processes([])
                if failed:
                    qa.compositor.wait.side_effect = subprocess.TimeoutExpired('owned-weston', 10)
                    with self.assertRaises(subprocess.TimeoutExpired):
                        owned.close_owned_envelope(qa)
                    qa.runtime.cleanup.assert_not_called()
                    qa.runtime._finalizer.detach.assert_called_once()
                else:
                    owned.close_owned_envelope(qa)
                    qa.runtime.cleanup.assert_called_once()
                    qa.runtime._finalizer.detach.assert_not_called()
                qa.clipboard_writer.terminate.assert_called_once()
                qa.compositor.terminate.assert_called_once()

    def test_s5a_public_reaper_is_the_shared_callable(self):
        import prove_s5a_native_selection as selection
        self.assertIs(selection.close_owned_companions, owned.close_owned_companions)

    def test_generic_main_writes_failure_and_actions_even_when_cleanup_raises(self):
        import prove_native_project_startup_gui as startup
        with tempfile.TemporaryDirectory() as temporary:
            parent = Path(temporary)
            run = parent / 'failed-run'
            qa = self.qa()
            qa.binary_hashes = {}
            qa.previous_focus = ''
            qa.execute = Mock()
            qa.stop = Mock(side_effect=RuntimeError('owned GUI teardown failed'))
            qa.close_envelope = Mock(side_effect=RuntimeError('retained runtime'))
            qa.actions.append({'synthetic_cleanup_control': True})
            args = SimpleNamespace(run_dir=run, native_launch_smoke=False, package_project=None,
                                   file_actions=False, minimum_dialogs=False, new_global=False, browse=False)
            with patch.object(startup, 'OWNED', parent), patch.object(startup, 'StartupQA', return_value=qa), \
                    patch.object(startup.argparse.ArgumentParser, 'parse_args', return_value=args):
                with self.assertRaisesRegex(RuntimeError, 'Owned QA teardown failed'):
                    startup.main()
            result = json.loads((run / 'result.json').read_text())
            self.assertEqual(result['status'], 'FAIL')
            self.assertEqual(set(result['cleanup_errors']), {'GUI', 'envelope'})
            self.assertEqual(json.loads((run / 'commands.json').read_text()), qa.actions)
            self.assertEqual(result['observations'], qa.observations)


if __name__ == '__main__':
    unittest.main()
