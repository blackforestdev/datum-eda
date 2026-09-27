"""Validate frame-observer delivery; this does not certify native admission."""
import collections


def integer(value):
    assert type(value) is int and value >= 0, ('invalid nonnegative integer', value)
    return value


def span(value):
    assert len(value) == 2
    start, end = map(integer, value)
    assert start <= end
    return end - start


def geometry(ranges):
    vertices = strokes = triangles = 0
    for entry in ranges:
        n = span([entry['start'], entry['end']])
        if entry['kind'] == 'vertices':
            vertices += n
            triangles += n // 3
        else:
            assert entry['kind'] == 'stroke_instances'
            strokes += n
            triangles += n * 2
    return vertices, strokes, triangles


def check(rows, expected_origins=None):
    end = rows[-1]
    assert end['phase'] == 'end'
    frames = [row for row in rows if row['phase'] == 'frame']
    if 'frames' not in end:
        assert not frames, 'frame delivery lacks final count'
        return {'available': False, 'frames': 0, 'qualification_pass': False,
                'limits': 'Historical trace has no per-attempt admission records.'}
    assert not end['frame_delivery_failed'], 'frame delivery reported failure'
    assert integer(end['frames']) == len(frames), 'frame count mismatch'
    gaps = collections.Counter()
    hosts = collections.Counter()
    submitted = 0
    last_time = 0
    for sequence, frame in enumerate(frames, 1):
        assert frame['sequence'] == sequence, 'frame sequence gap'
        timestamp = integer(frame['monotonic_ns'])
        assert timestamp >= last_time, 'frame time reversed'
        last_time = timestamp
        renderer = integer(frame['renderer_id'])
        if expected_origins is not None:
            assert renderer in expected_origins, 'unattributed frame renderer'
        hosts[renderer] += 1
        assert len(frame['extent']) == 2 and all(integer(v) > 0 for v in frame['extent'])
        outcome = frame['submitted_frame']
        assert outcome is None or type(outcome) is bool
        assert (outcome is None) == (frame['render_error'] is not None)
        if frame['render_error'] is not None:
            assert outcome is None
            gaps['render_errors'] += 1
        submitted += outcome is True
        prepared = {g['group']: g for g in frame['prepared_screen_geometry']}
        assert len(prepared) == len(frame['prepared_screen_geometry']) == 8
        for group in prepared.values():
            assert integer(group['payload_bytes']) == integer(group['prepared_vertices']) * 20
        screen = frame['submitted_screen_geometry']
        terminal = frame['submitted_terminal_geometry']
        world = frame['submitted_world_bundles']
        if outcome is not True:
            assert screen is None and terminal is None and world is None
        else:
            assert isinstance(screen, list) and isinstance(terminal, list)
            assert len({g['group'] for g in screen}) == len(screen)
            for group in screen:
                n = span(group['range'])
                assert group['range'][0] == 0
                assert group['submitted_commands'] == 1
                assert integer(group['submitted_vertices']) == n == prepared[group['group']]['prepared_vertices']
                assert integer(group['submitted_triangles']) == n // 3
                assert len(group['scissor']) == 4
                assert all(integer(v) >= 0 for v in group['scissor'])
            for draw in terminal:
                assert integer(draw['submitted_vertices']) == 6
                assert integer(draw['submitted_triangles']) == 2
                assert integer(draw['payload_bytes']) == 96
                assert len(draw['scissor']) == 4 and all(integer(v) >= 0 for v in draw['scissor'])
        panes = {p['pane_id']: p for p in frame['world_panes']}
        assert len(panes) == len(frame['world_panes'])
        for pane in panes.values():
            if pane['counts'] is None:
                assert pane['error'], 'missing world counts without reason'
                gaps['missing_world_counts'] += 1
                continue
            assert pane['error'] is None
            counts = pane['counts']
            v, s, t = geometry(pane['ranges'])
            assert integer(counts['prepared_commands']) == len(pane['ranges'])
            assert (counts['prepared_vertices'], counts['prepared_stroke_instances'], counts['prepared_triangles']) == (v, s, t)
            for entry in pane['ranges']:
                capacity = counts['retained_vertices' if entry['kind'] == 'vertices' else 'retained_stroke_instances']
                assert entry['end'] <= integer(capacity)
            if not pane['source'] or not pane['source'].get('scene_id') or not pane['source'].get('source_revision'):
                gaps['missing_world_source_identity'] += 1
        if world is not None:
            assert outcome is True
            assert len({p['pane_id'] for p in world}) == len(world)
            for pane in world:
                assert pane['pane_id'] in panes
                assert integer(pane['submitted_commands']) == len(pane['ranges'])
                v, s, t = geometry(pane['ranges'])
                assert (pane['submitted_vertices'], pane['submitted_stroke_instances'], pane['submitted_triangles']) == (v, s, t)
        elif outcome is True and panes:
            gaps['missing_world_submission_ranges'] += 1
        for grid in frame['prepared_surface_grids'] or []:
            n = span(grid['range'])
            assert grid['range'][1] <= integer(grid['generated_vertices'])
            assert integer(grid['prepared_vertices']) == n
            assert grid['pane_id'] in panes
            assert type(grid['encoded']) is bool
            if outcome is True:
                assert grid['submitted_commands'] == int(grid['encoded'])
                assert grid['submitted_vertices'] == (n if grid['encoded'] else 0)
                assert grid['submitted_triangles'] == (n // 3 if grid['encoded'] else 0)
            else:
                assert all(grid[k] is None for k in ('submitted_commands', 'submitted_vertices', 'submitted_triangles'))
        text, origins = frame['text_admission'], frame['text_origins']
        if text is None:
            assert origins is None
            if outcome is True:
                gaps['missing_submitted_text_observation'] += 1
        else:
            assert frame['text_observation_attempted'] and not frame['text_admission_failed']
            assert isinstance(origins, list)
            identities = set()
            for origin in origins:
                identity = origin['origin']
                kind = identity['kind']
                assert kind in ('host', 'viewport', 'terminal')
                key = (origin['overlay'], kind, identity.get('pane_id'), identity.get('leaf_index'))
                assert key not in identities
                identities.add(key)
                if kind == 'terminal' and not identity['session_id']:
                    gaps['missing_terminal_session_identity'] += 1
                assert integer(origin['unique_raster_keys']) <= integer(origin['shaped_instances'])
            for overlay, label in ((False, 'workspace'), (True, 'overlay')):
                group = text[label]
                for field in ('runs', 'layout_rows', 'shaped_instances'):
                    assert sum(integer(o[field]) for o in origins if o['overlay'] == overlay) == integer(group[field])
                assert integer(group['unique_raster_keys']) <= integer(group['shaped_instances'])
            unique = integer(text['union_unique_raster_keys'])
            a, b = text['workspace']['unique_raster_keys'], text['overlay']['unique_raster_keys']
            assert max(a, b) <= unique <= a + b
    return {'available': True, 'frames': len(frames), 'submitted_frames': submitted,
            'frames_by_renderer': dict(hosts), 'admission_gaps': dict(gaps), 'qualification_pass': False,
            'limits': 'Record consistency only. Missing-count gaps remain failures of admission; no native input, fixture/configuration coverage, raster/presentation, overhead, resource-cap or full S5 acceptance.'}
