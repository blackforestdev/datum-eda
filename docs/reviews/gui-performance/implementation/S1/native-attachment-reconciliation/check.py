"""Reconcile the recorded native owner traces; does not run or qualify the GUI."""
import collections
import gzip
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
INPUT = ROOT / 'docs/reviews/gui-performance/implementation/S1/native-window-lifetime'


def check(path):
    ledger = []
    allocations = {}
    current = {}
    presented = {}
    warm = collections.Counter()
    for line in gzip.decompress(path.read_bytes()).decode().splitlines():
        if 'native surface attachment ' in line:
            r = json.loads(line.split('native surface attachment ', 1)[1])
            assert r['format'] in ('Bgra8UnormSrgb', 'Bgra8Unorm', 'Rgba8UnormSrgb', 'Rgba8Unorm')
            assert r['payload_bytes'] == r['extent'][0] * r['extent'][1] * 4 * r['samples']
            key = (r['queue_epoch'], r['owner'], r['allocation'])
            identity = (r['host'], tuple(r['extent']), r['format'], r['samples'], r['payload_bytes'])
            assert key not in allocations or allocations[key] == identity
            allocations[key] = identity
            current[r['window']] = r
        if 'native_surface_lifecycle ' in line:
            r = json.loads(line.split('native_surface_lifecycle ', 1)[1])
            if r['reason'] == 'present':
                a = current[r['window']]
                assert a['extent'] == r['configured_extent']
                previous = presented.get(r['window'])
                if previous and previous['extent'] == a['extent']:
                    assert (previous['owner'], previous['allocation']) == (a['owner'], a['allocation'])
                    warm[str(r['host'])] += 1
                presented[r['window']] = a
        if 'native attachment ledger ' in line:
            outer = json.loads(line.split('native attachment ledger ', 1)[1])
            r = outer['attachments']
            assert r['unknown_payload_allocations'] == 0
            assert r['observed_allocations'] == r['completed_retirements'] + r['device_loss_retirements'] + len(r['allocations'])
            counts = collections.Counter()
            payload = collections.Counter()
            for a in r['allocations']:
                state = 'retiring' if a['release_reason'] else 'current'
                counts[a['host'], state] += 1
                assert counts[a['host'], state] <= 1
                payload[state] += a['payload_bytes']
            assert payload['current'] == r['current_payload_bytes']
            assert payload['retiring'] == r['retiring_payload_bytes']
            assert sum(payload.values()) <= r['peak_payload_bytes']
            ledger.append(r)
    assert allocations and ledger and presented
    last = ledger[-1]
    assert not last['allocations']
    assert last['current_payload_bytes'] == last['retiring_payload_bytes'] == 0
    assert last['observed_allocations'] == last['completed_retirements'] == len(allocations)
    assert last['device_loss_retirements'] == 0
    return {
        'trace': str(path.relative_to(ROOT)),
        'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
        'ledger_snapshots': len(ledger),
        'allocations_observed_and_retired': len(allocations),
        'peak_referenced_payload_bytes': last['peak_payload_bytes'],
        'same_extent_successive_presentations_by_queue_host': dict(warm),
        'observed_hosts': sorted({a[0] for a in allocations.values()}),
        'final_referenced_payload_bytes': 0,
    }


if __name__ == '__main__':
    results = [check(INPUT / backend / f'{host}-native.log.gz')
               for backend in ('candidate-wayland', 'candidate-x11')
               for host in ('main', 'global', 'project', 'new')]
    print(json.dumps({'scope': 'Recorded application MSAA attachment references only', 'runs': results}, indent=2))
