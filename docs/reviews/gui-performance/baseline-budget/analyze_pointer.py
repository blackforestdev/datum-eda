"""Summarize every frozen pointer attempt without deleting failures or pooling trials."""
from pathlib import Path
import json
import tarfile

ROOT = Path(__file__).resolve().parents[4]
BASE = ROOT/'docs/reviews/gui-performance/baseline-budget'
state = json.loads((ROOT/'target/gui-baseline-budget/pointer-state.json').read_text())
assert state['status'] in ('complete', 'stopped_failure')
archive = ROOT/'docs/reviews/gui-performance/gpu-redraw-proposal/r4-native-result/raw.tar.gz'
with tarfile.open(archive) as t:
    original = json.load(t.extractfile(next(n for n in t.getnames() if n.endswith('/result.json'))))

def row(r, label, historical=False):
    out = dict(trial=label, historical=historical, status=r['status'], gpu=r.get('gpu'), failures=r.get('failures', []))
    if 'active_start' in r and 'active_end' in r:
        a, b = r['active_start'], r['active_end']
        out['cpu_active_ms'] = (b['cpu_stat']['usage_usec']-a['cpu_stat']['usage_usec'])/1000
        out['observed_active_seconds'] = (b['monotonic_ns']-a['monotonic_ns'])/1e9
        out['cpu_percent_one_core'] = out['cpu_active_ms']/10/out['observed_active_seconds']
    if 'diagnosis' in r:
        out['input'] = r['diagnosis']['input']
        out['capture_state_matches'] = r['diagnosis']['capture_state_matches']
    out['readiness_pixel_difference'] = r.get('readiness_pixel_difference')
    out['final_pixel_difference'] = r.get('final_pixel_difference')
    return out

rows = [row(original, 'G1', True)]
for attempt in state['attempts']:
    path = Path(attempt['output'])/'result.json'
    if not path.is_file():
        rows.append(dict(trial=str(attempt['index']), status='setup_or_launcher_failure', attempt=attempt))
        continue
    r = json.loads(path.read_text())
    spec = attempt['spec']
    label = ('G' if spec['mode']=='gpu' else 'Q')+str(spec['pair'])
    rows.append(row(r, label))
valid = [r for r in rows if r['status'] in ('valid_descriptive_run', 'valid_budget_failure')]
gpu = [r for r in valid if isinstance(r.get('gpu'), dict)]
ranges = {k: dict(min=min(r['gpu'][k] for r in gpu), max=max(r['gpu'][k] for r in gpu)) for k in ('n','p95_ms','p99_ms','max_ms')}
pairs = []
for n in (2, 3):
    g = next((r for r in valid if r['trial']==f'G{n}'), None)
    q = next((r for r in valid if r['trial']==f'Q{n}'), None)
    if g and q:
        pairs.append(dict(pair=n, gpu_minus_quiet_cpu_ms=g['cpu_active_ms']-q['cpu_active_ms'], gpu_minus_quiet_cpu_percentage_points=g['cpu_percent_one_core']-q['cpu_percent_one_core'], gpu_elapsed_seconds=g['observed_active_seconds'], quiet_elapsed_seconds=q['observed_active_seconds']))
result = dict(status=state['status'], trials=rows, gpu_trial_ranges=ranges, contemporaneous_cpu_pairs=pairs, replacement_budgets='undecided', limitations=['Pointer subset only; not representative all-workload baseline.', 'Quiet retains semantic observation. Contrasts measure enabling GPU timestamps/polling/logging, not total instrumentation overhead.', 'Two contemporary pairs maximum; archived G1 has no matched quiet trial.', 'No uninstrumented GPU tail, physical display latency, full DRM duty or complete resource qualification.', 'No significance claim, pooled quantiles, favorable-run selection or automatic further experiment.'])
(BASE/'pointer-analysis.json').write_text(json.dumps(result, indent=2)+'\n')
print(json.dumps(dict(status=result['status'], trials=[{k:r[k] for k in ('trial','status','gpu','cpu_active_ms') if k in r} for r in rows], ranges=ranges, pairs=pairs), indent=2))
