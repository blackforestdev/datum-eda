from pathlib import Path
import json,math,statistics,collections,sys
out=Path(sys.argv[1])
classified=json.loads((out/'classified-gpu.json').read_text());active={tuple(r['identity']) for r in classified if r['phase']=='active'}
samples=[json.loads(line.split('gpu_measurement ',1)[1]) for line in (out/'native.log').read_text().splitlines() if line.startswith('gpu_measurement ')]
timed=[r for r in samples if (r['host'],r['device_epoch'],r['frame']) in active]
def stats(values):
 v=sorted(n/1e6 for n in values);return {'n':len(v),'min_ms':v[0],'median_ms':statistics.median(v),'p95_ms':v[math.ceil(.95*len(v))-1],'p99_ms':v[math.ceil(.99*len(v))-1],'max_ms':v[-1]}
metrics={'frame_span':stats([r['frame_span_ns'] for r in timed]),'own_pass_sum':stats([r['own_pass_sum_ns'] for r in timed]),'outside_passes':stats([r['frame_span_ns']-r['own_pass_sum_ns'] for r in timed])}
for name in ['upload-leading','restore','suffix']:
 metrics[name]=stats([sum(n for p,n in r['passes_ns'] if p==name) for r in timed])
receipt=json.loads((out/'input-receipt.json').read_text());transitions=[]
for r in receipt['records']:
 if r['workload'][1]==3 and r['before'].get('hover_utf8')!=r['after'].get('hover_utf8'):
  transitions.append({'demand':r['workload'][2],'before':r['before'].get('hover_utf8'),'after':r['after'].get('hover_utf8')})
archive=json.loads(Path('docs/reviews/gui-performance/gpu-redraw-proposal/timed-native-result/cold-demands.json').read_text());previous=[(r['preceding_hover_transition']['before'],r['preceding_hover_transition']['after']) for r in archive]
assert [(r['before'],r['after']) for r in transitions]==previous
r={'metrics':metrics,'over_4ms':sum(r['frame_span_ns']>4e6 for r in timed),'over_8ms':sum(r['frame_span_ns']>8e6 for r in timed),'total_gpu_frames':len(samples),'active_gpu_frames':len(timed),'input_records':len(receipt['records']),'input_limit':receipt['record_limit'],'input_overflow':receipt['overflow'],'active_world_submissions':sum(s['kind']=='world' for r in timed for s in r['submission_manifest']),'active_pass_sets':dict(collections.Counter(str([p for p,_ in r['passes_ns']]) for r in timed)),'hover_transitions':transitions,'matches_all14_previous_transitions':True,'limits':'Every active sample retained; nearest-rank quantiles match fixed runner. Pass quantiles are not additive. Suffix includes clipped drawing, attachment load/store and full exact8 resolve; existing timestamps cannot separate those costs. No matched baseline or quiet trial ran; no speedup, on/off overhead, DRM duty or full qualification inference.'}
(out/'cost-analysis.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps({k:v for k,v in r.items() if k!='hover_transitions'},indent=2))
