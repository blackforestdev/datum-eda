"""Real disposable Git fixtures; proposed destructive commands are never run."""
import contextlib
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import check_shared_tracker_restore as hook


class SharedTrackerRestoreTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.repo = self.root / 'project'
        self.foreign = self.root / 'unrelated'
        for repo in (self.repo, self.foreign):
            repo.mkdir()
            self.git(repo, 'init', '-q')
            (repo / '.beads').mkdir()
            (repo / hook.TRACKER).write_bytes(b'{"id":"baseline"}\n')
            (repo / 'ordinary.txt').write_text('baseline\n')
            (repo / 'subdir').mkdir()
            (repo / 'subdir/ordinary.txt').write_text('subdirectory baseline\n')
            self.git(repo, 'add', '.beads/issues.jsonl', 'ordinary.txt', 'subdir/ordinary.txt')
            self.git(repo, '-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid',
                     'commit', '-qm', 'disposable fixture baseline')
        self.tracker = self.repo / hook.TRACKER
        self.foreign_row = b'{"id":"another-worker-row","status":"in_progress"}\n'
        self.tracker.write_bytes(self.tracker.read_bytes() + self.foreign_row)

    def git(self, repo, *arguments):
        result = subprocess.run(['git', '--no-optional-locks', '-C', str(repo), *arguments],
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
                                env={**os.environ, 'GIT_OPTIONAL_LOCKS': '0'})
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        return result.stdout

    def decide(self, command, *, cwd=None):
        before = self.tracker.read_bytes()
        result = hook.decision(command, cwd or self.repo, protected_root=self.repo)
        self.assertEqual(self.tracker.read_bytes(), before)
        self.assertIn(self.foreign_row, before)
        return result

    def test_literal_checkout_restore_and_hard_reset_are_denied_without_losing_rows(self):
        for command in (
            'git checkout HEAD -- .beads/issues.jsonl',
            'git checkout -- .beads/issues.jsonl',
            'git checkout HEAD .beads/issues.jsonl',
            'git checkout .beads/issues.jsonl',
            'git restore .beads/issues.jsonl',
            'git restore --source HEAD -- .beads/issues.jsonl',
            'git restore --staged --worktree -- .beads/issues.jsonl',
            'git restore -SW -- .beads/issues.jsonl',
            'git reset --hard HEAD',
            'git checkout -f HEAD',
            'git checkout -f',
            'git checkout -f HEAD -- .beads/issues.jsonl',
        ):
            with self.subTest(command=command):
                reason = self.decide(command)
                self.assertEqual(reason, hook.DENIAL)
                self.assertIn('br row-level', reason)
                self.assertNotIn(command, reason)

    def test_whole_tree_directory_glob_and_excluded_pathspecs_use_git_matching(self):
        for pathspec in ('.', '.beads', "'*'", "':/'", "':(top).beads/issues.jsonl'"):
            with self.subTest(pathspec=pathspec):
                self.assertIsNotNone(self.decide('git restore -- ' + pathspec))
        self.assertIsNone(self.decide("git restore -- . ':(exclude).beads/issues.jsonl'"))
        self.assertIsNone(self.decide('git restore -- ordinary.txt'))
        self.assertIsNone(self.decide('git checkout -f HEAD -- ordinary.txt'))
        self.assertIsNone(self.decide('git checkout -f HEAD ordinary.txt'))
        self.assertIsNone(self.decide('git checkout -f ordinary.txt'))
        self.assertIsNone(self.decide('git restore -- subdir'))

    def test_global_git_c_subdirectories_and_absolute_paths_resolve_real_repo(self):
        quoted = json.dumps(str(self.repo))
        self.assertIsNotNone(self.decide(f'git -C {quoted} restore -- .', cwd=self.root))
        self.assertIsNotNone(self.decide(f'git -C {quoted} -C subdir restore -- ../.beads/issues.jsonl'))
        self.assertIsNotNone(self.decide('git restore -- ../.beads/issues.jsonl', cwd=self.repo / 'subdir'))
        self.assertIsNotNone(self.decide('git restore -- ' + json.dumps(str(self.tracker))))
        self.assertIsNone(self.decide('git restore -- .', cwd=self.repo / 'subdir'))
        self.assertIsNone(self.decide('git -C ' + json.dumps(str(self.foreign)) + ' restore -- .'))
        self.assertIsNotNone(self.decide('git --git-dir=' + str(self.repo / '.git')
                                         + ' --work-tree=' + str(self.repo) + ' restore -- .'))

    def test_same_common_directory_worktree_is_protected(self):
        worktree = self.root / 'linked'
        self.git(self.repo, 'worktree', 'add', '-qb', 'fixture-linked', str(worktree))
        tracker = worktree / hook.TRACKER
        tracker.write_bytes(tracker.read_bytes() + self.foreign_row)
        before = tracker.read_bytes()
        self.assertIsNotNone(self.decide('git restore -- .beads/issues.jsonl', cwd=worktree))
        self.assertEqual(tracker.read_bytes(), before)

    def test_index_only_and_read_only_git_and_br_commands_have_no_decision(self):
        for command in ('git status', 'git diff HEAD -- .beads/issues.jsonl', 'git show HEAD:.beads/issues.jsonl',
                        'git restore --staged -- .beads/issues.jsonl', 'git restore -S -- .',
                        'git reset HEAD -- .beads/issues.jsonl', 'git reset --mixed HEAD',
                        'br create task', 'br update dat-example --status closed', 'br delete dat-example',
                        'git checkout main'):
            with self.subTest(command=command):
                self.assertIsNone(self.decide(command))

    def test_benign_commands_do_not_spawn_git_inspection(self):
        with patch.object(hook.subprocess, 'run', side_effect=AssertionError('unexpected Git inspection')):
            for command in ('br delete dat-example', 'git status', 'git diff HEAD',
                            'git restore --staged -- .', 'git reset HEAD -- .',
                            "echo 'git restore -- .beads/issues.jsonl'"):
                with self.subTest(command=command):
                    self.assertIsNone(self.decide(command))

    def test_readonly_queries_override_caller_fsmonitor_and_hooks_after_scope_prefix(self):
        prefix = ['-C', str(self.repo), '-c', 'core.fsmonitor=caller-program',
                  '-c', 'core.hooksPath=caller-hooks']
        with patch.object(hook.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, b'')) as run:
            hook._git(self.root, prefix, ['status', '--porcelain=v1'])
        self.assertEqual(run.call_args.args[0], ['git', '--no-pager', '--no-optional-locks', *prefix,
                         '-c', 'core.fsmonitor=false', '-c', 'core.hooksPath=/dev/null',
                         'status', '--porcelain=v1'])
        self.assertEqual(run.call_args.kwargs['cwd'], self.root)

    def test_staged_dirty_file_is_protected_and_clean_tracker_is_allowed(self):
        self.git(self.repo, 'add', hook.TRACKER)
        self.assertIsNotNone(self.decide('git restore -- .beads/issues.jsonl'))
        self.assertIsNone(self.decide('git restore --staged -- .beads/issues.jsonl'))
        # Use row-level fixture writes, never a destructive Git restore.
        self.tracker.write_bytes(b'{"id":"baseline"}\n')
        self.assertIsNotNone(hook.decision('git checkout HEAD -- .', self.repo, protected_root=self.repo))
        self.git(self.repo, 'add', hook.TRACKER)
        self.assertIsNone(hook.decision('git restore -- .', self.repo, protected_root=self.repo))

    def test_compound_commands_and_cd_are_examined(self):
        for command in ('git status && git checkout HEAD -- .beads/issues.jsonl',
                        'git diff;\ngit restore -- .beads/issues.jsonl',
                        'git status || git reset --hard HEAD',
                        'cd subdir && git restore -- ../.beads/issues.jsonl',
                        'git restore -- .beads/issues.jsonl > /tmp/example-log 2>&1',
                        'if test -f .beads/issues.jsonl; then git restore -- .; fi'):
            with self.subTest(command=command):
                self.assertIsNotNone(self.decide(command))
        self.assertIsNone(self.decide('cd ' + json.dumps(str(self.foreign)) + ' && git restore -- .'))

    def test_subshell_cd_restores_outer_directory_in_both_directions(self):
        foreign, project = json.dumps(str(self.foreign)), json.dumps(str(self.repo))
        self.assertIsNotNone(self.decide(
            f'(cd {foreign}; git status); git checkout HEAD -- .beads/issues.jsonl'))
        self.assertIsNone(self.decide(
            f'(cd {project}; git status); git restore -- .beads/issues.jsonl', cwd=self.foreign))
        self.assertIsNotNone(self.decide(
            f'(cd {foreign}; (cd {project}; git status); git status); git restore -- .'))
        self.assertIsNone(self.decide(
            f'(cd {project}; (cd {foreign}; git status); git status); git restore -- .', cwd=self.foreign))

    def test_quoted_echo_comment_and_heredoc_examples_are_not_commands(self):
        for command in (
            "echo 'git checkout HEAD -- .beads/issues.jsonl'",
            'echo git checkout HEAD -- .beads/issues.jsonl',
            "printf '%s\\n' 'git reset --hard HEAD'",
            "echo ';' git checkout HEAD -- .beads/issues.jsonl",
            '# git restore -- .beads/issues.jsonl\ngit status',
            "cat <<'EXAMPLE'\ngit checkout HEAD -- .beads/issues.jsonl\nEXAMPLE\n",
            'cat <<-EXAMPLE\n\tgit reset --hard HEAD\n\tEXAMPLE\n',
        ):
            with self.subTest(command=command):
                self.assertIsNone(self.decide(command))
        self.assertIsNotNone(self.decide("cat <<'EXAMPLE'\ngit status\nEXAMPLE\ngit restore -- ."))

    def test_event_protocol_denies_only_pretooluse_bash_without_explicit_allow(self):
        event = {'hook_event_name': 'PreToolUse', 'tool_name': 'Bash', 'cwd': str(self.repo),
                 'tool_input': {'command': 'git restore -- .beads/issues.jsonl'}}
        output = hook.event_decision(event, protected_root=self.repo)
        self.assertEqual(output['hookSpecificOutput'], {
            'hookEventName': 'PreToolUse', 'permissionDecision': 'deny',
            'permissionDecisionReason': hook.DENIAL})
        for change in ({'tool_name': 'Read'}, {'hook_event_name': 'PostToolUse'},
                       {'tool_input': {'command': 'git status'}}):
            self.assertIsNone(hook.event_decision({**event, **change}, protected_root=self.repo))
        with patch.object(hook, 'event_decision', return_value=None), \
                patch.object(hook.sys, 'stdin', io.StringIO(json.dumps(event))), \
                contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(hook.main(['--hook']), 0)
        self.assertEqual(output.getvalue(), '')
        denial = {'hookSpecificOutput': {'hookEventName': 'PreToolUse',
                  'permissionDecision': 'deny', 'permissionDecisionReason': hook.DENIAL}}
        with patch.object(hook, 'event_decision', return_value=denial), \
                patch.object(hook.sys, 'stdin', io.StringIO(json.dumps(event))), \
                contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(hook.main(['--hook']), 0)
        self.assertEqual(json.loads(output.getvalue()), denial)


    def test_default_invocation_is_inert_without_reading_event_or_emitting_deny(self):
        with patch.object(hook, 'event_decision', side_effect=AssertionError('hook must be explicit')) as decide, \
                patch.object(hook.sys, 'stdin') as stdin, contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(hook.main([]), 0)
        stdin.read.assert_not_called()
        decide.assert_not_called()
        self.assertEqual(output.getvalue(), '')

    def test_semantic_diff_ignores_row_and_field_order_and_reports_all_ids(self):
        head = '\n'.join(json.dumps(row) for row in [
            {'id': 'same', 'status': 'open'}, {'id': 'removed'}, {'id': 'changed', 'status': 'open'}])
        working = '\n'.join(json.dumps(row) for row in [
            {'id': 'added'}, {'status': 'open', 'id': 'same'}, {'id': 'changed', 'status': 'closed'}])
        self.assertEqual(hook.tracker_diff(head, working), {
            'added': ['added'], 'removed': ['removed'], 'changed': {'changed': ['status']}})
        self.assertEqual(hook.tracker_diff('{"id":"a","status":"open"}\n{"id":"b"}\n',
                                          '{"id":"b"}\n{"status":"open","id":"a"}\n'),
                         {'added': [], 'removed': [], 'changed': {}})

    def test_semantic_diff_large_description_cannot_hide_later_foreign_row(self):
        head = [{'id': 'first', 'description': 'x' * 10000}, {'id': 'foreign', 'status': 'in_progress'}]
        working = [{'id': 'first', 'description': 'y' * 10000}, {'id': 'foreign', 'status': 'closed'}]
        encode = lambda rows: '\n'.join(json.dumps(row) for row in rows)
        result = hook.tracker_diff(encode(head), encode(working))
        self.assertEqual(result['changed'], {'first': ['description'], 'foreign': ['status']})
        self.assertNotIn('y' * 100, json.dumps(result))
        self.assertEqual(hook.tracker_diff('{"id":"a","value":true}', '{"id":"a","value":1}')['changed'],
                         {'a': ['value']})

    def test_semantic_diff_malformed_duplicate_or_missing_ids_fail_clearly(self):
        for value in ('not JSON', '{"id":"a"}\n{"id":"a"}', '{"status":"open"}',
                      '{"id":1}', '{"id":"a","id":"b"}', '{"id":"a","value":NaN}'):
            with self.subTest(value=value), self.assertRaisesRegex(ValueError, 'HEAD:'):
                hook.tracker_diff(value, '')
        with self.assertRaisesRegex(ValueError, 'working tree:'):
            hook.tracker_diff('', '{bad')

    def test_readonly_diff_uses_complete_head_and_working_rows_without_mutation(self):
        before = self.tracker.read_bytes()
        self.assertEqual(hook.working_tracker_diff(self.repo), {
            'added': ['another-worker-row'], 'removed': [], 'changed': {}})
        self.assertEqual(self.tracker.read_bytes(), before)
        with patch.object(hook, 'working_tracker_diff', return_value={'added': [], 'removed': [], 'changed': {}}), \
                patch.object(hook, 'event_decision', side_effect=AssertionError('diff is not interception')), \
                contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(hook.main(['--diff']), 0)
        self.assertEqual(json.loads(output.getvalue()), {'added': [], 'removed': [], 'changed': {}})
        with patch.object(hook, 'working_tracker_diff', side_effect=ValueError('malformed JSONL')), \
                contextlib.redirect_stderr(io.StringIO()) as error:
            self.assertEqual(hook.main(['--diff']), 2)
        self.assertIn('Tracker comparison failed: malformed JSONL', error.getvalue())


if __name__ == '__main__':
    unittest.main()
