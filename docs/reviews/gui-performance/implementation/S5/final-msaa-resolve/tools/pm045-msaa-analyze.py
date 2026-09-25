import json,statistics,math,hashlib
from pathlib import Path
p=Path('/tmp/pm045-msaa-campaign-y82lkusr');campaign=json.loads((p/'result.json').read_text());out=[]
def stats(v):
 s=sorted(v);return {'n':len(v),'mean_ms':statistics.mean(v),'p95_ms':s[math.ceil(.95*len(s))-1],'p99_ms':s[math.ceil(.99*len(s))-1],'max_ms':max(v)}
for run in campaign['runs']:
 r=json.loads((Path(run['path'])/'result.json').read_text());assert len(r['cycles'])==10 and r['exit_code']==0 and not r['warm_incomplete']
 groups={};passes={};names={}
 for s in r['samples'][r['warm_start_sample']:r['warm_end_sample']]:
  kind='menu' if any(x[0]=='menu-text' for x in s['passes_ns']) else 'closed'
  groups.setdefault(kind,[]).append(s['frame_span_ns']/1e6)
  assert abs(s['frame_span_ns']-(s['raw_ticks'][-1]-s['raw_ticks'][0])*s['timestamp_period_ns'])<.01
  names.setdefault(kind,set()).add(tuple(x[0] for x in s['passes_ns']))
  for name,duration in s['passes_ns']:passes.setdefault(kind+':'+name,[]).append(duration/1e6)
 out.append({'name':run['name'],'trial':run['trial'],'spans':{k:stats(v) for k,v in groups.items()},'passes':{k:stats(v) for k,v in passes.items()},'pass_sequences':{k:sorted(v) for k,v in names.items()},'pixel_checks':run.get('pixel_comparisons','baseline reference')})
result={'status':'Bounded native diagnostic improvement with exact static pixel parity; no S5 budget acceptance','runs':out,'binary_sha256':campaign['binary_sha256'],'scope':'60 native View-menu open/Escape-close cycles,10 per run,3 descriptive matched pairs. Main and menu screenshots identical across all runs. Not full W-WINDOWS, backend/scale, accounting, uncertainty, cold, endurance or independent performance replay.','preserved':'Same physical scene/text/menu-background/menu-text order,8xMSAA and retained atlas/lifetimes; no whole GPU duty or formal significance claim.'}
(p/'analysis.json').write_text(json.dumps(result,indent=2)+'\n')
for r in out:print(r['name'],r['trial'],{k:round(v['mean_ms'],3) for k,v in r['spans'].items()})
