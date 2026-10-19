"""Finite ordinary native Escape ownership cases over frozen S5A inputs."""
from __future__ import annotations

import json
from pathlib import Path

from prove_native_project_startup_gui import Atspi, digest
from s5a_native_selection_observation import qualify, selection_snapshot


def owned_open_text_keys(qa, main, state, result):
    expected = result['pin_source_oracle']['expected']
    retain = lambda: qualify(state(), selection_snapshot(qa, Atspi.StateType.SELECTED), expected)
    result['main_before_owned_open'] = qa.wait_presented()
    qa.menu_item(main, 'File', 'Open Project')
    dialog = qa.wait('owned Open form appears', lambda: qa.window('Open Project — Datum'))
    result['owned_open_presented'] = qa.wait_presented()
    assert result['owned_open_presented']['window'] != result['main_before_owned_open']['window']
    qa.fill_control(dialog, 'Location', 'N17_ORIGINAL')
    def location():
        node = qa.find('Location')
        assert node and node.get_process_id() == qa.process.pid
        assert node.get_state_set().contains(Atspi.StateType.FOCUSED)
        return {'pid': node.get_process_id(), 'id': node.get_accessible_id(),
                'role': node.get_role_name(), 'standard_focused': True,
                'attributes': qa.attributes(node)}
    result['location_original'] = location()
    result['selected_before_text_ctrl_a'] = retain()
    qa.capture('owned-open-original', dialog)
    qa.key(dialog, 'ctrl+a')
    result['selected_after_text_ctrl_a'] = retain()
    qa.text(dialog, 'N17_REPLACED')
    qa.wait('exact public Location replacement', lambda:
            qa.attributes(qa.find('Location')).get('value') == 'N17_REPLACED')
    result['location_replaced'] = location()
    result['selected_after_text_replace'] = retain()
    qa.capture('owned-open-replaced', dialog)
    diagnostic = Path(qa.env['DATUM_GUI_LOG'])
    offset = len(diagnostic.read_text())
    qa.key(dialog, 'Escape')
    qa.wait('owned Open dialog retired', lambda: qa.window('Open Project — Datum') is None)
    result['main_after_owned_cancel'] = qa.wait_presented()
    assert result['main_after_owned_cancel']['window'] == result['main_before_owned_open']['window']
    events = [line for line in diagnostic.read_text()[offset:].splitlines()
              if 'window event' in line and 'keyboard physical=Code(Escape)' in line]
    assert any('state=Pressed' in e for e in events) and any('state=Released' in e for e in events), events
    result['owned_dialog_escape_key_cycle'] = events
    qa.wait('owned form cancel restores Editor', lambda: state()['value']['focus'] == 'Editor(PaneId(1))')
    result['selected_after_owned_cancel'] = retain()
    qa.capture('owned-open-cancel-preserved', main)
    prior = state()['sequence']
    qa.key(main, 'Escape')
    result['main_idle_escape_clear'] = qa.wait('distinct Main Escape clears', lambda:
        state() if state()['sequence'] > prior and state()['value']['selection']['kind'] == 'none' else False)
    qa.wait('public selection root removed after Main Escape', lambda: not any(
        n.get_accessible_id() == '/org/a11y/atspi/accessible/selection' for n in qa.nodes()))
    qa.capture('owned-open-main-idle-clear', main)



def region_escape_cancel(qa, main, state, project, result):
    expected = result['pin_source_oracle']['expected']
    sheet = project / 'schematic/main.json'
    symbols = json.loads(sheet.read_text())['symbols']
    pinless = [(uid, s) for uid, s in symbols.items() if s['reference'] == 'U0']
    assert len(pinless) == 1
    uid, symbol = pinless[0]
    assert symbol['pins'] == [] and symbol['position'] == {'x': 30000000, 'y': 0}
    fresh_expected = {**expected, 'kind': 'compound', 'scope': 'symbol',
                      'members': [{'class': 'symbol', 'id': uid}], 'focus': None,
                      'oracle': 'Frozen majority-v1 pinless U0 placement anchor; region begun without prior selection has no focus'}

    def camera():
        value = state()['value']
        assert value['focus'] == 'Editor(PaneId(1))'
        assert value['focused_pane'] == 1 and value['focused_content'] == 'Some(Schematic)'
        assert value['window_size'] == [1344, 806] and value['scale'] == 1
        assert next(p for p in value['pane_cameras'] if p['pane'] == 1)['camera'] is None
        return {k: value[k] for k in ['focus', 'focused_pane', 'focused_content',
                                      'window_size', 'scale', 'layout', 'pane_cameras']}

    basis = camera()

    def observe(wanted):
        record = state()
        snapshot = selection_snapshot(qa, Atspi.StateType.SELECTED)
        observation = qualify(record, snapshot, wanted)
        assert state()['sequence'] == record['sequence'] and camera() == basis
        return observation

    result['region_prior'] = observe(expected)
    result['region_prior_presented'] = qa.capture('region-prior-current-frame', main)
    result['region_camera_basis'] = basis
    plan_path = qa.root / 'current-frame-region-plan.json'
    plan = qa.wait('source-bound U0 region plan for current presented P frame', lambda:
                   json.loads(plan_path.read_text()) if plan_path.exists() else False, timeout=300)
    assert plan['capture_sha256'] == digest(qa.root / 'region-prior-current-frame.png')
    assert plan['source_sheet_sha256'] == digest(sheet) and plan['u0_id'] == uid
    assert plan['camera_basis'] == basis and plan['world_rect_nm'] == [[29000000, -1000000], [31000000, 1000000]]
    assert plan['screen_rect_px'] == [[970, 397], [984, 410]] and plan['derivation']
    result['region_plan'] = plan
    diagnostic = Path(qa.env['DATUM_GUI_LOG'])
    native = result['region_prior_presented']['window']
    pressed = {'shift': False, 'primary': False}

    def events(offset):
        return [line for line in diagnostic.read_text()[offset:].splitlines()
                if f'window event {native} ' in line]

    def wait_event(offset, marker):
        return qa.wait('native ' + marker, lambda: next(
            (line for line in events(offset) if marker in line), False))

    def assert_held(offset):
        lines = events(offset)
        assert not any('mouse input Left Released' in line or 'focused false' in line
                       or ('Code(ShiftLeft)' in line and 'state=Released' in line)
                       or 'modifiers changed ModifiersState(0x0)' in line for line in lines), lines
        assert camera() == basis
        return lines

    def begin_region():
        start, end = plan['screen_rect_px']
        offset = len(diagnostic.read_text())
        qa.focus(main)
        qa.command('xdotool', 'mousemove', '--window', main, *map(str, start))
        wait_event(offset, f'cursor moved {start[0]:.2f},{start[1]:.2f}')
        qa.command('xdotool', 'keydown', 'Shift_L')
        pressed['shift'] = True
        wait_event(offset, 'modifiers changed ModifiersState(SHIFT)')
        before = qa.wait_presented()['presented_damage']
        qa.command('xdotool', 'mousedown', '1')
        pressed['primary'] = True
        wait_event(offset, 'mouse input Left Pressed')
        qa.command('xdotool', 'mousemove', '--window', main, *map(str, end))
        wait_event(offset, f'cursor moved {end[0]:.2f},{end[1]:.2f}')
        qa.wait_presented(after_damage=before, require_stable=False)
        assert_held(offset)
        return offset

    def release_inputs():
        if pressed['primary']:
            qa.command('xdotool', 'mouseup', '1')
            pressed['primary'] = False
        if pressed['shift']:
            qa.command('xdotool', 'keyup', 'Shift_L')
            pressed['shift'] = False

    try:
        held = begin_region()
        result['region_active_selection'] = observe(expected)
        result['region_active_presented'] = qa.capture('region-active-shift-rectangle', main, require_stable=False)
        result['region_active_after_capture'] = qa.wait_presented(require_stable=False)
        assert result['region_active_presented']['window'] == native
        result['region_held_before_escape'] = assert_held(held)
        escape = len(diagnostic.read_text())
        # Shift and primary remain held: never clear modifiers or change focus.
        qa.command('xdotool', 'key', 'Escape')
        wait_event(escape, 'keyboard physical=Code(Escape) logical=Named(Escape) state=Released')
        result['region_escape_events'] = [line for line in events(escape) if 'physical=Code(Escape)' in line]
        assert len(result['region_escape_events']) == 2
        assert 'state=Pressed' in result['region_escape_events'][0]
        assert 'state=Released' in result['region_escape_events'][1]
        assert_held(held)
        qa.wait_presented()
        result['region_after_escape'] = observe(expected)
        qa.capture('region-cancel-before-primary-release', main)
        result['region_held_through_cancel_capture'] = assert_held(held)
        trailing = len(diagnostic.read_text())
        release_inputs()
        wait_event(trailing, 'mouse input Left Released')
        wait_event(trailing, 'modifiers changed ModifiersState(0x0)')
        qa.wait_presented()
        result['region_after_trailing_release'] = observe(expected)
        result['region_trailing_events'] = events(trailing)
        prior = state()['sequence']
        idle = len(diagnostic.read_text())
        qa.key(main, 'Escape')
        result['region_idle_clear'] = qa.wait('distinct idle Escape clears P', lambda:
            state() if state()['sequence'] > prior and state()['value']['selection']['kind'] == 'none' else False)
        qa.wait('complete public selection root removed', lambda: not any(
            n.get_accessible_id() == '/org/a11y/atspi/accessible/selection' for n in qa.nodes()))
        result['region_idle_escape_events'] = [line for line in events(idle) if 'physical=Code(Escape)' in line]
        qa.capture('region-idle-escape-clear', main)
        fresh = begin_region()
        release_inputs()
        wait_event(fresh, 'mouse input Left Released')
        qa.wait_presented()
        def fresh_complete():
            if state()['value']['selection']['kind'] != 'compound':
                return False
            try:
                return observe(fresh_expected)
            except ValueError as error:
                result['region_pending_public_read'] = str(error)
                return False
        result['region_fresh_acquisition'] = qa.wait('fresh U0 region complete public membership', fresh_complete)
        result['region_fresh_events'] = events(fresh)
        result['region_fresh_source_oracle'] = fresh_expected
        # Complete public fields use the same compiled class inspection as the
        # renderer. Values come from the frozen authored source, not diagnostics.
        field_oracle = {semantic: symbol[source] for semantic, source in [
            ('reference', 'reference'), ('value', 'value'),
            ('placement_position_nm', 'position'), ('placement_rotation', 'rotation')]}
        fields_path = '/org/a11y/atspi/accessible/selection/fields'
        record = state()
        fields = {}
        for node in qa.nodes():
            path = node.get_accessible_id()
            if path.startswith(fields_path + '/n'):
                key = bytes.fromhex(path[len(fields_path + '/n'):]).decode()
                attrs = qa.attributes(node)
                assert node.get_process_id() == qa.process.pid
                assert all(attrs.get(k) == fresh_expected[v] for k, v in [
                    ('project-id', 'project'), ('model-revision', 'revision'),
                    ('subject-kind', 'kind')])
                assert key not in fields
                fields[key] = {'attributes': attrs, 'name': node.get_name(),
                               'description': node.get_description()}
        for key, value in field_oracle.items():
            attrs = fields[key]['attributes']
            assert attrs['value-state'] == 'common'
            assert json.loads(attrs['canonical-value']) == value
        assert state()['sequence'] == record['sequence'] and camera() == basis
        result['region_fresh_inspector'] = {
            'source_sha256': digest(sheet), 'source_oracle': field_oracle,
            'fields': fields, 'expected_rendered_header': 'Symbol: U0',
            'membership': observe(fresh_expected)}
        qa.capture('region-fresh-u0-acquired', main)
    finally:
        if any(pressed.values()):
            result['region_input_failure_housekeeping'] = dict(pressed)
            release_inputs()  # Release only input this case pressed; no success credit.
