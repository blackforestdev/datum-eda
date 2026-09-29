"""Focused output diagnosis; no GPU causal or performance acceptance."""
import copy
import time
from gpu_r3_pointer_input import pointer_input


def ensure_alive(process, report, stderr_path):
    code = process.poll()
    if code is not None:
        report['exit_code'] = code
        lines = stderr_path.read_text(errors='replace').splitlines() if stderr_path.exists() else []
        reason = next((line for line in lines if 'datum-gui error:' in line), 'no native fatal message')
        raise RuntimeError(f'native process exited {code}: {reason}')


def wait_until(deadline, process, report, stderr_path):
    while True:
        ensure_alive(process, report, stderr_path)
        remaining = (deadline-time.monotonic_ns())/1e9
        if remaining <= 0:
            return
        time.sleep(min(remaining, .05))


def semantic(state):
    return {k:v for k,v in state.items() if k not in ('render_revision','render_activity')}


def analyze(snapshot, schedule, report, declaration, receipt):
    assert snapshot['schema'] == 'datum.input-receipt/v1'
    assert snapshot['mode'] == 'output-diagnostic' and snapshot['complete'] is False
    assert snapshot['pid'] == report['pid']
    assert snapshot['coverage_complete'] and not snapshot['overflow'] and not snapshot['first_error']
    assert snapshot['pending'] is None and snapshot['gpu_drained'] is None
    records = snapshot['records']
    assert len(records) == snapshot['record_count'] <= snapshot['record_limit'] == 4096
    assert [r['sequence'] for r in records] == list(range(len(records)))
    assert records, 'no diagnostic records'
    origin = snapshot['monotonic_origin_ns']
    start, end = report['final_capture_started_ns'], report['final_capture_finished_ns']
    assert start <= end <= origin+snapshot['snapshot_ns'], 'capture outside diagnostic coverage'
    previous = None
    capture_state = None
    transitions = []
    for record in records:
        before, after = record['before'], record['after']
        received, completed = origin+record['received_ns'], origin+record['completed_ns']
        assert received <= completed and (previous is None or received >= previous)
        previous = completed
        assert after is not None and not before['truncated'] and not after['truncated']
        assert record['workload'] == [0,0,0], 'causal tracing unexpectedly enabled'
        if semantic(before) != semantic(after):
            transitions.append(dict(kind=record['demand_kind'],start_ns=received,end_ns=completed,
                                    before=before,after=after))
            assert not (received <= end and completed >= start), 'semantic transition overlaps capture'
        if completed <= start:
            capture_state = after
        elif received > end and capture_state is None:
            capture_state = before
    assert capture_state is not None, 'state at capture unavailable'
    # Reuse exact producer/applied-pointer oracle, excluding its post-close final
    # state assertion. Capture state is independently checked below, never fabricated.
    checked = copy.deepcopy(snapshot)
    checked.update({key:receipt[key] for key in declaration['expected_context']})
    input_result = pointer_input(schedule,checked,report['tail_end']['monotonic_ns'],
                                 declaration['expected_context'],declaration['max_producer_lateness_ns'],
                                 diagnostic=True)
    active = [r for r in records if r['position'] is not None
              and schedule['started_ns'] <= origin+r['received_ns'] <= report['tail_end']['monotonic_ns']]
    expected = active[-1]['after']
    correct_state = semantic(capture_state) == semantic(expected)
    return dict(input=input_result, capture_state=capture_state,
                expected_last_pointer_state=expected, capture_state_matches=correct_state,
                transitions_after_last_pointer=[r for r in transitions if r['start_ns'] >= origin+active[-1]['completed_ns']],
                limits='Native output diagnosis only; no GPU performance, original failure resolution or DRM qualification.')
