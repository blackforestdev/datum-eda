import hashlib,json,math,statistics,sys
from pathlib import Path
sys.path.insert(0,'/tmp')
from pm045_kernel_ledger import reconcile
root=Path('/tmp/pm045-window-campaign-o1yw7lsn')
campaign=json.loads((root/'result.json').read_text())

def stats(values):
 values=sorted(values)
 return {'n':len(values),'mean':statistics.mean(values),'min':values[0],'max':values[-1],'p95':values[math.ceil(.95*len(values))-1],'p99':values[math.ceil(.99*len(values))-1]}

summary={'status':'Bounded X11/Vulkan1x window-cycle evidence; S5 remains incomplete','qualification_pass':False,'candidate':campaign['candidate'],'binary_sha256':campaign['binary_sha256'],'trial_results':[],'failed_attempts':[],'comparison':'Three descriptive paired mode trials only; no formal seven-pair inference or overhead subtraction. Resource mode uses no ptrace/timestamp diagnostics.','scope':'One pinned F-DOA model and one native backend/scale profile. No full admission/observer uncertainty/scale/endurance/independent or temporal acceptance.'}
for run in campaign['runs']:
 if run['returncode']:
  summary['failed_attempts'].append(run);continue
 path=Path(run['path']);r=json.loads((path/'result.json').read_text())
 assert not r.get('error') and r['normal_exit_code']==0
 assert r['binary_sha256']==campaign['binary_sha256']
 assert len(r['cycles'])==93
 assert not Path(r['cpu_observer']['group']).exists()
 trial={'mode':run['mode'],'trial':run['trial'],'path':str(path),'hosts':{},'drain':r['drain'],'normal_exit_code':r['normal_exit_code'],'opaque_gpu_counter_qualification':False}
 for host in ['GLOBAL','PROJECT','NEW']:
  warm=[c for c in r['cycles'] if c['host']==host and c['phase']=='warm'];assert len(warm)==30
  assert all(c['focus_restored'] and c['open']['pixel_observations'][-1]['returncode']==0 for c in warm)
  trial['hosts'][host]={kind+'_cpu_ms':stats([c[kind]['cpu_ms'] for c in warm]) for kind in ['open','close']}
  trial['hosts'][host]['open_observed_wall_ms']=stats([c['open']['wall_ms'] for c in warm])
  trial['hosts'][host]['observed_mean_within_cpu_numbers']=trial['hosts'][host]['open_cpu_ms']['mean']<=50 and trial['hosts'][host]['close_cpu_ms']['mean']<=20
  trial['hosts'][host]['opens_above50ms']=sum(c['open']['cpu_ms']>50 for c in warm)
  samples=r['cpu_observer']['samples']
  trial['hosts'][host]['open_through_close_start_cpu_ms']=stats([(samples[c['close']['start_cpu_sample']]['group']['usage_usec']-samples[c['open']['start_cpu_sample']]['group']['usage_usec'])/1000 for c in warm])
 start=r['cpu_observer']['samples'][r['workload_first_cpu_sample']]
 end=r['cpu_observer']['samples'][-1]
 trial['whole_window']={'start_cpu_sample':start,'end_cpu_sample':end,'group_cpu_ms':(end['group']['usage_usec']-start['group']['usage_usec'])/1000,'elapsed_ms':(end['monotonic_ns']-start['monotonic_ns'])/1e6,'boundary_limit':'Counter observations are bracketed, not atomic; capture/observer overhead and shutdown semantic work remain explicit.'}
 if run['mode']=='kernel':
  ledger=reconcile(path/'kernel-ledger.jsonl');(path/'ledger-analysis.json').write_text(json.dumps(ledger,indent=2)+'\n')
  trial['supported_client_continuity']=ledger['supported_drm_continuity_complete'];trial['supported_clients']=len(ledger['clients']);trial['ledger_failures']=ledger['failures']
  records=[json.loads(s) for s in (path/'kernel-ledger.jsonl').read_text().splitlines()]
  opaque=[v for v in records if v['kind']=='opaque_gpu' and start['monotonic_ns']<=v['monotonic_ns']<=r['drain']['end_ns']]
  trial['opaque_in_warm_window']={'records':len(opaque),'paths':sorted({bytes.fromhex(v['path_hex']).decode() for v in opaque})}
  rows=(path/'native.log').read_text().splitlines()[r['action_log_start']:]
  samples=[json.loads(line.split('gpu_measurement ',1)[1]) for line in rows if 'gpu_measurement ' in line]
  assert not any('gpu_measurement_incomplete ' in line for line in rows)
  for sample in samples:
   expected=(sample['raw_ticks'][-1]-sample['raw_ticks'][0])*sample['timestamp_period_ns']
   assert abs(expected-sample['frame_span_ns'])<.01
  trial['gpu_span_ms']=stats([v['frame_span_ns']/1e6 for v in samples])
  trial['main_gpu_span_ms']=stats([v['frame_span_ns']/1e6 for v in samples if v['host']==1])
  trial['aux_gpu_span_ms']=stats([v['frame_span_ns']/1e6 for v in samples if v['host']!=1])
  trial['gpu_budget_qualification']='Unavailable: opaque GPU clients plus unqualified observer overhead. Raw timestamp excess retained; not an uninstrumented baseline claim.'
 summary['trial_results'].append(trial)
summary['descriptive_mode_comparisons']=[]
for trial in [1,2,3]:
 plain=next(v for v in summary['trial_results'] if v['trial']==trial and v['mode']=='plain')
 kernel=next(v for v in summary['trial_results'] if v['trial']==trial and v['mode']=='kernel')
 summary['descriptive_mode_comparisons'].append({'trial':trial,'open_cpu_ratios':{host:kernel['hosts'][host]['open_cpu_ms']['mean']/plain['hosts'][host]['open_cpu_ms']['mean'] for host in plain['hosts']},'whole_window_elapsed_ratio':kernel['whole_window']['elapsed_ms']/plain['whole_window']['elapsed_ms']})
(root/'analysis.json').write_text(json.dumps(summary,indent=2)+'\n')
for r in summary['trial_results']:
 print(r['mode'],r['trial'],{h:{'open_mean':round(v['open_cpu_ms']['mean'],3),'open_max':round(v['open_cpu_ms']['max'],3),'close_mean':round(v['close_cpu_ms']['mean'],3)} for h,v in r['hosts'].items()})
 if r['mode']=='kernel':print('GPU',r['gpu_span_ms'],'clients',r['supported_clients'],'supported_continuity',r['supported_client_continuity'],'opaque',r['opaque_in_warm_window'])
print('paired descriptive',summary['descriptive_mode_comparisons'])
