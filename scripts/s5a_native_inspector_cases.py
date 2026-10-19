"""Finite focused selection and Inspector cases over frozen native sources."""
from __future__ import annotations

import json
from pathlib import Path
import re
import subprocess

from prove_native_project_startup_gui import Atspi, digest
from s5a_native_selection_observation import qualify, selection_snapshot
from native_project_source_inventory import validate_frozen_sources


def hidden_junction_oracle(project):
    """Independent endpoint/source oracle for the frozen semantic-scope Run."""
    sheet = json.loads((project / 'schematic/selection.json').read_text())
    project_id = json.loads((project / 'project.json').read_text())['uuid']
    head = json.loads((project / '.datum/revision/v1/head.json').read_text())
    generation = json.loads((project / '.datum/revision/v1/generations' /
                             (head['integrity_root'].removeprefix('sha256:') + '.json')).read_text())
    start, middle, end = ({'x': x, 'y': 0} for x in [0, 10000000, 20000000])
    def unique(collection, predicate):
        matches = [v for v in sheet[collection].values() if predicate(v)]
        assert len(matches) == 1, (collection, matches)
        return matches[0]
    a = unique('wires', lambda w: w['from'] == start and w['to'] == middle)
    b = unique('junctions', lambda j: j['position'] == middle)
    c = unique('wires', lambda w: w['from'] == middle and w['to'] == end)
    label = unique('labels', lambda v: v['position'] == end)
    port = unique('ports', lambda v: v['position'] == end)
    assert all(w['from'] != middle and w['to'] != middle
               for w in sheet['wires'].values() if w['uuid'] not in [a['uuid'], c['uuid']])
    members = [{'class': cls, 'id': v['uuid']} for cls, v in
               [('wire', a), ('wire', c), ('junction', b), ('label', label), ('port', port)]]
    origin = {'class': 'wire', 'id': a['uuid']}
    assert len({m['id'] for m in members}) == 5 and project_id == head['project_id']
    return {'project': project_id, 'revision': generation['after_model_revision'],
            'kind': 'run', 'focus': None, 'scope': None, 'members': members,
            'origin': origin, 'subject': {'kind': 'run', 'identity': {'origin': origin}},
            'hidden_junction': b['uuid'], 'sheet_sha256': digest(project / 'schematic/selection.json'),
            'oracle': 'Frozen root[] Wire A endpoint -> Junction B -> Wire C, with Label/Port at C; exact source identities, not labels or acquisition output'}


def hidden_junction_run(qa, main, state, project, result, frozen_files):
    wanted = {**hidden_junction_oracle(project), 'pid': qa.process.pid}
    result['hidden_junction_oracle'] = wanted
    def checkpoint(label):
        result.setdefault('hidden_junction_source_checkpoints', {})[label] = validate_frozen_sources(project, frozen_files)
    def camera():
        value = state()['value']
        return {k: value[k] for k in ['focused_pane', 'focused_content', 'window_size',
                                      'scale', 'layout', 'pane_cameras']}
    qa.key(main, 'Escape')
    qa.wait('initial committed selection empty', lambda: state()['value']['selection']['kind'] == 'none')
    qa.key(main, 'f')
    qa.wait_presented()
    basis = camera()
    result['hidden_junction_fit_state'] = state()
    qa.capture('hidden-junction-fit-before-acquisition', main)
    path = qa.root / 'current-frame-hidden-junction-plan.json'
    plan = qa.wait('source/current-frame layer and wire point plan', lambda:
                   json.loads(path.read_text()) if path.exists() else False, timeout=300)
    assert plan['capture_sha256'] == digest(qa.root / 'hidden-junction-fit-before-acquisition.png')
    assert plan['sheet_sha256'] == wanted['sheet_sha256'] and plan['camera_basis'] == basis
    assert plan['origin'] == wanted['origin'] and plan['wire_world_nm'] == [5000000, 0]
    assert plan['layer'] == 'L202'
    assert plan['transform'] and plan['layer_row_derivation']
    result['hidden_junction_input_plan'] = plan
    diagnostic = Path(qa.env['DATUM_GUI_LOG'])
    def point(point):
        qa.focus(main)
        offset = len(diagnostic.read_text())
        x, y = point
        qa.command('xdotool', 'mousemove', '--window', main, str(x), str(y))
        qa.wait('native pointer delivery before input', lambda:
                f'cursor moved {x:.2f},{y:.2f}' in diagnostic.read_text()[offset:])
    def layer(visible):
        before = state()['sequence']
        x, y = plan['layer_screen_px']
        point([x + int(visible), y])
        qa.command('xdotool', 'click', '1')
        def observed():
            record = state()
            flag = re.search(r'"L202": (true|false)', record['value']['layer_filters'])
            return record if record['sequence'] > before and flag and (flag[1] == 'true') == visible else False
        return qa.wait('ordinary L202 visibility ' + str(visible), observed)
    result['junction_hidden_before_acquisition'] = layer(False)
    assert state()['value']['selection']['kind'] == 'none' and camera() == basis
    checkpoint('prehidden-before-origin')
    qa.capture('junction-hidden-no-selection', main)
    point(plan['wire_screen_px'])
    qa.key(main, 'shift+F10')
    prefix = 'menu:native-select:' + wanted['project']
    def rows():
        return [n for n in qa.nodes() if n.get_accessible_id().startswith(prefix + '/')]
    qa.wait('ordinary marking-menu Select entry', rows)
    assert state()['value']['selection']['kind'] == 'none'
    qa.key(main, 'Down')
    entry = qa.wait('source-origin Select Run row', lambda: next(
        (n for n in rows() if n.get_name() == 'Select Run'), None))
    assert wanted['origin']['id'] in entry.get_accessible_id()
    assert qa.attributes(entry).get('available') == 'true'
    for _ in range(len(rows()) + 1):
        entry = next(n for n in rows() if n.get_name() == 'Select Run')
        if entry.get_state_set().contains(Atspi.StateType.FOCUSED):
            break
        qa.key(main, 'Down')
    else:
        raise RuntimeError('Select Run not reachable by native menu keys')
    result['hidden_junction_menu_entry'] = {'id': entry.get_accessible_id(),
        'name': entry.get_name(), 'attributes': qa.attributes(entry), 'standard_focused': True}
    assert state()['value']['selection']['kind'] == 'none'
    qa.key(main, 'Return')

    def complete(hidden):
        try:
            record = state()
            snapshot = selection_snapshot(qa, Atspi.StateType.SELECTED)
            observation = qualify(record, snapshot, wanted)
            attrs = [m['attributes'] for m in snapshot['members']]
            if any(a.get('hidden') not in ['true', 'false'] for a in attrs):
                raise ValueError('missing public hidden status')
            ids = {json.loads(a['source-identity'])['id'] for a in attrs if a['hidden'] == 'true'}
            expected_hidden = {wanted['hidden_junction']} if hidden else set()
            if ids != expected_hidden or record['value']['selection']['hidden_count'] != int(hidden):
                raise ValueError('hidden Junction/public total not yet current')
            fields = {}
            fields_prefix = '/org/a11y/atspi/accessible/selection/fields/n'
            for node in qa.nodes():
                if node.get_accessible_id().startswith(fields_prefix):
                    key = bytes.fromhex(node.get_accessible_id()[len(fields_prefix):]).decode()
                    fields[key] = qa.attributes(node)
            types = {'wire': 2, 'junction': 1, 'label': 1, 'port': 1}
            for key, value in [('member_count', 5), ('member_types', types)]:
                field = fields[key]
                if (field.get('project-id') != wanted['project'] or field.get('model-revision') != wanted['revision']
                        or field.get('subject-kind') != 'run' or field.get('value-state') != 'common'
                        or json.loads(field['canonical-value']) != value):
                    raise ValueError('Inspector field disagrees: ' + key)
            if state()['sequence'] != record['sequence']:
                return False
            return {'selection': observation, 'fields': fields, 'hidden_ids': sorted(ids)}
        except (ValueError, KeyError, StopIteration) as error:
            result['hidden_junction_pending_read'] = str(error)
            return False
    def context(label, hidden):
        destination = qa.root / ('run-context-' + label + '.json')
        before = complete(hidden)
        assert before, 'current complete GUI before public ContextGet'
        argv = [str(qa.binary_dir / 'datum-eda'), '--format', 'json', 'context', 'get',
                '--project-root', str(project)]
        env = dict(qa.env)
        for key in ['DATUM_DISCOVERY', 'DATUM_TERMINAL_CONTEXT']:
            env.pop(key, None)  # These override explicit --project-root.
        query = subprocess.run(argv, cwd=qa.root, env=env, text=True,
                               stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, timeout=15)
        metadata = {'argv': argv, 'exit_code': query.returncode,
                    'discarded': 'Transient raw envelope stdout and stderr; no opaque context/runtime file reads.'}
        result.setdefault('hidden_junction_context_get', {})[label] = metadata
        assert query.returncode == 0, 'public ContextGet failed'
        selected = json.loads(query.stdout)['selection_context']
        assert isinstance(selected, dict)
        destination.write_text(json.dumps(selected, indent=2) + '\n')
        details = selected['details']
        assert selected['kind'] == 'run'
        for key in ['project', 'revision', 'subject', 'focus', 'scope']:
            assert details[key] == wanted[key], (key, details[key])
        assert details['total'] == 5 and details['hidden'] == int(hidden)
        assert details['per_type'] == {'wire': 2, 'junction': 1, 'label': 1, 'port': 1}
        for key in ['member_count', 'member_types']:
            assert details['inspection'][key] == {
                'state': 'common', 'value': json.loads(before['fields'][key]['canonical-value'])}
        assert details['truncated'] is False and details.get('omitted_inspection_fields', []) == []
        assert details['stale'] is False
        assert sorted(details['members'], key=lambda x: x['id']) == sorted(wanted['members'], key=lambda x: x['id'])
        after = complete(hidden)
        assert after and before['fields'] == after['fields']
        assert before['selection']['public_snapshot'] == after['selection']['public_snapshot']
        assert camera() == basis, 'public Get must not change focused pane/layout/camera'
        metadata.update(selection_context=selected, before=before, after=after,
                        persisted_scope='Allowlisted selection_context only; public emitted-context parity, not samePTY/AI acceptance.')
        checkpoint('public-context-' + label)
    result['hidden_junction_selected'] = qa.wait('complete derived Run across hidden Junction', lambda: complete(True))
    checkpoint('selected-hidden-junction')
    qa.capture('run-five-junction-hidden-wire9-selected', main)
    context('hidden', True)
    result['junction_layer_restored'] = layer(True)
    result['hidden_junction_restored'] = qa.wait('same complete Run after visibility restore', lambda: complete(False))
    checkpoint('same-run-junction-restored')
    qa.capture('run-five-junction-restored', main)
    context('restored', False)


def establish_focused_board_group(qa, main, state, project, result, baseline):
    board_path = project / 'board/board.json'
    board = json.loads(board_path.read_text())
    uid = '40000000-0000-4000-8000-000000000003'
    track = board['tracks'][uid]
    assert track['from'] == {'x': 40000000, 'y': 0}
    assert track['to'] == {'x': 50000000, 'y': 0} and track['width'] == 5080000
    assert len(board['tracks']) == 4 and board['outline']['vertices'] == []
    focus = {'class': 'track', 'id': uid}
    assert focus in baseline['members'] and len(baseline['members']) == 5
    wanted = {**baseline, 'focus': focus, 'scope': None,
              'oracle': 'Frozen scope-v2 complete four Tracks/empty BoardOutline; Shift-click existing isolated Track3 focuses without adding/removing membership'}
    widths = [t['width'] for t in board['tracks'].values()]
    nets = [t['net'] for t in board['tracks'].values()]
    assert widths == [5080000, 5080001, 5080000, 2]
    assert len(set(nets)) == 2 and {t['layer'] for t in board['tracks'].values()} == {1}
    result['focused_inspector_oracle'] = {'selection': wanted, 'widths_nm': widths,
                                          'board_net_ids': nets, 'board_sha256': digest(board_path)}

    def camera():
        value = state()['value']
        assert value['focus'] == 'Editor(PaneId(0))'
        assert value['focused_content'] == 'Some(Board)' and value['focused_pane'] == 0
        return {k: value[k] for k in ['focus', 'focused_pane', 'focused_content',
                                      'window_size', 'scale', 'layout', 'pane_cameras']}

    qa.key(main, 'f')  # Existing ordinary Fit command, never a camera setter.
    qa.wait_presented()
    basis = camera()
    result['focused_inspector_fit_state'] = state()
    result['focused_inspector_fit_presented'] = qa.capture('focused-inspector-fit-current-frame', main)
    path = qa.root / 'current-frame-focused-inspector-plan.json'
    plan = qa.wait('source/current-frame Track3 focus plan', lambda:
                   json.loads(path.read_text()) if path.exists() else False, timeout=300)
    assert plan['capture_sha256'] == digest(qa.root / 'focused-inspector-fit-current-frame.png')
    assert plan['source_board_sha256'] == digest(board_path) and plan['camera_basis'] == basis
    assert plan['focus'] == focus and plan['world_point_nm'] == [45000000, 0]
    assert plan['transform'] and plan['derivation']
    result['focused_inspector_point_plan'] = plan
    qa.focus(main)
    input_frame = qa.wait_presented()
    result['focused_inspector_input_frame'] = input_frame
    assert camera() == basis
    result['focused_inspector_before_shift'] = qualify(
        state(), selection_snapshot(qa, Atspi.StateType.SELECTED),
        {**baseline, 'scope': None, 'focus': None})
    diagnostic = Path(qa.env['DATUM_GUI_LOG'])
    offset = len(diagnostic.read_text())
    x, y = plan['screen_px']
    qa.command('xdotool', 'mousemove', '--window', main, str(x), str(y))
    qa.wait('native pointer at isolated Track3', lambda:
            f'cursor moved {x:.2f},{y:.2f}' in diagnostic.read_text()[offset:])
    held = False
    try:
        qa.command('xdotool', 'keydown', 'Shift_L')
        held = True
        qa.wait('native Shift before idempotent focus click', lambda:
                'modifiers changed ModifiersState(SHIFT)' in diagnostic.read_text()[offset:])
        qa.command('xdotool', 'click', '1')
        qa.command('xdotool', 'keyup', 'Shift_L')
        held = False
        qa.wait('native Shift release after focus click', lambda:
                'modifiers changed ModifiersState(0x0)' in diagnostic.read_text()[offset:])
    finally:
        if held:
            qa.command('xdotool', 'keyup', 'Shift_L')  # Only this case's key.
    events = [line for line in diagnostic.read_text()[offset:].splitlines()
              if f"window event {input_frame['window']} " in line]
    markers = ['modifiers changed ModifiersState(SHIFT)', 'mouse input Left Pressed',
               'mouse input Left Released', 'modifiers changed ModifiersState(0x0)']
    indices = [next(i for i, line in enumerate(events) if marker in line) for marker in markers]
    assert indices == sorted(indices) and not any('focused false' in line for line in events)
    result['focused_inspector_focus_click_events'] = events
    return wanted, basis, camera, focus


def focused_compound_scopes(qa, main, state, project, result, baseline):
    wanted, basis, camera, focus = establish_focused_board_group(
        qa, main, state, project, result, baseline)

    prefix = '/org/a11y/atspi/accessible/selection/fields'
    scopes = [(None, 'All', 5, {'track': 4, 'board_outline': 1}),
              ('track', 'Tracks', 4, {'track': 4}),
              ('board_outline', 'Board outline', 1, {'board_outline': 1})]

    def observe(scope):
        record = state()
        observation = qualify(record, selection_snapshot(qa, Atspi.StateType.SELECTED),
                              {**wanted, 'scope': scope})
        _, label, count, types = next(s for s in scopes if s[0] == scope)
        nodes = qa.nodes()
        root = next(n for n in nodes if n.get_accessible_id() == prefix)
        if root.get_name() != f'Read-only properties: {count} members in {label}':
            raise ValueError('scope field root not yet current')
        fields = {}
        for node in nodes:
            path = node.get_accessible_id()
            if not path.startswith(prefix + '/n'):
                continue
            key = bytes.fromhex(path[len(prefix + '/n'):]).decode()
            attrs = qa.attributes(node)
            if node.get_process_id() != qa.process.pid or any(attrs.get(k) != wanted[v]
                    for k, v in [('project-id', 'project'), ('model-revision', 'revision'),
                                 ('subject-kind', 'kind')]):
                raise ValueError('foreign field basis')
            if key in fields:
                raise ValueError('duplicate field')
            fields[key] = {'attributes': attrs, 'name': node.get_name(),
                           'description': node.get_description()}
        oracle = {'member_count': count, 'member_types': types}
        if scope == 'track':
            oracle['board_layer'] = 1
        for key, value in oracle.items():
            attrs = fields[key]['attributes']
            if attrs.get('value-state') != 'common' or json.loads(attrs['canonical-value']) != value:
                raise ValueError('canonical Common oracle mismatch: ' + key)
        if scope == 'track':
            if any(fields[k]['attributes'].get('value-state') != 'mixed'
                   for k in ['copper_width_nm', 'board_net_id']):
                raise ValueError('canonical full Track scope must remain Mixed')
            if fields['copper_width_nm']['description'] != 'Mixed':
                raise ValueError('width display must report Mixed')
        elif 'copper_width_nm' in fields:
            raise ValueError('aggregate scope exposes class-only width')
        if state()['sequence'] != record['sequence'] or camera() != basis:
            raise ValueError('selection/camera changed during complete read')
        return {'selection': observation, 'fields': fields, 'scope_field_oracle': oracle,
                'expected_rendered_header': 'Compound Selection — 5 objects',
                'expected_rendered_focus': focus}

    def complete(scope):
        try:
            return observe(scope)
        except (ValueError, KeyError, StopIteration) as error:
            result['focused_inspector_pending_read'] = str(error)
            return False  # Current public worker publication follows dispatch.

    result['focused_inspector_after_shift'] = qa.wait('complete five with explicit Track3 focus',
                                                    lambda: complete(None))
    qa.capture('focused-inspector-explicit-focus-all', main)
    menu_prefix = 'menu:native-select:' + wanted['project']

    def menu():
        root = next((n for n in qa.nodes() if n.get_accessible_id() == menu_prefix), None)
        if root is None:
            return False
        assert root.get_process_id() == qa.process.pid
        rows = []
        for i in range(root.get_child_count()):
            node = root.get_child_at_index(i)
            if node is None:
                return False
            assert node.get_process_id() == qa.process.pid
            rows.append({'id': node.get_accessible_id(), 'name': node.get_name(),
                         'attributes': qa.attributes(node),
                         'focused': node.get_state_set().contains(Atspi.StateType.FOCUSED)})
        return rows

    result['focused_inspector_scope_steps'] = []
    def retain(scope):
        try:
            record = state()
            observation = qualify(record, selection_snapshot(qa, Atspi.StateType.SELECTED),
                                  {**wanted, 'scope': scope})
            return observation if state()['sequence'] == record['sequence'] else False
        except ValueError as error:
            result['focused_inspector_pending_menu_read'] = str(error)
            return False

    previous_scope = None
    for scope, command, label in [('track', 'Some(Track)', 'Tracks'),
                                   ('board_outline', 'Some(BoardOutline)', 'Board outline'),
                                   (None, 'None', 'All')]:
        qa.key(main, 'shift+F10')
        qa.wait('owned native Select menu', lambda: menu() if state()['value']['focus'] == 'Overlay' else False)
        qa.key(main, 'Down')  # Existing Select-root keyboard expansion.
        target = f'{menu_prefix}/InspectScope({command})'
        rows = qa.wait('exact public Inspect command identity', lambda:
                       menu() if menu() and any(r['id'] == target for r in menu()) else False)
        index = next(i for i, row in enumerate(rows) if row['id'] == target)
        assert rows[index]['name'] == 'Inspect ' + label
        assert rows[index]['attributes'].get('available') == 'true'
        opened = qa.wait('complete focused selection while native menu is open',
                         lambda: retain(previous_scope))
        qa.key(main, 'Home')
        if index:
            qa.key(main, 'Down', repeat=index)  # Derived public order, never guessed.
        focused = qa.wait('standard focus on exact Inspect command before Enter', lambda:
                         menu() if menu() and any(r['id'] == target and r['focused'] for r in menu()) else False)
        before_enter = qa.wait('complete focused selection on target row before Enter',
                               lambda: retain(previous_scope))
        qa.key(main, 'Return')
        qa.wait('Inspect menu dismissed into owning Editor', lambda:
                not menu() and state()['value']['focus'] == 'Editor(PaneId(0))')
        observation = qa.wait('current focused Compound and scoped fields', lambda: complete(scope))
        receipt = qa.capture('focused-inspector-scope-' + (scope or 'all'), main)
        result['focused_inspector_scope_steps'].append({
            'command': target, 'public_rows_before': rows, 'public_focused_before_enter': focused,
            'selection_while_menu_open': opened, 'selection_before_enter': before_enter,
            'observation': observation, 'presented': receipt})
        previous_scope = scope


def focused_compound_pane_lifecycle(qa, main, state, project, result, baseline):
    wanted, _basis, _camera, focus = establish_focused_board_group(
        qa, main, state, project, result, baseline)
    menu_prefix = 'menu:View'

    def retain(pane, leaves=None):
        try:
            record = state()
            value = record['value']
            expected_focus = 'Overlay' if pane is None else f'Editor(PaneId({pane}))'
            if value['focus'] != expected_focus:
                return False
            snapshot = selection_snapshot(qa, Atspi.StateType.SELECTED)
            selection = qualify(record, snapshot, wanted)
            if snapshot['before'].get('active-pane') != ('' if pane is None else str(pane)):
                raise ValueError('public selection owner not yet current')
            actual_leaves = [(p['pane'], p['content']) for p in value['pane_cameras']]
            if leaves is not None:
                if actual_leaves != leaves or value['focused_pane'] != pane:
                    raise ValueError('current live pane topology not yet acknowledged')
                if value['focused_content'] != f'Some({dict(leaves)[pane]})':
                    raise ValueError('current pane content not yet acknowledged')
                if len(leaves) == 1 and (
                        f'root: Leaf {{ id: PaneId({pane}), content: {leaves[0][1]} }}'
                        not in value['layout']):
                    raise ValueError('single-leaf current layout not yet acknowledged')
                if f'Editor pane {pane}' not in snapshot['before'].get('description', ''):
                    raise ValueError('public current Editor description not yet published')
                presented = value['selection_inspector']['presented']
                if not presented or presented['basis_scope'] != 'None':
                    raise ValueError('current All Inspector basis not presented')
            fields = {}
            for field, expected in {'member_count': 5,
                                   'member_types': {'track': 4, 'board_outline': 1}}.items():
                path = '/org/a11y/atspi/accessible/selection/fields/n' + field.encode().hex()
                node = next(n for n in qa.nodes() if n.get_accessible_id() == path)
                attrs = qa.attributes(node)
                if (node.get_process_id() != qa.process.pid
                        or any(attrs.get(k) != wanted[v] for k, v in [
                            ('project-id', 'project'), ('model-revision', 'revision'),
                            ('subject-kind', 'kind')])
                        or attrs.get('active-pane') != ('' if pane is None else str(pane))
                        or attrs.get('value-state') != 'common'
                        or json.loads(attrs['canonical-value']) != expected):
                    raise ValueError('global Inspector aggregate not yet current: ' + field)
                fields[field] = {'attributes': attrs, 'name': node.get_name()}
            if state()['sequence'] != record['sequence']:
                return False
            return {'selection': selection, 'state': record, 'global_fields': fields,
                    'live_panes': actual_leaves,
                    'expected_rendered_header': 'Compound Selection — 5 objects',
                    'expected_rendered_focus': focus}
        except (ValueError, KeyError, StopIteration) as error:
            result['focused_pane_pending_read'] = str(error)
            return False

    def menu():
        root = next((n for n in qa.nodes() if n.get_accessible_id() == menu_prefix), None)
        if root is None:
            return False
        assert root.get_process_id() == qa.process.pid
        rows = []
        for i in range(root.get_child_count()):
            node = root.get_child_at_index(i)
            if node is None:
                return False
            assert node.get_process_id() == qa.process.pid
            rows.append({'id': node.get_accessible_id(), 'name': node.get_name(),
                         'attributes': qa.attributes(node),
                         'focused': node.get_state_set().contains(Atspi.StateType.FOCUSED)})
        return rows

    def view_action(action, label):
        qa.wait_presented()
        qa.focus(main)
        # Existing StartupQA.menu_item View menubar coordinate, same native size.
        assert state()['value']['window_size'] == [1344, 806] and state()['value']['scale'] == 1
        qa.command('xdotool', 'mousemove', '--window', main, '195', '17', 'click', '1')
        rows = qa.wait('own public View menu in Overlay', lambda:
                       menu() if state()['value']['focus'] == 'Overlay' else False)
        target = menu_prefix + '/' + action
        chosen = next(row for row in rows if row['id'] == target)
        assert chosen['name'] == label and chosen['attributes'].get('dispatch-key') == action
        assert chosen['attributes'].get('available') == 'true'
        opened = qa.wait('complete focused Compound while View menu open', lambda: retain(None))
        # Traverse only the finite current public rows, observing standard focus
        # at each step. General menubar navigation has no assumed Home binding.
        for _ in rows:
            current = menu()
            if current and any(row['id'] == target and row['focused'] for row in current):
                break
            qa.key(main, 'Down')
        else:
            raise RuntimeError('exact public View action did not receive focus: ' + target)
        focused = qa.wait('exact View action standardFOCUSED before Enter', lambda:
                         menu() if menu() and any(row['id'] == target and row['focused']
                                                 for row in menu()) else False)
        before_enter = qa.wait('complete focused Compound on View action', lambda: retain(None))
        qa.key(main, 'Return')
        qa.wait('View menu dismissed after action', lambda: not menu())
        return {'action': target, 'label': label, 'public_rows': rows,
                'public_focused_before_enter': focused,
                'selection_while_menu_open': opened, 'selection_before_enter': before_enter}

    result['focused_pane_before_close'] = qa.wait('focused five before closing resolving pane',
        lambda: retain(0, [(0, 'Board'), (1, 'Schematic')]))
    result['focused_pane_before_close_presented'] = qa.capture('focused-pane-before-close', main)
    result['focused_pane_close_action'] = view_action('view.close_pane', 'Close Pane')
    qa.wait_presented()
    result['focused_pane_without_board'] = qa.wait('same focused five with sole Schematic pane',
        lambda: retain(1, [(1, 'Schematic')]))
    result['focused_pane_without_board_presented'] = qa.capture('focused-pane-no-board', main)
    result['focused_pane_fill_action'] = view_action('view.fill_board', 'Fill Pane: Board')
    qa.wait_presented()
    result['focused_pane_board_rebound'] = qa.wait('same focused five rebound to surviving Board pane',
        lambda: retain(1, [(1, 'Board')]))
    result['focused_pane_board_rebound_presented'] = qa.capture('focused-pane-board-rebound', main)
    result['focused_pane_projection_limit'] = (
        'Live-pane absence is established by current pane/layout records and actual frames, '
        'not retained board_camera/scene_bounds or a fabricated unavailable count. '
        'Model source remains available; global unprojected can remain zero. '
        'New pane1 projection/camera/LOD/capacity may differ legitimately. '
        'No active-gesture-close, whole-view/camera equality or ordinary window-Close claim.')
