"""Analyze preserved native endurance data; no workload replay or clock inference."""
import collections
import gzip
import hashlib
import json
import math
from pathlib import Path
import sys

raw = Path(sys.argv[1])
evidence = Path(sys.argv[2])
with gzip.open(evidence / 'raw-sha256.json.gz', 'rt') as f:
    expected = json.load(f)
for name in ('result.json', 'resources.jsonl'):
    assert hashlib.sha256((raw / name).read_bytes()).hexdigest() == expected[name], name
run = json.loads((raw / 'result.json').read_text())
assert not any(run.get(k) for k in ('error', 'cleanup_errors', 'persistence_errors', 'forced_cleanup'))
assert run['normal_exit_code'] == 0
cycles = run['cycles']
samples = run['samples']
assert len(cycles) == 600 and all(c['completed'] for c in cycles)
assert all(a['monotonic_ns'] < b['monotonic_ns'] for a, b in zip(samples, samples[1:]))
assert all(a['group']['usage_usec'] <= b['group']['usage_usec'] for a,b in zip(samples,samples[1:]))
start = run['workload_start_ns']
assert run['workload_end_ns'] - start >= 3600 * 10**9

def bounds(lo, hi):
    # sample() records its timestamp before reading CPU counters. The next
    # sample timestamp is a conservative end bound for that counter read.
    before_lo = max(i for i in range(len(samples)-1) if samples[i+1]['monotonic_ns'] <= lo)
    after_lo = next(i for i,s in enumerate(samples) if s['monotonic_ns'] >= lo)
    before_hi = max(i for i in range(len(samples)-1) if samples[i+1]['monotonic_ns'] <= hi)
    after_hi = next(i for i,s in enumerate(samples) if s['monotonic_ns'] >= hi)
    def cpu(i): return samples[i]['group']['usage_usec']
    lower = cpu(before_hi)-cpu(after_lo)
    upper = cpu(after_hi)-cpu(before_lo)
    assert 0 <= lower <= upper
    seconds=(hi-lo)/1e9
    return {'interval_ns':[lo,hi], 'sample_indices':{'before_start':before_lo,'after_start':after_lo,'before_end':before_hi,'after_end':after_hi},
            'group_cpu_usec_bounds':[lower,upper], 'group_cpu_percent_bounds':[lower/(seconds*1e4),upper/(seconds*1e4)]}

windows={name:bounds(start+lo*10**9,start+hi*10**9) for name,lo,hi in [('first',0,600),('last',3000,3600)]}
a,b=windows['first']['group_cpu_usec_bounds'],windows['last']['group_cpu_usec_bounds']
ratio=[b[0]/a[1]-1,b[1]/a[0]-1]

def stats(values):
    v=sorted(values)
    return {'n':len(v),'mean_ms':sum(v)/len(v),'p95_ms':v[math.ceil(.95*len(v))-1],'max_ms':v[-1]}
latency={}
for name,lo,hi in [('first',0,600),('last',3000,3600)]:
    selected=[c for c in cycles if lo*10**9 <= c['scheduled_ns']-start < hi*10**9]
    latency[name]={}
    for host in ('GLOBAL','PROJECT','NEW'):
        group=[c for c in selected if c['host']==host]
        latency[name][host]={}
        for phase in ('open','close'):
            latency[name][host][phase]=stats([(samples[c[phase+'_end_sample']]['monotonic_ns']-samples[c[phase+'_begin_sample']]['monotonic_ns'])/1e6 for c in group])
latency_change={host:{phase:{metric:latency['last'][host][phase][metric]/latency['first'][host][phase][metric]-1 for metric in ('mean_ms','p95_ms','max_ms')} for phase in ('open','close')} for host in ('GLOBAL','PROJECT','NEW')}

snapshots=[]
with (raw/'resources.jsonl').open() as f:
    for line in f:
        row=json.loads(line)
        if row['phase']=='snapshot':snapshots.append(row)
closed={}
for row in snapshots:
    for host in row['hosts']:
        if not host['present'] and host['reservations']['released']:
            closed.setdefault(host['window'],row)

points=[]
for index in [0]+list(range(49,600,50)):
    c=cycles[index]; row=closed['WindowId('+str(c['window'])+')']
    assert all(h['host']=='MAIN' for h in row['hosts'] if h['present'])
    main=next(h for h in row['hosts'] if h['host']=='MAIN' and h['present'])
    scope=collections.Counter()
    for s in row['scoped_heap']:scope[s['label']]+=s['payload_bytes']
    values={'gui_rss_bytes':row['gui_rss']['rss_bytes'],'gui_hwm_bytes':row['gui_rss']['high_water_bytes'],
            'observer_window_strings_bytes':row['observer_storage']['window_strings_capacity_bytes'],
            'text_cache_registry_bytes':row['text_cache_registry_bytes'],
            'text_cache_owned_bytes':sum(s['bytes']+s['constructing_bytes'] for s in row['text_cache_owners']),
            'document_cpu_retained_bytes':sum(s['retained_bytes'] for s in row['documents_cpu']),
            'document_gpu_reserved_bytes':sum(s['reserved_bytes'] for s in row['documents_gpu'])}
    values.update({'main_'+k+'_reserved_bytes':main['reservations'][k]['reserved_bytes'] for k in ('screen','control_mesh','atlas','staging')})
    values.update({'scope_'+k+'_payload_bytes':v for k,v in scope.items()})
    points.append({'completed_cycle':index+1,'scheduled_seconds':(c['scheduled_ns']-start)/1e9,'window_id':c['window'],'snapshot_sequence':row['sequence'],'snapshot_relative_interval_ns':[row['started_ns'],row['finished_ns']],'values':values})
keys=set.intersection(*(set(p['values']) for p in points))
trends={}
for key in sorted(keys):
    v=[p['values'][key] for p in points]
    trends[key]={'first':v[0],'last':v[-1],'minimum':min(v),'maximum':max(v),'delta':v[-1]-v[0],'nondecreasing_with_growth':all(a<=b for a,b in zip(v,v[1:])) and v[-1]>v[0],'constant_after_cycle50':len(set(v[1:]))==1}
result={'qualification_pass':False,'source_hashes':{k:expected[k] for k in ('result.json','resources.jsonl')},
        'cpu_windows':windows,'cpu_relative_change_bounds':ratio,'diagnostic_cpu_5percent_regression_excluded_by_bounds':ratio[1]<=.05,
        'readiness_wall_intervals':latency,'readiness_relative_changes':latency_change,'post_close_resource_points':points,'resource_trends':trends,
        'limits':['CPU bounds use monotonically accumulated whole-cgroup counters over fixed first/last600second intervals, with adjacent timestamps bounding non-atomic sample reads. No interpolation, overhead subtraction or diagnostics-off cap inference.',
        'First/last windows have100cycles each, with differing round-robin host mixes; per-host readiness statistics are reported separately. This is within-run descriptive drift, not a formal relative candidate/baseline claim.',
        'Open wall intervals start before xdotool Return and end after exact screenshot-readiness polling; close intervals include external focus/destruction polling. These include harness/process scheduling overhead and are not calibrated displayed-state latency.',
        'Resource points join the first released native-window snapshot to the corresponding cycle by exact WindowId. They are lifecycle-selected near five-minute cycle boundaries, not asserted exact five-minute wall-clock samples. Resource Instant timestamps have no shared absolute origin in this receipt.',
        'Scoped heap, caches, documents and reservations overlap. Never sum these into a memory grand total. RSS includes diagnostic retention and mappings, and is not driver-residency measurement.',
        'These summaries cannot establish full MEM-03:20recoveries, native latency/idle-return, instantaneous resources, observer overhead, complete configuration coverage, distinct replay and owner acceptance remain open.']}
(evidence/'drift-analysis.json').write_text(json.dumps(result,indent=2)+'\n')
print('CPU relative change bounds',ratio)
print('Increasing resource endpoints',{k:v for k,v in trends.items() if v['delta']>0})
print('Readiness changes',latency_change)
