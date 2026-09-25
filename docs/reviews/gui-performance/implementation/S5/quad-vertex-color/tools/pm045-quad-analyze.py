import json,sys,math,subprocess,hashlib
from pathlib import Path
c=Path(sys.argv[1]);d=json.loads((c/'result.json').read_text());rows=[]
def summary(v):
 s=sorted(v);return {'count':len(s),'min':s[0],'p50':s[math.ceil(len(s)*.5)-1],'p95':s[math.ceil(len(s)*.95)-1],'p99':s[math.ceil(len(s)*.99)-1],'max':s[-1]}
for run in d['runs']:
 p=Path(run['path']);r=json.loads((p/'result.json').read_text());t=r.get('pointer_trial');row=dict(run)
 if not t:row['error']=r.get('error');rows.append(row);continue
 s=r['cpu_observer']['samples'];a=s[t['start_sample']];b=s[t['active_end_sample']];elapsed=(b['monotonic_ns']-a['monotonic_ns'])/1e9;cpu=(b['group']['usage_usec']-a['group']['usage_usec'])/1000
 data=(p/'active-and-tail.log').read_bytes();cut=t['active_end_log_offset']-t['start_log_offset']
 gpu=[json.loads(line.split('gpu_measurement ',1)[1]) for line in data.decode().splitlines() if 'gpu_measurement ' in line]
 tail=[json.loads(line.split('gpu_measurement ',1)[1]) for line in data[cut:].decode().splitlines() if 'gpu_measurement ' in line]
 cmp=subprocess.run(['compare','-metric','AE','/tmp/pm045-pointer-native-emp4nxo2/final.png',str(p/'final.png'),'null:'],capture_output=True,text=True);assert cmp.returncode in (0,1)
 row.update(cpu_ms=cpu,elapsed_seconds=elapsed,cpu_percent=cpu/elapsed/10,final_pixel_AE=cmp.stderr.strip(),normal_exit_code=r.get('normal_exit_code'),shutdown_receipt=r.get('shutdown_receipt'),error=r.get('error'),gpu_incomplete=r.get('gpu_incomplete'),action_gpu_incomplete=r.get('action_gpu_incomplete'),tail_gpu_samples=len(tail),tail_cpu_ms=t['tail_cpu_ms'])
 if gpu:
  row['gpu_frame_span_ms']=summary([x['frame_span_ns']/1e6 for x in gpu])
  row['gpu_pass_ms']={name:summary([dict(x['passes_ns'])[name]/1e6 for x in gpu if name in dict(x['passes_ns'])]) for name in sorted({name for x in gpu for name,_ in x['passes_ns']})}
 rows.append(row)
r={'qualification_pass':False,'runs':rows,'limits':['Three matched trial pairs; no seven-pair STAT relative acceptance.','All runs quiet withGPUtimestampsON; CPUincludesobserveroverhead and isnotquietCPUqualification. No per-frame forbiddenwork/shellsolve/worldupload orreceivedinputcounter coverage.','Existing3600scheduledintegerXTestcalls quantizeto1280physicalpositionchanges;120distinctsemantic-actions/s unqualified.','GPUrecords include measured active interval and5sectail;tail samples reported,not dropped. Nativeendpoints AE0 prove finalpixels/focusonly,notphysicalpresentationlatency/continuouspath/hits/ownerUX.','OnlyX11onXwaylandphysical60Hz1x oneMainleaf,hiddenterminal; no Wayland/otherdisplay-scale/multihost/endurance/resource/independentreviewer acceptance.']}
(c/'analysis.json').write_text(json.dumps(r,indent=2)+'\n')
for row in rows:print(row['role'],row['trial'],row.get('cpu_percent'),row.get('gpu_frame_span_ms'),row.get('final_pixel_AE'),row.get('error'))
