import json,re,math,subprocess,hashlib,sys
from pathlib import Path
campaign=Path(sys.argv[1]);c=json.loads((campaign/'result.json').read_text());out=[]
def pct(v,p):
 s=sorted(v);return s[max(0,math.ceil(len(s)*p)-1)] if s else None
for run in c['runs']:
 p=Path(run['path']);r=json.loads((p/'result.json').read_text());i=json.loads((p/'input.json').read_text());t=r['pointer_trial'];data=(p/'active-and-tail.log').read_bytes();cut=t['active_end_log_offset']-t['start_log_offset'];active=data[:cut].decode();tail=data[cut:].decode();assert len(data)==t['end_log_offset']-t['start_log_offset']
 planned=[tuple(x['physical_position']) for x in i['rows']];changed=[];prev=(300,150)
 for pos in planned:
  if pos!=prev:changed.append(pos)
  prev=pos
 received=[tuple(map(float,m)) for m in re.findall(r'cursor moved ([\d.]+),([\d.]+)',active)];j=0;missing=[]
 for pos in received:
  while j<len(changed) and changed[j]!=pos:missing.append(changed[j]);j+=1
  assert j<len(changed),(p,pos,j)
  j+=1
 missing+=changed[j:]
 assert received and received[-1]==planned[-1]==(300,150)
 life=[json.loads(line.split('native_surface_lifecycle ',1)[1]) for line in active.splitlines() if 'native_surface_lifecycle ' in line];reasons={k:sum(x['reason']==k for x in life) for k in sorted({x['reason'] for x in life})}
 s=r['cpu_observer']['samples'];a=s[t['start_sample']];b=s[t['active_end_sample']];elapsed=(b['monotonic_ns']-a['monotonic_ns'])/1e9;cpu=(b['group']['usage_usec']-a['group']['usage_usec'])/1000;gui=(b['gui_cpu_ns']-a['gui_cpu_ns'])/1e6
 assert abs(cpu-t['active_cpu_ms'])<0.001
 image=Path(c['runs'][0]['path'])/'final.png';cmp=subprocess.run(['compare','-metric','AE',str(image),str(p/'final.png'),'null:'],capture_output=True,text=True);assert cmp.returncode in (0,1)
 late=[(x['sent_ns']-x['scheduled_ns'])/1e6 for x in i['rows']]
 row={'role':run['role'],'trial':run['trial'],'gpu_diagnostic':bool(run['gpu']),'path':str(p),'returncode':run['returncode'],'normal_exit_code':r.get('normal_exit_code'),'error':r.get('error'),'shutdown_receipt':r.get('shutdown_receipt'),'scheduled_events':len(planned),'physical_position_changes':len(changed),'quantized_noop_events':len(planned)-len(changed),'received_cursor_events':len(received),'undelivered_changed_positions':missing,'final_position':received[-1],'cpu_ms':cpu,'gui_cpu_ms':gui,'family_minus_gui_ms':cpu-gui,'elapsed_seconds':elapsed,'cpu_percent':cpu/(elapsed*10),'cpu_ms_per_received_event':cpu/len(received),'cpu_ms_per_changed_position':cpu/len(changed),'cpu_10_percent_exceeded':cpu/(elapsed*10)>10,'cpu_0833_ms_per_received_exceeded':cpu/len(received)>0.833,'tail_cpu_ms':t['tail_cpu_ms'],'tail_log_lines':len(tail.splitlines()),'lifecycle_reason_counts':reasons,'retained_scene_build_begins':active.count('retained scene build begin'),'prepared_scene_build_begins':active.count('prepared scene build begin'),'aggregate_buffer_upload_payload_values':sorted(set(map(int,re.findall(r'buffer_payload_bytes: (\d+)',active)))),'final_pixel_AE_against_first_baseline':cmp.stderr.strip(),'schedule_lateness_ms':{'p95':pct(late,.95),'p99':pct(late,.99),'max':max(late)},'whole_run_gpu_sample_count':len(r.get('gpu_samples',[])),'gpu_incomplete':r.get('gpu_incomplete',[])}

 if run['gpu']:
  gpu=[json.loads(line.split('gpu_measurement ',1)[1]) for line in active.splitlines() if 'gpu_measurement ' in line]
  spans=[x['frame_span_ns']/1e6 for x in gpu];row['active_gpu_span_ms']={'count':len(spans),'p95':pct(spans,.95),'p99':pct(spans,.99),'max':max(spans),'sum':sum(spans)}
  row['active_gpu_pass_ms']={name:{'p95':pct([dict(x['passes_ns'])[name]/1e6 for x in gpu],.95),'p99':pct([dict(x['passes_ns'])[name]/1e6 for x in gpu],.99)} for name,_ in gpu[0]['passes_ns']}
 out.append(row)
 print(run['role'],run['trial'],run['gpu'],'CPU%',round(row['cpu_percent'],3),'received',len(received),'missing',len(missing),'AE',cmp.stderr.strip(),'tail',len(tail.splitlines()),'GPU',row['whole_run_gpu_sample_count'])
result={'qualification_pass':False,'runs':out,'limits':['Integer XTest quantization reduced3600 scheduled calls to1280 changed physical positions; missing changed positions reported individually. This does not prove120 distinct delivered semantic actions/second.','Per-event CPU denominators use actual received events or changed positions, never3600 scheduled calls or frames. No independent semantic-consumption acknowledgments exist in this trace.','Verbose native logging enabled equally in every run; overhead not subtracted or independently qualified. Family CPU includes kernel/descendants; residual against GUI clock includes sampling skew and is not exclusive engine attribution.','Endpoint pixel equality is relative to first baseline, not owner UX acceptance or physical presentation latency. No hit-state oracle, calibrated temporal/crosshair path proof, shell-solve counter or per-origin upload trace. Aggregate upload totals do not alone prove no world upload.','Five-second quiet tail does not alone establish250ms return-to-idle. No whole-driver GPU duty or NVIDIA counters, no negative controls, no formal seven-pair STAT, no Wayland/other-scale/endurance/independent-review acceptance.']}
(campaign/'analysis.json').write_text(json.dumps(result,indent=2)+'\n')
