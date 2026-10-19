#!/usr/bin/env python3
"""Finite ordinary native Wayland selection over existing frozen fixtures.

Use /usr/bin/python3 for the installed GI binding. No Cargo or product input
injection is performed. X11 drives only the owned Weston input envelope.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import signal
import time
import traceback

from prove_native_project_startup_gui import Atspi, StartupQA, digest
from native_project_source_inventory import validate_frozen_sources
from provision_s5a_replay_inputs import INPUTS, replay_location
from s5a_native_selection_observation import latest_state, qualify

ROOT = Path(__file__).resolve().parents[1]
OWNED = ROOT / '.datum/scratch/s5a-c04-managed-20261018'


def close_failed_transport(qa, result):
    if not qa.process or qa.process.poll() is not None or qa.terminal_ptys_closed():
        return
    app = qa.process.pid
    groups = []
    for child in Path(f'/proc/{app}/task/{app}/children').read_text().split():
        pid = int(child)
        try:
            stat = Path(f'/proc/{pid}/stat').read_text().rsplit(')', 1)[1].split()
            terminal = str(Path(f'/proc/{pid}/fd/0').readlink()).startswith('/dev/pts/')
            if int(stat[1]) == app and os.getpgid(pid) == pid and terminal:
                os.killpg(pid, signal.SIGHUP)
                groups.append(pid)
        except ProcessLookupError:
            continue
    qa.wait('owned failed-run terminal groups and PTYs close', lambda:
            not any(Path(f'/proc/{pid}').exists() for pid in groups)
            and qa.terminal_ptys_closed(), timeout=7)
    result['failure_transport_cleanup'] = {
        'verified_shell_groups_signalled_HUP': groups,
        'limit': 'Owned process cleanup, not ordinary shell/window-Close proof.'}


def close_owned_companions(qa, result):
    actions = []
    for proc in Path('/proc').iterdir():
        if not proc.name.isdecimal():
            continue
        try:
            binary = (qa.binary_dir / 'eda-engine-daemon').resolve()
            if proc.stat().st_uid != os.getuid() or (proc / 'exe').resolve() != binary:
                continue
            argv = [v.decode() for v in (proc / 'cmdline').read_bytes().split(b'\0') if v]
            if (len(argv) != 3 or argv[1] != '--socket'
                    or not argv[2].startswith(qa.runtime.name + '/datum-native-edit-')):
                continue
            pid = int(proc.name)
            fd = os.pidfd_open(pid)
            try:
                assert (proc / 'cmdline').read_bytes().split(b'\0')[:3] == [v.encode() for v in argv]
                assert (proc / 'exe').resolve() == binary
                signal.pidfd_send_signal(fd, signal.SIGTERM)
            finally:
                os.close(fd)
            until = time.monotonic() + 3
            while proc.exists() and time.monotonic() < until:
                time.sleep(.05)
            if proc.exists():
                raise RuntimeError(f'owned companion {pid} still present after cleanup')
            actions.append({'pid': pid, 'socket': argv[2], 'signal': 'SIGTERM', 'gone': True})
        except (FileNotFoundError, ProcessLookupError):
            continue
    result['owned_companion_housekeeping'] = actions


def selection_snapshot(qa):
    nodes = qa.nodes()
    root = next((n for n in nodes if n.get_accessible_id() ==
                 '/org/a11y/atspi/accessible/selection'), None)
    if root is None:
        raise ValueError('complete public selection root unavailable')
    before = qa.attributes(root)
    inventory = root.get_child_at_index(0)
    count = inventory.get_child_count()
    if count > 1000:
        raise ValueError('this finite fixture case does not traverse a dense selection')
    members = []
    for i in range(count):
        node = inventory.get_child_at_index(i)
        if node is None or node.get_process_id() != qa.process.pid:
            raise ValueError('missing or foreign public member')
        members.append({'attributes': qa.attributes(node),
                        'standard_selected': node.get_state_set().contains(Atspi.StateType.SELECTED)})
    return {'pid': root.get_process_id(), 'before': before, 'after': qa.attributes(root),
            'count': count, 'members': members}


def toggle_l1(qa, main, state, result, visible):
    before = state()['sequence']
    qa.wait_presented()
    qa.focus(main)
    # Same row in the pinned 1344x806 capture; move one pixel on restore so
    # native delivery, rather than an unchanged coordinate, acknowledges motion.
    x, y = (19 if visible else 18), 507
    diagnostic = Path(qa.env['DATUM_GUI_LOG'])
    offset = len(diagnostic.read_text())
    qa.command('xdotool', 'mousemove', '--window', main, str(x), str(y))
    qa.wait('native pointer acknowledges L1 row before press', lambda:
            f'cursor moved {x:.2f},{y:.2f}' in diagnostic.read_text()[offset:])
    qa.command('xdotool', 'click', '1')
    def acknowledged():
        record = state()
        flag = re.search(r'"L1": (true|false)', record['value']['layer_filters'])
        if record['sequence'] > before and flag and (flag[1] == 'true') == visible:
            return record
        return False
    result['layer_restore' if visible else 'pre_acquisition_layer_hide'] = qa.wait(
        'actual L1 visibility transition', acknowledged)


def execute(qa, project, result, frozen_files, surface, prehidden_l1):
    contract = json.loads((project / 's5a-fixture.json').read_text())
    if surface == 'schematic':
        expected = {'project': contract['project_id'], 'revision': contract['model_revision'],
                    'kind': 'compound', 'members': [{'class': 'symbol', 'id': s['source_id']}
                                                   for s in contract['subjects']],
                    'oracle': 'frozen majority fixture subjects/model_revision'}
        content, label, noun = 'Some(Schematic)', 'selection-majority', 'symbols'
    else:
        board = json.loads((project / 'board/board.json').read_text())
        basis = contract['scope_variant']['projects'][0]
        # Existing N14 accepted oracle: four Tracks plus the authored empty
        # BoardOutline. No visible primitive or acquisition result supplies IDs.
        assert len(board['tracks']) == 4 and not board['outline']['vertices']
        expected = {'project': basis['project_id'], 'revision': basis['revision'],
                    'kind': 'compound',
                    'members': [{'class': 'track', 'id': uid} for uid in board['tracks']]
                               + [{'class': 'board_outline', 'id': board['uuid']}],
                    'oracle': 'frozen scope-v2 board source and current project revision; accepted N14 four-Track/empty-outline oracle'}
        content, label, noun = 'Some(Board)', 'selection-board', 'board'
    expected['fixture_sha256'] = digest(project / 's5a-fixture.json')
    result['frozen_before'] = validate_frozen_sources(project, frozen_files)
    before = result['frozen_before']['canonical_files']
    result['canonical_before'] = before
    qa.start_envelope()
    main = qa.launch(label, project_root=project)
    expected['pid'] = qa.process.pid
    result['oracle'] = expected
    state = lambda: latest_state((qa.root / f'{label}.log').read_text())
    qa.wait('current diagnostic state', state)
    if state()['value']['focused_content'] != content:
        qa.key(main, 'Tab')
    qa.wait('ordinary target pane focus',
            lambda: state()['value']['focused_content'] == content)
    if prehidden_l1:
        qa.capture('initial-layer-state', main)
        toggle_l1(qa, main, state, result, False)
        qa.capture('l1-hidden-before-acquisition', main)
    prior = state()['sequence']
    qa.key(main, 'ctrl+a')
    expected_hidden_ids = set(board['tracks']) if prehidden_l1 else set()

    def complete():
        record = state()
        if record['sequence'] <= prior or record['value']['selection'].get('kind') != 'compound':
            return False
        try:
            snapshot = selection_snapshot(qa)
            observation = qualify(record, snapshot, expected)
            attrs = [m['attributes'] for m in snapshot['members']]
            if any(a.get('hidden') not in ['true', 'false'] for a in attrs):
                raise ValueError('complete public hidden statuses unavailable')
            hidden = {json.loads(a['source-identity'])['id'] for a in attrs if a['hidden'] == 'true'}
            if hidden != expected_hidden_ids or record['value']['selection']['hidden_count'] != len(hidden):
                raise ValueError('expected hidden projection not yet published consistently')
            if record['value']['selection']['focus'] is not None or any(
                    a.get('selection-focus') != 'false' for a in attrs):
                raise ValueError('global authored acquisition must retain no member focus')
            if state()['sequence'] != record['sequence']:
                return False
            return observation
        except ValueError as error:
            result['pending_public_read_failure'] = str(error)
            if 'snapshot' in locals():
                result['last_unqualified_public_snapshot'] = snapshot
            return False  # Public worker publication can follow native dispatch.

    result['selected'] = qa.wait('exact complete public selection against frozen oracle', complete)
    if prehidden_l1:
        members = result['selected']['public_snapshot']['members']
        hidden_ids = {json.loads(m['attributes']['source-identity'])['id'] for m in members
                      if m['attributes']['hidden'] == 'true'}
        assert hidden_ids == set(board['tracks']), 'all four L1 Tracks must be hidden members'
        assert len(hidden_ids) == 4 and state()['value']['selection']['hidden_count'] == 4
        result['hidden_track_ids'] = sorted(hidden_ids)
    qa.capture(f'selected-complete-{noun}', main)
    if prehidden_l1:
        prior = state()['sequence']
        expected_hidden_ids = set()
        toggle_l1(qa, main, state, result, True)
        result['restored_selection'] = qa.wait('same complete subject after layer restore', complete)
        assert all(m['attributes']['hidden'] == 'false'
                   for m in result['restored_selection']['public_snapshot']['members'])
        assert state()['value']['selection']['focus'] is None
        qa.capture('l1-restored-same-membership', main)
    prior = state()['sequence']
    qa.key(main, 'Escape')
    result['cleared'] = qa.wait('idle Escape clears selection', lambda:
        state() if state()['sequence'] > prior and state()['value']['selection']['kind'] == 'none' else False)
    qa.wait('public selection root removed', lambda: not any(
        n.get_accessible_id() == '/org/a11y/atspi/accessible/selection' for n in qa.nodes()))
    qa.capture(f'escape-cleared-{noun}', main)
    prior = state()['sequence']
    qa.key(main, 'ctrl+a')
    result['subsequent_selection'] = qa.wait('subsequent keyboard selection', complete)
    qa.capture('subsequent-keyboard-selection', main)
    result['frozen_after_selection'] = validate_frozen_sources(project, frozen_files)
    result['canonical_after_selection'] = result['frozen_after_selection']['canonical_files']
    assert before == result['canonical_after_selection'], 'source/journal changed during selection'
    qa.terminal_dock_open = state()['value']['dock'] != 'None'
    qa.terminal_command(main, 'exit')
    qa.wait('owned terminal PTYs closed before GUI process end', qa.terminal_ptys_closed)
    result['terminal_cleanup'] = 'Ordinary shell exit and observed PTY closure before final GUI SIGTERM.'
    result['frozen_after'] = validate_frozen_sources(project, frozen_files)
    result['canonical_after'] = result['frozen_after']['canonical_files']
    assert before == result['canonical_after'], 'source/journal changed during case'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run-dir', required=True, type=Path)
    parser.add_argument('--replay-root', required=True, type=Path)
    parser.add_argument('--surface', choices=['schematic', 'board'], default='schematic')
    parser.add_argument('--prehidden-l1', action='store_true')
    args = parser.parse_args()
    if args.prehidden_l1 and args.surface != 'board':
        parser.error('pre-hidden L1 control is only defined for this frozen Board case')
    run = args.run_dir.absolute()
    if run.resolve() != run or not run.is_relative_to(OWNED) or run == OWNED or run.exists():
        parser.error('run-dir must be a fresh unaliased child of the owned task scratch')
    replay = replay_location(args.replay_root.absolute())
    if (replay / 'index.json').read_bytes() != (INPUTS / 'index.json').read_bytes():
        parser.error('replay root must have the exact tracked catalogue marker')
    alias = 's5a-c03-u5-majority-v1' if args.surface == 'schematic' else 's5a-c03-u5-scope-v2'
    project = replay / alias
    catalogue = json.loads((INPUTS / 'index.json').read_text())
    frozen_files = {row['path'].removeprefix(alias + '/'): row['sha256']
                    for row in catalogue['files']
                    if row['path'].startswith(alias + '/')}
    if not frozen_files:
        parser.error('selected fixture has no tracked frozen source inventory')
    run.mkdir(parents=True)
    runtime_parent = Path(os.environ['XDG_RUNTIME_DIR'])
    stat = runtime_parent.stat()
    if (not runtime_parent.is_absolute() or runtime_parent.is_symlink()
            or stat.st_uid != os.getuid() or stat.st_mode & 0o077):
        parser.error('existing user IPC runtime must be a private owned directory')
    qa = StartupQA(run, runtime_parent=runtime_parent)
    for key in list(qa.env):
        if key.startswith(('DATUM_', 'WGPU_', 'VK_')) or key in ['EDA_CLI_BIN', 'EDA_ENGINE_SOCKET']:
            qa.env.pop(key)
    qa.env.update({'DATUM_GUI_VERBOSE_LOG': '1', 'DATUM_ACTION_EVIDENCE': '1',
                   'DATUM_GUI_PREFERENCES_PATH': str(run / 'config/gui-preferences.json'),
                   'EDA_CLI_BIN': str(qa.binary_dir / 'datum-eda')})
    if args.prehidden_l1:
        qa.env['DATUM_TRACE_CLICKS'] = '1'  # Existing passive dispatch observation.
    result = {'status': 'FAIL', 'surface': args.surface, 'prehidden_l1': args.prehidden_l1,
              'binary_sha256': qa.binary_hashes, 'project': str(project),
              'limits': 'Finite keyboard/complete public membership/nonmutation case only. Native Datum Wayland/wgpu, outer X11 Weston input. No HUMAN, KWin, C04/C05/C06 or ordinary window-Close acceptance.'}
    result['ipc_runtime'] = {'path': qa.runtime.name, 'uid': os.getuid(),
                             'mode': oct(Path(qa.runtime.name).stat().st_mode & 0o777),
                             'scope': 'Disposable private process runtime only; no frozen replay data.'}
    result['proof_source_sha256_before_launch'] = {
        name: digest(ROOT / 'scripts' / name)
        for name in ['prove_s5a_native_selection.py', 's5a_native_selection_observation.py',
                     'prove_native_project_startup_gui.py', 'native_project_source_inventory.py']}
    try:
        execute(qa, project, result, frozen_files, args.surface, args.prehidden_l1)
        result['status'] = 'PASS for executed finite selection observations'
    except Exception as error:
        result['error'] = str(error)
        (run / 'failure.txt').write_text(traceback.format_exc())
        raise
    finally:
        result['gui_pid'] = qa.process.pid if qa.process else None
        result['compositor_pid'] = qa.compositor.pid if qa.compositor else None
        try:
            close_failed_transport(qa, result)
        except Exception as error:
            result['failure_transport_cleanup_error'] = str(error)
        try:
            qa.stop()
            close_owned_companions(qa, result)
            result['frozen_after_teardown'] = validate_frozen_sources(project, frozen_files)
            qa.close_envelope()
            result['ipc_runtime']['removed_after_process_stop'] = not Path(qa.runtime.name).exists()
        except Exception as error:
            result['teardown_error'] = str(error)
            result['status'] = 'FAIL'
            raise
        finally:
            result['actions'] = qa.actions
            result['native_observations'] = qa.observations
            (run / 'result.json').write_text(json.dumps(result, indent=2) + '\n')


if __name__ == '__main__':
    main()
