import json,re,subprocess,sys
from pathlib import Path
campaigns=[Path(p) for p in sys.argv[1:]]
result={'qualification_pass':False,'runs':[],'limits':['XTest press/release pairs deliver two native LineDelta events each: 3600 received for 1800 scheduled pairs; positive control delivers two rather than one. Not the required 60 native events/s or single-reversal recipe.','Verbose logging is enabled and overhead is not removed; CPU/action uses received event denominator only, not acknowledged semantic actions.','Camera numeric state and unrelated cameras are not instrumented. Exact endpoint images and submission logs do not establish physical presentation latency or whole-client GPU inactivity.','No defect-injection negative control, full resource, backend/scale, endurance, distinct-reviewer or owner UX acceptance. Baseline1 post-exit harness failure retained; remaining five declared trials use corrected bookkeeping. No replacement trials.']}
for campaign in campaigns:
 c=json.loads((campaign/'result.json').read_text())
 for r in c['runs']:
  p=Path(r['path']);v=json.loads((p/'result.json').read_text());row={**r,'normal_exit_code':v.get('normal_exit_code'),'harness_error':v.get('error'),'shutdown_receipt':v.get('shutdown_receipt'),'bounds':[]}
  log=(p/'native.log').read_bytes()
  for b in v['clamp_trials']:
   label=b['bound'];active=log[b['start_log_offset']:b['active_end_log_offset']].decode();tail=log[b['active_end_log_offset']:b['end_log_offset']].decode();positive=(p/(label+'-positive.log')).read_text();scheduled=json.loads((p/(label+'-input.json')).read_text());events=re.findall(r'mouse wheel delta=LineDelta\(([^)]+)\)',active)
   counts=lambda s:{tag:s.count(tag) for tag in ['redraw handler begin','prepared scene build begin','retained scene build begin','frame present begin','native upload frame','mouse wheel delta=']}
   elapsed=(b['active_finished_ns']-b['started_ns'])/1e9
   x={'bound':label,'scheduled_pairs':scheduled['count'],'received_wheel_events':len(events),'deltas':sorted(set(events)),'active_elapsed_seconds':elapsed,'active_cpu_ms':b['active_cpu_ms'],'active_cpu_percent':b['active_cpu_ms']/elapsed/10,'cpu_ms_per_received_event':b['active_cpu_ms']/len(events) if events else None,'tail_cpu_ms':b['tail_cpu_ms'],'active_counts':counts(active),'tail_counts':counts(tail),'positive_counts':counts(positive),'endpoint_AE':b['endpoint_AE'],'positive_readback':b['positive']}
   row['bounds'].append(x)
  result['runs'].append(row)
Path(sys.argv[-1]+'/analysis.json').write_text(json.dumps(result,indent=2)+'\n')
for r in result['runs']:print(r['role'],r['trial'],r['harness_error'],[(b['bound'],round(b['active_cpu_percent'],3),b['received_wheel_events'],b['active_counts']['frame present begin']) for b in r['bounds']])
