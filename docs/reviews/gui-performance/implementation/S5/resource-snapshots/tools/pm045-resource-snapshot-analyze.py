import sys,json,copy,collections,importlib.util
from pathlib import Path
spec=importlib.util.spec_from_file_location('gpu_evidence',Path(__file__).with_name('pm045-resource-gpu-analyze.py'));gpu=importlib.util.module_from_spec(spec);spec.loader.exec_module(gpu)

def check(rows,expected_origins):
 assert rows[0]['phase']=='start' and rows[-1]['phase']=='end' and rows[-1]['complete_delivery'] and rows[-1]['event_loop_ok']
 seen=set();closed=set();peaks={};budgets={};violations=[];sequence=0;last_end=0;maxima=collections.defaultdict(int);hostnames={};last=None
 def bound(label,value,limit,seq):
  maxima[label]=max(maxima[label],value)
  if value>limit:violations.append({'snapshot':seq,'boundary':label,'value':value,'limit':limit})
 for row in rows[1:-1]:
  assert row['phase']=='snapshot';sequence+=1;assert row['sequence']==sequence
  assert row['started_ns']>=last_end and row['finished_ns']>=row['started_ns'];last_end=row['finished_ns'];last=row
  owners={x['owner_id'] for x in row['text_cache_owners']};assert len(owners)==len(row['text_cache_owners'])
  for host in row['hosts']:
   r=host['reservations'];rid=r['renderer_id'];assert rid not in closed;seen.add(rid);hostnames[rid]=host['host']
   limits={'screen':16*2**20,'control_mesh':4*2**20,'atlas':32*2**20,'staging':16*2**20}
   for name,limit in limits.items():
    b=r[name];bid=b['budget_id'];assert b['limit_bytes']==limit
    assert 0<=b['reserved_bytes']<=b['lifetime_peak_reserved_bytes']
    if bid in budgets:assert budgets[bid]==limit
    budgets[bid]=limit;assert b['lifetime_peak_reserved_bytes']>=peaks.get(bid,0);peaks[bid]=b['lifetime_peak_reserved_bytes']
    bound('local_'+name,b['lifetime_peak_reserved_bytes'],limit,sequence)
   assert r['released']==all(r[n]['reserved_bytes']==0 for n in limits)
   if not host['present']:
    assert host['renderer'] is None
    if r['released']:closed.add(rid)
   else:
    v=host['renderer'];assert v is not None;assert v['text_keys']['owner_id'] in owners
    bound('control_mesh_entries',v['control_mesh']['entries'],256,sequence)
    bound('control_mesh_cpu',v['control_mesh']['total_bytes'],4*2**20,sequence)
  scope_ids=[s['owner_id'] for s in row['scoped_heap']];assert len(scope_ids)==len(set(scope_ids))
  for scope in row['scoped_heap']:
   assert scope['allocator_installed'];assert scope['peak_payload_bytes']>=scope['payload_bytes']>=0
  for owner in row['text_cache_owners']:
   bound('text_cache_local',owner['bytes']+owner['constructing_bytes'],8*2**20,sequence)
   if owner['retention_overflow']:violations.append({'snapshot':sequence,'boundary':'text retention overflow','owner_id':owner['owner_id']})
  bound('text_cache_process',sum(o['bytes']+o['constructing_bytes'] for o in row['text_cache_owners'])+row['text_cache_registry_bytes'],32*2**20,sequence)
  for width in row['thread_measurements']:
   bound('thread_measurement_entries',width['entries'],256,sequence);bound('thread_measurement_keys',width['key_bytes'],64*2**10,sequence);bound('thread_measurement_retained',width['retained_bytes'],128*2**10,sequence)
  for doc in row['documents_cpu']:
   assert doc['limit_bytes']==64*2**20;bound('document_cpu_retained',doc['retained_bytes'],doc['limit_bytes'],sequence);bound('document_history_entries',doc['history_entries'],6,sequence)
  for doc in row['documents_gpu']:
   assert doc['limit_bytes']==64*2**20;assert doc['reserved_bytes']<=doc['lifetime_peak_reserved_bytes'];bound('document_gpu_reserved_peak',doc['lifetime_peak_reserved_bytes'],doc['limit_bytes'],sequence)
  rss=row['gui_rss'];assert rss['rss_bytes']<=rss['high_water_bytes'];bound('gui_combined_high_water',rss['high_water_bytes'],512*2**20,sequence)
 assert sequence==rows[-1]['snapshots'] and rows[-1]['unreleased_hosts']==0
 assert last['final'] and not any(h['present'] for h in last['hosts'])
 assert seen==closed==set(expected_origins),(seen-closed,set(expected_origins)-seen)
 assert not last['documents_gpu'] and not last['documents_cpu'] and not last['text_cache_owners']
 return {'snapshots':sequence,'renderer_origins':len(seen),'native_host_names':dict(collections.Counter(hostnames.values())),
  'unique_local_budget_ids':len(budgets),'observed_maxima':dict(maxima),'sampled_cap_violations':violations,
  'final_cpu_scopes':last['scoped_heap'],'final_thread_measurements':last['thread_measurements'],
  'final_local_reservations_zero':True,'all_gpu_origins_have_resource_views':True,'qualification_pass':False,
  'limits':'Point samples and lifetime admission peaks only; views overlap. Missing transient CPU peaks/ref incidence, unsampled workloads, observer overhead and all remaining S5 requirements are not inferred from these records.'}

def analyze(path):
 history=gpu.analyze(path)
 assert history['simultaneous_private_trace']['calls_without_renderer_origin']==0,'native private-call renderer attribution incomplete'
 rows=[json.loads(x) for x in (path/'resources.jsonl').read_text().splitlines()]
 result=check(rows,history['renderer_origins']);controls={}
 for label,mutate in [
  ('missing-end',lambda r:r.pop()),
  ('missing-snapshot',lambda r:r.pop(1)),
  ('missing-host-closure',lambda r:r[-2].__setitem__('hosts',[])),
  ('false-cap-pass',lambda r:next(s for s in r if s['phase']=='snapshot' and s['hosts'])['hosts'][0]['reservations']['staging'].__setitem__('lifetime_peak_reserved_bytes',2**40))]:
  data=copy.deepcopy(rows);mutate(data)
  try:negative=check(data,history['renderer_origins'])
  except (AssertionError,KeyError,IndexError):controls[label]='rejected'
  else:
   assert negative['sampled_cap_violations'],('negative control accepted',label)
   controls[label]='reported cap violation'
 result['negative_controls']=controls
 return {'gpu_and_private':history,'resources':result}

if __name__=='__main__':
 path=Path(sys.argv[1]);result=analyze(path);(path/'analysis.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
