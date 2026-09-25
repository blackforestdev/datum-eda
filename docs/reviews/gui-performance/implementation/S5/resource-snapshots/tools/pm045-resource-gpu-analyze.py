"""Reconcile bounded allocation delivery; never substitute it for full qualification."""
import json,sys,copy,re,collections
from pathlib import Path

def validate_private(rows):
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

def validate(rows):
 assert rows[0]['phase']=='start' and rows[-1]['phase']=='end'
 assert rows[-1]['complete_delivery'] and rows[-1]['event_loop_ok']
 observation=rows[0]['observation_id'];live={};seen=set();sequence=0;stamp=0;current=0;peak=0
 owners=set();origins=set();kinds=collections.Counter();reasons=collections.Counter();transitions=collections.Counter();batches=0;capacity=0
 immutable=('id','owner','renderer_origin','generation','kind','capacity_bytes')
 for row in rows[1:]:
  assert row['observation_id']==observation
  if row['phase']=='allocation':
   sequence+=1;assert row['sequence']==sequence;assert row['elapsed_ns']>=stamp;stamp=row['elapsed_ns']
   r=row['record'];identity=r['id'];transition=row['transition'];transitions[transition]+=1
   assert 0<=r['requested_bytes']<=r['capacity_bytes']
   assert 0<=r['prepared_references']<1000000 and 0<=r['submission_references']<1000000
   assert r['prepared_references'] or not r['prepared_consumers']
   assert r['submission_references'] or not r['submitted_consumers']
   assert r['retiring']==(r['retirement_reason'] is not None)
   if transition=='Registered':
    assert identity not in seen;seen.add(identity);live[identity]=r;current+=r['capacity_bytes'];peak=max(peak,current)
    assert not r['prepared_references'] and not r['submission_references'] and not r['retiring']
    owners.add(r['owner']);origins.add(r['renderer_origin']);kinds[r['kind']]+=1
   else:
    previous=live[identity]
    assert all(previous[k]==r[k] for k in immutable)
    if previous['retirement_reason'] is not None:assert previous['retirement_reason']==r['retirement_reason']
    assert r['submitted_source_bytes']>=previous['submitted_source_bytes']
    assert r['submitted_transfer_bytes']>=previous['submitted_transfer_bytes']
    if transition=='Released':
     assert not r['prepared_references'] and not r['submission_references']
     assert r['retirement_reason'] is not None
     reasons[r['retirement_reason']]+=1;current-=r['capacity_bytes'];del live[identity]
    else:live[identity]=r
  elif row['phase']=='batch':
   batches+=1;assert row['first_identity_id']==1
   assert row['total_events']==sequence and row['dropped_events']==0 and not row['invalid_accounting'] and not row['incomplete']
   assert row['active_allocations']==len(live) and row['tracked_capacity_bytes']==current and row['tracked_capacity_peak_bytes']==peak
   assert row['capacity']==rows[0]['capacity'];capacity=max(capacity,row['buffer_capacity_bytes'])
   # Bounds are lifetime pre-creation admission, not an atomic API-live sample.
   limits={'gpu_process':512*2**20,'atlas_process':128*2**20,'staging_process':64*2**20,'terminal_process':64*2**20}
   assert all(0<=v<=limits[k] for k,v in row['reservation_lifetime_peaks'].items())
  else:assert row['phase']=='end'
 assert batches and not live and current==0
 return {'events':sequence,'allocations':len(seen),'owners':len(owners),'renderer_origins':sorted(x for x in origins if x is not None),
  'unattributed_renderer_origin_present':None in origins,'kinds':dict(kinds),'release_reasons':dict(reasons),'transitions':dict(transitions),
  'tracked_registration_peak_bytes':peak,'buffer_capacity_bytes_each':capacity,'maximum_serial_drain_buffers':2,
  'final_reservation_lifetime_peaks':next(x for x in reversed(rows) if x['phase']=='batch')['reservation_lifetime_peaks'],
  'scope':'Delivery and internal reconciliation only. Registration/release callbacks are not exact API creation/drop instants; reservation peaks bound admitted capacity. Driver/RSS, local subcaps, observer overhead, workload and endurance qualification remain separate.'}

def analyze(path):
 rows=[json.loads(x) for x in (path/'gpu.jsonl').read_text().splitlines()];result=validate(rows)
 controls={}
 for label,mutate in [
  ('missing-registration',lambda data:data.pop(next(i for i,r in enumerate(data) if r.get('transition')=='Registered'))),
  ('missing-release',lambda data:data.pop(next(i for i,r in enumerate(data) if r.get('transition')=='Released'))),
  ('duplicate-sequence',lambda data:data.insert(2,copy.deepcopy(data[1]))),
  ('missing-end',lambda data:data.pop()),
  ('false-capacity',lambda data:next(r for r in data if r.get('transition')=='Registered')['record'].__setitem__('capacity_bytes',0)),
  ('false-reservation-cap',lambda data:next(r for r in data if r['phase']=='batch')['reservation_lifetime_peaks'].__setitem__('gpu_process',512*2**20+1))]:
  corrupted=copy.deepcopy(rows);mutate(corrupted)
  try:validate(corrupted)
  except (AssertionError,KeyError,IndexError):controls[label]='rejected'
  else:raise AssertionError(('negative control accepted',label))
 result['negative_controls']=controls
 native=(path/'native.log').read_text();mapping={}
 for host,renderer,epoch in re.findall(r'native upload frame window=.*? host=(\d+) renderer=(\d+) queue_epoch=(\d+)',native):
  mapping.setdefault(int(renderer),set()).add((int(host),int(epoch)))
 result['native_origin_incidence']={str(k):sorted(v) for k,v in mapping.items()}
 result['origins_missing_native_upload_mapping']=sorted(set(result['renderer_origins'])-mapping.keys())
 private=[json.loads(x) for x in (path/'private.jsonl').read_text().splitlines()]
 assert private[-1]['phase']=='end' and private[-1]['complete_delivery']
 result['simultaneous_private_trace']=validate_private(private)
 return result

if __name__=='__main__':
 path=Path(sys.argv[1]);result=analyze(path);(path/'analysis.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
