"""Narrow r2 measurement conformance; no native runs or performance inference."""
import math
from pathlib import Path

ROOT = Path('/home/bfadmin/Documents/datum-eda')


def encoding_order(source, images):
    """Pin actual encoder calls, not just sample labels or elapsed-time gaps.

    This deliberately narrow source contract must be reviewed when the encoder
    is restructured. It supplements the existing real-GPU query/copy tests.
    """
    begin = source.index('let mut encoder = device.create_command_encoder(')
    end = source.index('let encode_elapsed =', begin)
    body = source[begin:end]
    sites = ['label: Some("datum-prefix-copy-start")', 'm.pass("copy-start")',
             'images.copy(&mut encoder)', 'label: Some("datum-gui-final-resolve")']
    assert all(body.count(site) == 1 for site in sites), 'missing/duplicate encoder boundary'
    assert [body.index(s) for s in sites] == sorted(body.index(s) for s in sites), 'copy omitted from timed span'
    assert 'if reuse {' in body[:body.index(sites[0])], 'marker must precede warm copy'
    start = images.index('pub fn copy(')
    stop = images.index('pub fn logical_copy_bytes(', start)
    actual = images[start:stop]
    assert actual.count('encoder.copy_texture_to_texture(') == 1
    assert 'self.prefix.image.texture.as_image_copy()' in actual
    assert 'self.working.image.texture.as_image_copy()' in actual
    assert 'width: self.working.key.extent.0' in actual
    assert 'height: self.working.key.extent.1' in actual


def gpu_sample(sample, role, expected_epoch):
    assert sample['device_epoch'] == expected_epoch, 'foreign device epoch'
    names = [p[0] for p in sample['passes_ns']]
    allowed = [['frame']] if role == 'baseline' else [['copy-start', 'suffix'], ['frame', 'suffix'], ['frame']]
    assert names in allowed, 'missing full-frame/copy boundary'
    ticks = sample['raw_ticks']
    assert len(ticks) == len(names) * 2 and len(ticks) > 0
    period = sample['timestamp_period_ns']
    assert math.isfinite(period) and period > 0
    assert all(isinstance(t, int) and 0 <= t < 2**64 for t in ticks)
    assert ticks == sorted(ticks), 'reversed/wrapped ticks'
    durations = [(ticks[i+1]-ticks[i])*period for i in range(0, len(ticks), 2)]
    assert durations == [p[1] for p in sample['passes_ns']]
    assert sum(durations) == sample['own_pass_sum_ns']
    span = (ticks[-1]-ticks[0])*period
    assert 0 <= span < 2e9 and span == sample['frame_span_ns']
    return span


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
    records = receipt['records']
    assert [r['sequence'] for r in records] == list(range(len(records)))
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
    assert receipt['final_state'] == active[-1]['after'], 'final state changed after last acknowledged motion'
    assert receipt['final_state']['cursor'] == expected[-1] == [300,150]
    assert set(expected_context) == {'final_selection','final_focus','final_focused_pane'}
    assert all(receipt[key] == value for key,value in expected_context.items()), 'unexpected final selection/focus/pane'
    return {'scheduled':3600,'integer_changes':1280,'same_position_requests':2320,
            'received':len(active),'completed_viewport_routes':len(active),
            'producer_duration_ns':duration,'send_lateness_ns':{'p95':sorted(lags)[math.ceil(.95*len(lags))-1],
            'p99':sorted(lags)[math.ceil(.99*len(lags))-1],'max':max(lags)},
            'limits':'Completed routing is not display acknowledgement or CPU/action acceptance.'}


def rejected(call):
    try:
        call()
    except (AssertionError, ValueError, KeyError):
        return
    raise AssertionError('negative control was accepted')


def controls():
    source = (ROOT/'crates/gui-render/src/render/gpu_frame.rs').read_text()
    images = (ROOT/'crates/gui-render/src/render/gpu_prefix_images.rs').read_text()
    encoding_order(source, images)
    # Move the actual marker pass block after the copy; retain pass names.
    start = source.index('            if reuse {')
    split = source.index('            } else {', start)
    marker = source[start:split] + '            }\n'
    moved = source[:start] + '            if !reuse {' + source[split+len('            } else {'):]
    copy = '            images.copy(&mut encoder);'
    at = moved.index(copy) + len(copy)
    moved = moved[:at] + '\n' + marker + moved[at:]
    rejected(lambda: encoding_order(moved, images))
    rejected(lambda: encoding_order(source.replace('m.pass("copy-start")', 'm.pass("suffix")'), images))
    sample = {'device_epoch':7, 'raw_ticks':[10,12,30,50], 'timestamp_period_ns':2.5,
              'passes_ns':[['copy-start',5.0],['suffix',50.0]],
              'own_pass_sum_ns':55.0,'frame_span_ns':100.0}
    assert gpu_sample(sample,'candidate',7) == 100.0
    suffix = dict(sample,raw_ticks=[30,50],passes_ns=[['suffix',50.0]],own_pass_sum_ns=50.0,frame_span_ns=50.0)
    rejected(lambda: gpu_sample(suffix,'candidate',7))
    rejected(lambda: gpu_sample(sample,'candidate',8))
    rejected(lambda: gpu_sample(dict(sample,raw_ticks=[50,12,30,10]),'candidate',7))
    rejected(lambda: gpu_sample(dict(sample,frame_span_ns=55.0),'candidate',7))
    print('PASS: actual marker/copy/resolve source order; moved-marker, omitted-marker, suffix-only, epoch, reversed and pass-sum-substitution negatives rejected. No native run.')


if __name__ == '__main__':
    controls()
