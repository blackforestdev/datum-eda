import json,subprocess,sys
from pathlib import Path
c=Path(sys.argv[1]);d=json.loads((c/'result.json').read_text());rows=[]
for run in d['runs']:
 p=Path(run['path']);r=json.loads((p/'result.json').read_text());t=r['pointer_trial'];s=r['cpu_observer']['samples'];a=s[t['start_sample']];b=s[t['active_end_sample']];elapsed=(b['monotonic_ns']-a['monotonic_ns'])/1e9;cpu=(b['group']['usage_usec']-a['group']['usage_usec'])/1000
 v=subprocess.run(['compare','-metric','AE','/tmp/pm045-pointer-native-emp4nxo2/final.png',str(p/'final.png'),'null:'],capture_output=True,text=True);assert v.returncode==0,v.stderr
 row={'role':run['role'],'trial':run['trial'],'path':str(p),'cpu_ms':cpu,'elapsed_seconds':elapsed,'cpu_percent':cpu/elapsed/10,'normal_exit_code':r.get('normal_exit_code'),'shutdown_receipt':r.get('shutdown_receipt'),'error':r.get('error'),'native_log_bytes':(p/'native.log').stat().st_size,'final_pixel_AE':v.stderr.strip(),'received_event_count':(p/'active-and-tail.log').read_text().count('cursor moved') if run['role']=='verbose' else None}
 rows.append(row)
pairs=[]
for i in range(1,4):
 a=next((r for r in rows if r['role']=='verbose' and r['trial']==i),None);b=next((r for r in rows if r['role']=='quiet' and r['trial']==i),None)
 if a and b:pairs.append({'trial':i,'verbose_minus_quiet_cpu_ms':a['cpu_ms']-b['cpu_ms'],'increase_relative_to_quiet_percent':100*(a['cpu_ms']/b['cpu_ms']-1)})
r={'qualification_pass':False,'diagnostic_only':True,'runs':rows,'descriptive_pairs':pairs,'limits':['Three matched mode pairs are descriptive overhead evidence,not seven-pair formal STAT inference. Do not subtract an estimated overhead from a qualification result.','Quiet runs preserve warm/final pixel and focus checks plus family CPU and shutdown receipt; received events,per-frame counters and GPUtimestamps are absent,so output/path equivalence is not completely observed.','Both modes use the quantized XTest stream3600calls1280changes;neither establishes120distinctsemantic-actions/s.','Main-thread CPU includes application/backend/driver/logging. The separate GDBsample is perturbed,not performancequalification.']}
(c/'analysis.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r,indent=2))
