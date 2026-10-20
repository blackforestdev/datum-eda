"""Finite V3 public-export byte invariance over the governed Board selection case."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess

from native_project_source_inventory import validate_frozen_sources
from prove_native_project_startup_gui import Atspi
from s5a_native_selection_observation import qualify, selection_snapshot


def _empty(qa, state):
    record = state()
    selected = record['value']['selection']
    roots = [n for n in qa.nodes() if n.get_accessible_id() ==
             '/org/a11y/atspi/accessible/selection']
    return record if selected['kind'] == 'none' and selected['member_count'] == 0 and not roots else False


def _observe(qa, state, expected, selected):
    if not selected:
        return _empty(qa, state)
    try:
        return qualify(state(), selection_snapshot(qa, Atspi.StateType.SELECTED), expected)
    except ValueError:
        return False


def _context(qa, project, expected, selected):
    env = dict(qa.env)
    for key in ['DATUM_DISCOVERY', 'DATUM_TERMINAL_CONTEXT']:
        env.pop(key, None)
    argv = [str(qa.binary_dir / 'datum-eda'), '--format', 'json', 'context', 'get',
            '--project-root', str(project)]
    query = subprocess.run(argv, cwd=qa.root, env=env, text=True,
                           stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, timeout=15)
    assert query.returncode == 0, 'public ContextGet failed'
    # Persist only this emitted public allowlist, never opaque runtime context.
    context = json.loads(query.stdout)['selection_context']
    if selected:
        details = context['details']
        assert context['kind'] == 'compound'
        assert details['project'] == expected['project'] and details['revision'] == expected['revision']
        assert details['total'] == 5 and details['stale'] is False
        assert details['focus'] is None and details['scope'] is None
        assert details['per_type'] == {'track': 4, 'board_outline': 1}
        assert details['truncated'] is False and details['notice'] is None
        assert sorted(details['members'], key=lambda m: (m['class'], m['id'])) == sorted(
            expected['members'], key=lambda m: (m['class'], m['id']))
    else:
        assert context['kind'] == 'none'
    return {'argv': argv, 'exit_code': query.returncode, 'selection_context': context,
            'privacy': 'Only public selection_context persisted; transient raw envelope and stderr discarded.'}


def _checkpoint(qa, result, stage):
    (qa.root / 'export-invariance-checkpoint.json').write_text(
        json.dumps({'stage': stage, 'export_invariance': result['export_invariance_proof']}, indent=2) + '\n')


def _stage(qa, main, state, project, expected, frozen, result, name, selected):
    proof = result['export_invariance_proof']
    observed = qa.wait('complete public selection for export ' + name,
                       lambda: _observe(qa, state, expected, selected))
    context = _context(qa, project, expected, selected)
    after_context = qa.wait('complete public selection after ContextGet ' + name,
                            lambda: _observe(qa, state, expected, selected))
    if selected:
        assert observed['public_snapshot'] == after_context['public_snapshot']
    frame = qa.capture('export-' + name, main)
    destination = qa.root / (name + '.gbr')
    assert not destination.exists() and not destination.is_relative_to(project)
    argv = [str(qa.binary_dir / 'datum-eda'), '--format', 'json', 'project',
            'export-gerber-copper-layer', str(project), '--layer', '1', '--out', str(destination)]
    query = subprocess.run(argv, cwd=qa.root, env=qa.env, text=True,
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30)
    (qa.root / (name + '-export.stdout.json')).write_text(query.stdout)
    (qa.root / (name + '-export.stderr.log')).write_text(query.stderr)
    metadata = {'argv': argv, 'exit_code': query.returncode, 'selection_before': observed,
                'context': context, 'presented_frame': frame, 'destination': str(destination)}
    proof['stages'][name] = metadata
    _checkpoint(qa, result, name + '-command-returned')
    assert query.returncode == 0, 'public export failed at ' + name
    report = json.loads(query.stdout)
    metadata['report'] = report
    assert report['action'] == 'export_gerber_copper_layer'
    assert report['production_classification'] == 'manual_debug_export'
    assert report['project_root'] == str(project) and report['gerber_path'] == str(destination)
    assert report['layer'] == 1 and report['track_count'] == 4
    for key in ['pad_count', 'zone_count', 'unfilled_zone_count', 'via_count']:
        assert report[key] == 0, key
    assert report['unfilled_zone_ids'] == []
    projection = report['production_projection']
    assert projection['projection_kind'] == 'gerber_copper_layer'
    assert projection['projection_contract'] == 'datum.production_projection.gerber_copper_layer.v1'
    assert projection['model_revision'] == expected['revision']
    assert destination.is_file() and not destination.is_symlink()
    data = destination.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    assert data and projection['byte_count'] == len(data)
    assert projection['sha256'] == 'sha256:' + digest
    metadata.update(byte_count=len(data), sha256=digest,
                    selection_after=qa.wait('selection preserved by export ' + name,
                                            lambda: _observe(qa, state, expected, selected)))
    if selected:
        assert observed['public_snapshot'] == metadata['selection_after']['public_snapshot']
    inventory = validate_frozen_sources(project, frozen)
    metadata['source_inventory_after'] = inventory
    assert inventory['canonical_files'] == result['canonical_before']
    if name != 'before':
        assert data == (qa.root / 'before.gbr').read_bytes(), 'unnormalized bytes differ at ' + name
    _checkpoint(qa, result, name + '-qualified')


def export_before_selection(qa, main, state, project, expected, frozen, result):
    board = json.loads((project / 'board/board.json').read_text())
    assert len(board['tracks']) == 4 and not board['outline']['vertices']
    assert all(t['layer'] == 1 and t.get('midpoint') is None for t in board['tracks'].values())
    assert not board['pads'] and not board['vias'] and not board['zones']
    assert expected['project'] == json.loads((project / 'project.json').read_text())['uuid']
    assert expected['revision'] == '554096a8a1ead41134ae5c1e61a7c6df1b5b20228b9d5f4cb4ea923306217e6f'
    assert len(expected['members']) == 5
    result['export_invariance_proof'] = {
        'expected': expected, 'stages': {}, 'classification': 'manual_debug_export',
        'limits': 'Copper layer 1 successful public export byte invariance only; no full CAM, all-format, schematic export, HUMAN, C04 or successor acceptance.'}
    qa.key(main, 'Escape')
    qa.wait('qualified initial None for V3', lambda: _empty(qa, state))
    _stage(qa, main, state, project, expected, frozen, result, 'before', False)


def export_selected_and_cleared(qa, main, state, project, expected, frozen, result):
    _stage(qa, main, state, project, expected, frozen, result, 'selected', True)
    prior = state()['sequence']
    qa.key(main, 'Escape')
    qa.wait('ordinary Escape clears before V3 export', lambda:
            _empty(qa, state) if state()['sequence'] > prior else False)
    _stage(qa, main, state, project, expected, frozen, result, 'cleared', False)
    proof = result['export_invariance_proof']
    proof['equal_unnormalized_bytes'] = ((qa.root / 'before.gbr').read_bytes()
                                         == (qa.root / 'selected.gbr').read_bytes()
                                         == (qa.root / 'cleared.gbr').read_bytes())
    assert proof['equal_unnormalized_bytes']
    _checkpoint(qa, result, 'all-three-successful-and-byte-identical')
