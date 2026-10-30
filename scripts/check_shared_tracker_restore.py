#!/usr/bin/env python3
"""Narrow PreToolUse protection against restoring another worker's tracker rows.

Inspects ordinary literal Git commands, not shell aliases/plugins, substitutions
or scripts passed to other interpreters. It never executes the proposed command.
"""
from __future__ import annotations

import argparse
import json
import os
import re
from pathlib import Path
import shlex
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
TRACKER = '.beads/issues.jsonl'
DENIAL = ('Refusing a Git working-tree restore of dirty .beads/issues.jsonl: it can erase '
          "another worker's rows. Use br row-level create/update/delete operations; preserve "
          'and reconcile the current tracker bytes instead of restoring the whole file.')


def _tokens(text):
    lexer = shlex.shlex(text, posix=False, punctuation_chars=';&|()<>\n')
    lexer.whitespace = ' \t\r'
    lexer.commenters = ''
    tokens = []
    for token in lexer:
        if token and set(token) <= set(';&|()<>\n'):
            tokens.extend(re.findall(r'<<<|<<|>>|>&|<&|&>|&&|\|\||[;&|()<>\n]', token))
        else:
            tokens.append(token)
    return tokens


def _word(token):
    values = shlex.split(token)
    return values[0] if len(values) == 1 else token


def _literal_commands(text):
    # Here-documents are data. Keep their command headers but omit body examples.
    lines, pending = [], []
    for line in text.splitlines(keepends=True):
        if pending:
            delimiter, tabs = pending[0]
            if (line.lstrip('\t') if tabs else line).rstrip('\r\n') == delimiter:
                pending.pop(0)
            lines.append('\n')
            continue
        tokens = _tokens(line)
        for index, token in enumerate(tokens[:-1]):
            if token == '<<':
                raw = tokens[index + 1]
                tabs = raw.startswith('-')
                if tabs:
                    raw = raw[1:] or (tokens[index + 2] if index + 2 < len(tokens) else '')
                pending.append((_word(raw), tabs))
        lines.append(line)
    command, comment, redirect = [], False, False
    for token in _tokens(''.join(lines)):
        if token == '\n':
            comment = False
        if comment:
            continue
        if redirect:
            redirect = False
            continue
        if token in {'>', '>>', '<', '<<', '<<<', '>&', '<&', '&>'}:
            if command and command[-1].isdigit():
                command.pop()
            redirect = True
            continue
        if token.startswith('#'):
            comment = True
            continue
        if token in {';', '&&', '||', '|', '&', '\n', '(', ')'}:
            if command:
                yield command
                command = []
            if token in {'(', ')'}:
                yield [token]
        else:
            command.append(_word(token))
    if command:
        yield command


def _git(cwd, prefix, arguments):
    try:
        return subprocess.run(['git', '--no-pager', '--no-optional-locks', *prefix,
                               '-c', 'core.fsmonitor=false', '-c', 'core.hooksPath=/dev/null', *arguments],
                              cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                              env={**os.environ, 'GIT_OPTIONAL_LOCKS': '0'}, timeout=5, check=False)
    except (OSError, subprocess.SubprocessError):
        return None


def _repository(cwd, prefix=()):
    top = _git(cwd, prefix, ['rev-parse', '--show-toplevel'])
    common = _git(cwd, prefix, ['rev-parse', '--path-format=absolute', '--git-common-dir'])
    if top is None or common is None or top.returncode or common.returncode:
        return None
    return (Path(os.fsdecode(top.stdout).strip()).resolve(),
            Path(os.fsdecode(common.stdout).strip()).resolve())


def _git_command(words):
    while words and words[0] in {'then', 'do', 'else', 'if', 'while', 'until', '!', 'command'}:
        words = words[1:]
    if not words or Path(words[0]).name != 'git':
        return None
    prefix, index = [], 1
    takes_value = {'-C', '-c', '--git-dir', '--work-tree', '--namespace', '--config-env'}
    while index < len(words):
        token = words[index]
        if not token.startswith('-'):
            return prefix, token, words[index + 1:]
        prefix.append(token)
        if token in takes_value:
            index += 1
            if index >= len(words):
                return None
            prefix.append(words[index])
        index += 1
    return None


def _restore_paths(command, arguments):
    if command == 'reset':
        return [':/'] if '--hard' in arguments else None
    if command not in {'checkout', 'restore'}:
        return None
    before, separator, paths = arguments, False, []
    if '--' in arguments:
        index = arguments.index('--')
        before, paths, separator = arguments[:index], arguments[index + 1:], True
    staged = any(value == '--staged' or (value.startswith('-') and not value.startswith('--')
                                       and not value.startswith('-s') and 'S' in value) for value in before)
    worktree = any(value == '--worktree' or (value.startswith('-') and not value.startswith('--')
                                           and not value.startswith('-s') and 'W' in value) for value in before)
    if command == 'restore' and staged and not worktree:
        return None
    if separator:
        return paths
    positional, index = [], 0
    while index < len(before):
        token = before[index]
        if token in {'--source', '-s', '-b', '-B', '--orphan', '--conflict'}:
            index += 2
            continue
        if not token.startswith('-'):
            positional.append(token)
        index += 1
    # Checkout's optional tree-ish is resolved separately; force with an explicit
    # path still affects only that path, while force without paths affects all.
    if command == 'checkout' and not positional and any(value in {'-f', '--force'} for value in before):
        return [':/']
    return positional


def decision(command: str, cwd: Path, *, protected_root: Path = ROOT):
    protected = None
    current = Path(cwd)
    directories = []
    try:
        commands = list(_literal_commands(command))
    except ValueError:
        return None  # Unresolved shell syntax is outside this literal-command hook.
    for words in commands:
        if words == ['(']:
            directories.append(current)
            continue
        if words == [')']:
            if directories:
                current = directories.pop()
            continue
        if words and words[0] == 'cd' and len(words) == 2:
            destination = Path(words[1])
            destination = destination if destination.is_absolute() else current / destination
            if destination.is_dir():
                current = destination.resolve()
            continue
        parsed = _git_command(words)
        if parsed is None:
            continue
        prefix, name, arguments = parsed
        paths = _restore_paths(name, arguments)
        if not paths:
            continue
        if protected is None:
            protected = _repository(protected_root)
            if protected is None:
                return None
        repository = _repository(current, prefix)
        if repository is None or repository[1] != protected[1]:
            continue
        top, _ = repository
        if name == 'checkout' and '--' not in arguments and paths != [':/']:
            revision = _git(current, prefix, ['rev-parse', '--verify', '--quiet', paths[0] + '^{tree}'])
            if revision is not None and revision.returncode == 0:
                paths = paths[1:]
            if not paths:
                if any(value in {'-f', '--force'} for value in arguments):
                    paths = [':/']
                else:
                    continue
        selected = _git(current, prefix, ['ls-files', '--full-name', '--with-tree=HEAD', '-z', '--', *paths])
        if selected is None or selected.returncode or os.fsencode(TRACKER) not in selected.stdout.split(b'\0'):
            continue
        dirty = _git(current, prefix, ['status', '--porcelain=v1', '-z', '--untracked-files=all',
                                      '--', str(top / TRACKER)])
        if dirty is not None and dirty.returncode == 0 and dirty.stdout:
            return DENIAL
    return None


def event_decision(event, *, protected_root=ROOT):
    if (event.get('hook_event_name') != 'PreToolUse' or event.get('tool_name') != 'Bash'
            or not isinstance(event.get('tool_input'), dict)):
        return None
    command, cwd = event['tool_input'].get('command'), event.get('cwd')
    if not isinstance(command, str) or not isinstance(cwd, str):
        return None
    reason = decision(command, Path(cwd), protected_root=protected_root)
    if reason:
        return {'hookSpecificOutput': {'hookEventName': 'PreToolUse', 'permissionDecision': 'deny',
                                       'permissionDecisionReason': reason}}
    return None


def _rows(text, label):
    def pairs(entries):
        row = {}
        for key, value in entries:
            if key in row:
                raise ValueError("duplicate JSON field")
            row[key] = value
        return row

    def invalid_constant(_value):
        raise ValueError("non-JSON numeric constant")

    rows = {}
    for number, line in enumerate(text.splitlines(), 1):
        if not line.strip():
            continue
        try:
            row = json.loads(line, object_pairs_hook=pairs, parse_constant=invalid_constant)
        except ValueError:
            raise ValueError(f"{label}: malformed JSONL at line {number}") from None
        if not isinstance(row, dict) or not isinstance(row.get('id'), str) or not row['id'].strip():
            raise ValueError(f"{label}: row at line {number} needs a nonempty string id")
        if row['id'] in rows:
            raise ValueError(f"{label}: duplicate row id at line {number}")
        rows[row['id']] = row
    return rows


def tracker_diff(head, working):
    """Compare complete JSONL rows by ID; expose identities/fields, not payloads."""
    before, after = _rows(head, 'HEAD'), _rows(working, 'working tree')
    changed = {}
    encode = lambda value: json.dumps(value, sort_keys=True, ensure_ascii=False, allow_nan=False)
    for identity in sorted(before.keys() & after.keys()):
        fields = sorted(field for field in before[identity].keys() | after[identity].keys()
                        if field not in before[identity] or field not in after[identity]
                        or encode(before[identity][field]) != encode(after[identity][field]))
        if fields:
            changed[identity] = fields
    return {'added': sorted(after.keys() - before.keys()),
            'removed': sorted(before.keys() - after.keys()), 'changed': changed}


def working_tracker_diff(root=ROOT):
    head = _git(root, (), ['show', 'HEAD:' + TRACKER])
    if head is None or head.returncode:
        raise ValueError('cannot read HEAD tracker for comparison')
    try:
        return tracker_diff(head.stdout.decode('utf-8'), (Path(root) / TRACKER).read_text(encoding='utf-8'))
    except (OSError, UnicodeError):
        raise ValueError('cannot read UTF-8 tracker for comparison') from None


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--hook', action='store_true', help='process one PreToolUse event from stdin')
    mode.add_argument('--diff', action='store_true', help='read-only complete tracker comparison against HEAD')
    args = parser.parse_args(argv)
    if args.diff:
        try:
            print(json.dumps(working_tracker_diff(), sort_keys=True))
        except ValueError as error:
            print(f'Tracker comparison failed: {error}', file=sys.stderr)
            return 2
        return 0
    if not args.hook:
        return 0
    try:
        event = json.load(sys.stdin)
    except (ValueError, OSError):
        return 0
    if isinstance(event, dict):
        result = event_decision(event)
        if result is not None:
            print(json.dumps(result))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
