"""Classify requested block lifetimes inside observed private calls, not whole-process memory."""
import argparse
import hashlib
import json
from pathlib import Path


def require(condition, message):
    if not condition:
        raise ValueError(message)


def natural(value):
    require(type(value) is int and value >= 0, 'invalid unsigned counter')
    return value


def classify_call(rows):
    begin, end = rows[0], rows[-1]
    initial = natural(begin['scope_live_bytes'])
    remaining_input = initial
    live = initial
    peak = initial
    addresses = {}
    freed = set()
    blocks = []
    transitions = []
    for row in rows[1:-1]:
        allocation = row['allocation']
        address = natural(allocation['address'])
        size = natural(allocation['bytes'])
        require(address > 0 and size > 0, 'invalid block')
        require(type(allocation['allocated']) is bool, 'invalid allocation direction')
        if allocation['allocated']:
            require(address not in addresses, 'duplicate live address')
            block = {'birth': row['sequence'], 'bytes': size, 'freed': False}
            blocks.append(block)
            addresses[address] = block
            freed.discard(address)
            live += size
            transitions.append((row, block, size))
        else:
            block = addresses.pop(address, None)
            if block is None:
                require(address not in freed, 'duplicate input free')
                remaining_input -= size
                require(remaining_input >= 0, 'input frees exceed initial ownership')
            else:
                require(block['bytes'] == size, 'free size mismatch')
                block['freed'] = True
            freed.add(address)
            live -= size
            transitions.append((row, block, -size))
        require(live >= 0 and live == natural(row['scope_live_bytes']), 'transition live total mismatch')
        peak = max(peak, live)
    require(live == natural(end['scope_live_bytes']), 'final scope total mismatch')
    excluded = natural(begin['excluded_bytes'])
    report = end['report']
    require(natural(report['initial_bytes']) == max(0, initial - excluded), 'initial guard mismatch')
    require(natural(report['final_bytes']) == max(0, live - excluded), 'final guard mismatch')
    adjusted_peak = max(0, peak - excluded)
    require(natural(report['peak_bytes']) == adjusted_peak, 'guard peak mismatch')
    for kind in ('host', 'process'):
        require(natural(report[kind + '_peak_bytes']) >= adjusted_peak, 'admission peak below reconstructed scope')
    current = {'input': initial, 'output': 0, 'temporary': 0}
    maxima = dict(current)
    timeline = [(begin['sequence'], dict(current))]
    for row, block, delta in transitions:
        category = 'input' if block is None else ('temporary' if block['freed'] else 'output')
        current[category] += delta
        require(all(value >= 0 for value in current.values()), 'negative classified bytes')
        require(sum(current.values()) == row['scope_live_bytes'], 'classification mismatch')
        maxima = {key: max(maxima[key], value) for key, value in current.items()}
        timeline.append((row['sequence'], dict(current)))
    require(current['temporary'] == 0, 'temporary blocks remain live')
    require(current['input'] == remaining_input, 'remaining input mismatch')
    require(sum(block['bytes'] for block in addresses.values()) == current['output'], 'output mismatch')
    return {'call_id': report['call_id'], 'owner_id': report['owner_id'],
            'renderer_origin': begin['renderer_origin'], 'begin_sequence': begin['sequence'],
            'end_sequence': end['sequence'], 'initial_input_bytes': initial,
            'final_bytes_by_lifetime': current, 'peak_bytes_by_lifetime': maxima,
            'simultaneous_scope_peak_bytes': peak, 'excluded_bytes': excluded,
            'guard_exceeded': report['exceeded'], 'new_block_lifetimes': len(blocks)}, timeline


def analyze(rows):
    require(len(rows) >= 3 and rows[0]['phase'] == 'start' and rows[-1]['phase'] == 'end', 'missing envelope')
    require(rows[0]['allocation_events'] is True, 'allocation mode absent')
    require(rows[-1]['complete_delivery'] is True and rows[-1]['event_loop_ok'] is True, 'incomplete delivery')
    identity = rows[0]['observation_id']
    active, owners, completed = {}, set(), []
    known_addresses = {}
    sequence = elapsed = last_call = batches = 0
    immutable = ('renderer_origin', 'owner_label', 'excluded_bytes', 'credited_bytes', 'host_limit_bytes', 'process_limit_bytes')
    for row in rows[1:-1]:
        require(row['observation_id'] == identity, 'observation identity mismatch')
        if row['phase'] == 'batch':
            require(row['dropped_events'] == 0 and row['first_call_id'] == 1 and row['incomplete'] is False, 'lost events or startup coverage')
            require(row['total_events'] == sequence and row['active_calls'] == len(active), 'batch mismatch')
            batches += 1
            continue
        require(row['phase'] == 'call', 'unknown record')
        sequence += 1
        require(row['sequence'] == sequence, 'event sequence gap')
        stamp = natural(row['elapsed_ns'])
        require(stamp >= elapsed, 'time reversed')
        elapsed = stamp
        report = row['report']
        require(report['allocator_installed'] is True, 'allocator unavailable')
        call, owner = natural(report['call_id']), natural(report['owner_id'])
        require(call > 0 and owner > 0, 'invalid identity')
        phase = row['transition']
        if phase == 'Begin':
            require(call == last_call + 1 and owner not in owners, 'missing call or overlapping owner')
            require(row['allocation'] is None, 'allocation on begin')
            last_call = call
            active[call] = [row]
            owners.add(owner)
            continue
        require(call in active, 'event outside active call')
        begin = active[call][0]
        require(owner == begin['report']['owner_id'], 'owner changed')
        require(all(row[key] == begin[key] for key in immutable if key in row or phase != 'Allocation'), 'call attribution changed')
        active[call].append(row)
        if phase == 'Allocation':
            require(row['allocation'] is not None, 'missing block transition')
            allocation = row['allocation']
            address = natural(allocation['address'])
            if allocation['allocated']:
                require(address not in known_addresses, 'duplicate concurrent live address')
                known_addresses[address] = call
            elif address in known_addresses:
                require(known_addresses.pop(address) == call, 'cross-owner block release')
        else:
            require(phase == 'Finished' and row['allocation'] is None, 'abandoned or unknown transition')
            require(type(report['exceeded']) is bool, 'invalid overrun status')
            for kind in ('host', 'process'):
                high = natural(report[kind + '_peak_bytes'])
                require(high >= max(natural(report[kind + '_initial_bytes']), natural(report[kind + '_final_bytes'])), 'invalid admission peak')
                if high > natural(row[kind + '_limit_bytes']):
                    require(report['exceeded'], 'unreported cap violation')
            completed.append(active.pop(call))
            owners.remove(owner)
            # Post-call frees are unobserved; do not extend address identity past it.
            known_addresses = {address: c for address, c in known_addresses.items() if c != call}
    require(rows[-1]['observation_id'] == identity and not active and batches > 0, 'unclosed trace')
    require(rows[-2]['phase'] == 'batch' and rows[-2]['total_events'] == sequence, 'missing final batch')
    calls, sweep = [], []
    for call in completed:
        result, timeline = classify_call(call)
        calls.append(result)
        for seq, values in timeline:
            sweep.append((seq, result['call_id'], values))
        sweep.append((result['end_sequence'], result['call_id'], None))
    active_values = {}
    active_peaks = dict(input=0, output=0, temporary=0, total=0)
    for _, call, values in sorted(sweep):
        if values is None:
            del active_values[call]
        else:
            active_values[call] = values
        totals = {key: sum(v[key] for v in active_values.values()) for key in ('input', 'output', 'temporary')}
        totals['total'] = sum(totals.values())
        active_peaks = {key: max(active_peaks[key], value) for key, value in totals.items()}
    return {'valid_delivery_and_classification': True, 'qualification_pass': False,
            'events': sequence, 'calls': sorted(calls, key=lambda c: c['call_id']),
            'overrun_calls': [c['call_id'] for c in calls if c['guard_exceeded']],
            'simultaneous_active_call_scope_peaks': active_peaks,
            'limits': ['Input means live before call; output means allocated during and still live at call end; temporary means born and freed inside call. These are lifetime classes, not inferred semantic cache roles.',
                       'Category maxima need not coincide; never add them as a peak. Active-call aggregate excludes inactive owners and post-call retention; it is not whole-process memory.',
                       'Pre-call births and post-call frees are unobserved. Address identities are resolved only within a call. No ownership inference across unseen gaps.',
                       'Counts include requested payload and Datum header/alignment, not System slack/native mappings. Excluded budget bytes are not assigned to a category without further ownership evidence.',
                       'Complete native coverage, overhead, full process/host budget reconciliation and independent replay remain required.']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('trace', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    try:
        raw = args.trace.read_bytes()
        result = analyze([json.loads(line) for line in raw.splitlines()])
        result['source_sha256'] = hashlib.sha256(raw).hexdigest()
    except (ValueError, KeyError, TypeError, OSError) as error:
        result = {'valid_delivery_and_classification': False, 'qualification_pass': False, 'error': str(error)}
    with args.output.open('x') as stream:
        json.dump(result, stream, indent=2)
        stream.write('\n')
    return 0 if result['valid_delivery_and_classification'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
