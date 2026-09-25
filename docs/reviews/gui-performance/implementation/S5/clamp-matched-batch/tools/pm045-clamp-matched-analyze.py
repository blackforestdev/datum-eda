import json,re,subprocess,sys
from pathlib import Path
campaigns=[Path(p) for p in sys.argv[1:]]
result={'qualification_pass':False,'runs':[],'limits':['Corrected driver alternates pressed/released wheel edges at60Hz; original paired-driver method failure is separately retained. Verbose trials verify1800 native events per bound and one positive event. Quiet trials intentionally lack received-event and work counters. All endpoints compared with corrected pilot; no silent attribution of verbose counts to quiet trials.','Resource observations use diagnostics off, structural counters use separate matched diagnostics-on trials. No overhead subtraction; three pairs do not meet seven-pair formal relative inference. CPU/action quiet semantic acknowledgements are missing and therefore not inferred from scheduled input.','Camera numeric state and unrelated cameras are not instrumented. Exact endpoint images and submission logs do not establish physical presentation latency or whole-client GPU inactivity.','Release after single positive press produces a second inward event only in an explicitly separate cleanup interval; it is not included in the positive-control result. No defect-injection negative control, complete resource/backend-scale/endurance/distinct-reviewer or owner UX acceptance.']}
for campaign in campaigns:
 c=json.loads((campaign/'result.json').read_text())
 for r in c['runs']:
  p=Path(r['path']);v=json.loads((p/'result.json').read_text());row={**r,'verbose_log':v.get('verbose_log','1'),'normal_exit_code':v.get('normal_exit_code'),'harness_error':v.get('error'),'shutdown_receipt':v.get('shutdown_receipt'),'bounds':[]}
  log=(p/'native.log').read_bytes()
  for b in v['clamp_trials']:
   label=b['bound'];active=log[b['start_log_offset']:b['active_end_log_offset']].decode();tail=log[b['active_end_log_offset']:b['end_log_offset']].decode();positive=(p/(label+'-positive.log')).read_text();scheduled=json.loads((p/(label+'-input.json')).read_text());events=re.findall(r'mouse wheel delta=LineDelta\(([^)]+)\)',active)
   counts=lambda s:{tag:s.count(tag) for tag in ['redraw handler begin','prepared scene build begin','retained scene build begin','frame present begin','native upload frame','mouse wheel delta=']}
   elapsed=(b['active_finished_ns']-b['started_ns'])/1e9
   wall=[]
   for line in active.splitlines():
    if 'mouse wheel delta=' not in line:continue
    stamp=re.search(r'tv_sec: (\d+), tv_nsec: (\d+)',line);assert stamp,line
    wall.append(int(stamp[1])*10**9+int(stamp[2]))
   intervals=[(b-a)/1e6 for a,b in zip(wall,wall[1:])]
   lateness=[(x['sent_ns']-x['scheduled_ns'])/1e6 for x in scheduled['rows']]
   def distribution(xs):
    if not xs:return None
    ys=sorted(xs);n=len(ys);return {'n':n,'min':ys[0],'p50':ys[(n-1)//2],'p95':ys[(95*n+99)//100-1],'p99':ys[(99*n+99)//100-1],'max':ys[-1]}
   cadence={'native_interarrival_ms':distribution(intervals),'driver_lateness_ms':distribution(lateness),'native_first_to_last_seconds':(wall[-1]-wall[0])/1e9 if wall else None,'clock_limit':'Native wall-clock cadence only; no cross-clock or physical presentation latency inference.'}
   x={'bound':label,'cadence':cadence,'scheduled_transitions':scheduled['count'],'received_wheel_events':len(events),'deltas':sorted(set(events)),'active_elapsed_seconds':elapsed,'active_cpu_ms':b['active_cpu_ms'],'gui_cpu_ms':(v['cpu_observer']['samples'][b['active_end_sample']]['gui_cpu_ns']-v['cpu_observer']['samples'][b['start_sample']]['gui_cpu_ns'])/1e6,'active_cpu_percent':b['active_cpu_ms']/elapsed/10,'quiet_cpu_duty_cap_percent':1.0 if v.get('verbose_log')=='0' else None,'quiet_cpu_duty_pass':b['active_cpu_ms']/elapsed/10<=1.0 if v.get('verbose_log')=='0' else None,'cpu_ms_per_received_event':b['active_cpu_ms']/len(events) if events else None,'tail_cpu_ms':b['tail_cpu_ms'],'active_counts':counts(active),'tail_counts':counts(tail),'positive_counts':counts(positive),'endpoint_AE':b['endpoint_AE'],'pilot_endpoint_comparisons':b.get('pilot_endpoint_comparisons'),'cleanup_counts':counts((p/(label+'-release-cleanup.log')).read_text()),'positive_readback':b['positive']}
   row['bounds'].append(x)
  result['runs'].append(row)
Path(sys.argv[-1]+'/analysis.json').write_text(json.dumps(result,indent=2)+'\n')
for r in result['runs']:print(r['role'],r['trial'],r['harness_error'],[(b['bound'],round(b['active_cpu_percent'],3),b['received_wheel_events'],b['active_counts']['frame present begin']) for b in r['bounds']])
