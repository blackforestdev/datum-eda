import json,statistics,sys,hashlib
from pathlib import Path
campaign=Path(sys.argv[1]);run=json.loads((campaign/'result.json').read_text())
summary={'qualification_pass':False,'order':run['order'],'runs':[],'negative_controls':[]}
for item in run['runs']:
 p=Path(item['path']);r=json.loads((p/'result.json').read_text())
 if 'trial' not in item:
  summary['negative_controls'].append({'mode':item['mode'],'result':r});continue
 assert len(r['cycles'])==93 and r['passed_native_cycles']
 a={'mode':item['mode'],'trial':item['trial'],'cycles':93,'normal_exit_code':r.get('normal_exit_code'),'error':r.get('error'),'hosts':{},'extra_pixel_attempts':0}
 for host in ('GLOBAL','PROJECT','NEW'):
  c=[c for c in r['cycles'] if c['host']==host and c['phase']=='warm'];assert len(c)==30
  h={}
  for action,limit in [('open',50),('close',20)]:
   values=[x[action]['cpu_ms'] for x in c];h[action]={'mean_cpu_ms':statistics.mean(values),'maximum_cpu_ms':max(values),'over_budget_cycles':[x['cycle'] for x in c if x[action]['cpu_ms']>limit],'limit_ms':limit}
  def kib(v):assert v.endswith(' kB');return int(v.split()[0])*1024
  h['max_immediate_rss_open_increment_bytes']=max(kib(x['rss_open']['VmRSS'])-kib(x['rss_before']['VmRSS']) for x in c)
  h['max_immediate_rss_close_increment_bytes']=max(kib(x['rss']['VmRSS'])-kib(x['rss_before']['VmRSS']) for x in c)
  h['rss_boundary']='GUI PID point samples; immediate close is not five-second recovery proof, and baseline is per-cycle before-open, not a separate warmed endurance baseline.'
  a['hosts'][host]=h
 for c in r['cycles']:
  assert c['focus_restored'] and c['open']['pixel_observations'][-1]['returncode']==0
  a['extra_pixel_attempts']+=len(c['open']['pixel_observations'])-1
 if item['mode']=='on':a['observations']=json.loads((p/'analysis.json').read_text())
 summary['runs'].append(a)
model=json.loads(Path('/tmp/pm045-admission-hqugp169/project/board/board.json').read_text());model.pop('uuid',None)
h=hashlib.sha256(json.dumps(model,sort_keys=True,separators=(',',':')).encode()).hexdigest();assert h=='33e62de1c1da2020f0608444a4802eac23fb97a9f56cc8cf87c844b6077499ed'
summary['normalized_fixture_sha256_after']=h
summary['limits']='All valid observed CPU budget excesses retained; no retries/resampling. Planned three pairs stopped after first off shutdown failure; no pair or formal seven-pair inference. No whole GPU duty/cold/endurance/backend/scale or full memory acceptance.'
(campaign/'analysis.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps([{k:v for k,v in x.items() if k!='observations'} for x in summary['runs']],indent=2))
