import json,collections,copy
from pathlib import Path
campaign=Path('/tmp/pm045-private-campaign-_9v5nd5k');source=Path('/tmp/pm045-private-native-on-dcnch1gc/private.jsonl');rows=[json.loads(x) for x in source.read_text().splitlines()]
def check(rows):
 assert rows[0]['phase']=='start' and rows[-1]['phase']=='end' and rows[-1]['complete_delivery']
 ident=rows[0]['observation_id'];active={};calls=[];sequence=0;elapsed=0;batches=[]
 for row in rows:
  assert row['observation_id']==ident
  if row['phase']=='batch':
   assert row['dropped_events']==0 and row['first_call_id']==1 and not row['incomplete']
   assert row['total_events']==sequence and row['active_calls']==len(active);batches.append(row)
  if row['phase']!='call':continue
  sequence+=1;assert row['sequence']==sequence and row['elapsed_ns']>=elapsed;elapsed=row['elapsed_ns']
  r=row['report'];assert r['allocator_installed'];key=r['call_id']
  if row['transition']=='Begin':
   assert key not in active;active[key]=row
  else:
   assert row['transition']=='Finished' and key in active
   start=active.pop(key);assert start['report']['owner_id']==r['owner_id'] and start['renderer_origin']==row['renderer_origin'] and start['owner_label']==row['owner_label']
   assert r['peak_bytes']>=max(r['initial_bytes'],r['final_bytes'])
   assert r['host_peak_bytes']>=max(r['host_initial_bytes'],r['host_final_bytes'])
   assert r['process_peak_bytes']>=max(r['process_initial_bytes'],r['process_final_bytes'])
   assert not r['exceeded'] and r['host_peak_bytes']<=row['host_limit_bytes'] and r['process_peak_bytes']<=row['process_limit_bytes']
   calls.append(row)
 assert not active and sequence==batches[-1]['total_events']
 assert sorted(x['report']['call_id'] for x in calls)==list(range(1,len(calls)+1))
 return {'events':sequence,'calls':len(calls),'owner_labels':dict(collections.Counter(x['owner_label'] for x in calls)),'cpu_owner_count':len(set(x['report']['owner_id'] for x in calls)),'renderer_origins':sorted(set(x['renderer_origin'] for x in calls if x['renderer_origin'] is not None)),'calls_without_renderer_origin':sum(x['renderer_origin'] is None for x in calls),'reported_peak_maxima':{k:max(x['report'][k] for x in calls) for k in ['peak_bytes','host_peak_bytes','process_peak_bytes']},'buffer_capacity_bytes':max(x['buffer_capacity_bytes'] for x in batches),'complete_private_call_delivery':True,'observed_calls_within_reported_limits':True,'qualification_pass':False}
result=check(rows);controls={}
for name in ['missing_begin','missing_finish','missing_end','duplicate_sequence','false_cap_pass']:
 modified=copy.deepcopy(rows)
 if name=='missing_end':modified.pop()
 elif name=='false_cap_pass':next(x for x in modified if x['phase']=='call' and x['transition']=='Finished')['report']['host_peak_bytes']=2**50
 elif name=='duplicate_sequence':next(x for x in modified if x['phase']=='call')['sequence']=0
 else:
  phase='Begin' if name=='missing_begin' else 'Finished';index=next(i for i,x in enumerate(modified) if x['phase']=='call' and x['transition']==phase);modified.pop(index)
 try:check(modified)
 except AssertionError:controls[name]='rejected'
 else:raise AssertionError('negative accepted: '+name)
result['negative_controls']=controls
result['limits']='Complete bounded private-call delivery only.66 text-measurement calls have explicit null renderer origin; shared/per-thread ownership must be reconciled separately. No whole-resource/driver/RSS or observer-overhead, backend/scale,endurance,independent qualification.'
(campaign/'analysis.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
