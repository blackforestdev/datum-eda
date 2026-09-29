"""R3 input validation: preserve full demand records, inspect actual pointer events."""
import math

def pointer_input(schedule, receipt, tail_end_ns, expected_context, max_lateness_ns):
    """Require delivered path and completed viewport routing; no frame-count proxy."""
    assert receipt['complete'] and not receipt['overflow']
    rows = schedule['rows']
    assert len(rows) == schedule['scheduled_count'] == 3600
    assert [r['index'] for r in rows] == list(range(3600))
    assert all(r['physical_position'] == [round(v) for v in r['logical_position']] for r in rows)
    assert all(abs(r['scheduled_ns'] - schedule['started_ns'] - i*1e9/120) < 2 for i,r in enumerate(rows))
    for i, row in enumerate(rows):
        edge, fraction = i//900, (i%900)/900
        planned = [(300+400*fraction,150),(700,150+240*fraction),
                   (700-400*fraction,390),(300,390-240*fraction)][edge]
        assert row['logical_position'] == list(planned), 'changed workload path'
    assert max_lateness_ns == 8_333_334, 'undeclared cadence tolerance'
    lags = [r['sent_ns']-r['scheduled_ns'] for r in rows]
    assert all(0 <= lag <= max_lateness_ns for lag in lags), 'producer missed one-period deadline bound'
    assert all(a['sent_ns'] <= b['sent_ns'] for a,b in zip(rows,rows[1:])), 'producer clock/order error'
    duration = schedule['finished_ns']-schedule['started_ns']
    assert 30_000_000_000 <= duration <= 30_000_000_000+max_lateness_ns, 'producer duration outside bound'
    expected, previous = [], rows[0]['physical_position']
    assert previous == [300,150]
    for row in rows:
        if row['physical_position'] != previous:
            expected.append(row['physical_position'])
            previous = row['physical_position']
    origin = receipt['monotonic_origin_ns']
    all_records = receipt['records']
    assert [r['sequence'] for r in all_records] == list(range(len(all_records)))
    assert not any(r['button'] is not None for r in all_records), 'unexpected button input'
    records = [r for r in all_records if r['position'] is not None]
    before = [r for r in records if origin+r['received_ns'] < schedule['started_ns']]
    assert before and before[-1]['position'] == [300,150], 'wrong starting pointer'
    active = [r for r in records if schedule['started_ns'] <= origin+r['received_ns'] <= tail_end_ns]
    assert [r['position'] for r in active] == expected, 'missing/reordered/unexpected input'
    assert len(expected) == 1280
    epoch = before[-1]['after']['device_epoch']
    camera = before[-1]['after']['camera_center_nm'], before[-1]['after']['camera_zoom']
    for r in active:
        assert r['button'] is None and r['route'] == 'authoring_hover', 'unexpected capture/routing'
        assert r['completed_ns'] is not None and r['completed_ns'] >= r['received_ns']
        after = r['after']
        assert after['cursor'] == after['native_cursor'] == r['position'], 'cursor not semantically applied'
        assert not after['pan_active'] and not after['truncated']
        assert (after['camera_center_nm'],after['camera_zoom']) == camera
        assert after['device_epoch'] == epoch, 'device changed within stream'
    # Rendering revisions can advance for close/drain; cursor, camera, hover and
    # device identity must still equal the final applied pointer state.
    semantic = lambda state: {k:v for k,v in state.items() if k != 'render_revision'}
    assert semantic(receipt['final_state']) == semantic(active[-1]['after']), 'final input state changed after last acknowledged motion'
    assert receipt['final_state']['cursor'] == expected[-1] == [300,150]
    assert set(expected_context) == {'final_selection','final_focus','final_focused_pane'}
    assert all(receipt[key] == value for key,value in expected_context.items()), 'unexpected final selection/focus/pane'
    return {'scheduled':3600,'integer_changes':1280,'same_position_requests':2320,
            'received':len(active),'completed_viewport_routes':len(active),
            'producer_duration_ns':duration,'send_lateness_ns':{'p95':sorted(lags)[math.ceil(.95*len(lags))-1],
            'p99':sorted(lags)[math.ceil(.99*len(lags))-1],'max':max(lags)},
            'limits':'Completed routing is not display acknowledgement or CPU/action acceptance.'}

