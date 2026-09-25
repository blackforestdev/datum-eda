import json,collections
from pathlib import Path
p=Path('/tmp/pm045-panes-resource-izo05o5k');rows=[json.loads(l) for l in (p/'gpu.jsonl').read_text().splitlines()];active={};seen={};sequence=0;peak=0;uniform_peak=0;releases={};batches=0
for e in rows:
 if e['phase']=='allocation':
  sequence+=1;assert e['sequence']==sequence
  r=e['record'];i=r['id']
  if e['transition']=='Registered':assert i not in seen;seen[i]=e;active[i]=r
  elif e['transition']=='Released':
   assert i in active and r['prepared_references']==r['submission_references']==0
   del active[i];releases[i]=e
  else:assert i in active;active[i]=r
  peak=max(peak,sum(x['capacity_bytes'] for x in active.values()));uniform_peak=max(uniform_peak,sum(x['capacity_bytes'] for x in active.values() if x['kind']=='Uniform'))
 elif e['phase']=='batch':
  batches+=1;assert e['total_events']==sequence and e['active_allocations']==len(active)
  assert e['tracked_capacity_bytes']==sum(x['capacity_bytes'] for x in active.values())
  assert e['tracked_capacity_peak_bytes']==peak
  assert not e['incomplete'] and not e['invalid_accounting'] and not e['dropped_events'] and e['first_identity_id']==1
assert rows[-1]['phase']=='end' and rows[-1]['complete_delivery'] and not active and set(seen)==set(releases)
uniforms=[]
for i,e in seen.items():
 if e['record']['kind']=='Uniform':uniforms.append({'id':i,'owner':e['record']['owner'],'bytes':e['record']['capacity_bytes'],'registered_elapsed_ns':e['elapsed_ns'],'released_elapsed_ns':releases[i]['elapsed_ns'],'lifetime_ms':(releases[i]['elapsed_ns']-e['elapsed_ns'])/1e6,'final_consumers':releases[i]['record']['consumers']})
resources=[json.loads(l) for l in (p/'resources.jsonl').read_text().splitlines()];snaps=[s for s in resources if s['phase']=='snapshot'];assert resources[-1]['complete_delivery'] and resources[-1]['unreleased_hosts']==0 and snaps[-1]['final'];assert not snaps[-1]['documents_cpu'] and not snaps[-1]['documents_gpu'];assert all(h['reservations']['released'] for h in snaps[-1]['hosts'])
report={'allocation_events':sequence,'registrations':len(seen),'releases':len(releases),'batches_reconciled':batches,'tracked_capacity_peak_bytes':peak,'scope':'Application tracked allocation events; API registration follows creation. Not driver residency or complete memory total.','uniform_peak_bytes':uniform_peak,'uniform_lifetimes':uniforms,'resource_snapshots':len(snaps),'document_cpu_retained_values':sorted({d['retained_bytes'] for s in snaps for d in s['documents_cpu']}),'document_gpu_reserved_values':sorted({d['reserved_bytes'] for s in snaps for d in s['documents_gpu']}),'gui_rss_sample_min_bytes':min(s['gui_rss']['rss_bytes'] for s in snaps),'gui_rss_sample_max_bytes':max(s['gui_rss']['rss_bytes'] for s in snaps),'gui_rss_final_bytes':snaps[-1]['gui_rss']['rss_bytes'],'memory_limit':'GUI sampled RSS only, includes observer overhead; no summed engine/process-private/driver accounting or full no-leak/endurance proof. Document/cache/reservation/scoped views overlap.','final_tracked_capacity_bytes':0,'final_documents_cpu_gpu_empty':True,'unreleased_hosts':0,'qualification_pass':False}
(p/'allocation-analysis.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
