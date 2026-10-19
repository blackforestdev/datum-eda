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
from s5a_native_escape_cases import owned_open_text_keys, region_escape_cancel
from s5a_native_inspector_cases import focused_compound_scopes, focused_compound_pane_lifecycle, hidden_junction_run
from provision_s5a_replay_inputs import INPUTS, replay_location
from s5a_native_selection_observation import latest_state, qualify, selection_snapshot

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


def parent_points(qa, main, state, project, contract, result, setup_only=False):
    """One current-frame body acquisition sequence, never a point sweep."""
    symbols = json.loads((project / 'schematic/main.json').read_text())['symbols']
    by_ref = {s['reference']: s for s in symbols.values()}
    assert set(by_ref) == {'U1', 'U2'} and len(symbols) == 2
    parents = {ref: {'class': 'symbol', 'id': s['uuid']} for ref, s in by_ref.items()}
    result['owned_source_map'] = by_ref
    result['owned_library_source'] = json.loads((project / 'pool/symbols' /
                                               (by_ref['U1']['lib_id'] + '.json')).read_text())
    qa.key(main, 'Escape')  # Remove any launch-focus point acquisition.
    qa.wait('initial parent selection empty', lambda: state()['value']['selection']['kind'] == 'none')
    result['initial_presented_frame'] = qa.capture('parents-initial-current-frame', main)
    result['initial_state'] = state()
    # Bind a finite point plan to THIS presented capture after source/visual
    # inspection. The plan contains ordinary mouse coordinates, no App setters.
    plan_path = qa.root / 'current-frame-point-plan.json'
    plan = qa.wait('source-bound point plan for current presented frame', lambda:
                   json.loads(plan_path.read_text()) if plan_path.exists() else False, timeout=300)
    assert plan['capture_sha256'] == digest(qa.root / 'parents-initial-current-frame.png')
    assert set(plan['points']) == {'body_u1', 'body_u2'} and plan['basis'] and plan['transform']
    result['point_plan'] = plan
    expected = {'project': contract['project_id'], 'revision': contract['model_revision'],
                'pid': qa.process.pid, 'oracle': 'Frozen parents-v3 schematic parent IDs and source-owned primitives; independent checkpoint focus'}
    dispatch_log = qa.root / f'{qa.phase}.log'

    def body_dispatch(offset, ref):
        lines = [line for line in dispatch_log.read_text()[offset:].splitlines()
                 if line.startswith('[datum-click]') and 'primary click' in line]
        assert any(f'schematic-symbol:{parents[ref]["id"]}:library:' in line for line in lines), (
            'actual button dispatch must hit the source-owned library body, not a pin/text fallback', lines)
        return lines

    def observe(kind, refs, focus, prior):
        record = state()
        if record['sequence'] <= prior or record['value']['selection'].get('kind') != kind:
            return False
        try:
            snapshot = selection_snapshot(qa, Atspi.StateType.SELECTED)
            observation = qualify(record, snapshot, {**expected, 'kind': kind,
                'scope': 'symbol' if kind == 'object' else None,
                'members': [parents[r] for r in refs], 'focus': parents[focus]})
            if any(m['attributes'].get('hidden') != 'false' for m in snapshot['members']):
                raise ValueError('parent unexpectedly hidden')
            return observation if state()['sequence'] == record['sequence'] else False
        except ValueError as error:
            result['pending_public_read_failure'] = str(error)
            return False

    actions = [
            ('body-u1-object', 'body_u1', None, 'object', ['U1'], 'U1'),
            ('shift-body-u2-compound', 'body_u2', 'Shift_L', 'compound', ['U1', 'U2'], 'U2'),
            ('ctrl-body-u1-remove', 'body_u1', 'Control_L', 'object', ['U2'], 'U2')]
    for label, point, modifier, kind, refs, focus in (actions[:1] if setup_only else actions):
        prior = state()['sequence']
        qa.wait_presented()
        qa.focus(main)
        x, y = plan['points'][point]['screen_px']
        diagnostic = Path(qa.env['DATUM_GUI_LOG'])
        offset = len(diagnostic.read_text())
        qa.command('xdotool', 'mousemove', '--window', main, str(x), str(y))
        qa.wait('native pointer at source-bound target', lambda:
                f'cursor moved {x:.2f},{y:.2f}' in diagnostic.read_text()[offset:])
        if modifier:
            qa.command('xdotool', 'keydown', modifier)
        dispatch_offset = len(dispatch_log.read_text())
        try:
            qa.command('xdotool', 'click', '1')
        finally:
            if modifier:
                qa.command('xdotool', 'keyup', modifier)
        result[label] = qa.wait(label, lambda: observe(kind, refs, focus, prior))
        result[label]['body_dispatch'] = body_dispatch(dispatch_offset, 'U1' if point == 'body_u1' else 'U2')
        qa.capture(label, main)
    if setup_only:
        return
    prior = state()['sequence']
    qa.key(main, 'Escape')
    result['cleared'] = qa.wait('parent Escape clear', lambda: state() if
        state()['sequence'] > prior and state()['value']['selection']['kind'] == 'none' else False)
    qa.wait('public selection root removed', lambda: not any(
        n.get_accessible_id() == '/org/a11y/atspi/accessible/selection' for n in qa.nodes()))
    qa.capture('parents-escape-clear', main)
    prior = state()['sequence']
    x, y = plan['points']['body_u2']['screen_px']
    diagnostic = Path(qa.env['DATUM_GUI_LOG'])
    offset = len(diagnostic.read_text())
    qa.command('xdotool', 'mousemove', '--window', main, str(x), str(y))
    qa.wait('fresh body native pointer acknowledgement', lambda:
            f'cursor moved {x:.2f},{y:.2f}' in diagnostic.read_text()[offset:])
    qa.wait_presented()
    dispatch_offset = len(dispatch_log.read_text())
    qa.command('xdotool', 'click', '1')
    result['fresh-body-u2-object'] = qa.wait('fresh point acquisition after Escape', lambda:
        observe('object', ['U2'], 'U2', prior))
    result['fresh-body-u2-object']['body_dispatch'] = body_dispatch(dispatch_offset, 'U2')
    qa.capture('fresh-body-u2-object', main)


def pin_points(qa, main, state, project, contract, result, setup_only=False):
    symbols = json.loads((project / 'schematic/main.json').read_text())['symbols']
    parent = next(s for s in symbols.values() if s['reference'] == 'U14')
    index, pin = next((i, p) for i, p in enumerate(parent['pins']) if p['name'] == 'P13')
    assert pin['position'] == {'x': 13_000_000, 'y': 0}
    assert sum(p['position'] == pin['position'] for s in symbols.values() for p in s['pins']) == 1
    identity = {'class': 'symbol', 'id': parent['uuid']}
    expected = {'pid': qa.process.pid, 'project': contract['project_id'],
                'revision': contract['model_revision'], 'kind': 'object',
                'scope': identity['class'], 'members': [identity], 'focus': identity,
                'oracle': 'Frozen majority-v1 unique P13 absolute terminal and owning U14; no acquired child supplies the parent identity'}
    result['pin_source_oracle'] = {'parent': parent, 'pin': pin, 'index': index, 'expected': expected}
    qa.key(main, 'Escape')
    qa.wait('initial pin selection empty', lambda: state()['value']['selection']['kind'] == 'none')
    result['initial_presented_frame'] = qa.capture('pin-initial-current-frame', main)
    result['initial_state'] = state()
    path = qa.root / 'current-frame-point-plan.json'
    plan = qa.wait('source-bound pin point plan', lambda:
                   json.loads(path.read_text()) if path.exists() else False, timeout=300)
    assert plan['capture_sha256'] == digest(qa.root / 'pin-initial-current-frame.png')
    assert plan['pin_uuid'] == pin['uuid'] and plan['transform']
    result['point_plan'] = plan
    diagnostic, dispatch = Path(qa.env['DATUM_GUI_LOG']), qa.root / f'{qa.phase}.log'
    for label in (['pin-setup-parent'] if setup_only else ['pin-acquired-parent', 'pin-reacquired-parent']):
        prior = state()['sequence']
        qa.wait_presented()
        qa.focus(main)
        x, y = plan['screen_px']
        offset = len(diagnostic.read_text())
        qa.command('xdotool', 'mousemove', '--window', main, str(x), str(y))
        qa.wait('native pointer at unique source pin', lambda:
                f'cursor moved {x:.2f},{y:.2f}' in diagnostic.read_text()[offset:])
        offset = len(dispatch.read_text())
        qa.command('xdotool', 'click', '1')
        def observe():
            record = state()
            if record['sequence'] <= prior or record['value']['selection'].get('kind') != 'object':
                return False
            try:
                observation = qualify(record, selection_snapshot(qa, Atspi.StateType.SELECTED), expected)
                assert all(m['attributes']['hidden'] == 'false' for m in observation['public_snapshot']['members'])
                return observation if state()['sequence'] == record['sequence'] else False
            except ValueError as error:
                result['pending_public_read_failure'] = str(error)
                return False
        result[label] = qa.wait('complete public exact pin parent/focus/scope', observe)
        lines = [line for line in dispatch.read_text()[offset:].splitlines() if line.startswith('[datum-click]')]
        assert any(f'schematic-symbol-pin-terminal:{parent["uuid"]}:{index}' in line for line in lines), lines
        result[label]['pin_dispatch'] = lines
        qa.command('xdotool', 'mousemove', '--window', main, '100', '100')
        qa.wait_presented()
        qa.capture(label, main)
        if label == 'pin-acquired-parent':
            prior = state()['sequence']
            qa.key(main, 'Escape')
            result['cleared'] = qa.wait('pin parent Escape clears', lambda: state() if
                state()['sequence'] > prior and state()['value']['selection']['kind'] == 'none' else False)
            qa.wait('public pin selection root removed', lambda: not any(
                n.get_accessible_id() == '/org/a11y/atspi/accessible/selection' for n in qa.nodes()))
            qa.capture('pin-escape-clear', main)


def terminal_key_ownership(qa, main, state, result):
    expected = result['pin_source_oracle']['expected']

    def retain():
        return qualify(state(), selection_snapshot(qa, Atspi.StateType.SELECTED), expected)

    def terminal_focus():
        return next((n for n in qa.nodes() if n.get_role_name() == 'terminal'
                     and n.get_process_id() == qa.process.pid
                     and n.get_state_set().contains(Atspi.StateType.FOCUSED)), None)

    def focus_receipt():
        node = qa.wait('owned Terminal standard FOCUSED and matching focus authority', lambda:
                       terminal_focus() if state()['value']['focus'] == 'Terminal' else False)
        return {'pid': node.get_process_id(), 'accessible_id': node.get_accessible_id(),
                'standard_focused': True, 'diagnostic_focus': state()['value']['focus']}

    def lines():
        return [line.rstrip() for line in qa.terminal_text().splitlines()]

    qa.terminal_dock_open = state()['value']['dock'] != 'None'
    # Shell-local deterministic readline binding; no profile/config writes.
    qa.terminal_command(main, "set -o emacs && printf 'N17_EDIT_READY\\n'")
    qa.wait('executed shell editing-ready output', lambda: 'N17_EDIT_READY' in lines())
    result['terminal_focus_before_ctrl_a'] = focus_receipt()
    result['selection_before_terminal_keys'] = retain()
    qa.capture('terminal-focused-before-keys', main)
    qa.text(main, "printf 'N17_TAIL\\n'")
    qa.key(main, 'ctrl+a')
    result['terminal_focus_after_ctrl_a'] = focus_receipt()
    result['selection_after_ctrl_a'] = retain()
    qa.text(main, "printf 'N17_HEAD\\n'; ")
    qa.key(main, 'Return')
    def ordered_output():
        rows = lines()
        return any(rows[i:i + 2] == ['N17_HEAD', 'N17_TAIL'] for i in range(len(rows) - 1))
    qa.wait('executed HEAD then TAIL, not echoed command text', ordered_output)
    result['executed_ctrl_a_output'] = ['N17_HEAD', 'N17_TAIL']
    result['selection_after_shell_output'] = retain()
    qa.capture('terminal-ctrl-a-executed', main)
    # Bash read -n1 consumes the actual key, independently proving PTY delivery.
    qa.terminal_command(main,
        "printf 'N17_ESC_READY\\n'; IFS= read -rsn1 N17_KEY; "
        "printf 'N17_ESC:%d\\n' \"'$N17_KEY\"")
    qa.wait('executed Escape reader ready', lambda: 'N17_ESC_READY' in lines())
    result['terminal_focus_before_escape'] = focus_receipt()
    result['selection_before_escape'] = retain()
    prior = state()['sequence']
    qa.key(main, 'Escape')  # Ordinary press AND release; release returns Editor.
    qa.wait('actual shell Escape byte27', lambda: 'N17_ESC:27' in lines())
    qa.wait('Escape release returns editor ownership', lambda:
            state()['sequence'] > prior and state()['value']['focus'] == 'Editor(PaneId(1))'
            and terminal_focus() is None)
    result['executed_escape_output'] = 'N17_ESC:27'
    result['focus_after_escape_release'] = state()
    result['selection_after_terminal_escape'] = retain()
    qa.capture('terminal-escape-release-editor', main)
    # No click, reset or reacquisition between focus release and this control.
    prior = state()['sequence']
    qa.key(main, 'Escape')
    result['editor_escape_clear'] = qa.wait('second distinct editor Escape clears', lambda:
        state() if state()['sequence'] > prior
        and state()['value']['selection']['kind'] == 'none' else False)
    qa.wait('public selection root removed after editor Escape', lambda: not any(
        n.get_accessible_id() == '/org/a11y/atspi/accessible/selection' for n in qa.nodes()))
    qa.capture('editor-second-escape-clear', main)


def overlap_menu(qa, main, state, project, contract, result):
    symbols = json.loads((project / 'schematic/main.json').read_text())['symbols']
    ids = {s['reference']: s['uuid'] for s in symbols.values()}
    assert set(ids) == {'U1', 'U2'}
    pins = [next(p for p in s['pins'] if p['name'] == 'OUT') for s in symbols.values()]
    assert all(p['position'] == {'x': 1_500_000, 'y': 0} for p in pins)
    prior_id = {'class': 'symbol', 'id': ids['U1']}
    expected = {'pid': qa.process.pid, 'project': contract['project_id'],
                'revision': contract['model_revision'], 'kind': 'object', 'scope': 'symbol',
                'members': [prior_id], 'focus': prior_id, 'oracle': 'Frozen parents-v3 U1 prior selection; both coincident OUT pin owners deduplicated to U1/U2'}
    result['overlap_source_oracle'] = {'parents': ids, 'pins': pins, 'prior': expected}
    prefix = 'menu:native-select:' + expected['project']
    def menu():
        root = next((n for n in qa.nodes() if n.get_accessible_id() == prefix), None)
        if root is None:
            return False
        assert root.get_process_id() == qa.process.pid and root.get_child_count() <= 1000
        rows = []
        for i in range(root.get_child_count()):
            n = root.get_child_at_index(i)
            if n is None:
                return False
            assert n.get_process_id() == qa.process.pid
            rows.append({'index': i, 'id': n.get_accessible_id(), 'name': n.get_name(),
                         'attributes': qa.attributes(n),
                         'focused': n.get_state_set().contains(Atspi.StateType.FOCUSED)})
        return {'pid': root.get_process_id(), 'root': prefix, 'rows': rows}
    def complete_menu():
        snap = menu()
        if not snap:
            return False
        candidates = [r for r in snap['rows'] if r['id'].startswith(prefix + '/Object(')]
        if not candidates:
            return False
        assert len(candidates) == 2 and all('class: Symbol' in r['id'] and 'instance_path: []' in r['id'] for r in candidates)
        assert {uid for uid in ids.values() if any(uid in r['id'] for r in candidates)} == set(ids.values())
        assert all(r['attributes'].get('available') == 'true' for r in candidates)
        return snap
    retain = lambda: qualify(state(), selection_snapshot(qa, Atspi.StateType.SELECTED), expected)
    x, y = result['point_plan']['shared_pin_screen_px']
    diagnostic = Path(qa.env['DATUM_GUI_LOG']); offset = len(diagnostic.read_text())
    qa.command('xdotool', 'mousemove', '--window', main, str(x), str(y))
    qa.wait('native cursor at shared source pin', lambda: f'cursor moved {x:.2f},{y:.2f}' in diagnostic.read_text()[offset:])
    qa.key(main, 'shift+F10')
    result['menu_root'] = qa.wait('Overlay owns native Select root', lambda: menu() if state()['value']['focus'] == 'Overlay' else False)
    result['prior_after_open'] = retain()
    qa.key(main, 'Down')
    result['complete_candidates'] = qa.wait('exact two deduplicated public parent candidates', complete_menu)
    for ref in ['U1', 'U2']:
        snap = complete_menu()
        index = next(r['index'] for r in snap['rows'] if r['id'].startswith(prefix + '/Object(') and ids[ref] in r['id'])
        qa.key(main, 'Home')
        qa.key(main, 'Down', repeat=max(1, index))
        if index == 0:
            qa.key(main, 'Up')
        result['menu_focus_' + ref] = qa.wait('public menu row focus ' + ref, lambda:
            menu() if any(r['focused'] and ids[ref] in r['id'] for r in menu()['rows']) else False)
        result['committed_while_menu_focus_' + ref] = retain()
        qa.capture('overlap-menu-' + ref.lower() + '-focus', main)
    result['canvas_preview_qualified'] = False  # Only actual capture review can qualify visible preview.
    qa.key(main, 'Escape')
    qa.wait('whole native menu dismissed and Editor restored', lambda: not menu() and state()['value']['focus'] == 'Editor(PaneId(1))')
    result['dismissed_prior_preserved'] = retain()
    qa.capture('overlap-dismiss-preserved-u1', main)
    prior = state()['sequence']; qa.key(main, 'Escape')
    result['idle_escape_clear'] = qa.wait('distinct idle Escape clears', lambda: state() if state()['sequence'] > prior and state()['value']['selection']['kind'] == 'none' else False)
    qa.wait('public prior selection removed', lambda: not any(n.get_accessible_id() == '/org/a11y/atspi/accessible/selection' for n in qa.nodes()))
    qa.capture('overlap-idle-escape-clear', main)




def execute(qa, project, result, frozen_files, surface, prehidden_l1, parents_case=False, pin_case=False, terminal_keys=False, overlap_case=False, owned_open=False, region_escape=False, focused_inspector=False, focused_pane_lifecycle=False, hidden_junction=False):
    contract_path = project / ('project.json' if hidden_junction else 's5a-fixture.json')
    contract = json.loads(contract_path.read_text())
    if parents_case or pin_case or terminal_keys or overlap_case or owned_open or region_escape or hidden_junction:
        content = 'Some(Schematic)'
        label = 'selection-terminal' if terminal_keys else ('selection-pin' if pin_case else 'selection-parents')
        if overlap_case:
            label = 'selection-overlap'
        if owned_open:
            label = 'selection-owned-open'
        if region_escape:
            label = 'selection-region-escape'
        if hidden_junction:
            label = 'selection-hidden-junction'
        expected = {'fixture_sha256': digest(contract_path)}
    elif surface == 'schematic':
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
    expected['fixture_sha256'] = digest(contract_path)
    result['frozen_before'] = validate_frozen_sources(project, frozen_files)
    before = result['frozen_before']['canonical_files']
    result['canonical_before'] = before
    qa.start_envelope()
    main = qa.launch(label, project_root=project)
    expected['pid'] = qa.process.pid
    result['oracle'] = expected
    state = lambda: latest_state((qa.root / f'{label}.log').read_text())
    qa.wait('current diagnostic state', state)
    qa.wait_presented()
    if state()['value']['focused_content'] != content:
        qa.key(main, 'Tab')
    qa.wait('ordinary target pane focus',
            lambda: state()['value']['focused_content'] == content)
    if parents_case or pin_case or terminal_keys or overlap_case or owned_open or region_escape or hidden_junction:
        if hidden_junction:
            hidden_junction_run(qa, main, state, project, result, frozen_files)
        elif region_escape:
            pin_points(qa, main, state, project, contract, result, setup_only=True)
            region_escape_cancel(qa, main, state, project, result)
        elif owned_open:
            pin_points(qa, main, state, project, contract, result, setup_only=True)
            owned_open_text_keys(qa, main, state, result)
        elif overlap_case:
            parent_points(qa, main, state, project, contract, result, setup_only=True)
            overlap_menu(qa, main, state, project, contract, result)
        elif terminal_keys:
            pin_points(qa, main, state, project, contract, result, setup_only=True)
            terminal_key_ownership(qa, main, state, result)
        else:
            (pin_points if pin_case else parent_points)(qa, main, state, project, contract, result)
        finish_case(qa, main, state, project, frozen_files, result, before)
        return
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
            snapshot = selection_snapshot(qa, Atspi.StateType.SELECTED)
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
    if focused_inspector or focused_pane_lifecycle:
        if focused_pane_lifecycle:
            focused_compound_pane_lifecycle(qa, main, state, project, result, expected)
        else:
            focused_compound_scopes(qa, main, state, project, result, expected)
        finish_case(qa, main, state, project, frozen_files, result, before)
        return
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
    finish_case(qa, main, state, project, frozen_files, result, before)


def finish_case(qa, main, state, project, frozen_files, result, before):
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
    parser.add_argument('--parents-point', action='store_true')
    parser.add_argument('--pin-point', action='store_true')
    parser.add_argument('--terminal-keys', action='store_true')
    parser.add_argument('--overlap-menu', action='store_true')
    parser.add_argument('--owned-open-text', action='store_true')
    parser.add_argument('--region-escape', action='store_true')
    parser.add_argument('--focused-inspector', action='store_true')
    parser.add_argument('--focused-pane-lifecycle', action='store_true')
    parser.add_argument('--hidden-junction-run', action='store_true')
    args = parser.parse_args()
    if args.hidden_junction_run and (args.surface != 'schematic' or any([
            v for k, v in vars(args).items() if isinstance(v, bool) and k != 'hidden_junction_run'])):
        parser.error('hidden Junction Run is exclusive to the frozen Schematic semantic-scope case')
    if args.focused_pane_lifecycle and (args.surface != 'board' or any([
            args.prehidden_l1, args.parents_point, args.pin_point, args.terminal_keys,
            args.overlap_menu, args.owned_open_text, args.region_escape, args.focused_inspector])):
        parser.error('focused pane lifecycle is exclusive to the frozen visible Board case')
    if args.focused_inspector and (args.surface != 'board' or any([
            args.prehidden_l1, args.parents_point, args.pin_point, args.terminal_keys,
            args.overlap_menu, args.owned_open_text, args.region_escape])):
        parser.error('focused Inspector is exclusive to the frozen visible Board case')
    if args.prehidden_l1 and args.surface != 'board':
        parser.error('pre-hidden L1 control is only defined for this frozen Board case')
    if args.parents_point and (args.surface != 'schematic' or args.prehidden_l1):
        parser.error('parent body points are defined only for the frozen Schematic parents case')
    if args.pin_point and (args.surface != 'schematic' or args.prehidden_l1 or args.parents_point):
        parser.error('pin point is defined only for the frozen Schematic majority case')
    if args.terminal_keys and (args.surface != 'schematic' or args.prehidden_l1 or args.parents_point or args.pin_point):
        parser.error('terminal key ownership is only defined over majority P13 setup')
    if args.overlap_menu and (args.surface != 'schematic' or args.prehidden_l1 or args.parents_point or args.pin_point or args.terminal_keys):
        parser.error('overlap menu is defined only over frozen parents setup')
    if args.owned_open_text and (args.surface != 'schematic' or args.prehidden_l1 or args.parents_point or args.pin_point or args.terminal_keys or args.overlap_menu):
        parser.error('owned Open text keys are defined only over majority P13 setup')
    if args.region_escape and (args.surface != 'schematic' or any([args.prehidden_l1, args.parents_point, args.pin_point, args.terminal_keys, args.overlap_menu, args.owned_open_text])):
        parser.error('region Escape is defined only over majority P13 setup')
    run = args.run_dir.absolute()
    if run.resolve() != run or not run.is_relative_to(OWNED) or run == OWNED or run.exists():
        parser.error('run-dir must be a fresh unaliased child of the owned task scratch')
    replay = replay_location(args.replay_root.absolute())
    if (replay / 'index.json').read_bytes() != (INPUTS / 'index.json').read_bytes():
        parser.error('replay root must have the exact tracked catalogue marker')
    alias = 's5a-c03-u5-majority-v1' if args.surface == 'schematic' else 's5a-c03-u5-scope-v2'
    if args.parents_point or args.overlap_menu:
        alias = 's5a-c03-u5-parents-v3'
    if args.hidden_junction_run:
        alias = 's5a-c03-u5-semantic-v1/variants/semantic-scope'
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
    if args.prehidden_l1 or args.parents_point or args.pin_point or args.terminal_keys or args.overlap_menu or args.owned_open_text or args.region_escape or args.focused_inspector or args.focused_pane_lifecycle or args.hidden_junction_run:
        qa.env['DATUM_TRACE_CLICKS'] = '1'  # Existing passive dispatch observation.
    result = {'status': 'FAIL', 'surface': args.surface, 'prehidden_l1': args.prehidden_l1,
              'parents_point': args.parents_point,
              'pin_point': args.pin_point,
              'terminal_keys': args.terminal_keys,
              'overlap_menu': args.overlap_menu,
              'owned_open_text': args.owned_open_text, 'region_escape': args.region_escape,
              'focused_inspector': args.focused_inspector,
              'focused_pane_lifecycle': args.focused_pane_lifecycle,
              'binary_sha256': qa.binary_hashes, 'project': str(project),
              'limits': 'Finite ordinary selection/complete public membership/nonmutation cases only. Native Datum Wayland/wgpu, outer X11 Weston input. No HUMAN, KWin, C04/C05/C06 or ordinary window-Close acceptance.'}
    result['ipc_runtime'] = {'path': qa.runtime.name, 'uid': os.getuid(),
                             'mode': oct(Path(qa.runtime.name).stat().st_mode & 0o777),
                             'scope': 'Disposable private process runtime only; no frozen replay data.'}
    result['proof_source_sha256_before_launch'] = {
        name: digest(ROOT / 'scripts' / name)
        for name in ['prove_s5a_native_selection.py', 's5a_native_selection_observation.py',
                     'prove_native_project_startup_gui.py', 'native_project_source_inventory.py',
                     's5a_native_escape_cases.py', 's5a_native_inspector_cases.py']}
    try:
        execute(qa, project, result, frozen_files, args.surface, args.prehidden_l1, args.parents_point, args.pin_point, args.terminal_keys, args.overlap_menu, args.owned_open_text, args.region_escape, args.focused_inspector, args.focused_pane_lifecycle, args.hidden_junction_run)
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
