import json,sys,collections,hashlib,copy
from pathlib import Path
campaign=Path(sys.argv[1]);r=json.loads((campaign/'result.json').read_text());result={'qualification_pass':False,'complete_planned_order':len(r['runs'])==6 and all(x['returncode']==0 for x in r['runs']),'trials':[],'failed_runs':[]}
for run in r['runs']:
 p=Path(run['path']);native=json.loads((p/'result.json').read_text())
 if run['returncode'] or native.get('error'):result['failed_runs'].append(run)
 for row in native.get('idle_trials',[]):
  samples=native['cpu_observer']['samples'];start=samples[row['start_sample']];end=samples[row['end_sample']]
  assert start['gui_pid']==end['gui_pid'] and start['gui_start_ticks']==end['gui_start_ticks']
  assert row['elapsed_s']>=60 and row['finished_ns']>row['started_ns']
  lines=(p/(row['host']+'-timed.log')).read_text().splitlines();life=[json.loads(line.split('native_surface_lifecycle ',1)[1]) for line in lines if 'native_surface_lifecycle ' in line]
  submits=sum(x['reason'] in ('submit','submit_upload') for x in life)
  assert submits==row['timed_submission_receipts']
  assert sum('frame present begin' in x for x in lines)==row['timed_presentations']
  group_usec=end['group']['usage_usec']-start['group']['usage_usec'];assert group_usec>=0
  assert abs(group_usec/1e6-row['cpu_seconds'])<1e-8
  gui_ns=end['gui_cpu_ns']-start['gui_cpu_ns'];assert gui_ns>=0
  violations=[]
  if row['cpu_duty_percent']>1:violations.append('CPU duty exceeds1percent')
  if submits or row['timed_presentations']:violations.append('Idle GPU submission/presentation')
  if row['pixels_returncode']!=0:violations.append('Idle content changed')
  result['trials'].append({'role':run['role'],'trial':run['trial'],'host':row['host'],'elapsed_s':row['elapsed_s'],'cpu_duty_percent':row['cpu_duty_percent'],'cgroup_cpu_usec':group_usec,'gui_cpu_ns':gui_ns,'cooperating_descendants_cpu_residual_ns':group_usec*1000-gui_ns,'counter_boundary':'Cgroup includes kernel/all descendants and exits. GUI clock sampled separately; residual is not exclusive engine attribution and includes sampling skew. No clamping.','presentations':row['timed_presentations'],'submission_receipts':submits,'redraw_requests':row['timed_redraw_requests'],'timed_log_lines':len(lines),'pixels_difference':row['pixels_difference'],'violations':violations,'normal_drained_exit':native.get('normal_exit_code')==0,'start_members':start['members'],'end_members':end['members']})
 result.setdefault('runtime_identity',[]).append({'role':run['role'],'trial':run['trial'],'surface_identity':list(dict.fromkeys(line.split('surface identity ',1)[1] for line in (p/'native.log').read_text().splitlines() if 'surface identity ' in line))})
model=json.loads(Path('/tmp/pm045-admission-hqugp169/project/board/board.json').read_text());model.pop('uuid',None)
h=hashlib.sha256(json.dumps(model,sort_keys=True,separators=(',',':')).encode()).hexdigest();assert h=='33e62de1c1da2020f0608444a4802eac23fb97a9f56cc8cf87c844b6077499ed';result['normalized_fixture_sha256_after']=h
result['limits']=['No wholeGPUdriver-client counters: existing opaque NVIDIA handles remain a missing method boundary. Zero application log receipts cannot replace required wholeGPU duty.','X11 actual60Hz1x single-leaf hidden-terminal only. Wayland,other supported scales,otherleaf configurations and visibleterminal remain.','Planned three pairs stopped after candidate2 final-close timeout; only first pair completed normally. Remaining three launches not run. No required three-trial or seven-pair inference. No software perpetual-redraw negative run performed; no complete SH04 or S5 acceptance.','CPU family total captures descendants via cgroup; separate GUI clock residual is not dedicated engine lifetime accounting. Endpoint images do not establish continuous displayed temporal qualification.']
(campaign/'analysis.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'complete_planned_order':result['complete_planned_order'],'trials':result['trials'],'failed_runs':result['failed_runs']},indent=2))
