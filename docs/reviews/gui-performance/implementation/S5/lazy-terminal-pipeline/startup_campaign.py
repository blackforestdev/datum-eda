"""Six fixed affected window trials. No retry, build, or acceptance promotion."""
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

ROOT = Path(subprocess.check_output(['git', 'rev-parse', '--show-toplevel'], text=True).strip())
sys.path.insert(0, str(ROOT/'scripts'))
from run_cargo_guarded import acquire_lock, load_policy

DRIVER = Path(__file__).resolve().with_name('startup_windows.py')
OUT = Path(tempfile.mkdtemp(prefix='pm045-startup-batch-'))
print(OUT, flush=True)
order = [('baseline', 1), ('candidate', 1), ('candidate', 2), ('baseline', 2), ('baseline', 3), ('candidate', 3)]
record = {'status': 'predeclared; not complete', 'qualification_pass': False,
          'decision': 'Assess unchanged50ms warm-open and20ms warm-close CPU caps after consolidated startup fixes, with exact pixels/focus/destruction.',
          'order': order, 'warm_cycles_per_host_per_run': 30,
          'hosts': ['GLOBAL', 'PROJECT', 'NEW'], 'measurement_mode': 'diagnostics off',
          'scope': 'X11/Xwayland1x only; GUI+descendant cgroupCPU. No formal relative claim, full resource/endurance/independent replay or owner UX acceptance.',
          'stop': 'Retain numerical failures and finish fixed schedule. Stop on structural/fixture/input/output/cleanup failure; never replace failed attempts.',
          'driver_sha256': hashlib.sha256(DRIVER.read_bytes()).hexdigest(),
          'runs': []}
(OUT/'declaration.json').write_text(json.dumps(record, indent=2)+'\n')
def save():
    (OUT/'result.json').write_text(json.dumps(record, indent=2)+'\n')
def summarize(result):
    assert result.get('normal_exit_code') == 0
    assert not result.get('forced_cleanup') and not result.get('remaining_before_cleanup')
    assert not any(result.get(k) for k in ('error', 'cleanup_errors', 'persistence_errors'))
    assert len(result['cycles']) == 90
    samples = result['samples']
    hosts = {}
    for host in record['hosts']:
        rows = [r for r in result['cycles'] if r['host'] == host]
        assert len(rows) == 30 and all(r.get('completed') and r.get('focus_restored') for r in rows)
        metrics = {}
        for phase, limit in [('open', 50), ('close', 20)]:
            values = [(samples[r[phase+'_end_sample']]['group']['usage_usec']-samples[r[phase+'_begin_sample']]['group']['usage_usec'])/1000 for r in rows]
            assert all(v >= 0 for v in values)
            ordered = sorted(values)
            metrics[phase] = {'cpu_ms': values, 'mean_ms': sum(values)/len(values),
                'p95_ms': ordered[math.ceil(.95*len(values))-1], 'max_ms': max(values),
                'limit_ms': limit, 'exceeding_cycle_indices': [r['index'] for r,v in zip(rows,values) if v>limit]}
        hosts[host] = metrics
    return hosts
save()
# Hold the existing compilation lock throughout native measurements.
policy = load_policy()
with acquire_lock(policy.lock_path, policy.lock_timeout_seconds):
    for role, trial in order:
        assert hashlib.sha256(DRIVER.read_bytes()).hexdigest() == record['driver_sha256']
        env = os.environ.copy()
        env.update(PM045_STARTUP_ROLE=role, PM045_STARTUP_TRIAL=str(trial),
                   PM045_REPLAY_PROJECT='/tmp/pm045-independent-replay-fixture/project')
        log = OUT/f'{role}-{trial}.log'
        entry = {'role': role, 'trial': trial, 'begin_ns': time.monotonic_ns()}
        record['runs'].append(entry)
        save()
        with log.open('w') as stream:
            rc = subprocess.run([sys.executable, str(DRIVER)], cwd=ROOT, env=env,
                                stdout=stream, stderr=subprocess.STDOUT).returncode
        entry.update(returncode=rc, end_ns=time.monotonic_ns())
        text = log.read_text()
        location = next((s for s in text.splitlines() if s.startswith('/tmp/pm045-startup-')), None)
        entry['raw_directory'] = location
        try:
            assert rc == 0, text[-1500:]
            result = json.loads((Path(location)/'result.json').read_text())
            entry['host_cpu'] = summarize(result)
        except BaseException as error:
            entry['error'] = repr(error)
            record['status'] = 'Stopped on failed trial; no replacement'
            save()
            raise
        save()
        print(role, trial, {h:{p:v['max_ms'] for p,v in phases.items()} for h,phases in entry['host_cpu'].items()}, flush=True)
    record['status'] = 'All six fixed trials complete; per-cycle excesses retained. No full acceptance.'
    save()
