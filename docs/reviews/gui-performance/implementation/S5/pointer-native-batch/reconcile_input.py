"""Reconcile existing pointer schedules with native receipts; no new native run."""
import hashlib
import json
import re
import tarfile
from pathlib import Path

root = Path(__file__).resolve().parent
runs = []
for path in sorted(root.glob('*-gpu[01].tar.gz')):
    with tarfile.open(path) as archive:
        schedule = json.load(archive.extractfile('input.json'))
        result = json.load(archive.extractfile('result.json'))
        native = archive.extractfile('active-and-tail.log').read().decode()
        prefix = archive.extractfile('native.log').read()[
            :result['pointer_trial']['start_log_offset']].decode()
    rows = schedule['rows']
    assert len(rows) == schedule['scheduled_count'] == 3600
    assert [row['index'] for row in rows] == list(range(3600))
    assert all(row['physical_position'] == [round(v) for v in row['logical_position']]
               for row in rows)
    assert all(abs(row['scheduled_ns'] - schedule['started_ns'] - i * 1e9 / 120) < 2
               for i, row in enumerate(rows))
    assert all(rows[i]['sent_ns'] <= rows[i+1]['sent_ns'] for i in range(3599))
    previous = rows[0]['physical_position']
    assert previous == [300, 150]
    changed = []
    for row in rows:
        if row['physical_position'] != previous:
            changed.append(tuple(row['physical_position']))
            previous = row['physical_position']
    pattern = (rf'window event WindowId\({result["main_window"]}\) '
               r'cursor moved ([0-9.]+),([0-9.]+)')
    prior_positions = re.findall(pattern, prefix)
    assert prior_positions and tuple(map(float, prior_positions[-1])) == (300, 150)
    received = [tuple(map(float, pair)) for pair in re.findall(pattern, native)]
    assert changed == received and len(changed) == 1280
    assert received[-1] == (300, 150)
    assert result['pointer_trial']['final_focus'] == result['main_window']
    assert result['normal_exit_code'] == 0
    runs.append({'archive': path.name,
                 'sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
                 'scheduled_positions': len(rows), 'changed_integer_positions': len(changed),
                 'same_integer_position_requests': len(rows) - len(changed),
                 'received_positions_in_exact_order': len(received),
                 'maximum_coordinate_rounding_error_px': max(
                     abs(a-b) for row in rows
                     for a, b in zip(row['physical_position'], row['logical_position'])),
                 'maximum_send_lateness_ms': max(
                     (row['sent_ns']-row['scheduled_ns'])/1e6 for row in rows),
                 'exact_final_position_and_focus': True, 'normal_exit_code': 0})
assert len(runs) == 7
report = {'new_native_runs': 0, 'qualification_pass': False, 'runs': runs,
          'finding': 'All seven archived schedules contain3600positions at120Hz. '
                     '2320requests repeat the current rounded integer position; '
                     'all1280changed positions appear in native receipts in exact order.',
          'limits': 'Native cursor receipts do not establish semantic hit/crosshair '
                    'correctness, complete counter conformance or numerical budgets. '
                    'Integer rounding is explicit; no fractional native delivery claim. '
                    'CPU/action acceptance requires acknowledged semantic actions; '
                    'cursor receipts alone are not that denominator. '
                    'All prior CPU/GPU failures and other scope gaps remain.'}
(root/'input-reconciliation.json').write_text(json.dumps(report, indent=2)+'\n')
print('7 archived runs reconciled; 0 new native runs; no missing changed integer positions.')
