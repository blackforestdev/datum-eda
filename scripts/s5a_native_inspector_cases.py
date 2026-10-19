"""Finite focused selection and Inspector cases over frozen native sources."""
from __future__ import annotations

import json
from pathlib import Path

from prove_native_project_startup_gui import Atspi, digest
from s5a_native_selection_observation import qualify, selection_snapshot


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
