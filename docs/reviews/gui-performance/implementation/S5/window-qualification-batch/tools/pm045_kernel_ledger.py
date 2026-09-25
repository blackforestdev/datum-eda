"""Fail-closed reconciliation for the pinned kernel descriptor observer.

This proves captured descriptor/client continuity, not zero observer overhead or
performance acceptance. Earlier exits with live GPU ownership require a drain
receipt; process death alone is not completion of queued GPU work.
"""
import collections
import json
from pathlib import Path


def decode(row):
    fields = dict(line.split(':', 1) for line in bytes.fromhex(row['raw_hex']).decode().splitlines() if ':' in line)
    fields = {key: value.strip() for key, value in fields.items()}
    key = tuple(fields[k] for k in ('drm-driver', 'drm-pdev', 'drm-client-id'))
    counters = {}
    for name, value in fields.items():
        if name.startswith('drm-engine-'):
            count, unit = value.split()
            if unit != 'ns' or int(count) < 0:
                raise ValueError('unsupported engine counter')
            counters[name] = int(count)
    if not counters:
        raise ValueError('engine counters unavailable')
    return key, counters


def reconcile(path):
    rows = [json.loads(line) for line in Path(path).read_text().splitlines()]
    failures = []
    if not rows or rows[-1].get('kind') != 'observer_end':
        failures.append('observer did not finish')
    groups = collections.defaultdict(list)
    identities = {}
    for row in rows:
        if row['kind'] == 'drm' or row['kind'] == 'identity':
            identities[row['tid']] = (row['pid'], row['start'])
        if 'sequence' in row and row['sequence']:
            groups[row['sequence']].append(row)
    bindings = {}
    clients = {}
    operations = []
    for sequence, group in groups.items():
        result = next((r for r in group if r['kind'] == 'result'), None)
        if result:
            operations.append((result['monotonic_ns'], sequence, result, group))
        else:
            event = next((r for r in group if r['kind'] == 'lifecycle' and r['event'] == 4), None)
            if event:
                operations.append((event['monotonic_ns'], sequence, {'syscall': 59, 'result': 0, 'tid': event['tid'], 'exec_event': True}, group))
    receipts = []

    def observe(row):
        key, counters = decode(row)
        client = clients.setdefault(key, {'epoch': 1, 'first_ns': row['monotonic_ns'], 'last_ns': row['monotonic_ns'], 'engines_ns': counters, 'first_engines_ns': dict(counters), 'closed': False, 'births': 0, 'retirements': 0})
        old = client['engines_ns']
        if old.keys() != counters.keys() or any(value < old.get(k, 0) for k, value in counters.items()):
            failures.append({'counter_reset': key, 'sequence': row['sequence']})
        client['engines_ns'] = counters
        client['last_ns'] = row['monotonic_ns']
        return key, client

    def birth(row):
        key, client = observe(row)
        owner = (row['pid'], row['start'], row['fd'])
        if owner in bindings:
            failures.append({'duplicate_live_descriptor': owner, 'sequence': row['sequence']})
        if client['closed']:
            client['epoch'] += 1
            failures.append({'unbridged_client_reappearance': key, 'epoch': client['epoch']})
        bindings[owner] = key
        client['births'] += 1
        client['closed'] = False
        receipts.append({'phase': 'birth', 'key': key, 'owner': owner, 'ns': row['monotonic_ns'], 'sequence': row['sequence']})

    def retire(row):
        key, client = observe(row)
        owner = (row['pid'], row['start'], row['fd'])
        if bindings.pop(owner, None) != key:
            failures.append({'retirement_without_matching_birth': owner, 'key': key, 'sequence': row['sequence']})
        client['retirements'] += 1
        client['closed'] = key not in bindings.values()
        receipts.append({'phase': 'retire', 'key': key, 'owner': owner, 'ns': row['monotonic_ns'], 'sequence': row['sequence']})

    # Inherited descriptors are real new owners of existing shared clients.
    inherited = [r for r in rows if r['kind'] == 'drm' and r['phase'] == 'inherited']
    events = [(ns, 'operation', (sequence, result, group)) for ns, sequence, result, group in operations]
    events.extend((r['monotonic_ns'], 'inherit', r) for r in inherited)
    for ns, kind, payload in sorted(events, key=lambda e: e[0]):
        if kind == 'inherit':
            birth(payload)
            continue
        sequence, result, group = payload
        nr, value = result['syscall'], result['result']
        args = result.get('args', [])
        if len(args) != 3 and not result.get('exec_event'):
            failures.append({'missing_syscall_arguments': sequence});continue
        observed = [r for r in group if r['kind'] in ('drm', 'non_drm', 'lookup_missing', 'opaque_gpu')]
        before = [r for r in observed if r['phase'] in ('retire_before', 'replace_before', 'range_before', 'exec_before')]
        after = [r for r in observed if r['phase'] in ('birth_after', 'receive_after')]
        if value < 0:
            if nr == 3 and any(r['kind'] == 'drm' for r in before):
                failures.append({'ambiguous_failed_drm_close': sequence, 'result': value})
            continue
        if nr in (2, 85, 257, 437, 32, 304, 438) or (nr == 72 and args[1] in (0, 1030)) or (nr in (33, 292) and args[0] != args[1]):
            if not any(r['fd'] == value and r['phase'] == 'birth_after' and r['kind'] != 'lookup_missing' for r in after):
                failures.append({'missing_successful_fd_classification': sequence, 'fd': value})
        if nr in (3, 33, 292, 436, 59, 322):
            for row in before:
                identity = identities.get(row['tid'])
                owner = (*identity, row['fd']) if identity else None
                if row['kind'] == 'drm':
                    retire(row)
                elif owner in bindings:
                    failures.append({'lost_final_drm_reading': owner, 'sequence': sequence})
            if nr == 3 and args:
                identity = identities.get(result['tid'])
                owner = (*identity, args[0]) if identity else None
                if owner in bindings:
                    failures.append({'missing_close_receipt': owner, 'sequence': sequence})
        for row in after:
            if row['kind'] == 'drm':
                birth(row)
    if bindings:
        failures.append({'unclosed_descriptors_without_final_retirement': [list(k) for k in bindings]})
    supported_failures=list(failures)
    opaque=[r for r in rows if r['kind']=='opaque_gpu']
    if opaque:failures.append({'unsupported_gpu_observations':len(opaque),'devices':sorted({bytes.fromhex(r['path_hex']).decode() for r in opaque})})
    return {
        'supported_drm_continuity_complete':not supported_failures,
        'supported_drm_failures':supported_failures,
        'descriptor_continuity_complete': not failures,
        'failures': failures,
        'clients': [{'device': list(k[:2]), 'client_id': k[2], **v} for k, v in clients.items()],
        'receipts': receipts,
        'raw_record_count': len(rows),
        'supported_only_engine_delta_ns': {name:sum(v['engines_ns'][name]-v['first_engines_ns'][name] for v in clients.values()) for name in next(iter(clients.values()))['engines_ns']} if clients and not supported_failures else None,
        'engine_delta_ns': {name:sum(v['engines_ns'][name]-v['first_engines_ns'][name] for v in clients.values()) for name in next(iter(clients.values()))['engines_ns']} if clients and not failures else None,
        'scope': 'Captured descriptor continuity only; no timing, queue-drain, API coverage or numerical-budget admission implied.',
    }


if __name__ == '__main__':
    import sys
    print(json.dumps(reconcile(sys.argv[1]), indent=2))
